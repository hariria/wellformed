/**
 * Zod-like builder DSL for creating wellformed schemas.
 *
 * @example
 * ```ts
 * import { w } from "wellformed-ts";
 *
 * const payeeSchema = w.object({
 *   tin: w.string().trim().tin(),
 *   name: w.string().trim().minLen(1).maxLen(100),
 *   address: w.object({
 *     street: w.string(),
 *     city: w.string(),
 *     state: w.string().usState(),
 *     zip: w.string().usZip(),
 *   }),
 * });
 *
 * // Convert to IR for serialization
 * const ir = payeeSchema.toSchema();
 * ```
 */

export * from "./array.js";
export * from "./condition.js";
export * from "./enum.js";
export * from "./intersection.js";
export * from "./literal.js";
export * from "./number.js";
export * from "./object.js";
export * from "./record.js";
export * from "./string.js";
export * from "./tuple.js";
export * from "./types.js";
export * from "./union.js";
export * from "./w.js";
