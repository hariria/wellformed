import type { Resolver } from "react-hook-form";
import type { Schema } from "wellformed-ts/ir";
import { validate } from "wellformed-ts/runtime";

export function wellformedResolver(schema: Schema): Resolver {
  return async (values) => {
    const result = validate(schema, values);
    if (result.valid) {
      return { values: result.value as Record<string, unknown>, errors: {} };
    }

    const errors: Record<string, { type: string; message: string }> = {};
    for (const err of result.errors) {
      // Convert JSON Pointer "/foo/bar" → dot path "foo.bar"
      const path = err.path.replace(/^\//, "").replace(/\//g, ".");
      if (path && !errors[path]) {
        errors[path] = { type: err.code, message: err.message };
      }
    }
    return { values: {}, errors };
  };
}
