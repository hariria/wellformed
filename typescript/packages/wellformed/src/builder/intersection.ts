/**
 * Intersection schema builder.
 */

import type { TypeSchema } from "../ir/types.js";
import { BaseBuilder, type SchemaBuilder } from "./types.js";

/**
 * Builder for intersection schemas.
 */
export class IntersectionBuilder<
  T extends readonly SchemaBuilder[] = readonly SchemaBuilder[],
> extends BaseBuilder<TypeSchema & { type: "intersection" }> {
  /** @internal Exposed for type inference */
  readonly _variants: T;
  private _variantSchemas: TypeSchema[];

  constructor(variants: T) {
    super();
    this._variants = variants;
    this._variantSchemas = [...variants].map((v) => v.toTypeSchema());
  }

  toTypeSchema(): TypeSchema & { type: "intersection" } {
    return {
      type: "intersection",
      allOf: this._variantSchemas,
    };
  }
}
