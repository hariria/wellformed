/**
 * Enum schema builder.
 */

import type { TypeSchema } from "../ir/types.js";
import { BaseBuilder } from "./types.js";

/**
 * Builder for enum schemas.
 */
export class EnumBuilder<
  T extends readonly string[] = readonly string[],
> extends BaseBuilder<TypeSchema & { type: "enum" }> {
  /** @internal Exposed for type inference */
  readonly _values: T;

  constructor(values: T) {
    super();
    this._values = values;
  }

  toTypeSchema(): TypeSchema & { type: "enum" } {
    // Rust uses flattened enum schema with values directly on the object
    return {
      type: "enum",
      values: [...this._values],
    };
  }
}
