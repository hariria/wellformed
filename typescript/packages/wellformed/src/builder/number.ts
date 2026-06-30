/**
 * Number, integer, and money schema builders.
 */

import type { TypeSchema } from "../ir/types.js";
import {
  type ConstraintOptions,
  makeConstraint,
  TransformableBuilder,
} from "./types.js";

/**
 * Builder for number schemas.
 */
export class NumberBuilder extends TransformableBuilder<
  TypeSchema & { type: "number" }
> {
  /**
   * Provide default value if null/undefined.
   */
  default(value: number): this {
    return this.addTransform({ fn: "default", value });
  }

  /**
   * Require value in range.
   */
  range(min?: number, max?: number, options?: ConstraintOptions): this {
    const parts: string[] = [];
    if (min !== undefined) parts.push(`>= ${min}`);
    if (max !== undefined) parts.push(`<= ${max}`);
    return this.addConstraint(
      makeConstraint(
        { type: "range", min, max },
        "OUT_OF_RANGE",
        options?.message ?? `Must be ${parts.join(" and ")}`,
        options,
      ),
    );
  }

  /**
   * Require minimum value.
   */
  min(value: number, options?: ConstraintOptions): this {
    return this.range(value, undefined, options);
  }

  /**
   * Require maximum value.
   */
  max(value: number, options?: ConstraintOptions): this {
    return this.range(undefined, value, options);
  }

  /**
   * Require non-negative value (>= 0).
   */
  nonNegative(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_non_negative" },
        "NEGATIVE_VALUE",
        options?.message ?? "Must be non-negative",
        options,
      ),
    );
  }

  /**
   * Require positive value (> 0).
   */
  positive(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_positive" },
        "NOT_POSITIVE",
        options?.message ?? "Must be positive",
        options,
      ),
    );
  }

  /**
   * Require negative value (< 0).
   */
  negative(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_negative" },
        "NOT_NEGATIVE",
        options?.message ?? "Must be negative",
        options,
      ),
    );
  }

  /**
   * Require non-positive value (<= 0).
   */
  nonPositive(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_non_positive" },
        "POSITIVE_VALUE",
        options?.message ?? "Must be non-positive",
        options,
      ),
    );
  }

  /**
   * Require value to be a multiple of the provided step.
   */
  multipleOf(step: number, options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_multiple_of", args: { value: step } },
        "NOT_MULTIPLE",
        options?.message ?? `Must be a multiple of ${step}`,
        options,
      ),
    );
  }

  /**
   * Validate as percentage (0-100 or 0-1 for decimal).
   */
  percentage(
    options?: ConstraintOptions & {
      format?: "percent" | "decimal";
      allowOver100?: boolean;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_percentage",
          args: {
            format: options?.format,
            allow_over_100: options?.allowOver100,
          },
        },
        "INVALID_PERCENTAGE",
        options?.message ?? "Invalid percentage",
        options,
      ),
    );
  }

  /**
   * Require value less than or equal to another.
   */
  lte(value: number, options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "less_than_or_equal", args: { value } },
        "TOO_LARGE",
        options?.message ?? `Must be at most ${value}`,
        options,
      ),
    );
  }

  /**
   * Require value greater than or equal to another.
   */
  gte(value: number, options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "greater_than_or_equal", args: { value } },
        "TOO_SMALL",
        options?.message ?? `Must be at least ${value}`,
        options,
      ),
    );
  }

  toTypeSchema(): TypeSchema & { type: "number" } {
    return {
      type: "number",
      transforms: this._transforms.length > 0 ? this._transforms : undefined,
      constraints: this._constraints.length > 0 ? this._constraints : undefined,
    };
  }
}

/**
 * Builder for integer schemas.
 */
export class IntegerBuilder extends TransformableBuilder<
  TypeSchema & { type: "integer" }
