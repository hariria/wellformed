/**
 * String schema builder.
 */

import type { TemplateLiteralPart, TypeSchema } from "../ir/types.js";
import {
  type ConstraintOptions,
  makeConstraint,
  TransformableBuilder,
} from "./types.js";

/**
 * Builder for string schemas.
 */
export class StringBuilder extends TransformableBuilder<
  TypeSchema & { type: "string" }
> {
  // ============================================================================
  // Transforms
  // ============================================================================

  /**
   * Trim whitespace from both ends.
   */
  trim(): this {
    return this.addTransform({ fn: "trim" });
  }

  /**
   * Collapse multiple whitespace to single space and trim.
   */
  collapseWhitespace(): this {
    return this.addTransform({ fn: "collapse_whitespace" });
  }

  /**
   * Remove all non-digit characters.
   */
  digitsOnly(): this {
    return this.addTransform({ fn: "digits_only" });
  }

  /**
   * Convert to uppercase.
   */
  upper(): this {
    return this.addTransform({ fn: "upper" });
  }

  /**
   * Convert to lowercase.
   */
  lower(): this {
    return this.addTransform({ fn: "lower" });
  }

  /**
   * Replace pattern with replacement.
   */
  replace(pattern: string, replacement: string): this {
    return this.addTransform({ fn: "replace", pattern, replacement });
  }

  /**
   * Normalize a flight number (uppercase + remove spaces/hyphens).
   */
  normalizeFlightNumber(): this {
    return this.addTransform({ fn: "normalize_flight_number" });
  }

  /**
   * Normalize an ICD-10 code to canonical dotted uppercase form.
   */
  normalizeIcd10(): this {
    return this.addTransform({ fn: "normalize_icd10" });
  }

  /**
   * Normalize a CPT code to uppercase alphanumeric form.
   */
  normalizeCpt(): this {
    return this.addTransform({ fn: "normalize_cpt" });
  }

  /**
   * Normalize an HCPCS code to uppercase alphanumeric form.
   */
  normalizeHcpcs(): this {
    return this.addTransform({ fn: "normalize_hcpcs" });
  }

  /**
   * Normalize an NDC code to 11-digit format when possible.
   */
  normalizeNdc11(): this {
    return this.addTransform({ fn: "normalize_ndc11" });
  }

  /**
   * Provide default value if null/undefined.
   */
  default(value: string): this {
    return this.addTransform({ fn: "default", value });
  }

  /**
   * Format number with thousands separators.
   */
  formatThousands(options?: { separator?: string }): this {
    return this.addTransform({
      fn: "format_thousands",
      separator: options?.separator,
    });
  }

  /**
   * Format number to fixed decimal places.
   */
  formatDecimal(places: number): this {
    return this.addTransform({ fn: "format_decimal", places });
  }

  // ============================================================================
  // Basic Constraints
  // ============================================================================

  /**
   * Require regex pattern match.
   */
  regex(
    pattern: string,
    options?: ConstraintOptions & { flags?: string },
  ): this {
    return this.addConstraint(
      makeConstraint(
        { type: "regex", pattern, flags: options?.flags },
        "INVALID_FORMAT",
        options?.message ?? `Must match pattern: ${pattern}`,
        options,
      ),
    );
  }

  /**
   * Require a template literal-style pattern with linear scanning.
   */
  templateLiteral(
    parts: Array<string | TemplateLiteralPart>,
    options?: ConstraintOptions,
  ): this {
    const normalized: TemplateLiteralPart[] = parts.map((part) =>
      typeof part === "string" ? { kind: "literal", value: part } : part,
    );
    return this.addConstraint(
      makeConstraint(
        { type: "template_literal", parts: normalized },
        "INVALID_TEMPLATE_LITERAL",
        options?.message ?? "Invalid template literal format",
        options,
      ),
    );
  }

  /**
   * Require the string to start with the given prefix.
   */
  startsWith(prefix: string, options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "starts_with", args: { value: prefix } },
        "INVALID_PREFIX",
        options?.message ?? `Must start with "${prefix}"`,
        options,
      ),
    );
  }

  /**
   * Require the string to end with the given suffix.
   */
  endsWith(suffix: string, options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "ends_with", args: { value: suffix } },
        "INVALID_SUFFIX",
        options?.message ?? `Must end with "${suffix}"`,
        options,
      ),
    );
  }

  /**
   * Require the string to contain the given substring.
   */
  includes(substr: string, options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "contains", args: { value: substr } },
        "MISSING_SUBSTRING",
        options?.message ?? `Must include "${substr}"`,
        options,
      ),
    );
  }

  /**
   * Require minimum length.
   */
  minLen(len: number, options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "min_len", len },
        "TOO_SHORT",
        options?.message ?? `Must be at least ${len} characters`,
        options,
      ),
    );
  }

  /**
   * Require maximum length.
   */
  maxLen(len: number, options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "max_len", len },
        "TOO_LONG",
        options?.message ?? `Must be at most ${len} characters`,
        options,
      ),
    );
  }

  /**
   * Require exact length.
   */
  length(len: number, options?: ConstraintOptions): this {
    return this.minLen(len, options).maxLen(len, options);
  }

  /**
   * Require non-empty string.
   */
  nonEmpty(options?: ConstraintOptions): this {
    return this.minLen(1, { message: "Cannot be empty", ...options });
  }

  /**
   * Validate number of decimal places.
   */
  decimalPlaces(
    places: number,
    options?: ConstraintOptions & { max?: boolean },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_decimal_places",
          args: { places, max: options?.max },
        },
        "INVALID_DECIMAL_PLACES",
        options?.message ??
          (options?.max
            ? `Must have at most ${places} decimal places`
            : `Must have exactly ${places} decimal places`),
        options,
      ),
    );
  }

  // ============================================================================
  // Numeric Type Predicates
  // ============================================================================

  /**
   * Validate value is a whole number (integer).
   */
  integer(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_integer" },
        "NOT_INTEGER",
        options?.message ?? "Must be a whole number",
        options,
      ),
    );
  }

  /**
   * Validate value is a valid floating point number.
   */
  float(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_float" },
        "NOT_FLOAT",
        options?.message ?? "Must be a number",
        options,
      ),
    );
  }

  /**
   * Validate value fits in u8 (0 to 255).
   */
  u8(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_u8" },
        "OUT_OF_RANGE",
        options?.message ?? "Must be 0-255",
        options,
      ),
    );
  }

  /**
   * Validate value fits in u16 (0 to 65535).
   */
  u16(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_u16" },
        "OUT_OF_RANGE",
        options?.message ?? "Must be 0-65535",
        options,
      ),
    );
  }

  /**
   * Validate value fits in u32 (0 to 4294967295).
   */
  u32(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_u32" },
        "OUT_OF_RANGE",
        options?.message ?? "Must be 0-4294967295",
        options,
      ),
    );
  }

  /**
   * Validate value fits in u64.
   */
  u64(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_u64" },
        "OUT_OF_RANGE",
        options?.message ?? "Must be a non-negative 64-bit integer",
        options,
      ),
    );
  }

  /**
   * Validate value fits in i8 (-128 to 127).
   */
  i8(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_i8" },
        "OUT_OF_RANGE",
        options?.message ?? "Must be -128 to 127",
        options,
      ),
    );
  }

  /**
   * Validate value fits in i16 (-32768 to 32767).
   */
  i16(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_i16" },
        "OUT_OF_RANGE",
        options?.message ?? "Must be -32768 to 32767",
        options,
      ),
    );
  }

  /**
   * Validate value fits in i32 (-2147483648 to 2147483647).
   */
  i32(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_i32" },
        "OUT_OF_RANGE",
        options?.message ?? "Must be a 32-bit integer",
        options,
      ),
    );
  }

  /**
   * Validate value fits in i64.
   */
  i64(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_i64" },
        "OUT_OF_RANGE",
        options?.message ?? "Must be a 64-bit integer",
        options,
      ),
    );
  }

  // ============================================================================
  // TIN Predicates
  // ============================================================================

  /**
   * Validate as any TIN (SSN, EIN, ITIN, or ATIN).
   */
  tin(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_tin" },
        "INVALID_TIN",
        options?.message ?? "Invalid taxpayer identification number",
        options,
      ),
    );
  }

  /**
   * Validate as SSN.
   */
  ssn(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_ssn" },
        "INVALID_SSN",
        options?.message ?? "Invalid Social Security Number",
        options,
      ),
    );
  }

  /**
   * Validate as EIN.
   */
  ein(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_ein" },
        "INVALID_EIN",
        options?.message ?? "Invalid Employer Identification Number",
        options,
      ),
    );
  }

  /**
   * Validate as ITIN.
   */
  itin(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_itin" },
        "INVALID_ITIN",
        options?.message ?? "Invalid Individual Taxpayer Identification Number",
        options,
      ),
    );
  }

  /**
   * Validate as ATIN.
   */
  atin(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_atin" },
        "INVALID_ATIN",
        options?.message ?? "Invalid Adoption Taxpayer Identification Number",
        options,
      ),
    );
  }

  /**
   * Validate Luhn checksum.
   */
  luhn(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "luhn" },
        "INVALID_CHECKSUM",
        options?.message ?? "Invalid checksum",
        options,
      ),
    );
  }

  /**
   * Format digits as SSN: XXX-XX-XXXX.
   */
  formatSsn(): this {
    return this.addTransform({ fn: "format_ssn" });
  }

  /**
   * Format digits as EIN: XX-XXXXXXX.
   */
  formatEin(): this {
    return this.addTransform({ fn: "format_ein" });
  }

  /**
   * Mask SSN showing only last 4 digits: ***-**-XXXX.
   */
  maskSsn(): this {
    return this.addTransform({ fn: "mask_ssn" });
  }

  /**
   * Mask EIN showing only last 4 digits: **-***XXXX.
   */
  maskEin(): this {
    return this.addTransform({ fn: "mask_ein" });
  }

  // ============================================================================
  // Financial Predicates
  // ============================================================================

  /**
   * Validate as CUSIP.
   */
  cusip(options?: ConstraintOptions & { validateChecksum?: boolean }): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_cusip",
          args: { validate_checksum: options?.validateChecksum },
        },
        "INVALID_CUSIP",
        options?.message ?? "Invalid CUSIP",
        options,
      ),
    );
  }

  /**
   * Validate as ABA routing number.
   */
  abaRouting(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_aba_routing" },
        "INVALID_ROUTING",
        options?.message ?? "Invalid ABA routing number",
        options,
      ),
    );
  }

  /**
   * Validate as MCC (Merchant Category Code).
   */
  mcc(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_mcc" },
        "INVALID_MCC",
        options?.message ?? "Invalid merchant category code",
        options,
      ),
    );
  }

  /**
   * Validate as account number.
   */
  accountNumber(
    options?: ConstraintOptions & {
      minLen?: number;
      maxLen?: number;
      allowHyphens?: boolean;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_account_number",
          args: {
            min_len: options?.minLen,
            max_len: options?.maxLen,
            allow_hyphens: options?.allowHyphens,
          },
        },
        "INVALID_ACCOUNT",
        options?.message ?? "Invalid account number",
        options,
      ),
    );
  }

  // ============================================================================
  // Payment Card Predicates
  // ============================================================================

  /**
   * Validate as credit card number (Luhn + IIN prefix/length).
   */
  creditCard(
    options?: ConstraintOptions & {
      network?: "visa" | "mastercard" | "amex" | "discover";
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_credit_card",
          args: { network: options?.network },
        },
        "INVALID_CREDIT_CARD",
        options?.message ?? "Invalid credit card number",
        options,
      ),
    );
  }

  /**
   * Validate as CVV code.
   */
  cvv(
    options?: ConstraintOptions & {
      network?: "visa" | "mastercard" | "amex" | "discover";
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_cvv",
          args: { network: options?.network },
        },
        "INVALID_CVV",
        options?.message ?? "Invalid CVV",
        options,
      ),
    );
  }

  /**
   * Validate as card expiry date (MM/YY or MM/YYYY).
   */
  cardExpiry(options?: ConstraintOptions & { rejectExpired?: boolean }): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_card_expiry",
          args: { reject_expired: options?.rejectExpired },
        },
        "INVALID_CARD_EXPIRY",
        options?.message ?? "Invalid card expiry date",
        options,
      ),
    );
  }

  /**
   * Mask card number showing only last 4 digits.
   */
  maskCardNumber(): this {
    return this.addTransform({ fn: "card_mask_last4" });
  }

  /**
   * Format digits as credit card with spaces every 4 digits.
   */
  formatCreditCard(): this {
    return this.addTransform({ fn: "format_credit_card" });
  }

  // ============================================================================
  // International Banking Predicates
  // ============================================================================

  /**
   * Validate as IBAN (International Bank Account Number).
   */
  iban(options?: ConstraintOptions & { country?: string }): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_iban",
          args: { country: options?.country },
        },
        "INVALID_IBAN",
        options?.message ?? "Invalid IBAN",
        options,
      ),
    );
  }

  /**
   * Validate as BIC/SWIFT code.
   */
  bic(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_bic" },
        "INVALID_BIC",
        options?.message ?? "Invalid BIC/SWIFT code",
        options,
      ),
    );
  }

  /**
   * Validate as SWIFT code (alias for .bic()).
   */
  swift(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_swift" },
        "INVALID_SWIFT",
        options?.message ?? "Invalid SWIFT code",
        options,
      ),
    );
  }

  /**
   * Format IBAN with spaces every 4 characters.
   */
  formatIban(): this {
    return this.addTransform({ fn: "format_iban" });
  }

  // ============================================================================
  // Date Predicates
  // ============================================================================

  /**
   * Validate as time string (HH:MM, HH:MM:SS, 12h/24h).
   */
  time(options?: ConstraintOptions & { format?: "24h" | "12h" }): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_time",
          args: { format: options?.format },
        },
        "INVALID_TIME",
        options?.message ?? "Invalid time",
        options,
      ),
    );
  }

  /**
   * Validate time is before a given time.
   */
  timeBefore(
    targetTime: string,
    options?: ConstraintOptions & { allowEqual?: boolean },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "time_before",
          args: { time: targetTime, allow_equal: options?.allowEqual },
        },
        "TIME_TOO_LATE",
        options?.message ?? `Must be before ${targetTime}`,
        options,
      ),
    );
  }

  /**
   * Validate time is after a given time.
   */
  timeAfter(
    targetTime: string,
    options?: ConstraintOptions & { allowEqual?: boolean },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "time_after",
          args: { time: targetTime, allow_equal: options?.allowEqual },
        },
        "TIME_TOO_EARLY",
        options?.message ?? `Must be after ${targetTime}`,
        options,
      ),
    );
  }

  /**
   * Validate time is within a range. Supports overnight ranges (min > max wraps around midnight).
   */
  timeInRange(
    options: ConstraintOptions & {
      min?: string;
      max?: string;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "time_in_range",
          args: {
            min: options.min,
            max: options.max,
          },
        },
        "TIME_OUT_OF_RANGE",
        options?.message ?? "Time out of range",
        options,
      ),
    );
  }

  /**
   * Validate as ISO 8601 datetime.
   */
  isoDatetime(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_iso_datetime" },
        "INVALID_DATETIME",
        options?.message ?? "Invalid ISO 8601 datetime",
        options,
      ),
    );
  }

  /**
   * Validate as date string.
   */
  date(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_date" },
        "INVALID_DATE",
        options?.message ?? "Invalid date",
        options,
      ),
    );
  }

  /**
   * Validate date is in range.
   */
  dateInRange(
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
  dateBefore(
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
  dateAfter(
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

  // ============================================================================
  // Reference Predicates
  // ============================================================================

  /**
   * Validate as ISO country code.
   */
  countryCode(options?: ConstraintOptions & { excludeUs?: boolean }): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_country_code",
          args: { exclude_us: options?.excludeUs },
        },
        "INVALID_COUNTRY",
        options?.message ?? "Invalid country code",
        options,
      ),
    );
  }

  /**
   * Validate as ISO 4217 currency code.
   */
  currencyCode(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_currency_code" },
        "INVALID_CURRENCY",
        options?.message ?? "Invalid currency code",
        options,
      ),
    );
  }

  /**
   * Validate as US state code.
   */
  usState(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_us_state" },
        "INVALID_STATE",
        options?.message ?? "Invalid US state code",
        options,
      ),
    );
  }

  /**
   * Validate as US ZIP code.
   */
  usZip(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_us_zip" },
        "INVALID_ZIP",
        options?.message ?? "Invalid ZIP code",
        options,
      ),
    );
  }

  /**
   * Validate as street address (starts with a number, contains letters, min 5 chars).
   */
  streetAddress(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_street_address" },
        "INVALID_STREET_ADDRESS",
        options?.message ?? "Invalid street address",
        options,
      ),
    );
  }

  // ============================================================================
  // Text character class predicates
  // ============================================================================

  /**
   * Validate as alphabetic characters only (A-Za-z).
   */
  alpha(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_alpha" },
        "INVALID_ALPHA",
        options?.message ?? "Must contain only letters",
        options,
      ),
    );
  }

  /**
   * Validate as digits only (0-9).
   */
  digits(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_digits" },
        "INVALID_DIGITS",
        options?.message ?? "Must contain only digits",
        options,
      ),
    );
  }

  /**
   * Validate as alphanumeric characters only (A-Za-z0-9).
   */
  alphanumeric(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_alphanumeric" },
        "INVALID_ALPHANUMERIC",
        options?.message ?? "Must contain only letters and digits",
        options,
      ),
    );
  }

  /**
   * Validate as alphabetic characters and spaces only, with at least one letter.
   */
  alphaSpaces(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_alpha_spaces" },
        "INVALID_ALPHA_SPACES",
        options?.message ?? "Must contain only letters and spaces",
        options,
      ),
    );
  }

  /**
   * Validate as alphanumeric characters and spaces only, with at least one letter or digit.
   */
  alphanumericSpaces(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_alphanumeric_spaces" },
        "INVALID_ALPHANUMERIC_SPACES",
        options?.message ?? "Must contain only letters, digits, and spaces",
        options,
      ),
    );
  }

  /**
   * Validate as name characters only (letters, hyphens, apostrophes), with at least one letter.
   */
  nameChars(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_name_chars" },
        "INVALID_NAME_CHARS",
        options?.message ??
          "Must contain only letters, hyphens, and apostrophes",
        options,
      ),
    );
  }

  /**
   * Validate as uppercase letters only (A-Z).
   */
  uppercase(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_uppercase" },
        "INVALID_UPPERCASE",
        options?.message ?? "Must contain only uppercase letters",
        options,
      ),
    );
  }

  /**
   * Validate as lowercase letters only (a-z).
   */
  lowercase(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_lowercase" },
        "INVALID_LOWERCASE",
        options?.message ?? "Must contain only lowercase letters",
        options,
      ),
    );
  }

  /**
   * Validate as title case (starts with uppercase letter followed by one or more lowercase letters).
   */
  titleCase(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_title_case" },
        "INVALID_TITLE_CASE",
        options?.message ?? "Must be title case",
        options,
      ),
    );
  }

  /**
   * Validate as W-2 Box 12 code.
   */
  w2Box12Code(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_w2_box12_code" },
        "INVALID_BOX12_CODE",
        options?.message ?? "Invalid W-2 Box 12 code",
        options,
      ),
    );
  }

  /**
   * Validate as 1099-B code.
   */
  code1099B(
    options?: ConstraintOptions & { codeType?: "term" | "basis" | "type" },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_1099b_code",
          args: { type: options?.codeType },
        },
        "INVALID_1099B_CODE",
        options?.message ?? "Invalid 1099-B code",
        options,
      ),
    );
  }

  // ============================================================================
  // Product / Ecommerce Predicates
  // ============================================================================

  /**
   * Validate as VIN (Vehicle Identification Number).
   */
  vin(options?: ConstraintOptions & { validateChecksum?: boolean }): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_vin",
          args: { validate_checksum: options?.validateChecksum },
        },
        "INVALID_VIN",
        options?.message ?? "Invalid VIN",
        options,
      ),
    );
  }

  /**
   * Validate as UPC-A barcode (12 digits with check digit).
   */
  upc(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_upc" },
        "INVALID_UPC",
        options?.message ?? "Invalid UPC",
        options,
      ),
    );
  }

  /**
   * Validate as EAN-13 barcode (13 digits with check digit).
   */
  ean(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_ean" },
        "INVALID_EAN",
        options?.message ?? "Invalid EAN",
        options,
      ),
    );
  }

  /**
   * Validate as ISBN (ISBN-10 or ISBN-13).
   */
  isbn(options?: ConstraintOptions & { version?: 10 | 13 }): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_isbn",
          args: { version: options?.version },
        },
        "INVALID_ISBN",
        options?.message ?? "Invalid ISBN",
        options,
      ),
    );
  }

  // ============================================================================
  // Aviation Predicates
  // ============================================================================

  /**
   * Validate as IATA airport code (3 letters, e.g. SFO).
   */
  iataAirportCode(
    options?: ConstraintOptions & {
      knownOnly?: boolean;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_iata_airport_code",
          args: { known_only: options?.knownOnly },
        },
        "INVALID_IATA_AIRPORT_CODE",
        options?.message ?? "Invalid IATA airport code",
        options,
      ),
    );
  }

  /**
   * Validate as ICAO airport code (4 letters, e.g. KSFO).
   */
  icaoAirportCode(
    options?: ConstraintOptions & {
      knownOnly?: boolean;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_icao_airport_code",
          args: { known_only: options?.knownOnly },
        },
        "INVALID_ICAO_AIRPORT_CODE",
        options?.message ?? "Invalid ICAO airport code",
        options,
      ),
    );
  }

  /**
   * Validate as IATA airline code (2 chars, e.g. UA, B6).
   */
  iataAirlineCode(
    options?: ConstraintOptions & {
      knownOnly?: boolean;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_iata_airline_code",
          args: { known_only: options?.knownOnly },
        },
        "INVALID_IATA_AIRLINE_CODE",
        options?.message ?? "Invalid IATA airline code",
        options,
      ),
    );
  }

  /**
   * Validate as ICAO airline code (3 letters, e.g. UAL).
   */
  icaoAirlineCode(
    options?: ConstraintOptions & {
      knownOnly?: boolean;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_icao_airline_code",
          args: { known_only: options?.knownOnly },
        },
        "INVALID_ICAO_AIRLINE_CODE",
        options?.message ?? "Invalid ICAO airline code",
        options,
      ),
    );
  }

  /**
   * Validate as flight number (e.g. UA123, UAL1234A).
   */
  flightNumber(
    options?: ConstraintOptions & {
      carrierFormat?: "ANY" | "IATA" | "ICAO";
      knownCarrier?: boolean;
      allowSuffix?: boolean;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_flight_number",
          args: {
            carrier_format: options?.carrierFormat,
            known_carrier: options?.knownCarrier,
            allow_suffix: options?.allowSuffix,
          },
        },
        "INVALID_FLIGHT_NUMBER",
        options?.message ?? "Invalid flight number",
        options,
      ),
    );
  }

  /**
   * Validate as airport code in IATA/ICAO/ANY system.
   */
  airportCode(
    options?: ConstraintOptions & {
      system?: "ANY" | "IATA" | "ICAO";
      knownOnly?: boolean;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_airport_code",
          args: {
            system: options?.system,
            known_only: options?.knownOnly,
          },
        },
        "INVALID_AIRPORT_CODE",
        options?.message ?? "Invalid airport code",
        options,
      ),
    );
  }

  /**
   * Validate as airline code in IATA/ICAO/ANY system.
   */
  airlineCode(
    options?: ConstraintOptions & {
      system?: "ANY" | "IATA" | "ICAO";
      knownOnly?: boolean;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_airline_code",
          args: {
            system: options?.system,
            known_only: options?.knownOnly,
          },
        },
        "INVALID_AIRLINE_CODE",
        options?.message ?? "Invalid airline code",
        options,
      ),
    );
  }

  // ============================================================================
  // Color Predicates
  // ============================================================================

  /**
   * Validate as hex color (#RGB, #RRGGBB, with optional alpha).
   */
  hexColor(
    options?: ConstraintOptions & {
      allowAlpha?: boolean;
      requireHash?: boolean;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_hex_color",
          args: {
            allow_alpha: options?.allowAlpha,
            require_hash: options?.requireHash,
          },
        },
        "INVALID_HEX_COLOR",
        options?.message ?? "Invalid hex color",
        options,
      ),
    );
  }

  /**
   * Validate as RGB/RGBA color string.
   */
  rgbColor(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_rgb_color" },
        "INVALID_RGB_COLOR",
        options?.message ?? "Invalid RGB color",
        options,
      ),
    );
  }

  /**
   * Validate as HSL/HSLA color string.
   */
  hslColor(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_hsl_color" },
        "INVALID_HSL_COLOR",
        options?.message ?? "Invalid HSL color",
        options,
      ),
    );
  }

  // ============================================================================
  // Text Analysis Predicates
  // ============================================================================

  /**
   * Detect if text contains RTL (right-to-left) characters.
   */
  rtl(options?: ConstraintOptions & { threshold?: number }): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_rtl",
          args: { threshold: options?.threshold },
        },
        "NOT_RTL",
        options?.message ?? "Must contain RTL text",
        options,
      ),
    );
  }

  /**
   * Validate text is exclusively LTR (no RTL characters).
   */
  ltr(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_ltr" },
        "NOT_LTR",
        options?.message ?? "Must not contain RTL characters",
        options,
      ),
    );
  }

  // ============================================================================
  // Insurance / Healthcare Predicates
  // ============================================================================

  /**
   * Validate as NPI (National Provider Identifier).
   */
  npi(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_npi" },
        "INVALID_NPI",
        options?.message ?? "Invalid NPI",
        options,
      ),
    );
  }

  /**
   * Validate as DEA registration number.
   */
  deaNumber(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_dea_number" },
        "INVALID_DEA_NUMBER",
        options?.message ?? "Invalid DEA number",
        options,
      ),
    );
  }

  /**
   * Validate as ICD-10 diagnosis code.
   */
  icd10Code(
    options?: ConstraintOptions & {
      strictFormat?: boolean;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_icd10_code",
          args: { strict_format: options?.strictFormat },
        },
        "INVALID_ICD10_CODE",
        options?.message ?? "Invalid ICD-10 code",
        options,
      ),
    );
  }

  /**
   * Validate as CPT code.
   */
  cptCode(
    options?: ConstraintOptions & {
      allowCategoryIi?: boolean;
      allowCategoryIii?: boolean;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_cpt_code",
          args: {
            allow_category_ii: options?.allowCategoryIi,
            allow_category_iii: options?.allowCategoryIii,
          },
        },
        "INVALID_CPT_CODE",
        options?.message ?? "Invalid CPT code",
        options,
      ),
    );
  }

  /**
   * Validate as HCPCS Level II code.
   */
  hcpcsCode(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_hcpcs_code" },
        "INVALID_HCPCS_CODE",
        options?.message ?? "Invalid HCPCS code",
        options,
      ),
    );
  }

  /**
   * Validate as NDC code (10-digit or 11-digit).
   */
  ndcCode(
    options?: ConstraintOptions & {
      format?: "10" | "11";
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_ndc_code",
          args: { format: options?.format },
        },
        "INVALID_NDC_CODE",
        options?.message ?? "Invalid NDC code",
        options,
      ),
    );
  }

  // ============================================================================
  // Encoding / Crypto Predicates
  // ============================================================================

  /**
   * Validate as base58-encoded string (Bitcoin alphabet).
   */
  base58(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_base58" },
        "INVALID_BASE58",
        options?.message ?? "Invalid base58 encoding",
        options,
      ),
    );
  }

  /**
   * Validate as base64-encoded string.
   */
  base64(options?: ConstraintOptions & { urlSafe?: boolean }): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_base64",
          args: { url_safe: options?.urlSafe },
        },
        "INVALID_BASE64",
        options?.message ?? "Invalid base64 encoding",
        options,
      ),
    );
  }

  /**
   * Validate as Bitcoin address (P2PKH, P2SH, Bech32, Taproot).
   */
  bitcoinAddress(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_bitcoin_address" },
        "INVALID_BITCOIN_ADDRESS",
        options?.message ?? "Invalid Bitcoin address",
        options,
      ),
    );
  }

  /**
   * Validate as Ethereum address (0x + 40 hex chars).
   */
  ethereumAddress(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_ethereum_address" },
        "INVALID_ETHEREUM_ADDRESS",
        options?.message ?? "Invalid Ethereum address",
        options,
      ),
    );
  }

  /**
   * Validate as Solana address (base58, 32-44 chars).
   */
  solanaAddress(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_solana_address" },
        "INVALID_SOLANA_ADDRESS",
        options?.message ?? "Invalid Solana address",
        options,
      ),
    );
  }

  /**
   * Validate as compact JWT string.
   */
  jwt(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_jwt" },
        "INVALID_JWT",
        options?.message ?? "Invalid JWT",
        options,
      ),
    );
  }

  /**
   * Validate as a hexadecimal hash digest.
   */
  hash(
    options?: ConstraintOptions & {
      algorithm?: "md5" | "sha1" | "sha224" | "sha256" | "sha384" | "sha512";
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_hash",
          args: { algorithm: options?.algorithm },
        },
        "INVALID_HASH",
        options?.message ?? "Invalid hash",
        options,
      ),
    );
  }

  // ============================================================================
  // Contact Predicates
  // ============================================================================

  /**
   * Validate as phone number (US or international).
   */
  phone(
    options?: ConstraintOptions & {
      requireAreaCode?: boolean;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "phone_number",
          args: {
            require_area_code: options?.requireAreaCode,
          },
        },
        "INVALID_PHONE",
        options?.message ?? "Invalid phone number",
        options,
      ),
    );
  }

  /**
   * Validate as US phone number only (rejects international format).
   */
  phoneUS(
    options?: ConstraintOptions & {
      requireAreaCode?: boolean;
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "phone_number_us",
          args: {
            require_area_code: options?.requireAreaCode,
          },
        },
        "INVALID_PHONE",
        options?.message ?? "Invalid US phone number",
        options,
      ),
    );
  }

  /**
   * Format phone number as (XXX) XXX-XXXX.
   */
  formatPhoneUS(): this {
    return this.addTransform({ fn: "phone_us" });
  }

  /**
   * Normalize phone number to E.164 (e.g. +16501234567).
   */
  formatPhoneE164(): this {
    return this.addTransform({ fn: "phone_e164" });
  }

  /**
   * Validate as UUID.
   */
  uuid(options?: ConstraintOptions & { version?: number }): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_uuid",
          args: { version: options?.version },
        },
        "INVALID_UUID",
        options?.message ?? "Invalid UUID",
        options,
      ),
    );
  }

  /**
   * Validate as email address.
   */
  email(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_email" },
        "INVALID_EMAIL",
        options?.message ?? "Invalid email address",
        options,
      ),
    );
  }

  /**
   * Validate as URL (http or https).
   */
  url(options?: ConstraintOptions & { requireHttps?: boolean }): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_url",
          args: { require_https: options?.requireHttps },
        },
        "INVALID_URL",
        options?.message ?? "Invalid URL",
        options,
      ),
    );
  }

  /**
   * Validate as IP address.
   */
  ip(
    options?: ConstraintOptions & {
      version?: "v4" | "v6";
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_ip",
          args: { version: options?.version },
        },
        "INVALID_IP",
        options?.message ?? "Invalid IP address",
        options,
      ),
    );
  }

  /**
   * Validate as CIDR block.
   */
  cidr(
    options?: ConstraintOptions & {
      version?: "v4" | "v6";
    },
  ): this {
    return this.addConstraint(
      makeConstraint(
        {
          type: "call",
          name: "is_cidr",
          args: { version: options?.version },
        },
        "INVALID_CIDR",
        options?.message ?? "Invalid CIDR block",
        options,
      ),
    );
  }

  /**
   * Validate as MAC address.
   */
  macAddress(options?: ConstraintOptions): this {
    return this.addConstraint(
      makeConstraint(
        { type: "call", name: "is_mac_address" },
        "INVALID_MAC_ADDRESS",
        options?.message ?? "Invalid MAC address",
        options,
      ),
    );
  }

  // ============================================================================
  // Output
  // ============================================================================

  toTypeSchema(): TypeSchema & { type: "string" } {
    return {
      type: "string",
      transforms: this._transforms.length > 0 ? this._transforms : undefined,
      constraints: this._constraints.length > 0 ? this._constraints : undefined,
    };
  }
}
