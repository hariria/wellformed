/**
 * Base types and interfaces for the builder DSL.
 */

import type {
  Constraint,
  ErrorMeta,
  Predicate,
  Schema,
  Transform,
  TypeSchema,
} from "../ir/types.js";

/**
 * Brand for optional builders (used for type inference).
 */
export interface OptionalBrand {
  readonly _optional: true;
}

/**
 * Base interface for all schema builders.
 */
export interface SchemaBuilder<T extends TypeSchema = TypeSchema> {
  /**
   * Convert to IR TypeSchema.
   */
  toTypeSchema(): T;

  /**
   * Convert to a complete Schema with version.
   */
  toSchema(version?: string): Schema;

  /**
   * Serialize to JSON string.
   */
  toJSON(version?: string, pretty?: boolean): string;

  /**
   * Mark this field as optional when used in an object schema.
   */
  optional(): this & OptionalBrand;

  /**
   * Allow `null` in addition to the current schema.
   */
  nullable(): NullableBuilder<this>;

  /**
   * Allow `null` and mark optional in object contexts.
   */
  nullish(): NullableBuilder<this> & OptionalBrand;

  /**
   * Apply transforms before validating this schema.
   */
  preprocess(transforms: Transform | Transform[]): PreprocessBuilder<this>;

  /**
   * Return fallback value when validation fails.
   */
  catch(value: unknown): CatchBuilder<this>;

  /**
   * Check if this builder is marked as optional.
   * @internal
   */
  isOptional(): boolean;
}

/**
 * Options for creating a constraint.
 */
export interface ConstraintOptions {
  id?: string;
  message?: string;
  code?: string;
  help?: string;
  source?: string;
}

/**
 * Helper to create a constraint from a predicate.
 */
export function makeConstraint(
  pred: Predicate,
  defaultCode: string,
  defaultMessage: string,
  options?: ConstraintOptions,
): Constraint {
  const error: ErrorMeta = {
    code: options?.code ?? defaultCode,
    message: options?.message ?? defaultMessage,
    severity: "error",
  };

  // Only add optional properties if they have values
  if (options?.help) error.help = options.help;
  if (options?.source) error.source = options.source;

  const constraint: Constraint = {
    pred,
    error,
  };

  // Only add id if it has a value
  if (options?.id) constraint.id = options.id;

  return constraint;
}

/**
 * Base class for schema builders with common functionality.
 */
export abstract class BaseBuilder<T extends TypeSchema>
  implements SchemaBuilder<T>
{
  /** @internal */
  protected _isOptional = false;

  abstract toTypeSchema(): T;

  toSchema(version = "1.0"): Schema {
    return {
      version,
      root: this.toTypeSchema(),
    };
  }

  toJSON(version = "1.0", pretty = false): string {
    return JSON.stringify(this.toSchema(version), null, pretty ? 2 : undefined);
  }

  /**
   * Mark this field as optional when used in an object schema.
   */
  optional(): this & OptionalBrand {
    this._isOptional = true;
    return this as this & OptionalBrand;
  }

  /**
   * Allow `null` in addition to this schema.
   */
  nullable(): NullableBuilder<this> {
    return new NullableBuilder(this);
  }

  /**
   * Allow `null` and mark optional in object contexts.
   */
  nullish(): NullableBuilder<this> & OptionalBrand {
    return this.nullable().optional();
  }

  /**
   * Apply transforms before validating this schema.
   */
  preprocess(transforms: Transform | Transform[]): PreprocessBuilder<this> {
    const transformList = Array.isArray(transforms) ? transforms : [transforms];
    return new PreprocessBuilder(this, transformList);
  }

  /**
   * Return fallback value when validation fails.
   */
  catch(value: unknown): CatchBuilder<this> {
    return new CatchBuilder(this, value);
  }

  /**
   * Check if this builder is marked as optional.
   * @internal
   */
  isOptional(): boolean {
    return this._isOptional;
  }
}

/**
 * Wrapper builder for nullable schemas.
 */
export class NullableBuilder<
  T extends SchemaBuilder = SchemaBuilder,
> extends BaseBuilder<TypeSchema & { type: "union" }> {
  /** @internal Exposed for type inference */
  readonly _inner: T;

  constructor(inner: T) {
    super();
    this._inner = inner;
  }

  toTypeSchema(): TypeSchema & { type: "union" } {
    return {
      type: "union",
      oneOf: [this._inner.toTypeSchema(), { type: "literal", value: null }],
    };
  }
}

/**
 * Wrapper builder for preprocess schemas.
 */
export class PreprocessBuilder<
  T extends SchemaBuilder = SchemaBuilder,
> extends BaseBuilder<TypeSchema & { type: "preprocess" }> {
  /** @internal Exposed for type inference */
  readonly _inner: T;
  readonly _transforms: Transform[];

  constructor(inner: T, transforms: Transform[]) {
    super();
    this._inner = inner;
    this._transforms = transforms;
  }

  toTypeSchema(): TypeSchema & { type: "preprocess" } {
    return {
      type: "preprocess",
      schema: this._inner.toTypeSchema(),
      transforms: this._transforms.length > 0 ? this._transforms : undefined,
    };
  }
}

/**
 * Wrapper builder for catch schemas.
 */
export class CatchBuilder<
  T extends SchemaBuilder = SchemaBuilder,
> extends BaseBuilder<TypeSchema & { type: "catch" }> {
  /** @internal Exposed for type inference */
  readonly _inner: T;
  readonly _value: unknown;

  constructor(inner: T, value: unknown) {
    super();
    this._inner = inner;
    this._value = value;
  }

  toTypeSchema(): TypeSchema & { type: "catch" } {
    return {
      type: "catch",
      schema: this._inner.toTypeSchema(),
      value: this._value,
    };
  }
}

/**
 * Base class for builders that support transforms and constraints.
 */
export abstract class TransformableBuilder<
  T extends TypeSchema,
> extends BaseBuilder<T> {
  protected _transforms: Transform[] = [];
  protected _constraints: Constraint[] = [];

  protected addTransform(transform: Transform): this {
    this._transforms = [...this._transforms, transform];
    return this;
  }

  protected addConstraint(constraint: Constraint): this {
    this._constraints = [...this._constraints, constraint];
    return this;
  }
}