> {
  /**
   * Provide default value if null/undefined.
   */
  default(value: number): this {
    return this.addTransform({ fn: "default", value });
  }

  /**
   * Require value in range.
   */
  range(min?: number, max?: number, options?: ConstraintOptions): this {
    const parts: string[] = [];
    if (min !== undefined) parts.push(`>= ${min}`);
    if (max !== undefined) parts.push(`<= ${max}`);
    return this.addConstraint(
      makeConstraint(
        { type: "range", min, max },
        "OUT_OF_RANGE",
        options?.message ?? `Must be ${parts.join(" and ")}`,
        options,
      ),
    );
  }

  /**
   * Require minimum value.
   */
  min(value: number, options?: ConstraintOptions): this {
    return this.range(value, undefined, options);
  }

  /**
   * Require maximum value.
   */
  max(value: number, options?: ConstraintOptions): this {
    return this.range(undefined, value, options);
  }

  /**
   * Require non-negative value (>= 0).
   */
  nonNegative(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_non_negative" },
        "NEGATIVE_VALUE",
        options?.message ?? "Must be non-negative",
        options,
      ),
    );
  }

  /**
   * Require positive value (> 0).
   */
  positive(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_positive" },
        "NOT_POSITIVE",
        options?.message ?? "Must be positive",
        options,
      ),
    );
  }

  /**
   * Require negative value (< 0).
   */
  negative(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_negative" },
        "NOT_NEGATIVE",
        options?.message ?? "Must be negative",
        options,
      ),
    );
  }

  /**
   * Require non-positive value (<= 0).
   */
  nonPositive(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_non_positive" },
        "POSITIVE_VALUE",
        options?.message ?? "Must be non-positive",
        options,
      ),
    );
  }

  /**
   * Require value to be a multiple of the provided step.
   */
  multipleOf(step: number, options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_multiple_of", args: { value: step } },
        "NOT_MULTIPLE",
        options?.message ?? `Must be a multiple of ${step}`,
        options,
      ),
    );
  }

  /**
   * Validate as tax year.
   */
  taxYear(options?: ConstraintOptions & { min?: number; max?: number }): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_tax_year",
          args: { min: options?.min, max: options?.max },
        },
        "INVALID_TAX_YEAR",
        options?.message ?? "Invalid tax year",
        options,
      ),
    );
  }

  toTypeSchema(): TypeSchema & { type: "integer" } {
    return {
      type: "integer",
      transforms: this._transforms.length > 0 ? this._transforms : undefined,
      constraints: this._constraints.length > 0 ? this._constraints : undefined,
    };
  }
}

/**
 * Builder for money schemas (amounts in cents).
 */
export class MoneyBuilder extends TransformableBuilder<
  TypeSchema & { type: "money" }
> {
  private _scale?: number;

  /**
   * Set the scale (decimal places). Default is 2.
   */
  scale(value: number): this {
    this._scale = value;
    return this;
  }

  /**
   * Provide default value if null/undefined.
   */
  default(value: number): this {
    return this.addTransform({ fn: "default", value });
  }

  /**
   * Require non-negative value (>= 0).
   */
  nonNegative(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_non_negative" },
        "NEGATIVE_AMOUNT",
        options?.message ?? "Amount cannot be negative",
        options,
      ),
    );
  }

  /**
   * Require positive value (> 0).
   */
  positive(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_positive" },
        "ZERO_AMOUNT",
        options?.message ?? "Amount must be positive",
        options,
      ),
    );
  }

  /**
   * Require value in range.
   */
  range(min?: number, max?: number, options?: ConstraintOptions): this {
    const parts: string[] = [];
    if (min !== undefined) parts.push(`>= ${min}`);
    if (max !== undefined) parts.push(`<= ${max}`);
    return this.addConstraint(
      makeConstraint(
        { type: "range", min, max },
        "AMOUNT_OUT_OF_RANGE",
        options?.message ?? `Amount must be ${parts.join(" and ")}`,
        options,
      ),
    );
  }

  toTypeSchema(): TypeSchema & { type: "money" } {
    return {
      type: "money",
      scale: this._scale,
      transforms: this._transforms.length > 0 ? this._transforms : undefined,
      constraints: this._constraints.length > 0 ? this._constraints : undefined,
    };
  }
}

