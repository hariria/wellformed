/**
 * Fluent API for building conditional validation rules.
 *
 * @example
 * ```ts
 * schema
 *   .when("type").equals("individual").require("ssn")
 *   .when("type").equals("business").require("ein")
 *   .when("country").equals("US").and("type").equals("individual").require("ssn")
 * ```
 */

import type { Predicate } from "../ir/types.js";
import type { ConstraintOptions } from "./types.js";

/**
 * Callback to add a rule to the parent object builder.
 */
export type RuleAdder<T> = (
  pred: Predicate,
  code: string,
  message: string,
  options?: ConstraintOptions,
) => T;

/**
 * Initial condition builder - starts with a field reference.
 */
export class ConditionBuilder<T> {
  constructor(
    private field: string,
    private addRule: RuleAdder<T>,
    private parent: T,
  ) {}

  /**
   * Field equals a specific value.
   */
  equals(value: unknown): ConditionChain<T> {
    const pred: Predicate = { type: "eq", path: `/${this.field}`, value };
    return new ConditionChain(pred, this.addRule, this.parent);
  }

  /**
   * Field is one of the specified values.
   */
  in(values: unknown[]): ConditionChain<T> {
    const pred: Predicate = { type: "in", path: `/${this.field}`, values };
    return new ConditionChain(pred, this.addRule, this.parent);
  }

  /**
   * Field exists (is not null/undefined).
   */
  exists(): ConditionChain<T> {
    const pred: Predicate = { type: "exists", path: `/${this.field}` };
    return new ConditionChain(pred, this.addRule, this.parent);
  }

  /**
   * Field does not exist.
   */
  notExists(): ConditionChain<T> {
    const pred: Predicate = {
      type: "not",
      predicate: { type: "exists", path: `/${this.field}` },
    };
    return new ConditionChain(pred, this.addRule, this.parent);
  }

  /**
   * Field does not equal a value.
   */
  notEquals(value: unknown): ConditionChain<T> {
    const pred: Predicate = {
      type: "not",
      predicate: { type: "eq", path: `/${this.field}`, value },
    };
    return new ConditionChain(pred, this.addRule, this.parent);
  }

  /**
   * Field value is greater than or equal.
   */
  gte(value: number): ConditionChain<T> {
    const pred: Predicate = { type: "range", min: value };
    return new ConditionChain(pred, this.addRule, this.parent);
  }

  /**
   * Field value is less than or equal.
   */
  lte(value: number): ConditionChain<T> {
    const pred: Predicate = { type: "range", max: value };
    return new ConditionChain(pred, this.addRule, this.parent);
  }
}

/**
 * Condition chain - allows composing conditions with and/or.
 */
export class ConditionChain<T> {
  constructor(
    private condition: Predicate,
    private addRule: RuleAdder<T>,
    private parent: T,
  ) {}

  /**
   * Add another condition with AND.
   */
  and(field: string): ConditionAndBuilder<T> {
    return new ConditionAndBuilder(
      this.condition,
      field,
      this.addRule,
      this.parent,
    );
  }

  /**
   * Add another condition with OR.
   */
  or(field: string): ConditionOrBuilder<T> {
    return new ConditionOrBuilder(
      this.condition,
      field,
      this.addRule,
      this.parent,
    );
  }

  /**
   * Require a field to exist when condition is true.
   */
  require(field: string, options?: ConstraintOptions): T {
    const consequent: Predicate = { type: "exists", path: `/${field}` };
    return this.addImpliesRule(
      consequent,
      options?.code ?? "REQUIRED",
      options?.message ?? `${field} is required`,
      options,
    );
  }

  /**
   * Require a field to have a specific value when condition is true.
   */
  requireEquals(field: string, value: unknown, options?: ConstraintOptions): T {
    const consequent: Predicate = { type: "eq", path: `/${field}`, value };
    return this.addImpliesRule(
      consequent,
      options?.code ?? "INVALID_VALUE",
      options?.message ?? `${field} must equal ${JSON.stringify(value)}`,
      options,
    );
  }

  /**
   * Require a field to be one of the specified values.
   */
  requireIn(field: string, values: unknown[], options?: ConstraintOptions): T {
    const consequent: Predicate = { type: "in", path: `/${field}`, values };
    return this.addImpliesRule(
      consequent,
      options?.code ?? "INVALID_VALUE",
      options?.message ?? `${field} must be one of: ${values.join(", ")}`,
      options,
    );
  }

  /**
   * Require a field to match a regex pattern.
   */
  requireMatch(
    field: string,
    pattern: string | RegExp,
    options?: ConstraintOptions,
  ): T {
    const patternStr = typeof pattern === "string" ? pattern : pattern.source;
    const flags = typeof pattern === "string" ? undefined : pattern.flags;
    const consequent: Predicate = {
      type: "and",
      predicates: [
        { type: "exists", path: `/${field}` },
        { type: "regex", pattern: patternStr, flags },
      ],
    };
    return this.addImpliesRule(
      consequent,
      options?.code ?? "INVALID_FORMAT",
      options?.message ?? `${field} has invalid format`,
      options,
    );
  }

