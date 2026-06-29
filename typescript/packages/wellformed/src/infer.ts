/**
 * TypeScript type inference from wellformed schemas.
 *
 * Similar to Zod's `z.infer<typeof schema>`, this allows extracting
 * the TypeScript type that a schema validates.
 *
 * @example
 * ```ts
 * const userSchema = w.object({
 *   name: w.string(),
 *   age: w.integer(),
 *   email: optional(w.string().email()),
 * });
 *
 * type User = Infer<typeof userSchema>;
 * // { name: string; age: number; email?: string | undefined }
 * ```
 */

import type { ArrayBuilder } from "./builder/array.js";
import type { EnumBuilder } from "./builder/enum.js";
import type { IntersectionBuilder } from "./builder/intersection.js";
import type { LiteralBuilder, NeverBuilder } from "./builder/literal.js";
import type {
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
} from "./builder/number.js";
import type {
  ObjectBuilder,
  ObjectShape,
  PropertyDef,
} from "./builder/object.js";
import type { RecordBuilder } from "./builder/record.js";
import type { StringBuilder } from "./builder/string.js";
import type { TupleBuilder } from "./builder/tuple.js";
import type {
  CatchBuilder,
  NullableBuilder,
  OptionalBrand,
  PreprocessBuilder,
  SchemaBuilder,
} from "./builder/types.js";
import type { UnionBuilder } from "./builder/union.js";

/**
 * Infer the TypeScript type from a schema builder.
 */
export type Infer<T extends SchemaBuilder> = InferBuilder<T>;

/**
 * Infer from specific builder types.
 */
type InferBuilder<T> = T extends StringBuilder
  ? string
  : T extends NumberBuilder
    ? number
    : T extends IntegerBuilder
      ? number
      : T extends
            | MoneyBuilder
            | CurrencyBuilder
            | DecimalBuilder
            | PercentageBuilder
            | Int32Builder
            | Int64Builder
            | Uint32Builder
            | Uint64Builder
        ? number
        : T extends DateBuilder
          ? string
          : T extends BooleanBuilder
            ? boolean
            : T extends EnumBuilder<infer Values>
              ? Values[number]
              : T extends LiteralBuilder<infer Value>
                ? Value
                : T extends NeverBuilder
                  ? never
                  : T extends NullableBuilder<infer Inner>
                    ? InferBuilder<Inner> | null
                    : T extends PreprocessBuilder<infer Inner>
                      ? InferBuilder<Inner>
                      : T extends CatchBuilder<infer Inner>
                        ? InferBuilder<Inner>
                        : T extends RecordBuilder<infer Value, infer _Key>
                          ? Record<string, InferBuilder<Value>>
                          : T extends IntersectionBuilder<infer Variants>
                            ? InferIntersectionVariants<Variants>
                            : T extends TupleBuilder<infer Items>
                              ? InferTupleItems<Items>
                              : T extends ArrayBuilder<infer Item>
                                ? InferBuilder<Item>[]
                                : T extends ObjectBuilder<infer S>
                                  ? S extends ObjectShape
                                    ? Simplify<InferShape<S>>
                                    : Record<string, unknown>
                                  : T extends UnionBuilder<infer Variants>
                                    ? InferUnionVariants<Variants>
                                    : unknown;

/**
 * Infer from an object shape.
 */
type InferShape<S extends ObjectShape> = {
  [K in keyof S as IsOptional<S[K]> extends true ? never : K]: InferProperty<
    S[K]
  >;
} & {
  [K in keyof S as IsOptional<S[K]> extends true ? K : never]?: InferProperty<
    S[K]
  >;
};

/**
 * Check if a value is marked as optional (either PropertyDef or OptionalBrand).
 */
type IsOptional<T> =
  T extends PropertyDef<SchemaBuilder, infer Required>
    ? Required extends false
      ? true
      : false
    : T extends OptionalBrand
      ? true
      : false;

/**
 * Infer from a property (either SchemaBuilder or PropertyDef).
 */
type InferProperty<P> = P extends {
  builder: infer Builder extends SchemaBuilder;
}
  ? InferBuilder<Builder>
  : P extends SchemaBuilder
    ? InferBuilder<P>
    : unknown;

/**
 * Infer union type from array of variants.
 */
type InferUnionVariants<T extends readonly SchemaBuilder[]> = {
  [K in keyof T]: T[K] extends SchemaBuilder ? InferBuilder<T[K]> : never;
}[number];

type UnionToIntersection<U> = (
  U extends unknown
    ? (arg: U) => void
    : never
) extends (arg: infer I) => void
  ? I
  : never;

/**
 * Infer intersection type from variants.
 */
type InferIntersectionVariants<T extends readonly SchemaBuilder[]> =
  UnionToIntersection<
    {
      [K in keyof T]: T[K] extends SchemaBuilder ? InferBuilder<T[K]> : never;
    }[number]
  >;

/**
 * Infer tuple output type from tuple builders.
 */
type InferTupleItems<T extends readonly SchemaBuilder[]> = T extends readonly [
  infer Head extends SchemaBuilder,
  ...infer Tail extends readonly SchemaBuilder[],
]
  ? [InferBuilder<Head>, ...InferTupleItems<Tail>]
  : [];

/**
 * Helper type to simplify object types for better display.
 */
export type Simplify<T> = { [K in keyof T]: T[K] } & {};

/**
 * Input type - allows undefined for optional fields.
 * Use this when accepting input data before validation.
 */
export type InferInput<T extends SchemaBuilder> = InferBuilder<T>;

/**
 * Output type - after validation and transforms.
 * Same as Infer for now, but could differ if transforms change types.
 */
export type InferOutput<T extends SchemaBuilder> = Infer<T>;
