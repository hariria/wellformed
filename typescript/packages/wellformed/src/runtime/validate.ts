// biome-ignore-all lint/complexity/noExcessiveCognitiveComplexity: Runtime schema validation keeps per-type validation branches near their error handling.
/**
 * Validation engine - validates values against schemas and collects errors.
 */

import type {
  Constraint,
  FormError,
  Schema,
  Transform,
  TypeSchema,
} from "../ir/types.js";
import { toPointer } from "./pointer.js";
import { createEvalContext, type EvalContext, evaluate } from "./predicate.js";
import { applyTransforms } from "./transform.js";

/**
 * Validation result containing errors and warnings.
 */
export interface ValidationResult {
  valid: boolean;
  errors: FormError[];
  warnings: FormError[];
  /** The value after transforms have been applied */
  value: unknown;
}

/**
 * Options for validation.
 */
export interface ValidateOptions {
  /** Custom predicate registry */
  context?: EvalContext;
  /** Whether to apply transforms to the value (default: true) */
  applyTransforms?: boolean;
  /** Whether to collect all errors or stop at first (default: collect all) */
  collectAll?: boolean;
}

interface RuntimeValidationContext {
  definitions?: Record<string, TypeSchema>;
  refStack: string[];
}

/**
 * Maximum `$ref` recursion depth. Bounds runaway/cyclic schemas so they error
 * cleanly instead of recursing forever, while still allowing legitimate
 * recursive schemas over finite data to validate. Mirrors the Rust runtime's
 * validation-depth bound.
 */
const MAX_REF_DEPTH = 128;

const defaultEvalContext = createEvalContext();

/**
 * Validate a value against a schema.
 */
export function validate(
  schema: Schema | TypeSchema,
  value: unknown,
  options?: ValidateOptions,
): ValidationResult {
  const typeSchema = "root" in schema ? schema.root : schema;
  const runtime: RuntimeValidationContext = {
    definitions: "root" in schema ? schema.definitions : undefined,
    refStack: [],
  };
  const ctx = options?.context ?? defaultEvalContext;
  const shouldTransform = options?.applyTransforms ?? true;

  const errors: FormError[] = [];
  const warnings: FormError[] = [];

  const transformedValue = validateTypeSchema(
    typeSchema,
    value,
    "",
    ctx,
    errors,
    warnings,
    shouldTransform,
    runtime,
  );

  return {
    valid: errors.length === 0,
    errors,
    warnings,
    value: transformedValue,
  };
}

/**
 * Validate a value and throw if invalid.
 */
export function validateOrThrow(
  schema: Schema | TypeSchema,
  value: unknown,
  options?: ValidateOptions,
): unknown {
  const result = validate(schema, value, options);
  if (!result.valid) {
    const messages = result.errors
      .map((e) => `${e.path}: ${e.message}`)
      .join("; ");
    throw new ValidationError(messages, result.errors);
  }
  return result.value;
}

/**
 * Error thrown when validation fails.
 */
export class ValidationError extends Error {
  constructor(
    message: string,
    public readonly errors: FormError[],
  ) {
    super(message);
    this.name = "ValidationError";
  }
}

// ============================================================================
// Internal validation functions
// ============================================================================

/**
 * Helper to get transforms from a flattened schema.
 */
function getTransforms(schema: TypeSchema): Transform[] | undefined {
  switch (schema.type) {
    case "string":
    case "number":
    case "integer":
    case "int32":
    case "int64":
    case "uint32":
    case "uint64":
    case "money":
    case "currency":
    case "decimal":
    case "percentage":
    case "date":
    case "preprocess":
      return schema.transforms;
    default:
      return undefined;
  }
}

/**
 * Helper to get constraints from a flattened schema.
 */
function getConstraints(schema: TypeSchema): Constraint[] | undefined {
  if ("constraints" in schema) {
    return schema.constraints;
  }
  return undefined;
}

function hasDefaultTransform(transforms: Transform[] | undefined): boolean {
  return transforms?.some((transform) => transform.fn === "default") === true;
}

function isFiniteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

