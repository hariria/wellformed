/**
 * Record schema builder.
 */

import type { TypeSchema } from "../ir/types.js";
import { BaseBuilder, type SchemaBuilder } from "./types.js";

/**
 * Builder for record schemas.
 */
export class RecordBuilder<
  V extends SchemaBuilder = SchemaBuilder,
  K extends SchemaBuilder | undefined = undefined,
> extends BaseBuilder<TypeSchema & { type: "record" }> {
  /** @internal Exposed for type inference */
  readonly _valueBuilder: V;
  /** @internal Exposed for type inference */
  readonly _keyBuilder: K;
  private _valueSchema: TypeSchema;
  private _keySchema?: TypeSchema;
  private _partial?: boolean;

  constructor(valueBuilder: V, keyBuilder?: K) {
    super();
    this._valueBuilder = valueBuilder;
    this._keyBuilder = keyBuilder as K;
    this._valueSchema = valueBuilder.toTypeSchema();
    this._keySchema = keyBuilder?.toTypeSchema();
  }

  /**
   * Mark known-key records as partial.
   */
  partial(): this {
    this._partial = true;
    return this;
  }

  toTypeSchema(): TypeSchema & { type: "record" } {
    return {
      type: "record",
      value: this._valueSchema,
      key: this._keySchema,
      partial: this._partial,
    };
  }
}
