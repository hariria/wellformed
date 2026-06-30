/**
 * Object schema builder.
 */

import type {
  Constraint,
  Predicate,
  PropertySchema,
  TypeSchema,
  UnknownKeysBehavior,
} from "../ir/types.js";
import { ConditionBuilder } from "./condition.js";
import {
  BaseBuilder,
  type ConstraintOptions,
  makeConstraint,
  type SchemaBuilder,
} from "./types.js";

/**
 * Property definition for object builder.
 */
export interface PropertyDef<
  T extends SchemaBuilder = SchemaBuilder,
  Required extends boolean = boolean,
> {
  builder: T;
  required?: Required;
}

/**
 * Shape definition - maps property names to builders or property defs.
 */
export type ObjectShape = Record<string, SchemaBuilder | PropertyDef>;

/**
 * Check if a value is a PropertyDef.
 */
function isPropertyDef(
  value: SchemaBuilder | PropertyDef,
): value is PropertyDef {
  return "builder" in value && "toTypeSchema" in value.builder;
}

/**
 * Create a flattened PropertySchema from a builder and required flag.
 * Rust uses #[serde(flatten)] so the TypeSchema fields are merged with required.
 */
function makePropertySchema(
  builder: SchemaBuilder,
  required: boolean,
): PropertySchema {
  const schema = builder.toTypeSchema();
  // Flatten: merge required into the type schema
  // Only include required if it's false (Rust defaults to true)
  if (required) {
    return schema; // Rust defaults required to true, so omit it
  }
  return { ...schema, required: false };
}

/**
 * Builder for object schemas.
 */
export class ObjectBuilder<
  S extends ObjectShape = ObjectShape,