function schemaFillsMissing(
  schema: TypeSchema,
  runtime: RuntimeValidationContext,
  seenRefs = new Set<string>(),
): boolean {
  if (hasDefaultTransform(getTransforms(schema))) {
    return true;
  }

  switch (schema.type) {
    case "preprocess":
      return schemaFillsMissing(schema.schema, runtime, seenRefs);
    case "catch":
      return schemaFillsMissing(schema.schema, runtime, seenRefs);
    case "ref": {
      if (seenRefs.has(schema.$ref)) return false;
      const definition = runtime.definitions?.[schema.$ref];
      if (!definition) return false;
      seenRefs.add(schema.$ref);
      const fills = schemaFillsMissing(definition, runtime, seenRefs);
      seenRefs.delete(schema.$ref);
      return fills;
    }
    default:
      return false;
  }
}

function schemaAllowsNull(
  schema: TypeSchema,
  runtime: RuntimeValidationContext,
  seenRefs = new Set<string>(),
): boolean {
  if (hasDefaultTransform(getTransforms(schema))) {
    return true;
  }

  switch (schema.type) {
    case "literal":
      return isEqualValue(schema.value, null);
    case "enum":
      return schema.values.some((value) => isEqualValue(value, null));
    case "union":
      return schema.oneOf.some((variant) =>
        schemaAllowsNull(variant, runtime, seenRefs),
      );
    case "preprocess":
      return schemaAllowsNull(schema.schema, runtime, seenRefs);
    case "catch":
      return schemaAllowsNull(schema.schema, runtime, seenRefs);
    case "ref": {
      if (seenRefs.has(schema.$ref)) return false;
      const definition = runtime.definitions?.[schema.$ref];
      if (!definition) return false;
      seenRefs.add(schema.$ref);
      const allows = schemaAllowsNull(definition, runtime, seenRefs);
      seenRefs.delete(schema.$ref);
      return allows;
    }
    case "any":
      return true;
    default:
      return false;
  }
}

function validateTypeSchema(
  schema: TypeSchema,
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
  runtime: RuntimeValidationContext,
): unknown {
  switch (schema.type) {
    case "string":
      return validateString(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
      );

    case "number":
      return validateNumber(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
      );

    case "integer":
      return validateInteger(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
      );

    case "int32":
      return validateInt32(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
      );

    case "int64":
      return validateInt64(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
      );

    case "uint32":
      return validateUint32(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
      );

    case "uint64":
      return validateUint64(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
      );

    case "boolean":
      return validateBoolean(value, path, errors);

    case "money":
      return validateMoney(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
      );

    case "currency":
      return validateCurrency(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
      );

    case "decimal":
      return validateDecimal(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
      );

    case "percentage":
      return validatePercentage(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
      );

    case "date":
      return validateDate(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
      );

    case "object":
      return validateObject(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
        runtime,
      );

    case "array":
      return validateArray(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
        runtime,
      );

    case "tuple":
      return validateTuple(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
        runtime,
      );

    case "enum":
      return validateEnum(schema.values, value, path, errors);

    case "literal":
      return validateLiteral(schema.value, value, path, errors);

    case "never":
      return validateNever(value, path, errors);

    case "union":
      return validateUnion(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
        runtime,
      );

    case "intersection":
      return validateIntersection(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
        runtime,
      );

    case "record":
      return validateRecord(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
        runtime,
      );

    case "preprocess":
      return validatePreprocess(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
        runtime,
      );

    case "catch":
      return validateCatch(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
        runtime,
      );

    case "any":
      return value;

    case "ref":
      return validateRef(
        schema,
        value,
        path,
        ctx,
        errors,
        warnings,
        shouldTransform,
        runtime,
      );

    default:
      return value;
  }
}

