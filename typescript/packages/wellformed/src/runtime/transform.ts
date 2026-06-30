// biome-ignore-all lint/complexity/noExcessiveCognitiveComplexity: Transform parsing intentionally keeps individual normalization cases together.
/**
 * Transform execution - applies transforms to normalize data.
 */

import type { Transform } from "../ir/types.js";

/**
 * Round half away from zero (so -2.5 -> -3, matching the Rust runtime's
 * f64::round). JavaScript's Math.round rounds half toward +Infinity
 * (-2.5 -> -2), which would diverge across runtimes for negative money values.
 * See conformance: money-to-cents-negative-half.
 */
function roundTiesAwayFromZero(n: number): number {
  return Math.sign(n) * Math.round(Math.abs(n));
}

/**
 * Format `f` to `places` decimals using the same multiply-then-round model as
 * money_to_cents, then build the string from the rounded integer. This is
 * deterministic across runtimes (the Rust runtime uses the identical algorithm)
 * and keeps format_decimal consistent with money_to_cents within a runtime. We
 * deliberately do NOT use Number.toFixed, whose true-value rounding model would
 * disagree with both. See conformance: format-decimal-half-rounding.
 */
function formatDecimalPlaces(f: number, places: number): string {
  if (!Number.isFinite(f)) {
    return String(f);
  }
  const factor = 10 ** places;
  const scaled = roundTiesAwayFromZero(f * factor);
  const negative = scaled < 0;
  const magnitude = Math.abs(scaled);
  if (places === 0) {
    return `${negative ? "-" : ""}${magnitude}`;
  }
  const intPart = Math.floor(magnitude / factor);
  const fracPart = magnitude - intPart * factor;
  return `${negative ? "-" : ""}${intPart}.${String(fracPart).padStart(places, "0")}`;
}

/**
 * Apply a single transform to a value.
 */
export function applyTransform(value: unknown, transform: Transform): unknown {
  // Most transforms only work on strings
  if (typeof value !== "string") {
    // Handle money_to_cents for numbers
    if (transform.fn === "money_to_cents" && typeof value === "number") {
      const scale = transform.scale ?? 2;
      return roundTiesAwayFromZero(value * 10 ** scale);
    }
    // Handle default for null/undefined
    if (transform.fn === "default" && (value === null || value === undefined)) {
      return transform.value;
    }
    // Handle format_thousands for numbers
    if (transform.fn === "format_thousands" && typeof value === "number") {
      const sep = transform.separator ?? ",";
      return formatWithThousands(String(value), sep) ?? String(value);
    }
    // Handle format_decimal for numbers
    if (transform.fn === "format_decimal" && typeof value === "number") {
      return formatDecimalPlaces(value, transform.places ?? 0);
    }
    return value;
  }

  switch (transform.fn) {
    case "trim":
      return value.trim();

    case "collapse_whitespace":
      return value.replace(/\s+/g, " ").trim();

    case "digits_only":
      return value.replace(/\D/g, "");

    case "upper":
      return value.toUpperCase();

    case "lower":
      return value.toLowerCase();

    case "money_to_cents": {
      const scale = transform.scale ?? 2;
      const cleaned = cleanNumberString(value);
      const num = parseStrictFiniteNumber(cleaned);
      if (num === null) {
        return value; // Can't parse, return as-is
      }
      return roundTiesAwayFromZero(num * 10 ** scale);
    }

    case "date_parse": {
      // Parse date from format to canonical YYYY-MM-DD
      const parsed = parseDateWithFormat(value, transform.format);
      return parsed ?? value;
    }

    case "replace": {
      const { pattern, replacement } = transform;
      if (pattern === "") {
        return `${replacement}${Array.from(value).join(replacement)}${replacement}`;
      }
      return value.split(pattern).join(replacement);
    }

    case "normalize_flight_number":
      return value
        .trim()
        .replace(/[\s-]+/g, "")
        .toUpperCase();

    case "normalize_icd10": {
      const normalized = normalizeIcd10Code(value);
      return normalized ?? value;
    }

    case "normalize_cpt": {
      const normalized = normalizeCptCode(value);
      return normalized ?? value;
    }

    case "normalize_hcpcs": {
      const normalized = normalizeHcpcsCode(value);
      return normalized ?? value;
    }

    case "normalize_ndc11": {
      const normalized = normalizeNdc11Code(value);
      return normalized ?? value;
    }

    case "default":
      // String is not null/undefined, return as-is
      return value;

    case "phone_us": {
      // Extract digits from value
      const digits = value.replace(/\D/g, "");
      let normalized = digits;

      // If 11 digits starting with 1, drop the leading 1
      if (digits.length === 11 && digits[0] === "1") {
        normalized = digits.slice(1);
      }

      // If 10 digits, format as (XXX) XXX-XXXX
      if (normalized.length === 10) {
        return `(${normalized.slice(0, 3)}) ${normalized.slice(3, 6)}-${normalized.slice(6)}`;
      }

      // Otherwise return value as-is
      return value;
    }

    case "phone_e164": {
      const trimmed = value.trim();
      const digits = value.replace(/\D/g, "");

      if (trimmed.startsWith("+")) {
        if (digits.length >= 7 && digits.length <= 15) {
          return `+${digits}`;
        }
        return value;
      }

      if (digits.length === 10) {
        return `+1${digits}`;
      }

      if (digits.length === 11 && digits.startsWith("1")) {
        return `+${digits}`;
      }

      return value;
    }

    case "card_mask_last4": {
      const digits = value.replace(/\D/g, "");
      if (digits.length > 4) {
        const maskedCount = digits.length - 4;
        return "*".repeat(maskedCount) + digits.slice(maskedCount);
      }
      return digits;
    }

    case "format_ssn": {
      const digits = value.replace(/\D/g, "");
      if (digits.length === 9) {
        return `${digits.slice(0, 3)}-${digits.slice(3, 5)}-${digits.slice(5)}`;
      }
      return value;
    }

    case "format_ein": {
      const digits = value.replace(/\D/g, "");
      if (digits.length === 9) {
        return `${digits.slice(0, 2)}-${digits.slice(2)}`;
      }
      return value;
    }

    case "mask_ssn": {
      const digits = value.replace(/\D/g, "");
      if (digits.length === 9) {
        return `***-**-${digits.slice(5)}`;
      }
      return value;
    }

    case "mask_ein": {
      const digits = value.replace(/\D/g, "");
      if (digits.length === 9) {
        return `**-***${digits.slice(5)}`;
      }
      return value;
    }

    case "format_iban": {
      const clean = value.replace(/\s/g, "").toUpperCase();
      if (clean.length >= 5) {
        return clean.replace(/(.{4})/g, "$1 ").trim();
      }
      return value;
    }

    case "format_credit_card": {
      const digits = value.replace(/\D/g, "");
      if (digits.length >= 13) {
        return digits.replace(/(.{4})/g, "$1 ").trim();
      }
      return value;
    }

    case "format_thousands": {
      const sep = transform.separator ?? ",";
      return formatWithThousands(value, sep) ?? value;
    }

    case "format_decimal": {
      const cleaned = cleanNumberString(value);
      const num = parseStrictFiniteNumber(cleaned);
      if (num === null) return value;
      return formatDecimalPlaces(num, transform.places ?? 0);
    }

    default:
      return value;
  }
}