/**
 * ISO 4217 currency codes.
 * Common currencies with autocomplete support.
 */
export type CurrencyCode =
  // Major currencies
  | "USD" // US Dollar
  | "EUR" // Euro
  | "GBP" // British Pound
  | "JPY" // Japanese Yen (0 decimal places)
  | "CHF" // Swiss Franc
  | "CAD" // Canadian Dollar
  | "AUD" // Australian Dollar
  | "NZD" // New Zealand Dollar
  // Asian currencies
  | "CNY" // Chinese Yuan
  | "HKD" // Hong Kong Dollar
  | "SGD" // Singapore Dollar
  | "KRW" // South Korean Won (0 decimal places)
  | "TWD" // Taiwan Dollar
  | "INR" // Indian Rupee
  | "THB" // Thai Baht
  | "MYR" // Malaysian Ringgit
  | "IDR" // Indonesian Rupiah (0 decimal places)
  | "PHP" // Philippine Peso
  | "VND" // Vietnamese Dong (0 decimal places)
  // European currencies
  | "SEK" // Swedish Krona
  | "NOK" // Norwegian Krone
  | "DKK" // Danish Krone
  | "PLN" // Polish Zloty
  | "CZK" // Czech Koruna
  | "HUF" // Hungarian Forint
  | "RUB" // Russian Ruble
  | "TRY" // Turkish Lira
  | "RON" // Romanian Leu
  | "BGN" // Bulgarian Lev
  | "HRK" // Croatian Kuna
  | "ISK" // Icelandic Krona (0 decimal places)
  // Americas currencies
  | "MXN" // Mexican Peso
  | "BRL" // Brazilian Real
  | "ARS" // Argentine Peso
  | "CLP" // Chilean Peso (0 decimal places)
  | "COP" // Colombian Peso
  | "PEN" // Peruvian Sol
  // Middle East currencies
  | "AED" // UAE Dirham
  | "SAR" // Saudi Riyal
  | "ILS" // Israeli Shekel
  | "QAR" // Qatari Riyal
  | "KWD" // Kuwaiti Dinar (3 decimal places)
  | "BHD" // Bahraini Dinar (3 decimal places)
  | "OMR" // Omani Rial (3 decimal places)
  | "JOD" // Jordanian Dinar (3 decimal places)
  // African currencies
  | "ZAR" // South African Rand
  | "EGP" // Egyptian Pound
  | "NGN" // Nigerian Naira
  | "KES" // Kenyan Shilling
  | "MAD" // Moroccan Dirham
  // Oceania currencies
  | "FJD" // Fiji Dollar
  // Cryptocurrencies (commonly used in finance)
  | "XBT" // Bitcoin (ISO 4217 proposed)
  | "XRP" // Ripple
  | "ETH" // Ethereum (not ISO but widely used)
  // Precious metals
  | "XAU" // Gold (troy ounce)
  | "XAG" // Silver (troy ounce)
  // Special
  | "XXX" // No currency
  // Allow any string for other ISO 4217 codes
  | (string & {});

/**
 * Builder for currency schemas with ISO 4217 currency code.
 * Unlike money (which just stores a numeric value in cents), currency
 * includes the currency code for multi-currency support.
 */
export class CurrencyBuilder extends TransformableBuilder<
  TypeSchema & { type: "currency" }
