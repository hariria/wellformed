/**
 * Array schema builder.
 */

import type { TypeSchema } from "../ir/types.js";
import {
  type ConstraintOptions,
  makeConstraint,
  type SchemaBuilder,
  TransformableBuilder,
} from "./types.js";

/**
 * Builder for array schemas.
 */
export class ArrayBuilder<
  T extends SchemaBuilder = SchemaBuilder,
> extends TransformableBuilder<TypeSchema & { type: "array" }> {
  /** @internal Exposed for type inference */
  readonly _itemBuilder: T;
  private _items: TypeSchema;
  private _minItems?: number;
  private _maxItems?: number;

  constructor(itemBuilder: T) {
    super();
    this._itemBuilder = itemBuilder;
    this._items = itemBuilder.toTypeSchema();
  }

  /**
   * Require minimum number of items.
   */
  minItems(count: number, options?: ConstraintOptions): this {
    this._minItems = count;
    return this.addConstraint(
      makeConstraint(
        { type: "min_len", len: count },
        "TOO_FEW_ITEMS",
        options?.message ??
          `Must have at least ${count} item${count === 1 ? "" : "s"}`,
        options,
      ),
    );
  }

  /**
   * Require maximum number of items.
   */
  maxItems(count: number, options?: ConstraintOptions): this {
    this._maxItems = count;
    return this.addConstraint(
      makeConstraint(
        { type: "max_len", len: count },
        "TOO_MANY_ITEMS",
        options?.message ??
          `Must have at most ${count} item${count === 1 ? "" : "s"}`,
        options,
      ),
    );
  }

  /**
   * Require non-empty array.
   */
  nonEmpty(options?: ConstraintOptions): this {
    return this.minItems(1, { message: "Cannot be empty", ...options });
  }

  /**
   * Require exact number of items.
   */
  length(count: number, options?: ConstraintOptions): this {
    return this.minItems(count, options).maxItems(count, options);
  }

  toTypeSchema(): TypeSchema & { type: "array" } {
    // Rust uses flattened array schema, not nested under "schema"
    return {
      type: "array",
      items: this._items,
      min_items: this._minItems,
      max_items: this._maxItems,
      constraints: this._constraints.length > 0 ? this._constraints : undefined,
    };
  }
}
