/**
 * Literal and never schema builders.
 */

import type { TypeSchema } from "../ir/types.js";
import { BaseBuilder } from "./types.js";

export type PrimitiveLiteral = string | number | boolean | null;

/**
 * Builder for literal schemas.
 */
export class LiteralBuilder<
  T extends PrimitiveLiteral = PrimitiveLiteral,
> extends BaseBuilder<TypeSchema & { type: "literal" }> {
  /** @internal Exposed for type inference */
  readonly _value: T;

  constructor(value: T) {
    super();
    this._value = value;
  }

  toTypeSchema(): TypeSchema & { type: "literal" } {
    return {
      type: "literal",
      value: this._value,
    };
  }
}

/**
 * Builder for never schemas.
 */
export class NeverBuilder extends BaseBuilder<TypeSchema & { type: "never" }> {
  toTypeSchema(): TypeSchema & { type: "never" } {
    return {
      type: "never",
    };
  }
}
