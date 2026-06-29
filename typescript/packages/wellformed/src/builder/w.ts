/**
 * Main entry point for the builder DSL.
 *
 * @example
 * ```ts
 * import { w } from "wellformed-ts";
 *
 * const schema = w.object({
 *   name: w.string().trim().minLen(1),
 *   age: w.integer().min(0).max(150),
 *   email: w.string().email(),
 * });
 * ```
 */

import type { Transform } from "../ir/types.js";
import { ArrayBuilder } from "./array.js";
import { EnumBuilder } from "./enum.js";
import { IntersectionBuilder } from "./intersection.js";
import {
  LiteralBuilder,
  NeverBuilder,
  type PrimitiveLiteral,
} from "./literal.js";
import {
  BooleanBuilder,
  CurrencyBuilder,
  DateBuilder,
  DecimalBuilder,
  Int32Builder,
  Int64Builder,
  IntegerBuilder,
  MoneyBuilder,
  NumberBuilder,
  PercentageBuilder,
  Uint32Builder,
  Uint64Builder,
} from "./number.js";
import { ObjectBuilder, type ObjectShape } from "./object.js";
import { RecordBuilder } from "./record.js";
import { StringBuilder } from "./string.js";
import { TupleBuilder } from "./tuple.js";
import {
  type CatchBuilder,
  NullableBuilder,
  type OptionalBrand,
  type PreprocessBuilder,
  type SchemaBuilder,
} from "./types.js";
import { UnionBuilder } from "./union.js";

/**
 * Create a string schema builder.
 */
function string(): StringBuilder {
  return new StringBuilder();
}

/**
 * Create a number schema builder.
 */
function number(): NumberBuilder {
  return new NumberBuilder();
}

/**
 * Create an integer schema builder.
 */
function integer(): IntegerBuilder {
  return new IntegerBuilder();
}

/**
 * Create a signed 32-bit integer schema builder.
 */
function int32(): Int32Builder {
  return new Int32Builder();
}

/**
 * Create a signed 64-bit integer schema builder.
 */
function int64(): Int64Builder {
  return new Int64Builder();
}

/**
 * Create an unsigned 32-bit integer schema builder.
 */
function uint32(): Uint32Builder {
  return new Uint32Builder();
}

/**
 * Create an unsigned 64-bit integer schema builder.
 */
function uint64(): Uint64Builder {
  return new Uint64Builder();
}

/**
 * Create a boolean schema builder.
 */
function boolean(): BooleanBuilder {
  return new BooleanBuilder();
}

/**
 * Create a money schema builder.
 */
function money(): MoneyBuilder {
  return new MoneyBuilder();
}

/**
 * Create a currency schema builder with ISO 4217 code support.
 */
function currency(): CurrencyBuilder {
  return new CurrencyBuilder();
}

/**
 * Create a decimal schema builder with configurable precision and scale.
 */
function decimal(): DecimalBuilder {
  return new DecimalBuilder();
}

/**
 * Create a percentage schema builder.
 */
function percentage(): PercentageBuilder {
  return new PercentageBuilder();
}

/**
 * Create a date schema builder.
 */
function date(): DateBuilder {
  return new DateBuilder();
}

/**
 * Create an object schema builder.
 */
function object<S extends ObjectShape>(shape?: S): ObjectBuilder<S> {
  return new ObjectBuilder(shape);
}

/**
 * Create an array schema builder.
 */
function array<T extends SchemaBuilder>(items: T): ArrayBuilder<T> {
  return new ArrayBuilder(items);
}

/**
 * Create a tuple schema builder.
 */
function tuple<T extends readonly SchemaBuilder[]>(items: T): TupleBuilder<T> {
  return new TupleBuilder(items);
}

/**
 * Create an enum schema builder.
 */
function enumType<T extends readonly string[]>(values: T): EnumBuilder<T> {
  return new EnumBuilder(values);
}

/**
 * Create a literal schema builder.
 */
function literal<T extends PrimitiveLiteral>(value: T): LiteralBuilder<T> {
  return new LiteralBuilder(value);
}

/**
 * Create a never schema builder.
 */
function never(): NeverBuilder {
  return new NeverBuilder();
}

/**
 * Create a union schema builder.
 */
function union<T extends readonly SchemaBuilder[]>(
  variants: T,
): UnionBuilder<T> {
  return new UnionBuilder(variants);
}

/**
 * Create an intersection schema builder.
 */
function intersection<T extends readonly SchemaBuilder[]>(
  variants: T,
): IntersectionBuilder<T> {
  return new IntersectionBuilder(variants);
}

/**
 * Create a record schema builder.
 */
function record<
  V extends SchemaBuilder,
  K extends SchemaBuilder | undefined = undefined,
>(value: V, key?: K): RecordBuilder<V, K> {
  return new RecordBuilder(value, key);
}

/**
 * Wrap a schema to also allow null.
 */
function nullable<T extends SchemaBuilder>(schema: T): NullableBuilder<T> {
  return new NullableBuilder(schema);
}

/**
 * Wrap a schema to allow null and make it optional in objects.
 */
function nullish<T extends SchemaBuilder>(
  schema: T,
): NullableBuilder<T> & OptionalBrand {
  return nullable(schema).optional();
}

/**
 * Wrap a schema with preprocess transforms.
 */
function preprocess<T extends SchemaBuilder>(
  transforms: Transform | Transform[],
  schema: T,
): PreprocessBuilder<T> {
  return schema.preprocess(transforms);
}

/**
 * Wrap a schema with catch fallback.
 */
function catchType<T extends SchemaBuilder>(
  schema: T,
  value: unknown,
): CatchBuilder<T> {
  return schema.catch(value);
}

/**
 * The `w` namespace provides factory functions for all schema types.
 */
export const w = {
  // Primitive types
  string,
  number,
  integer,
  // Specific integer types
  int32,
  int64,
  uint32,
  uint64,
  // Boolean
  boolean,
  // Domain-specific numeric types
  money,
  currency,
  decimal,
  percentage,
  // Date
  date,
  // Composite types
  object,
  array,
  tuple,
  enum: enumType,
  literal,
  never,
  union,
  intersection,
  record,
  nullable,
  nullish,
  preprocess,
  catch: catchType,
} as const;

// Re-export for convenience
export { optional, required } from "./object.js";