function validateString(
  schema: TypeSchema & { type: "string" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
): unknown {
  // Apply transforms
  let transformed = value;
  const transforms = getTransforms(schema);
  if (shouldTransform && transforms) {
    transformed = applyTransforms(value, transforms);
  }

  // Type check (after transforms, allow null/undefined to pass through for optional handling)
  if (
    transformed !== null &&
    transformed !== undefined &&
    typeof transformed !== "string"
  ) {
    addError(
      errors,
      path,
      "TYPE_ERROR",
      `Expected string, got ${typeof transformed}`,
    );
    return transformed;
  }

  // Evaluate constraints
  const constraints = getConstraints(schema);
  if (constraints && typeof transformed === "string") {
    evaluateConstraints(constraints, transformed, path, ctx, errors, warnings);
  }

  return transformed;
}

function validateNumber(
  schema: TypeSchema & { type: "number" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
): unknown {
  let transformed = value;
  const transforms = getTransforms(schema);
  if (shouldTransform && transforms) {
    transformed = applyTransforms(value, transforms);
  }

  if (transformed !== null && transformed !== undefined) {
    if (!isFiniteNumber(transformed)) {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `Expected number, got ${typeof transformed}`,
      );
      return transformed;
    }
  }

  const constraints = getConstraints(schema);
  if (constraints && isFiniteNumber(transformed)) {
    evaluateConstraints(constraints, transformed, path, ctx, errors, warnings);
  }

  return transformed;
}

function validateInteger(
  schema: TypeSchema & { type: "integer" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
): unknown {
  let transformed = value;
  const transforms = getTransforms(schema);
  if (shouldTransform && transforms) {
    transformed = applyTransforms(value, transforms);
  }

  if (transformed !== null && transformed !== undefined) {
    if (typeof transformed !== "number") {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `Expected integer, got ${typeof transformed}`,
      );
      return transformed;
    }
    if (!Number.isInteger(transformed)) {
      addError(errors, path, "TYPE_ERROR", "Expected integer, got float");
      return transformed;
    }
  }

  const constraints = getConstraints(schema);
  if (
    constraints &&
    typeof transformed === "number" &&
    Number.isInteger(transformed)
  ) {
    evaluateConstraints(constraints, transformed, path, ctx, errors, warnings);
  }

  return transformed;
}

// Constants for integer validation
const INT32_MIN = -2147483648;
const INT32_MAX = 2147483647;
const UINT32_MAX = 4294967295;
const MAX_SAFE_INTEGER = Number.MAX_SAFE_INTEGER;

function validateInt32(
  schema: TypeSchema & { type: "int32" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
): unknown {
  let transformed = value;
  const transforms = getTransforms(schema);
  if (shouldTransform && transforms) {
    transformed = applyTransforms(value, transforms);
  }

  if (transformed !== null && transformed !== undefined) {
    if (typeof transformed !== "number") {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `Expected int32, got ${typeof transformed}`,
      );
      return transformed;
    }
    if (!Number.isInteger(transformed)) {
      addError(errors, path, "TYPE_ERROR", "Expected int32, got float");
      return transformed;
    }
    if (transformed < INT32_MIN || transformed > INT32_MAX) {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `Expected int32 (${INT32_MIN} to ${INT32_MAX}), got ${transformed}`,
      );
      return transformed;
    }
  }

  const constraints = getConstraints(schema);
  if (
    constraints &&
    typeof transformed === "number" &&
    Number.isInteger(transformed) &&
    transformed >= INT32_MIN &&
    transformed <= INT32_MAX
  ) {
    evaluateConstraints(constraints, transformed, path, ctx, errors, warnings);
  }

  return transformed;
}

function validateInt64(
  schema: TypeSchema & { type: "int64" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
): unknown {
  let transformed = value;
  const transforms = getTransforms(schema);
  if (shouldTransform && transforms) {
    transformed = applyTransforms(value, transforms);
  }

  if (transformed !== null && transformed !== undefined) {
    if (typeof transformed !== "number") {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `Expected int64, got ${typeof transformed}`,
      );
      return transformed;
    }
    if (!Number.isInteger(transformed)) {
      addError(errors, path, "TYPE_ERROR", "Expected int64, got float");
      return transformed;
    }
    // Note: JavaScript can only safely represent integers up to 2^53-1
    if (Math.abs(transformed) > MAX_SAFE_INTEGER) {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `int64 value exceeds JavaScript safe integer range`,
      );
      return transformed;
    }
  }

  const constraints = getConstraints(schema);
  if (
    constraints &&
    typeof transformed === "number" &&
    Number.isInteger(transformed) &&
    Math.abs(transformed) <= MAX_SAFE_INTEGER
  ) {
    evaluateConstraints(constraints, transformed, path, ctx, errors, warnings);
  }

  return transformed;
}

function validateUint32(
  schema: TypeSchema & { type: "uint32" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
): unknown {
  let transformed = value;
  const transforms = getTransforms(schema);
  if (shouldTransform && transforms) {
    transformed = applyTransforms(value, transforms);
  }

  if (transformed !== null && transformed !== undefined) {
    if (typeof transformed !== "number") {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `Expected uint32, got ${typeof transformed}`,
      );
      return transformed;
    }
    if (!Number.isInteger(transformed)) {
      addError(errors, path, "TYPE_ERROR", "Expected uint32, got float");
      return transformed;
    }
    if (transformed < 0 || transformed > UINT32_MAX) {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `Expected uint32 (0 to ${UINT32_MAX}), got ${transformed}`,
      );
      return transformed;
    }
  }

  const constraints = getConstraints(schema);
  if (
    constraints &&
    typeof transformed === "number" &&
    Number.isInteger(transformed) &&
    transformed >= 0 &&
    transformed <= UINT32_MAX
  ) {
    evaluateConstraints(constraints, transformed, path, ctx, errors, warnings);
  }

  return transformed;
}