> {
  private _code?: string;
  private _scale?: number;

  /**
   * Set the ISO 4217 currency code (e.g., "USD", "EUR", "GBP").
   * Provides autocomplete for common currencies.
   */
  code(value: CurrencyCode): this {
    this._code = value;
    return this;
  }

  /**
   * Set the scale (decimal places). Default is 2.
   * Some currencies have different scales:
   * - JPY, KRW, VND: 0 decimal places
   * - BHD, KWD, OMR: 3 decimal places
   */
  scale(value: number): this {
    this._scale = value;
    return this;
  }

  /**
   * Provide default value if null/undefined.
   */
  default(value: number): this {
    return this.addTransform({ fn: "default", value });
  }

  /**
   * Require non-negative value (>= 0).
   */
  nonNegative(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_non_negative" },
        "NEGATIVE_AMOUNT",
        options?.message ?? "Amount cannot be negative",
        options,
      ),
    );
  }

  /**
   * Require positive value (> 0).
   */
  positive(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_positive" },
        "ZERO_AMOUNT",
        options?.message ?? "Amount must be positive",
        options,
      ),
    );
  }

  /**
   * Require value in range.
   */
  range(min?: number, max?: number, options?: ConstraintOptions): this {
    const parts: string[] = [];
    if (min !== undefined) parts.push(`>= ${min}`);
    if (max !== undefined) parts.push(`<= ${max}`);
    return this.addConstraint(
      makeConstraint(
        { type: "range", min, max },
        "AMOUNT_OUT_OF_RANGE",
        options?.message ?? `Amount must be ${parts.join(" and ")}`,
        options,
      ),
    );
  }

  toTypeSchema(): TypeSchema & { type: "currency" } {
    return {
      type: "currency",
      code: this._code,
      scale: this._scale,
      transforms: this._transforms.length > 0 ? this._transforms : undefined,
      constraints: this._constraints.length > 0 ? this._constraints : undefined,
    };
  }
}

/**
 * Builder for date schemas.
 */
export class DateBuilder extends TransformableBuilder<
  TypeSchema & { type: "date" }
> {
  private _format?: string;

  /**
   * Set the expected date format.
   */
  format(value: string): this {
    this._format = value;
    return this;
  }

  /**
   * Provide default value if null/undefined.
   */
  default(value: string): this {
    return this.addTransform({ fn: "default", value });
  }

  /**
   * Validate date is in range.
   */
  inRange(
    options: ConstraintOptions & {
      minYear?: number;
      maxYear?: number;
      min?: string;
      max?: string;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "date_in_range",
          args: {
            min_year: options.minYear,
            max_year: options.maxYear,
            min: options.min,
            max: options.max,
          },
        },
        "DATE_OUT_OF_RANGE",
        options?.message ?? "Date out of range",
        options,
      ),
    );
  }

  /**
   * Validate date is before another date.
   */
  before(
    targetDate: string,
    options?: ConstraintOptions & { allowEqual?: boolean },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "date_before",
          args: { date: targetDate, allow_equal: options?.allowEqual },
        },
        "DATE_TOO_LATE",
        options?.message ?? `Must be before ${targetDate}`,
        options,
      ),
    );
  }

  /**
   * Validate date is after another date.
   */
  after(
    targetDate: string,
    options?: ConstraintOptions & { allowEqual?: boolean },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "date_after",
          args: { date: targetDate, allow_equal: options?.allowEqual },
        },
        "DATE_TOO_EARLY",
        options?.message ?? `Must be after ${targetDate}`,
        options,
      ),
    );
  }

  toTypeSchema(): TypeSchema & { type: "date" } {
    return {
      type: "date",
      format: this._format,
      transforms: this._transforms.length > 0 ? this._transforms : undefined,
      constraints: this._constraints.length > 0 ? this._constraints : undefined,
    };
  }
}

/**
 * Builder for boolean schemas.
 */
export class BooleanBuilder extends TransformableBuilder<
  TypeSchema & { type: "boolean" }
> {
  toTypeSchema(): TypeSchema & { type: "boolean" } {
    return { type: "boolean" };
  }
}

// ============================================================================
// Specific Integer Types
// ============================================================================

/**
 * Builder for signed 32-bit integer schemas (-2,147,483,648 to 2,147,483,647).
 */
export class Int32Builder extends TransformableBuilder<
  TypeSchema & { type: "int32" }