  /**
   * Require a custom predicate to be true.
   */
  requirePredicate(
    consequent: Predicate,
    code: string,
    message: string,
    options?: ConstraintOptions,
  ): T {
    return this.addImpliesRule(consequent, code, message, options);
  }

  /**
   * Forbid a field from existing when condition is true.
   */
  forbid(field: string, options?: ConstraintOptions): T {
    const consequent: Predicate = {
      type: "not",
      predicate: { type: "exists", path: `/${field}` },
    };
    return this.addImpliesRule(
      consequent,
      options?.code ?? "FORBIDDEN",
      options?.message ?? `${field} is not allowed`,
      options,
    );
  }

  private addImpliesRule(
    consequent: Predicate,
    code: string,
    message: string,
    options?: ConstraintOptions,
  ): T {
    const impliesPred: Predicate = {
      type: "implies",
      if: this.condition,
      // biome-ignore lint/suspicious/noThenProperty: `then` is a legitimate property in our Predicate IR
      then: consequent,
    };
    return this.addRule(impliesPred, code, message, options);
  }
}

/**
 * Builder for AND conditions.
 */
export class ConditionAndBuilder<T> {
  constructor(
    private leftCondition: Predicate,
    private field: string,
    private addRule: RuleAdder<T>,
    private parent: T,
  ) {}

  equals(value: unknown): ConditionChain<T> {
    const rightPred: Predicate = { type: "eq", path: `/${this.field}`, value };
    const combined: Predicate = {
      type: "and",
      predicates: [this.leftCondition, rightPred],
    };
    return new ConditionChain(combined, this.addRule, this.parent);
  }

  in(values: unknown[]): ConditionChain<T> {
    const rightPred: Predicate = { type: "in", path: `/${this.field}`, values };
    const combined: Predicate = {
      type: "and",
      predicates: [this.leftCondition, rightPred],
    };
    return new ConditionChain(combined, this.addRule, this.parent);
  }

  exists(): ConditionChain<T> {
    const rightPred: Predicate = { type: "exists", path: `/${this.field}` };
    const combined: Predicate = {
      type: "and",
      predicates: [this.leftCondition, rightPred],
    };
    return new ConditionChain(combined, this.addRule, this.parent);
  }

  notExists(): ConditionChain<T> {
    const rightPred: Predicate = {
      type: "not",
      predicate: { type: "exists", path: `/${this.field}` },
    };
    const combined: Predicate = {
      type: "and",
      predicates: [this.leftCondition, rightPred],
    };
    return new ConditionChain(combined, this.addRule, this.parent);
  }

  notEquals(value: unknown): ConditionChain<T> {
    const rightPred: Predicate = {
      type: "not",
      predicate: { type: "eq", path: `/${this.field}`, value },
    };
    const combined: Predicate = {
      type: "and",
      predicates: [this.leftCondition, rightPred],
    };
    return new ConditionChain(combined, this.addRule, this.parent);
  }

  gte(value: number): ConditionChain<T> {
    const rightPred: Predicate = { type: "range", min: value };
    const combined: Predicate = {
      type: "and",
      predicates: [this.leftCondition, rightPred],
    };
    return new ConditionChain(combined, this.addRule, this.parent);
  }

  lte(value: number): ConditionChain<T> {
    const rightPred: Predicate = { type: "range", max: value };
    const combined: Predicate = {
      type: "and",
      predicates: [this.leftCondition, rightPred],
    };
    return new ConditionChain(combined, this.addRule, this.parent);
  }
}

/**
 * Builder for OR conditions.
 */
export class ConditionOrBuilder<T> {
  constructor(
    private leftCondition: Predicate,
    private field: string,
    private addRule: RuleAdder<T>,
    private parent: T,
  ) {}

  equals(value: unknown): ConditionChain<T> {
    const rightPred: Predicate = { type: "eq", path: `/${this.field}`, value };
    const combined: Predicate = {
      type: "or",
      predicates: [this.leftCondition, rightPred],
    };
    return new ConditionChain(combined, this.addRule, this.parent);
  }

  in(values: unknown[]): ConditionChain<T> {
    const rightPred: Predicate = { type: "in", path: `/${this.field}`, values };
    const combined: Predicate = {
      type: "or",
      predicates: [this.leftCondition, rightPred],
    };
    return new ConditionChain(combined, this.addRule, this.parent);
  }

  exists(): ConditionChain<T> {
    const rightPred: Predicate = { type: "exists", path: `/${this.field}` };
    const combined: Predicate = {
      type: "or",
      predicates: [this.leftCondition, rightPred],
    };
    return new ConditionChain(combined, this.addRule, this.parent);
  }

  notExists(): ConditionChain<T> {
    const rightPred: Predicate = {
      type: "not",
      predicate: { type: "exists", path: `/${this.field}` },
    };
    const combined: Predicate = {
      type: "or",
      predicates: [this.leftCondition, rightPred],
    };
    return new ConditionChain(combined, this.addRule, this.parent);
  }

  notEquals(value: unknown): ConditionChain<T> {
    const rightPred: Predicate = {
      type: "not",
      predicate: { type: "eq", path: `/${this.field}`, value },
    };
    const combined: Predicate = {
      type: "or",
      predicates: [this.leftCondition, rightPred],
    };
    return new ConditionChain(combined, this.addRule, this.parent);
  }
}