function validateUint64(
  schema: TypeSchema & { type: "uint64" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
): unknown {
  let transformed = value;
  const transforms = getTransforms(schema);
  if (shouldTransform && transforms) {
    transformed = applyTransforms(value, transforms);
  }

  if (transformed !== null && transformed !== undefined) {
    if (typeof transformed !== "number") {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `Expected uint64, got ${typeof transformed}`,
      );
      return transformed;
    }
    if (!Number.isInteger(transformed)) {
      addError(errors, path, "TYPE_ERROR", "Expected uint64, got float");
      return transformed;
    }
    if (transformed < 0) {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `Expected uint64 (non-negative integer), got ${transformed}`,
      );
      return transformed;
    }
    // Note: JavaScript can only safely represent integers up to 2^53-1
    if (transformed > MAX_SAFE_INTEGER) {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `uint64 value exceeds JavaScript safe integer range`,
      );
      return transformed;
    }
  }

  const constraints = getConstraints(schema);
  if (
    constraints &&
    typeof transformed === "number" &&
    Number.isInteger(transformed) &&
    transformed >= 0 &&
    transformed <= MAX_SAFE_INTEGER
  ) {
    evaluateConstraints(constraints, transformed, path, ctx, errors, warnings);
  }

  return transformed;
}

function validateBoolean(
  value: unknown,
  path: string,
  errors: FormError[],
): unknown {
  if (value !== null && value !== undefined && typeof value !== "boolean") {
    addError(
      errors,
      path,
      "TYPE_ERROR",
      `Expected boolean, got ${typeof value}`,
    );
  }
  return value;
}

function validateMoney(
  schema: TypeSchema & { type: "money" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
): unknown {
  let transformed = value;
  const transforms = getTransforms(schema);
  if (shouldTransform && transforms) {
    transformed = applyTransforms(value, transforms);
  }

  if (transformed !== null && transformed !== undefined) {
    if (!isFiniteNumber(transformed)) {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `Expected money (number), got ${typeof transformed}`,
      );
      return transformed;
    }
  }

  const constraints = getConstraints(schema);
  if (constraints && isFiniteNumber(transformed)) {
    evaluateConstraints(constraints, transformed, path, ctx, errors, warnings);
  }

  return transformed;
}

function validateCurrency(
  schema: TypeSchema & { type: "currency" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
): unknown {
  let transformed = value;
  const transforms = getTransforms(schema);
  if (shouldTransform && transforms) {
    transformed = applyTransforms(value, transforms);
  }

  if (transformed !== null && transformed !== undefined) {
    if (!isFiniteNumber(transformed)) {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `Expected currency (number), got ${typeof transformed}`,
      );
      return transformed;
    }

    // Validate scale (decimal places)
    const scale = schema.scale ?? 2; // Default to 2 decimal places
    const scaled = transformed * 10 ** scale;
    if (Math.abs(scaled - Math.round(scaled)) > 1e-10) {
      addError(
        errors,
        path,
        "CURRENCY_SCALE_EXCEEDED",
        `Currency value has more than ${scale} decimal places`,
      );
    }
  }

  const constraints = getConstraints(schema);
  if (constraints && isFiniteNumber(transformed)) {
    evaluateConstraints(constraints, transformed, path, ctx, errors, warnings);
  }

  return transformed;
}