/**
 * Apply multiple transforms in sequence.
 */
export function applyTransforms(
  value: unknown,
  transforms: Transform[],
): unknown {
  let result = value;
  for (const transform of transforms) {
    result = applyTransform(result, transform);
  }
  return result;
}

/**
 * Parse a date string with a given format and return canonical YYYY-MM-DD.
 */
function parseDateWithFormat(value: string, format: string): string | null {
  const trimmed = value.trim();

  // Handle common formats
  if (
    format === "%m/%d/%Y" ||
    format === "%m-%d-%Y" ||
    format === "MM/DD/YYYY" ||
    format === "MM-DD-YYYY"
  ) {
    const sep = format.includes("/") ? "/" : "-";
    const parts = trimmed.split(sep);
    if (parts.length === 3) {
      const [month, day, year] = parts;
      if (month && day && year && isValidDate(year, month, day)) {
        return `${year}-${month.padStart(2, "0")}-${day.padStart(2, "0")}`;
      }
    }
  }

  if (format === "%Y-%m-%d" || format === "YYYY-MM-DD") {
    // Already in canonical format, just validate
    const parts = trimmed.split("-");
    if (parts.length === 3) {
      const [year, month, day] = parts;
      if (year && month && day && isValidDate(year, month, day)) {
        return `${year}-${month.padStart(2, "0")}-${day.padStart(2, "0")}`;
      }
    }
  }

  if (format === "MMDDYYYY") {
    if (trimmed.length === 8 && /^\d{8}$/.test(trimmed)) {
      const month = trimmed.slice(0, 2);
      const day = trimmed.slice(2, 4);
      const year = trimmed.slice(4, 8);
      if (isValidDate(year, month, day)) {
        return `${year}-${month}-${day}`;
      }
    }
  }

  if (
    format === "%d/%m/%Y" ||
    format === "%d-%m-%Y" ||
    format === "DD/MM/YYYY" ||
    format === "DD-MM-YYYY"
  ) {
    const sep = format.includes("/") ? "/" : "-";
    const parts = trimmed.split(sep);
    if (parts.length === 3) {
      const [day, month, year] = parts;
      if (month && day && year && isValidDate(year, month, day)) {
        return `${year}-${month.padStart(2, "0")}-${day.padStart(2, "0")}`;
      }
    }
  }

  return null;
}

