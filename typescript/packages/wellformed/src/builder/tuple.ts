/**
 * Tuple schema builder.
 */

import type { TypeSchema } from "../ir/types.js";
import { BaseBuilder, type SchemaBuilder } from "./types.js";

/**
 * Builder for tuple schemas.
 */
export class TupleBuilder<
  T extends readonly SchemaBuilder[] = readonly SchemaBuilder[],
> extends BaseBuilder<TypeSchema & { type: "tuple" }> {
  /** @internal Exposed for type inference */
  readonly _items: T;
  private _itemSchemas: TypeSchema[];

  constructor(items: T) {
    super();
    this._items = items;
    this._itemSchemas = [...items].map((item) => item.toTypeSchema());
  }

  toTypeSchema(): TypeSchema & { type: "tuple" } {
    return {
      type: "tuple",
      items: this._itemSchemas,
    };
  }
}