> {
  /**
   * Provide default value if null/undefined.
   */
  default(value: number): this {
    return this.addTransform({ fn: "default", value });
  }

  /**
   * Require value in range.
   */
  range(min?: number, max?: number, options?: ConstraintOptions): this {
    const parts: string[] = [];
    if (min !== undefined) parts.push(`>= ${min}`);
    if (max !== undefined) parts.push(`<= ${max}`);
    return this.addConstraint(
      makeConstraint(
        { type: "range", min, max },
        "OUT_OF_RANGE",
        options?.message ?? `Must be ${parts.join(" and ")}`,
        options,
      ),
    );
  }

  /**
   * Require minimum value.
   */
  min(value: number, options?: ConstraintOptions): this {
    return this.range(value, undefined, options);
  }

  /**
   * Require maximum value.
   */
  max(value: number, options?: ConstraintOptions): this {
    return this.range(undefined, value, options);
  }

  toTypeSchema(): TypeSchema & { type: "int32" } {
    return {
      type: "int32",
      transforms: this._transforms.length > 0 ? this._transforms : undefined,
      constraints: this._constraints.length > 0 ? this._constraints : undefined,
    };
  }
}

/**
 * Builder for signed 64-bit integer schemas.
 * Note: JavaScript only supports safe integers up to 2^53-1.
 */
export class Int64Builder extends TransformableBuilder<
  TypeSchema & { type: "int64" }
> {
  /**
   * Provide default value if null/undefined.
   */
  default(value: number): this {
    return this.addTransform({ fn: "default", value });
  }

  /**
   * Require value in range.
   */
  range(min?: number, max?: number, options?: ConstraintOptions): this {
    const parts: string[] = [];
    if (min !== undefined) parts.push(`>= ${min}`);
    if (max !== undefined) parts.push(`<= ${max}`);
    return this.addConstraint(
      makeConstraint(
        { type: "range", min, max },
        "OUT_OF_RANGE",
        options?.message ?? `Must be ${parts.join(" and ")}`,
        options,
      ),
    );
  }

  /**
   * Require minimum value.
   */
  min(value: number, options?: ConstraintOptions): this {
    return this.range(value, undefined, options);
  }

  /**
   * Require maximum value.
   */
  max(value: number, options?: ConstraintOptions): this {
    return this.range(undefined, value, options);
  }

  toTypeSchema(): TypeSchema & { type: "int64" } {
    return {
      type: "int64",
      transforms: this._transforms.length > 0 ? this._transforms : undefined,
      constraints: this._constraints.length > 0 ? this._constraints : undefined,
    };
  }
}

/**
 * Builder for unsigned 32-bit integer schemas (0 to 4,294,967,295).
 */
export class Uint32Builder extends TransformableBuilder<
  TypeSchema & { type: "uint32" }
> {
  /**
   * Provide default value if null/undefined.
   */
  default(value: number): this {
    return this.addTransform({ fn: "default", value });
  }

  /**
   * Require value in range.
   */
  range(min?: number, max?: number, options?: ConstraintOptions): this {
    const parts: string[] = [];
    if (min !== undefined) parts.push(`>= ${min}`);
    if (max !== undefined) parts.push(`<= ${max}`);
    return this.addConstraint(
      makeConstraint(
        { type: "range", min, max },
        "OUT_OF_RANGE",
        options?.message ?? `Must be ${parts.join(" and ")}`,
        options,
      ),
    );
  }

  /**
   * Require maximum value.
   */
  max(value: number, options?: ConstraintOptions): this {
    return this.range(undefined, value, options);
  }

  toTypeSchema(): TypeSchema & { type: "uint32" } {
    return {
      type: "uint32",
      transforms: this._transforms.length > 0 ? this._transforms : undefined,
      constraints: this._constraints.length > 0 ? this._constraints : undefined,
    };
  }
}

/**
 * Builder for unsigned 64-bit integer schemas.
 * Note: JavaScript only supports safe integers up to 2^53-1.
 */
export class Uint64Builder extends TransformableBuilder<
  TypeSchema & { type: "uint64" }