> extends BaseBuilder<TypeSchema & { type: "object" }> {
  /** @internal Exposed for type inference */
  readonly _shape: S;
  private _properties: Record<string, PropertySchema> = {};
  private _additionalProperties = false;
  private _unknownKeys?: UnknownKeysBehavior;
  private _catchall?: TypeSchema;
  private _rules: Constraint[] = [];

  constructor(shape?: S) {
    super();
    this._shape = (shape ?? {}) as S;
    if (shape) {
      for (const [key, value] of Object.entries(shape)) {
        if (isPropertyDef(value)) {
          this._properties[key] = makePropertySchema(
            value.builder,
            value.required !== false,
          );
        } else {
          // Check if builder is marked as optional via .optional()
          const isOptionalBuilder = "isOptional" in value && value.isOptional();
          this._properties[key] = makePropertySchema(value, !isOptionalBuilder);
        }
      }
    }
  }

  /**
   * Add a property to the object.
   */
  prop(name: string, builder: SchemaBuilder, required = true): this {
    this._properties[name] = makePropertySchema(builder, required);
    return this;
  }

  /**
   * Add an optional property.
   * @deprecated Use the shape syntax with .optional() instead: `w.object({ field: w.string().optional() })`
   */
  optionalProp(name: string, builder: SchemaBuilder): this {
    return this.prop(name, builder, false);
  }

  /**
   * Allow additional properties beyond those defined.
   */
  additionalProperties(allow = true): this {
    this._additionalProperties = allow;
    this._unknownKeys = allow ? "passthrough" : "strict";
    return this;
  }

  /**
   * Reject unknown keys.
   */
  strict(): this {
    this._additionalProperties = false;
    this._unknownKeys = "strict";
    return this;
  }

  /**
   * Keep unknown keys.
   */
  passthrough(): this {
    this._additionalProperties = true;
    this._unknownKeys = "passthrough";
    return this;
  }

  /**
   * Remove unknown keys from validated output.
   */
  strip(): this {
    this._additionalProperties = false;
    this._unknownKeys = "strip";
    return this;
  }

  /**
   * Validate unknown keys against a schema.
   */
  catchall(builder: SchemaBuilder): this {
    this._catchall = builder.toTypeSchema();
    this._additionalProperties = true;
    this._unknownKeys = "passthrough";
    return this;
  }

  /**
   * Add a cross-field validation rule.
   */
  rule(
    pred: Predicate,
    code: string,
    message: string,
    options?: ConstraintOptions,
  ): this {
    this._rules.push(makeConstraint(pred, code, message, options));
    return this;
  }

  /**
   * Start a fluent conditional rule.
   *
   * @example
   * ```ts
   * schema
   *   .when("type").equals("individual").require("ssn")
   *   .when("type").equals("business").require("ein")
   *   .when("country").equals("US").and("type").equals("individual").require("ssn")
   * ```
   */
  when(field: string): ConditionBuilder<this>;
  /**
   * Add a conditional requirement: if condition is true, then consequent must be true.
   * @deprecated Use the fluent API: `.when("field").equals(value).require("other")`
   */
  when(
    condition: Predicate,
    consequent: Predicate,
    code: string,
    message: string,
    options?: ConstraintOptions,
  ): this;
  when(
    fieldOrCondition: string | Predicate,
    consequent?: Predicate,
    code?: string,
    message?: string,
    options?: ConstraintOptions,
  ): this | ConditionBuilder<this> {
    // Fluent API: when("fieldName")
    if (typeof fieldOrCondition === "string") {
      return new ConditionBuilder<this>(
        fieldOrCondition,
        (pred, c, m, opts) => {
          this.rule(pred, c, m, opts);
          return this;
        },
        this,
      );
    }

    // Legacy API: when(predicate, predicate, code, message)
    return this.rule(
      // biome-ignore lint/suspicious/noThenProperty lint/style/noNonNullAssertion: guaranteed by overload signature
      { type: "implies", if: fieldOrCondition, then: consequent! },
      // biome-ignore lint/style/noNonNullAssertion: guaranteed by overload signature
      code!,
      // biome-ignore lint/style/noNonNullAssertion: guaranteed by overload signature
      message!,
      options,
    );
  }

  /**
   * Require that if one field exists, another must also exist.
   */
  requireWith(
    field1: string,
    field2: string,
    options?: ConstraintOptions,
  ): this {
    return this.rule(
      { type: "required_with", field: `/${field2}`, with: `/${field1}` },
      options?.code ?? "MISSING_REQUIRED_FIELD",
      options?.message ?? `${field2} is required when ${field1} is present`,
      options,
    );
  }

  /**
   * Require that a field exists when another field is absent.
   */
  requireWithout(
    field: string,
    without: string,
    options?: ConstraintOptions,
  ): this {
    return this.rule(
      { type: "required_without", field: `/${field}`, without: `/${without}` },
      options?.code ?? "MISSING_REQUIRED_FIELD",
      options?.message ?? `${field} is required when ${without} is missing`,
      options,
    );
  }

  /**
   * Require that fields are mutually exclusive.
   */
  mutuallyExclusive(
    field1: string,
    field2: string,
    options?: ConstraintOptions,
  ): this {
    return this.rule(
      {
        type: "not",
        predicate: {
          type: "and",
          predicates: [
            { type: "exists", path: `/${field1}` },
            { type: "exists", path: `/${field2}` },
          ],
        },
      },
      options?.code ?? "MUTUALLY_EXCLUSIVE",
      options?.message ?? `${field1} and ${field2} cannot both be present`,
      options,
    );
  }

  /**
   * Require at least one of the specified fields.
   */
  requireOneOf(fields: string[], options?: ConstraintOptions): this {
    return this.rule(
      {
        type: "or",
        predicates: fields.map((f) => ({
          type: "exists" as const,
          path: `/${f}`,
        })),
      },
      options?.code ?? "MISSING_REQUIRED_FIELD",
      options?.message ?? `At least one of ${fields.join(", ")} is required`,
      options,
    );
  }

  /**
   * Require exactly one of the specified fields.
   */
  requireExactlyOneOf(fields: string[], options?: ConstraintOptions): this {
    return this.rule(
      {
        type: "exactly_one_of",
        paths: fields.map((f) => `/${f}`),
      },
      options?.code ?? "EXACTLY_ONE_REQUIRED",
      options?.message ?? `Exactly one of ${fields.join(", ")} is required`,
      options,
    );
  }

  /**
   * Require two fields to have equal values.
   *
   * @example
   * ```ts
   * // Schedule B total must equal Form 941 liability
   * schema.requireFieldsMatch("scheduleB/total", "form941/liability")
   * ```
   */
  requireFieldsMatch(
    field1: string,
    field2: string,
    options?: ConstraintOptions,
  ): this {
    return this.rule(
      { type: "eq_fields", left: `/${field1}`, right: `/${field2}` },
      options?.code ?? "FIELDS_MISMATCH",
      options?.message ?? `${field1} must equal ${field2}`,
      options,
    );
  }

  /**
   * Require that field1 > field2.
   */
  requireFieldGreaterThan(
    field1: string,
    field2: string,
    options?: ConstraintOptions,
  ): this {
    return this.rule(
      { type: "gt_field", left: `/${field1}`, right: `/${field2}` },
      options?.code ?? "FIELD_COMPARISON_FAILED",
      options?.message ?? `${field1} must be greater than ${field2}`,
      options,
    );
  }

  /**
   * Require that field1 >= field2.
   */
  requireFieldGreaterOrEqual(
    field1: string,
    field2: string,
    options?: ConstraintOptions,
  ): this {
    return this.rule(
      { type: "gte_field", left: `/${field1}`, right: `/${field2}` },
      options?.code ?? "FIELD_COMPARISON_FAILED",
      options?.message ??
        `${field1} must be greater than or equal to ${field2}`,
      options,
    );
  }

  /**
   * Require that field1 < field2.
   */
  requireFieldLessThan(
    field1: string,
    field2: string,
    options?: ConstraintOptions,
  ): this {
    return this.rule(
      { type: "lt_field", left: `/${field1}`, right: `/${field2}` },
      options?.code ?? "FIELD_COMPARISON_FAILED",
      options?.message ?? `${field1} must be less than ${field2}`,
      options,
    );
  }

  /**
   * Require that field1 <= field2.
   */
  requireFieldLessOrEqual(
    field1: string,
    field2: string,
    options?: ConstraintOptions,
  ): this {
    return this.rule(
      { type: "lte_field", left: `/${field1}`, right: `/${field2}` },
      options?.code ?? "FIELD_COMPARISON_FAILED",
      options?.message ?? `${field1} must be less than or equal to ${field2}`,
      options,
    );
  }

  /**
   * Require that the sum of specified fields equals a target field.
   *
   * @example
   * ```ts
   * // Lines 1-5 must sum to line 6
   * schema.requireSum(["line1", "line2", "line3", "line4", "line5"], "line6")
   * ```
   */
  requireSum(
    fields: string[],
    target: string,
    options?: ConstraintOptions,
  ): this {
    return this.rule(
      {
        type: "sum_equals",
        paths: fields.map((f) => `/${f}`),
        target: `/${target}`,
      },
      options?.code ?? "SUM_MISMATCH",
      options?.message ?? `Sum of ${fields.join(", ")} must equal ${target}`,
      options,
    );
  }

  /**
   * Require that the sum of specified fields equals a specific value.
   *
   * @example
   * ```ts
   * // Percentages must sum to 100
   * schema.requireSumEquals(["percent1", "percent2", "percent3"], 100)
   * ```
   */
  requireSumEquals(
    fields: string[],
    value: number,
    options?: ConstraintOptions,
  ): this {
    return this.rule(
      {
        type: "sum_equals_value",
        paths: fields.map((f) => `/${f}`),
        value,
      },
      options?.code ?? "SUM_MISMATCH",
      options?.message ?? `Sum of ${fields.join(", ")} must equal ${value}`,
      options,
    );
  }

  /**
   * Extend this object schema with additional properties.
   *
   * @example
   * ```ts
   * const base = w.object({ id: w.string() });
   * const extended = base.extend({ name: w.string(), age: w.integer() });
   * ```
   */
  extend<E extends ObjectShape>(shape: E): ObjectBuilder<S & E> {
    const newBuilder = new ObjectBuilder<S & E>();
    // Copy existing properties
    newBuilder._properties = { ...this._properties };
    newBuilder._additionalProperties = this._additionalProperties;
    newBuilder._unknownKeys = this._unknownKeys;
    newBuilder._catchall = this._catchall;
    newBuilder._rules = [...this._rules];
    // Add new properties
    for (const [key, value] of Object.entries(shape)) {
      if (isPropertyDef(value)) {
        newBuilder._properties[key] = makePropertySchema(
          value.builder,
          value.required !== false,
        );
      } else {
        const isOptionalBuilder = "isOptional" in value && value.isOptional();
        newBuilder._properties[key] = makePropertySchema(
          value,
          !isOptionalBuilder,
        );
      }
    }
    // Merge shapes for type inference
    (newBuilder as { _shape: S & E })._shape = {
      ...this._shape,
      ...shape,
    } as S & E;
    return newBuilder;
  }

  /**
   * Merge with another object schema.
   *
   * @example
   * ```ts
   * const a = w.object({ name: w.string() });
   * const b = w.object({ age: w.integer() });
   * const merged = a.merge(b);
   * ```
   */
  merge<T extends ObjectShape>(other: ObjectBuilder<T>): ObjectBuilder<S & T> {
    const newBuilder = new ObjectBuilder<S & T>();
    // Copy from this
    newBuilder._properties = { ...this._properties };
    newBuilder._additionalProperties =
      this._additionalProperties || other._additionalProperties;
    newBuilder._unknownKeys = other._unknownKeys ?? this._unknownKeys;
    newBuilder._catchall = other._catchall ?? this._catchall;
    newBuilder._rules = [...this._rules, ...other._rules];
    // Copy from other
    for (const [key, prop] of Object.entries(other._properties)) {
      newBuilder._properties[key] = prop;
    }
    // Merge shapes
    (newBuilder as { _shape: S & T })._shape = {
      ...this._shape,
      ...other._shape,
    } as S & T;
    return newBuilder;
  }

  /**
   * Create a new schema with only the specified keys.
   *
   * @example
   * ```ts
   * const user = w.object({ id: w.string(), name: w.string(), email: w.string() });
   * const nameOnly = user.pick("name");
   * ```
   */
  pick<K extends keyof S & string>(...keys: K[]): ObjectBuilder<Pick<S, K>> {
    const newBuilder = new ObjectBuilder<Pick<S, K>>();
    newBuilder._additionalProperties = this._additionalProperties;
    newBuilder._unknownKeys = this._unknownKeys;
    newBuilder._catchall = this._catchall;
    const keySet = new Set(keys);
    for (const [key, prop] of Object.entries(this._properties)) {
      if (keySet.has(key as K)) {
        newBuilder._properties[key] = prop;
      }
    }
    // Pick from shape
    const newShape: Partial<S> = {};
    for (const key of keys) {
      if (key in this._shape) {
        newShape[key] = this._shape[key];
      }
    }
    (newBuilder as { _shape: Pick<S, K> })._shape = newShape as Pick<S, K>;
    return newBuilder;
  }

  /**
   * Create a new schema without the specified keys.
   *
   * @example
   * ```ts
   * const user = w.object({ id: w.string(), name: w.string(), password: w.string() });
   * const safe = user.omit("password");
   * ```
   */
  omit<K extends keyof S & string>(...keys: K[]): ObjectBuilder<Omit<S, K>> {
    const newBuilder = new ObjectBuilder<Omit<S, K>>();
    const keySet = new Set(keys);
    for (const [key, prop] of Object.entries(this._properties)) {
      if (!keySet.has(key as K)) {
        newBuilder._properties[key] = prop;
      }
    }
    newBuilder._additionalProperties = this._additionalProperties;
    newBuilder._unknownKeys = this._unknownKeys;
    newBuilder._catchall = this._catchall;
    // Omit from shape
    const newShape: Partial<S> = {};
    for (const key of Object.keys(this._shape) as (keyof S)[]) {
      if (!keySet.has(key as K)) {
        newShape[key] = this._shape[key];
      }
    }
    (newBuilder as { _shape: Omit<S, K> })._shape = newShape as Omit<S, K>;
    return newBuilder;
  }

  /**
   * Make all properties optional.
   *
   * @example
   * ```ts
   * const user = w.object({ name: w.string(), age: w.integer() });
   * const partialUser = user.partial();
   * ```
   */
  partial(): ObjectBuilder<{
    [K in keyof S]: S[K] extends PropertyDef<infer B, boolean>
      ? PropertyDef<B, false>
      : S[K] extends SchemaBuilder
        ? PropertyDef<S[K], false>
        : PropertyDef<SchemaBuilder, false>;
  }> {
    const newBuilder = new ObjectBuilder<{
      [K in keyof S]: S[K] extends PropertyDef<infer B, boolean>
        ? PropertyDef<B, false>
        : S[K] extends SchemaBuilder
          ? PropertyDef<S[K], false>
          : PropertyDef<SchemaBuilder, false>;
    }>();
    for (const [key, prop] of Object.entries(this._properties)) {
      newBuilder._properties[key] = { ...prop, required: false };
    }
    newBuilder._additionalProperties = this._additionalProperties;
    newBuilder._unknownKeys = this._unknownKeys;
    newBuilder._catchall = this._catchall;
    newBuilder._rules = [...this._rules];
    // Transform shape to PropertyDefs with required: false
    const newShape: Record<string, PropertyDef> = {};
    for (const key of Object.keys(this._shape)) {
      const value = this._shape[key];
      if (value === undefined) continue;
      if (isPropertyDef(value)) {
        newShape[key] = { ...value, required: false };
      } else {
        newShape[key] = { builder: value, required: false };
      }
    }
    (
      newBuilder as {
        _shape: {
          [K in keyof S]: S[K] extends PropertyDef<infer B>
            ? PropertyDef<B, false>
            : S[K] extends SchemaBuilder
              ? PropertyDef<S[K], false>
              : PropertyDef<SchemaBuilder, false>;
        };
      }
    )._shape = newShape as {
      [K in keyof S]: S[K] extends PropertyDef<infer B>
        ? PropertyDef<B, false>
        : S[K] extends SchemaBuilder
          ? PropertyDef<S[K], false>
          : PropertyDef<SchemaBuilder, false>;
    };
    return newBuilder;
  }

  /**
   * Make all properties required.
   *
   * @example
   * ```ts
   * const partialUser = w.object({ name: optional(w.string()) });
   * const requiredUser = partialUser.required();
   * ```
   */
  required(): ObjectBuilder<{
    [K in keyof S]: S[K] extends PropertyDef ? S[K]["builder"] : S[K];
  }> {
    const newBuilder = new ObjectBuilder<{
      [K in keyof S]: S[K] extends PropertyDef ? S[K]["builder"] : S[K];
    }>();
    for (const [key, prop] of Object.entries(this._properties)) {
      // Remove the 'required' field - Rust defaults to true so we omit it
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { required: _removed, ...rest } = prop as PropertySchema & {
        required?: boolean;
      };
      newBuilder._properties[key] = rest;
    }
    newBuilder._additionalProperties = this._additionalProperties;
    newBuilder._unknownKeys = this._unknownKeys;
    newBuilder._catchall = this._catchall;
    newBuilder._rules = [...this._rules];
    // Transform shape to required
    const newShape: Record<string, SchemaBuilder> = {};
    for (const key of Object.keys(this._shape)) {
      const value = this._shape[key];
      if (value === undefined) continue;
      if (isPropertyDef(value)) {
        newShape[key] = value.builder;
      } else {
        newShape[key] = value;
      }
    }
    (
      newBuilder as {
        _shape: {
          [K in keyof S]: S[K] extends PropertyDef ? S[K]["builder"] : S[K];
        };
      }
    )._shape = newShape as {
      [K in keyof S]: S[K] extends PropertyDef ? S[K]["builder"] : S[K];
    };
    return newBuilder;
  }

  toTypeSchema(): TypeSchema & { type: "object" } {
    // Rust uses flattened object schema, not nested under "schema"
    return {
      type: "object",
      properties:
        Object.keys(this._properties).length > 0 ? this._properties : undefined,
      additional_properties: this._additionalProperties || undefined,
      unknown_keys: this._unknownKeys,
      catchall: this._catchall,
      rules: this._rules.length > 0 ? this._rules : undefined,
    };
  }
}

/**
 * Helper to create an optional property definition.
 */
export function optional<T extends SchemaBuilder>(
  builder: T,
): PropertyDef<T, false> {
  return { builder, required: false } as PropertyDef<T, false>;
}

/**
 * Helper to create a required property definition (explicit).
 */
export function required<T extends SchemaBuilder>(
  builder: T,
): PropertyDef<T, true> {
  return { builder, required: true } as PropertyDef<T, true>;
}
