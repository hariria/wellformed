/**
 * Union schema builder.
 */

import type { TypeSchema } from "../ir/types.js";
import { BaseBuilder, type SchemaBuilder } from "./types.js";

/**
 * Builder for union (discriminated or regular) schemas.
 */
export class UnionBuilder<
  T extends readonly SchemaBuilder[] = readonly SchemaBuilder[],
> extends BaseBuilder<TypeSchema & { type: "union" }> {
  /** @internal Exposed for type inference */
  readonly _variants: T;
  private _variantSchemas: TypeSchema[];
  private _discriminator?: string;

  constructor(variants: T) {
    super();
    this._variants = variants;
    this._variantSchemas = [...variants].map((v) => v.toTypeSchema());
  }

  /**
   * Set the discriminator field for tagged unions.
   */
  discriminator(field: string): this {
    this._discriminator = field;
    return this;
  }

  toTypeSchema(): TypeSchema & { type: "union" } {
    // Rust uses "oneOf" for variants, not "variants"
    return {
      type: "union",
      oneOf: this._variantSchemas,
      discriminator: this._discriminator,
    };
  }
}