function validateDecimal(
  schema: TypeSchema & { type: "decimal" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
): unknown {
  let transformed = value;
  const transforms = getTransforms(schema);
  if (shouldTransform && transforms) {
    transformed = applyTransforms(value, transforms);
  }

  if (transformed !== null && transformed !== undefined) {
    if (!isFiniteNumber(transformed)) {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `Expected decimal (number), got ${typeof transformed}`,
      );
      return transformed;
    }

    // Validate scale (decimal places)
    if (schema.scale !== undefined) {
      const scaled = transformed * 10 ** schema.scale;
      if (Math.abs(scaled - Math.round(scaled)) > 1e-10) {
        addError(
          errors,
          path,
          "DECIMAL_SCALE_EXCEEDED",
          `Value has more than ${schema.scale} decimal places`,
        );
      }
    }

    // Validate precision (total digits)
    if (schema.precision !== undefined) {
      const absN = Math.abs(transformed);
      const s = absN.toFixed(10).replace(/\.?0+$/, "");
      const digitCount = s.replace(".", "").length;
      if (digitCount > schema.precision) {
        addError(
          errors,
          path,
          "DECIMAL_PRECISION_EXCEEDED",
          `Value exceeds ${schema.precision} total digits`,
        );
      }
    }
  }

  const constraints = getConstraints(schema);
  if (constraints && isFiniteNumber(transformed)) {
    evaluateConstraints(constraints, transformed, path, ctx, errors, warnings);
  }

  return transformed;
}