> {
  /**
   * Provide default value if null/undefined.
   */
  default(value: number): this {
    return this.addTransform({ fn: "default", value });
  }

  /**
   * Require value in range.
   */
  range(min?: number, max?: number, options?: ConstraintOptions): this {
    const parts: string[] = [];
    if (min !== undefined) parts.push(`>= ${min}`);
    if (max !== undefined) parts.push(`<= ${max}`);
    return this.addConstraint(
      makeConstraint(
        { type: "range", min, max },
        "OUT_OF_RANGE",
        options?.message ?? `Must be ${parts.join(" and ")}`,
        options,
      ),
    );
  }

  /**
   * Require maximum value.
   */
  max(value: number, options?: ConstraintOptions): this {
    return this.range(undefined, value, options);
  }

  toTypeSchema(): TypeSchema & { type: "uint64" } {
    return {
      type: "uint64",
      transforms: this._transforms.length > 0 ? this._transforms : undefined,
      constraints: this._constraints.length > 0 ? this._constraints : undefined,
    };
  }
}

// ============================================================================
// Domain-Specific Numeric Types
// ============================================================================

/**
 * Builder for decimal schemas with configurable precision and scale.
 * Useful for exact decimal representation (unlike floating point).
 */
export class DecimalBuilder extends TransformableBuilder<
  TypeSchema & { type: "decimal" }
> {
  private _precision?: number;
  private _scale?: number;

  /**
   * Set precision (total digits).
   */
  precision(value: number): this {
    this._precision = value;
    return this;
  }

  /**
   * Set scale (decimal places).
   */
  scale(value: number): this {
    this._scale = value;
    return this;
  }

  /**
   * Provide default value if null/undefined.
   */
  default(value: number): this {
    return this.addTransform({ fn: "default", value });
  }

  /**
   * Require value in range.
   */
  range(min?: number, max?: number, options?: ConstraintOptions): this {
    const parts: string[] = [];
    if (min !== undefined) parts.push(`>= ${min}`);
    if (max !== undefined) parts.push(`<= ${max}`);
    return this.addConstraint(
      makeConstraint(
        { type: "range", min, max },
        "OUT_OF_RANGE",
        options?.message ?? `Must be ${parts.join(" and ")}`,
        options,
      ),
    );
  }

  /**
   * Require non-negative value (>= 0).
   */
  nonNegative(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_non_negative" },
        "NEGATIVE_VALUE",
        options?.message ?? "Must be non-negative",
        options,
      ),
    );
  }

  toTypeSchema(): TypeSchema & { type: "decimal" } {
    return {
      type: "decimal",
      precision: this._precision,
      scale: this._scale,
      transforms: this._transforms.length > 0 ? this._transforms : undefined,
      constraints: this._constraints.length > 0 ? this._constraints : undefined,
    };
  }
}

/**
 * Builder for percentage schemas.
 * Useful for tax rates, ownership percentages, withholding rates, etc.
 */
export class PercentageBuilder extends TransformableBuilder<
  TypeSchema & { type: "percentage" }
> {
  private _format: "decimal" | "whole" = "decimal";
  private _allowOver100 = false;
  private _scale?: number;

  /**
   * Use decimal format (0-1). This is the default.
   */
  decimal(): this {
    this._format = "decimal";
    return this;
  }

  /**
   * Use whole number format (0-100).
   */
  whole(): this {
    this._format = "whole";
    return this;
  }

  /**
   * Allow values over 100% (or 1.0 in decimal format).
   */
  allowOver100(): this {
    this._allowOver100 = true;
    return this;
  }

  /**
   * Set scale (decimal places).
   */
  scale(value: number): this {
    this._scale = value;
    return this;
  }

  /**
   * Provide default value if null/undefined.
   */
  default(value: number): this {
    return this.addTransform({ fn: "default", value });
  }

  toTypeSchema(): TypeSchema & { type: "percentage" } {
    return {
      type: "percentage",
      format: this._format,
      allow_over_100: this._allowOver100 || undefined,
      scale: this._scale,
      transforms: this._transforms.length > 0 ? this._transforms : undefined,
      constraints: this._constraints.length > 0 ? this._constraints : undefined,
    };
  }
}