/**
 * Basic date validation.
 */
function isValidDate(year: string, month: string, day: string): boolean {
  if (!/^\d+$/.test(year) || !/^\d+$/.test(month) || !/^\d+$/.test(day)) {
    return false;
  }

  const y = Number(year);
  const m = Number(month);
  const d = Number(day);

  if (Number.isNaN(y) || Number.isNaN(m) || Number.isNaN(d)) {
    return false;
  }

  if (m < 1 || m > 12) return false;
  if (d < 1 || d > 31) return false;

  // Simple validation (doesn't check all edge cases)
  const daysInMonth = [
    31,
    isLeapYear(y) ? 29 : 28,
    31,
    30,
    31,
    30,
    31,
    31,
    30,
    31,
    30,
    31,
  ];
  const maxDays = daysInMonth[m - 1];
  return maxDays !== undefined && d <= maxDays;
}

function isLeapYear(year: number): boolean {
  return (year % 4 === 0 && year % 100 !== 0) || year % 400 === 0;
}

function cleanNumberString(value: string): string {
  return value.replace(/[^0-9.-]/g, "");
}

function parseStrictFiniteNumber(value: string): number | null {
  const s = value.trim();
  if (s.length === 0) return null;
  if (!/^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?$/.test(s)) {
    return null;
  }
  const n = Number(s);
  return Number.isFinite(n) ? n : null;
}

function normalizeIcd10Code(input: string): string | null {
  const compact = input.replace(/[^A-Za-z0-9]/g, "").toUpperCase();
  if (!isIcd10Plain(compact)) return null;
  if (compact.length > 3) return `${compact.slice(0, 3)}.${compact.slice(3)}`;
  return compact;
}

function isIcd10Plain(s: string): boolean {
  if (s.length < 3 || s.length > 7) return false;
  if (!/^[A-Z][A-Z0-9][A-Z0-9]/.test(s)) return false;
  return /^[A-Z0-9]+$/.test(s);
}

function normalizeCptCode(input: string): string | null {
  const compact = input.replace(/[^A-Za-z0-9]/g, "").toUpperCase();
  if (compact.length !== 5) return null;
  if (/^\d{5}$/.test(compact)) return compact;
  if (/^\d{4}[FT]$/.test(compact)) return compact;
  return null;
}

function normalizeHcpcsCode(input: string): string | null {
  const compact = input.replace(/[^A-Za-z0-9]/g, "").toUpperCase();
  return /^[A-V]\d{4}$/.test(compact) ? compact : null;
}

function normalizeNdc11Code(input: string): string | null {
  const trimmed = input.trim();
  const digits = trimmed.replace(/\D/g, "");
  if (digits.length === 11) return digits;

  const parts = trimmed.split("-");
  if (parts.length !== 3 || !parts.every((p) => /^\d+$/.test(p))) return null;
  const [a, b, c] = parts;
  if (!a || !b || !c) return null;

  if (a.length === 4 && b.length === 4 && c.length === 2)
    return `0${a}${b}${c}`;
  if (a.length === 5 && b.length === 3 && c.length === 2)
    return `${a}0${b}${c}`;
  if (a.length === 5 && b.length === 4 && c.length === 1)
    return `${a}${b}0${c}`;
  if (a.length === 5 && b.length === 4 && c.length === 2) return `${a}${b}${c}`;
  return null;
}

/**
 * Format a number string with thousands separators.
 */
function formatWithThousands(s: string, separator: string): string | null {
  const cleaned = s.replace(/[^0-9.-]/g, "");
  if (cleaned.length === 0) return null;

  const isNegative = cleaned.startsWith("-");
  const abs = cleaned.replace(/^-/, "");

  const dotIdx = abs.indexOf(".");
  const intPart = dotIdx >= 0 ? abs.slice(0, dotIdx) : abs;
  const decPart = dotIdx >= 0 ? abs.slice(dotIdx) : "";

  if (intPart.length === 0 || !/^\d+$/.test(intPart)) return null;

  // Add separators from right to left
  let result = "";
  for (let i = 0; i < intPart.length; i++) {
    if (i > 0 && (intPart.length - i) % 3 === 0) {
      result += separator;
    }
    result += intPart[i];
  }

  return (isNegative ? "-" : "") + result + decPart;
}