function validatePercentage(
  schema: TypeSchema & { type: "percentage" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
): unknown {
  let transformed = value;
  const transforms = getTransforms(schema);
  if (shouldTransform && transforms) {
    transformed = applyTransforms(value, transforms);
  }

  if (transformed !== null && transformed !== undefined) {
    if (!isFiniteNumber(transformed)) {
      addError(
        errors,
        path,
        "TYPE_ERROR",
        `Expected percentage (number), got ${typeof transformed}`,
      );
      return transformed;
    }

    // Validate range based on format
    const format = schema.format ?? "decimal";
    const allowOver100 = schema.allow_over_100 ?? false;
    const max = format === "decimal" ? 1.0 : 100.0;

    if (transformed < 0) {
      addError(
        errors,
        path,
        "PERCENTAGE_NEGATIVE",
        "Percentage cannot be negative",
      );
    } else if (transformed > max && !allowOver100) {
      const maxDisplay = format === "decimal" ? "1.0 (100%)" : "100";
      addError(
        errors,
        path,
        "PERCENTAGE_TOO_HIGH",
        `Percentage cannot exceed ${maxDisplay}`,
      );
    }

    // Validate scale (decimal places)
    if (schema.scale !== undefined) {
      const scaled = transformed * 10 ** schema.scale;
      if (Math.abs(scaled - Math.round(scaled)) > 1e-10) {
        addError(
          errors,
          path,
          "PERCENTAGE_SCALE_EXCEEDED",
          `Percentage has more than ${schema.scale} decimal places`,
        );
      }
    }
  }

  const constraints = getConstraints(schema);
  if (constraints && isFiniteNumber(transformed)) {
    evaluateConstraints(constraints, transformed, path, ctx, errors, warnings);
  }

  return transformed;
}

function validateDate(
  schema: TypeSchema & { type: "date" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
): unknown {
  let transformed = value;
  const transforms = getTransforms(schema);
  if (shouldTransform && transforms) {
    transformed = applyTransforms(value, transforms);
  }

  if (
    transformed !== null &&
    transformed !== undefined &&
    typeof transformed !== "string"
  ) {
    addError(
      errors,
      path,
      "TYPE_ERROR",
      `Expected date string, got ${typeof transformed}`,
    );
    return transformed;
  }

  const constraints = getConstraints(schema);
  if (constraints && typeof transformed === "string") {
    evaluateConstraints(constraints, transformed, path, ctx, errors, warnings);
  }

  return transformed;
}

function validateObject(
  schema: TypeSchema & { type: "object" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
  runtime: RuntimeValidationContext,
): unknown {
  if (value === null || value === undefined) {
    return value;
  }

  if (typeof value !== "object" || Array.isArray(value)) {
    addError(
      errors,
      path,
      "TYPE_ERROR",
      `Expected object, got ${Array.isArray(value) ? "array" : typeof value}`,
    );
    return value;
  }

  const obj = value as Record<string, unknown>;
  const result: Record<string, unknown> = {};
  const properties = schema.properties ?? schema.fields ?? {};
  const unknownBehavior =
    schema.unknown_keys ??
    (schema.additional_properties ? "passthrough" : "strict");
  const catchall = schema.catchall;

  // Validate each defined property
  for (const [propName, propSchema] of Object.entries(properties)) {
    const propPath = joinPathSegment(path, propName);
    const propValue = obj[propName];

    // PropertySchema is a flattened TypeSchema with optional 'required' field
    // Check if required (defaults to true if not specified)
    const isRequired =
      !("required" in propSchema) || propSchema.required !== false;

    if (propValue === undefined) {
      if (schemaFillsMissing(propSchema as TypeSchema, runtime)) {
        result[propName] = validateTypeSchema(
          propSchema as TypeSchema,
          undefined,
          propPath,
          ctx,
          errors,
          warnings,
          shouldTransform,
          runtime,
        );
        continue;
      }

      if (isRequired) {
        addError(errors, propPath, "REQUIRED", `${propName} is required`);
      }
      continue;
    }

    if (
      isRequired &&
      propValue === null &&
      !schemaAllowsNull(propSchema as TypeSchema, runtime)
    ) {
      result[propName] = propValue;
      addError(errors, propPath, "REQUIRED", `${propName} is required`);
      continue;
    }

    // Validate property value - propSchema IS a TypeSchema (flattened)
    result[propName] = validateTypeSchema(
      propSchema as TypeSchema,
      propValue,
      propPath,
      ctx,
      errors,
      warnings,
      shouldTransform,
      runtime,
    );
  }

  // Handle additional properties based on unknown-key behavior/catchall.
  for (const key of Object.keys(obj)) {
    if (!(key in properties)) {
      const propPath = joinPathSegment(path, key);

      if (catchall) {
        result[key] = validateTypeSchema(
          catchall,
          obj[key],
          propPath,
          ctx,
          errors,
          warnings,
          shouldTransform,
          runtime,
        );
        continue;
      }

      if (unknownBehavior === "passthrough") {
        result[key] = obj[key];
        continue;
      }

      if (unknownBehavior === "strict") {
        result[key] = obj[key];
        addError(
          errors,
          propPath,
          "ADDITIONAL_PROPERTY_NOT_ALLOWED",
          `Additional property '${key}' is not allowed`,
        );
      }
    }
  }

  // Evaluate cross-field rules
  if (schema.rules) {
    evaluateConstraints(schema.rules, result, path, ctx, errors, warnings);
  }

  return result;
}

function validateArray(
  schema: TypeSchema & { type: "array" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
  runtime: RuntimeValidationContext,
): unknown {
  if (value === null || value === undefined) {
    return value;
  }

  if (!Array.isArray(value)) {
    addError(errors, path, "TYPE_ERROR", `Expected array, got ${typeof value}`);
    return value;
  }

  if (
    schema.min_items !== undefined &&
    value.length < schema.min_items &&
    !hasLengthConstraint(schema.constraints, "min_len", schema.min_items)
  ) {
    addError(
      errors,
      path,
      "ARRAY_TOO_SHORT",
      `Array must have at least ${schema.min_items} items`,
    );
  }

  if (
    schema.max_items !== undefined &&
    value.length > schema.max_items &&
    !hasLengthConstraint(schema.constraints, "max_len", schema.max_items)
  ) {
    addError(
      errors,
      path,
      "ARRAY_TOO_LONG",
      `Array must have at most ${schema.max_items} items`,
    );
  }

  // Validate each item
  const result: unknown[] = [];
  for (let i = 0; i < value.length; i++) {
    const itemPath = joinPathSegment(path, String(i));
    result.push(
      validateTypeSchema(
        schema.items,
        value[i],
        itemPath,
        ctx,
        errors,
        warnings,
        shouldTransform,
        runtime,
      ),
    );
  }

  // Rust evaluates array constraints after item validation, so predicates see
  // item transforms that were applied during validation.
  const constraints = getConstraints(schema);
  if (constraints) {
    evaluateConstraints(constraints, result, path, ctx, errors, warnings);
  }

  return result;
}

function hasLengthConstraint(
  constraints: Constraint[] | undefined,
  type: "min_len" | "max_len",
  len: number,
): boolean {
  return (
    constraints?.some((constraint) => {
      const pred = constraint.pred;
      return pred.type === type && pred.len === len;
    }) === true
  );
}

function validateTuple(
  schema: TypeSchema & { type: "tuple" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
  runtime: RuntimeValidationContext,
): unknown {
  if (value === null || value === undefined) {
    return value;
  }

  if (!Array.isArray(value)) {
    addError(
      errors,
      path,
      "TYPE_ERROR",
      `Expected tuple (array), got ${typeof value}`,
    );
    return value;
  }

  if (value.length !== schema.items.length) {
    addError(
      errors,
      path,
      "INVALID_TUPLE",
      `Tuple must have exactly ${schema.items.length} items, got ${value.length}`,
    );
  }

  const result: unknown[] = [];
  const count = Math.min(value.length, schema.items.length);
  for (let i = 0; i < count; i++) {
    const itemPath = joinPathSegment(path, String(i));
    result.push(
      validateTypeSchema(
        schema.items[i] as TypeSchema,
        value[i],
        itemPath,
        ctx,
        errors,
        warnings,
        shouldTransform,
        runtime,
      ),
    );
  }

  // Preserve extra items as-is when input is longer than the tuple definition.
  for (let i = count; i < value.length; i++) {
    result.push(value[i]);
  }

  return result;
}

function validateEnum(
  values: unknown[],
  value: unknown,
  path: string,
  errors: FormError[],
): unknown {
  if (value === null || value === undefined) {
    return value;
  }

  // Rust skips enum validation for empty strings so blank optional form fields
  // are not rejected before higher-level requiredness decides whether they
  // matter.
  if (value === "") {
    return value;
  }

  // Enum values are arbitrary JSON values in the IR, so arrays and objects must
  // compare structurally rather than by reference.
  const found = values.some((v) => isEqualValue(v, value));

  if (!found) {
    const valuesStr = values.map((v) => String(v)).join(", ");
    addError(errors, path, "INVALID_ENUM", `Must be one of: ${valuesStr}`);
  }

  return value;
}

function validateLiteral(
  expected: unknown,
  value: unknown,
  path: string,
  errors: FormError[],
): unknown {
  if (!isEqualValue(value, expected)) {
    addError(
      errors,
      path,
      "INVALID_LITERAL",
      `Expected literal ${JSON.stringify(expected)}, got ${JSON.stringify(value)}`,
    );
  }
  return value;
}

function validateNever(
  value: unknown,
  path: string,
  errors: FormError[],
): unknown {
  addError(errors, path, "TYPE_ERROR", "Expected never");
  return value;
}

function validateUnion(
  schema: TypeSchema & { type: "union" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
  runtime: RuntimeValidationContext,
): unknown {
  if (value === null) {
    return value;
  }

  // Try each variant until one succeeds
  for (const variant of schema.oneOf) {
    const variantErrors: FormError[] = [];
    const variantWarnings: FormError[] = [];
    const result = validateTypeSchema(
      variant,
      value,
      path,
      ctx,
      variantErrors,
      variantWarnings,
      shouldTransform,
      runtime,
    );

    if (variantErrors.length === 0) {
      warnings.push(...variantWarnings);
      return result;
    }
  }

  // No variant matched
  addError(
    errors,
    path,
    "INVALID_UNION",
    "Value does not match any union variant",
  );
  return value;
}

function validateIntersection(
  schema: TypeSchema & { type: "intersection" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
  runtime: RuntimeValidationContext,
): unknown {
  let current = value;

  for (const variant of schema.allOf) {
    const variantErrors: FormError[] = [];
    const variantWarnings: FormError[] = [];
    const next = validateTypeSchema(
      variant,
      current,
      path,
      ctx,
      variantErrors,
      variantWarnings,
      shouldTransform,
      runtime,
    );

    warnings.push(...variantWarnings);
    if (variantErrors.length > 0) {
      errors.push(...variantErrors);
      return current;
    }

    current = next;
  }

  return current;
}

function validateRecord(
  schema: TypeSchema & { type: "record" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
  runtime: RuntimeValidationContext,
): unknown {
  if (value === null || value === undefined) {
    return value;
  }

  if (typeof value !== "object" || Array.isArray(value)) {
    addError(
      errors,
      path,
      "TYPE_ERROR",
      `Expected record (object), got ${Array.isArray(value) ? "array" : typeof value}`,
    );
    return value;
  }

  const obj = value as Record<string, unknown>;
  const out: Record<string, unknown> = {};

  for (const [key, itemValue] of Object.entries(obj)) {
    const itemPath = joinPathSegment(path, key);

    if (schema.key) {
      const keyPath = joinPathSegment(itemPath, "$key");
      validateTypeSchema(
        schema.key,
        key,
        keyPath,
        ctx,
        errors,
        warnings,
        shouldTransform,
        runtime,
      );
    }

    out[key] = validateTypeSchema(
      schema.value,
      itemValue,
      itemPath,
      ctx,
      errors,
      warnings,
      shouldTransform,
      runtime,
    );
  }

  return out;
}

function validatePreprocess(
  schema: TypeSchema & { type: "preprocess" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
  runtime: RuntimeValidationContext,
): unknown {
  const transformed =
    shouldTransform && schema.transforms
      ? applyTransforms(value, schema.transforms)
      : value;

  return validateTypeSchema(
    schema.schema,
    transformed,
    path,
    ctx,
    errors,
    warnings,
    shouldTransform,
    runtime,
  );
}

function validateCatch(
  schema: TypeSchema & { type: "catch" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  _errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
  runtime: RuntimeValidationContext,
): unknown {
  const innerErrors: FormError[] = [];
  const innerWarnings: FormError[] = [];
  const parsed = validateTypeSchema(
    schema.schema,
    value,
    path,
    ctx,
    innerErrors,
    innerWarnings,
    shouldTransform,
    runtime,
  );

  if (innerErrors.length === 0) {
    warnings.push(...innerWarnings);
    return parsed;
  }

  return schema.value;
}

function validateRef(
  schema: TypeSchema & { type: "ref" },
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
  shouldTransform: boolean,
  runtime: RuntimeValidationContext,
): unknown {
  const definition = runtime.definitions?.[schema.$ref];
  if (!definition) {
    addError(
      errors,
      path,
      "REF_NOT_FOUND",
      `Schema reference not found: ${schema.$ref}`,
    );
    return value;
  }

  // Bound recursion by depth rather than name membership. A recursive schema
  // over finite data (e.g. a linked list or tree) re-enters the same $ref as it
  // descends, which is legitimate and must validate; only unbounded recursion (a
  // true cycle that consumes no data) is an error. This mirrors the Rust runtime,
  // which bounds total validation depth. See conformance: recursive-ref-schema.
  if (runtime.refStack.length >= MAX_REF_DEPTH) {
    addError(
      errors,
      path,
      "REF_CYCLE",
      `Schema reference recursion limit (${MAX_REF_DEPTH}) exceeded (possible cycle): ${[
        ...runtime.refStack,
        schema.$ref,
      ].join(" -> ")}`,
    );
    return value;
  }

  runtime.refStack.push(schema.$ref);
  try {
    return validateTypeSchema(
      definition,
      value,
      path,
      ctx,
      errors,
      warnings,
      shouldTransform,
      runtime,
    );
  } finally {
    runtime.refStack.pop();
  }
}

// ============================================================================
// Helpers
// ============================================================================

function evaluateConstraints(
  constraints: Constraint[],
  value: unknown,
  path: string,
  ctx: EvalContext,
  errors: FormError[],
  warnings: FormError[],
): void {
  for (const constraint of constraints) {
    const result = evaluate(constraint.pred, value, ctx);
    if (!result) {
      const error: FormError = {
        code: constraint.error.code,
        message: constraint.error.message,
        path: constraint.error.path ?? path,
        severity: constraint.error.severity ?? "error",
        help: constraint.error.help,
        source: constraint.error.source,
      };

      if (error.severity === "warning") {
        warnings.push(error);
      } else {
        errors.push(error);
      }
    }
  }
}

function addError(
  errors: FormError[],
  path: string,
  code: string,
  message: string,
): void {
  errors.push({
    code,
    message,
    path,
    severity: "error",
  });
}

function joinPathSegment(base: string, segment: string): string {
  const child = toPointer([segment]);
  return base === "" ? child : `${base}${child}`;
}

function isEqualValue(a: unknown, b: unknown): boolean {
  if (a === b) {
    return true;
  }

  if (a === null || b === null) {
    return a === b;
  }

  if (typeof a !== "object" || typeof b !== "object") {
    return false;
  }

  if (Array.isArray(a) !== Array.isArray(b)) {
    return false;
  }

  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) {
      return false;
    }
    for (let i = 0; i < a.length; i++) {
      if (!isEqualValue(a[i], b[i])) {
        return false;
      }
    }
    return true;
  }

  const aKeys = Object.keys(a as Record<string, unknown>);
  const bKeys = Object.keys(b as Record<string, unknown>);
  if (aKeys.length !== bKeys.length) {
    return false;
  }

  for (const key of aKeys) {
    if (!(key in (b as Record<string, unknown>))) {
      return false;
    }
    if (
      !isEqualValue(
        (a as Record<string, unknown>)[key],
        (b as Record<string, unknown>)[key],
      )
    ) {
      return false;
    }
  }

  return true;
}
