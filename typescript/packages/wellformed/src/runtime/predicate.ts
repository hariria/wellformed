// biome-ignore-all lint/complexity/noExcessiveCognitiveComplexity: Predicate evaluation keeps parser and dispatch logic grouped by runtime domain.
/**
 * Predicate evaluation - evaluates predicates against values.
 */

import type { Predicate, TemplateLiteralPart } from "../ir/types.js";
import { resolveJsonPointer } from "./pointer.js";

/**
 * Named predicate function signature.
 */
export type NamedPredicateFn = (value: unknown, args: unknown) => boolean;

/**
 * Registry of named predicates.
 */
export class PredicateRegistry {
  private predicates: Map<string, NamedPredicateFn> = new Map();

  /**
   * Register a named predicate.
   */
  register(name: string, fn: NamedPredicateFn): void {
    this.predicates.set(name, fn);
  }

  /**
   * Get a named predicate by name.
   */
  get(name: string): NamedPredicateFn | undefined {
    return this.predicates.get(name);
  }

  /**
   * Create a registry with built-in predicates.
   */
  static withBuiltins(): PredicateRegistry {
    const registry = new PredicateRegistry();

    // TIN predicates
    registry.register("is_tin", isTin);
    registry.register("is_ssn", isSsn);
    registry.register("is_ein", isEin);
    registry.register("is_itin", isItin);
    registry.register("is_atin", isAtin);
    registry.register("luhn", luhn);

    // Financial predicates
    registry.register("is_cusip", isCusip);
    registry.register("is_aba_routing", isAbaRouting);
    registry.register("is_mcc", isMcc);
    registry.register("is_account_number", isAccountNumber);

    // Date predicates
    registry.register("is_date", isDate);
    registry.register("is_time", isTime);
    registry.register("is_iso_datetime", isIsoDatetime);
    registry.register("is_tax_year", isTaxYear);
    registry.register("date_in_range", dateInRange);
    registry.register("date_before", dateBefore);
    registry.register("date_after", dateAfter);
    registry.register("time_before", timeBefore);
    registry.register("time_after", timeAfter);
    registry.register("time_in_range", timeInRange);

    // Amount predicates
    registry.register("is_non_negative", isNonNegative);
    registry.register("is_positive", isPositive);
    registry.register("is_negative", isNegative);
    registry.register("is_non_positive", isNonPositive);
    registry.register("is_percentage", isPercentage);
    registry.register("is_money_format", isMoneyFormat);
    registry.register("is_money_no_symbol", isMoneyNoSymbol);
    registry.register("is_multiple_of", isMultipleOf);
    registry.register("less_than_or_equal", lessThanOrEqual);
    registry.register("greater_than_or_equal", greaterThanOrEqual);
    registry.register("format:decimal-2", formatDecimal2);

    // Reference predicates
    registry.register("is_country_code", isCountryCode);
    registry.register("is_country_name", isCountryName);
    registry.register("is_currency_code", isCurrencyCode);
    registry.register("is_state_name", isStateName);
    registry.register("is_us_state", isUsState);
    registry.register("is_us_zip", isUsZip);
    registry.register("is_street_address", isStreetAddress);
    registry.register("is_w2_box12_code", isW2Box12Code);
    registry.register("is_1099b_code", is1099BCode);
    registry.register("is_filing_status", isFilingStatus);

    // Aviation predicates
    registry.register("is_iata_airport_code", isIataAirportCode);
    registry.register("is_icao_airport_code", isIcaoAirportCode);
    registry.register("is_airport_code", isAirportCode);
    registry.register("is_iata_airline_code", isIataAirlineCode);
    registry.register("is_icao_airline_code", isIcaoAirlineCode);
    registry.register("is_airline_code", isAirlineCode);
    registry.register("is_flight_number", isFlightNumber);

    // Contact predicates
    registry.register("phone_number", isPhoneNumber);
    registry.register("phone_number_us", isPhoneNumberUS);
    registry.register("is_phone", isPhoneNumber); // backwards compat
    registry.register("is_email", isEmail);
    registry.register("is_url", isUrl);
    registry.register("is_uuid", isUuid);
    registry.register("is_ip", isIp);
    registry.register("is_cidr", isCidr);
    registry.register("is_mac_address", isMacAddress);

    // Payment card predicates
    registry.register("is_credit_card", isCreditCard);
    registry.register("is_cvv", isCvv);
    registry.register("is_card_expiry", isCardExpiry);

    // Product / ecommerce predicates
    registry.register("is_vin", isVin);
    registry.register("is_upc", isUpc);
    registry.register("is_ean", isEan);
    registry.register("is_isbn", isIsbn);

    // International banking predicates
    registry.register("is_iban", isIban);
    registry.register("is_bic", isBic);
    registry.register("is_swift", isBic); // alias

    // Text predicates
    registry.register("is_rtl", isRtl);
    registry.register("is_ltr", isLtr);
    registry.register("starts_with", startsWith);
    registry.register("ends_with", endsWith);
    registry.register("contains", containsSubstring);

    // Color predicates
    registry.register("is_hex_color", isHexColor);
    registry.register("is_rgb_color", isRgbColor);
    registry.register("is_hsl_color", isHslColor);

    // Decimal places predicate
    registry.register("is_decimal_places", isDecimalPlaces);

    // Numeric type predicates
    registry.register("is_integer", isInteger);
    registry.register("is_float", isFloat);
    registry.register("is_u8", makeIntRangeCheck(0n, 255n));
    registry.register("is_u16", makeIntRangeCheck(0n, 65535n));
    registry.register("is_u32", makeIntRangeCheck(0n, 4294967295n));
    registry.register("is_u64", makeIntRangeCheck(0n, 18446744073709551615n));
    registry.register("is_i8", makeIntRangeCheck(-128n, 127n));
    registry.register("is_i16", makeIntRangeCheck(-32768n, 32767n));
    registry.register("is_i32", makeIntRangeCheck(-2147483648n, 2147483647n));
    registry.register(
      "is_i64",
      makeIntRangeCheck(-9223372036854775808n, 9223372036854775807n),
    );

    // Insurance predicates
    registry.register("is_npi", isNpi);
    registry.register("is_dea_number", isDeaNumber);
    registry.register("is_icd10_code", isIcd10Code);
    registry.register("is_cpt_code", isCptCode);
    registry.register("is_hcpcs_code", isHcpcsCode);
    registry.register("is_ndc_code", isNdcCode);

    // Encoding/crypto predicates
    registry.register("is_base58", isBase58);
    registry.register("is_base64", isBase64);
    registry.register("is_bitcoin_address", isBitcoinAddress);
    registry.register("is_ethereum_address", isEthereumAddress);
    registry.register("is_solana_address", isSolanaAddress);
    registry.register("is_jwt", isJwt);
    registry.register("is_hash", isHash);

    // Text character class predicates
    registry.register("is_alpha", isAlpha);
    registry.register("is_digits", isDigits);
    registry.register("is_alphanumeric", isAlphanumeric);
    registry.register("is_alpha_spaces", isAlphaSpaces);
    registry.register("is_alphanumeric_spaces", isAlphanumericSpaces);
    registry.register("is_name_chars", isNameChars);
    registry.register("is_uppercase", isUppercase);
    registry.register("is_lowercase", isLowercase);
    registry.register("is_title_case", isTitleCase);

    return registry;
  }
}

/**
 * Evaluation context for predicates.
 */
export interface EvalContext {
  registry: PredicateRegistry;
  regexCache?: Map<string, RegExp>;
}

/**
 * Create a new evaluation context.
 */
export function createEvalContext(registry?: PredicateRegistry): EvalContext {
  return {
    registry: registry ?? PredicateRegistry.withBuiltins(),
    regexCache: new Map(),
  };
}

/**
 * Evaluate a predicate against a value.
 */
export function evaluate(
  pred: Predicate,
  value: unknown,
  ctx: EvalContext,
): boolean {
  switch (pred.type) {
    case "true":
      return true;

    case "false":
      return false;

    case "regex": {
      if (typeof value !== "string") return true;
      const cacheKey = `${pred.pattern}:${pred.flags ?? ""}`;
      let regex = ctx.regexCache?.get(cacheKey);
      if (!regex) {
        regex = new RegExp(pred.pattern, pred.flags);
        ctx.regexCache?.set(cacheKey, regex);
      }
      regex.lastIndex = 0;
      return regex.test(value);
    }

    case "template_literal":
      if (typeof value !== "string") return true;
      return matchesTemplateLiteral(value, pred.parts);

    case "min_len": {
      const len = getLength(value);
      return len !== null && len >= pred.len;
    }

    case "max_len": {
      const len = getLength(value);
      return len !== null && len <= pred.len;
    }

    case "range": {
      const n = typeof value === "number" ? value : null;
      if (n === null) return false;
      const aboveMin = pred.min === undefined || n >= pred.min;
      const belowMax = pred.max === undefined || n <= pred.max;
      return aboveMin && belowMax;
    }

    case "exists": {
      const results = resolveJsonPointer(value, pred.path);
      return (
        results.length > 0 &&
        results.every((v) => v !== null && v !== undefined)
      );
    }

    case "eq": {
      const results = resolveJsonPointer(value, pred.path);
      return (
        results.length > 0 && results.every((v) => deepEqual(v, pred.value))
      );
    }

    case "in": {
      const results = resolveJsonPointer(value, pred.path);
      return (
        results.length > 0 &&
        results.every((v) => pred.values.some((val) => deepEqual(v, val)))
      );
    }

    case "required_with": {
      const withExists = pathExists(value, pred.with);
      const fieldExists = pathExists(value, pred.field);
      return !withExists || fieldExists;
    }

    case "required_without": {
      const withoutExists = pathExists(value, pred.without);
      const fieldExists = pathExists(value, pred.field);
      return withoutExists || fieldExists;
    }

    case "exactly_one_of": {
      let count = 0;
      for (const path of pred.paths) {
        if (pathExists(value, path)) {
          count += 1;
          if (count > 1) return false;
        }
      }
      return count === 1;
    }

    case "and":
      return pred.predicates.every((p) => evaluate(p, value, ctx));

    case "or":
      return pred.predicates.some((p) => evaluate(p, value, ctx));

    case "not":
      return !evaluate(pred.predicate, value, ctx);

    case "implies": {
      // P => Q is equivalent to !P || Q
      const antecedent = evaluate(pred.if, value, ctx);
      if (!antecedent) return true;
      return evaluate(pred.then, value, ctx);
    }

    case "call": {
      const fn = ctx.registry.get(pred.name);
      if (!fn) {
        throw new Error(`Unknown predicate: ${pred.name}`);
      }
      return fn(value, pred.args);
    }

    // Cross-field comparisons
    case "eq_fields": {
      const leftVals = resolveJsonPointer(value, pred.left);
      const rightVals = resolveJsonPointer(value, pred.right);
      if (leftVals.length === 0 || rightVals.length === 0) return false;
      return (
        leftVals.length === rightVals.length &&
        leftVals.every((left, i) => deepEqual(left, rightVals[i]))
      );
    }

    case "gt_field": {
      const leftVals = resolveJsonPointer(value, pred.left);
      const rightVals = resolveJsonPointer(value, pred.right);
      if (leftVals.length === 0 || rightVals.length === 0) return false;
      const left = toNumber(leftVals[0]);
      const right = toNumber(rightVals[0]);
      if (left === null || right === null) return false;
      return left > right;
    }

    case "gte_field": {
      const leftVals = resolveJsonPointer(value, pred.left);
      const rightVals = resolveJsonPointer(value, pred.right);
      if (leftVals.length === 0 || rightVals.length === 0) return false;
      const left = toNumber(leftVals[0]);
      const right = toNumber(rightVals[0]);
      if (left === null || right === null) return false;
      return left >= right;
    }

    case "lt_field": {
      const leftVals = resolveJsonPointer(value, pred.left);
      const rightVals = resolveJsonPointer(value, pred.right);
      if (leftVals.length === 0 || rightVals.length === 0) return false;
      const left = toNumber(leftVals[0]);
      const right = toNumber(rightVals[0]);
      if (left === null || right === null) return false;
      return left < right;
    }

    case "lte_field": {
      const leftVals = resolveJsonPointer(value, pred.left);
      const rightVals = resolveJsonPointer(value, pred.right);
      if (leftVals.length === 0 || rightVals.length === 0) return false;
      const left = toNumber(leftVals[0]);
      const right = toNumber(rightVals[0]);
      if (left === null || right === null) return false;
      return left <= right;
    }

    // Sum/computed predicates
    case "sum_equals": {
      const sum = computeSum(value, pred.paths);
      if (sum === null) return false;
      const targetVals = resolveJsonPointer(value, pred.target);
      if (targetVals.length === 0) return false;
      const target = toNumber(targetVals[0]);
      if (target === null) return false;
      // Use tolerance for floating point comparison
      return Math.abs(sum - target) < 1e-10;
    }

    case "sum_equals_value": {
      const sum = computeSum(value, pred.paths);
      if (sum === null) return false;
      return Math.abs(sum - pred.value) < 1e-10;
    }

    default:
      return false;
  }
}

/**
 * Compute sum of numeric values at given paths.
 */
function computeSum(value: unknown, paths: string[]): number | null {
  let sum = 0;
  for (const path of paths) {
    const vals = resolveJsonPointer(value, path);
    if (vals.length === 0) return null; // Missing field
    const n = toNumber(vals[0]);
    if (n === null) return null; // Non-numeric value
    sum += n;
  }
  return sum;
}

/**
 * Check whether a JSON pointer path resolves to at least one non-null value.
 */
function pathExists(value: unknown, path: string): boolean {
  const vals = resolveJsonPointer(value, path);
  return vals.length > 0 && vals.every((v) => v !== null && v !== undefined);
}

/**
 * Convert a value to a number if possible.
 */
function toNumber(val: unknown): number | null {
  if (typeof val === "number") return val;
  return null;
}

// ============================================================================
// Helper Functions
// ============================================================================

function getLength(value: unknown): number | null {
  if (typeof value === "string") return Array.from(value).length;
  if (Array.isArray(value)) return value.length;
  return null;
}

function deepEqual(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (typeof a !== typeof b) return false;
  if (a === null || b === null) return a === b;
  if (typeof a !== "object") return false;

  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) return false;
    return a.every((v, i) => deepEqual(v, b[i]));
  }

  if (Array.isArray(a) || Array.isArray(b)) return false;

  const aKeys = Object.keys(a as object);
  const bKeys = Object.keys(b as object);
  if (aKeys.length !== bKeys.length) return false;

  return aKeys.every((key) =>
    deepEqual(
      (a as Record<string, unknown>)[key],
      (b as Record<string, unknown>)[key],
    ),
  );
}

function matchesTemplateLiteral(
  input: string,
  parts: TemplateLiteralPart[],
): boolean {
  return matchesTemplateFrom(input, 0, parts, 0);
}

function matchesTemplateFrom(
  input: string,
  inputPos: number,
  parts: TemplateLiteralPart[],
  partPos: number,
): boolean {
  if (partPos >= parts.length) {
    return inputPos === input.length;
  }

  const part = parts[partPos];
  if (!part) return false;

  if (part.kind === "literal") {
    if (!input.startsWith(part.value, inputPos)) return false;
    return matchesTemplateFrom(
      input,
      inputPos + part.value.length,
      parts,
      partPos + 1,
    );
  }

  const bounds = templatePartBounds(part);
  if (bounds === null) return false;
  const remaining = input.length - inputPos;
  const maxLen = Math.min(bounds.max, remaining);
  if (bounds.min > maxLen) return false;

  const runLen = templatePartRunLength(part, input, inputPos);
  if (runLen < bounds.min) return false;
  const maxRun = Math.min(maxLen, runLen);

  if (partPos + 1 >= parts.length) {
    const tailLen = remaining;
    return tailLen >= bounds.min && tailLen <= maxRun;
  }

  const nextPart = parts[partPos + 1];
  if (nextPart?.kind === "literal" && nextPart.value.length > 0) {
    let searchFrom = inputPos + bounds.min;
    while (searchFrom <= inputPos + maxRun) {
      const nextIdx = input.indexOf(nextPart.value, searchFrom);
      if (nextIdx === -1) break;

      const consumed = nextIdx - inputPos;
      if (consumed > maxRun) break;
      if (matchesTemplateFrom(input, nextIdx, parts, partPos + 1)) return true;
      searchFrom = nextIdx + 1;
    }
    return false;
  }

  for (let consumed = maxRun; consumed >= bounds.min; consumed--) {
    if (matchesTemplateFrom(input, inputPos + consumed, parts, partPos + 1)) {
      return true;
    }
  }
  return false;
}

function templatePartBounds(
  part: Exclude<TemplateLiteralPart, { kind: "literal" }>,
): { min: number; max: number } | null {
  const min = Math.max(0, part.min ?? 1);
  const max = Math.max(min, part.max ?? Number.POSITIVE_INFINITY);
  if (min > max) return null;
  return { min, max };
}

function templatePartRunLength(
  part: Exclude<TemplateLiteralPart, { kind: "literal" }>,
  input: string,
  start: number,
): number {
  let len = 0;
  for (let i = start; i < input.length; i++) {
    const code = input.charCodeAt(i);
    if (templatePartMatchesCode(part, code)) {
      len++;
    } else {
      break;
    }
  }
  return len;
}

function templatePartMatchesCode(
  part: Exclude<TemplateLiteralPart, { kind: "literal" }>,
  code: number,
): boolean {
  const isDigit = code >= 48 && code <= 57;
  const isUpper = code >= 65 && code <= 90;
  const isLower = code >= 97 && code <= 122;

  switch (part.kind) {
    case "digits":
      return isDigit;
    case "ascii_letters":
      return isUpper || isLower;
    case "ascii_alphanumeric":
      return isDigit || isUpper || isLower;
    case "uppercase":
      return isUpper;
    case "lowercase":
      return isLower;
    case "hex":
      return (
        isDigit || (code >= 65 && code <= 70) || (code >= 97 && code <= 102)
      );
    default:
      return false;
  }
}

// ============================================================================
// Built-in Named Predicates
// ============================================================================

function extractDigits(value: unknown): string {
  if (typeof value !== "string") return "";
  let digits = "";
  let changed = false;
  for (let i = 0; i < value.length; i++) {
    const code = value.charCodeAt(i);
    if (isAsciiDigit(code)) {
      if (changed) digits += value.charAt(i);
    } else if (!changed) {
      digits = value.slice(0, i);
      changed = true;
    }
  }
  return changed ? digits : value;
}

function extractNumber(value: unknown): number | null {
  if (typeof value === "number") return Number.isFinite(value) ? value : null;
  if (typeof value === "string") {
    const cleaned = value.replace(/[^0-9.-]/g, "");
    return parseStrictFiniteNumber(cleaned);
  }
  return null;
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

// TIN predicates
const VALID_EIN_CAMPUSES = new Set([
  10, 12, 60, 67, 50, 53, 1, 2, 3, 4, 5, 6, 11, 13, 14, 16, 21, 22, 23, 25, 34,
  51, 52, 54, 55, 56, 57, 58, 59, 65, 30, 32, 35, 36, 37, 38, 61, 15, 24, 40,
  44, 94, 95, 80, 90, 33, 39, 41, 42, 43, 46, 48, 62, 63, 64, 66, 68, 71, 72,
  73, 74, 75, 76, 77, 81, 82, 83, 84, 85, 86, 87, 88, 91, 92, 93, 98, 99, 20,
  26, 27, 45, 47,
]);

function isTin(value: unknown, args: unknown): boolean {
  const digits = extractDigits(value);
  if (digits.length !== 9) return false;
  if (isAllSameDigit(digits, 48)) return false;

  const kind =
    typeof args === "object" && args !== null
      ? (args as { kind?: string }).kind
      : "ANY";

  switch (kind) {
    case "SSN":
      return isValidSsn(digits);
    case "EIN":
      return isValidEin(digits);
    case "ITIN":
      return isValidItin(digits);
    case "ATIN":
      return isValidAtin(digits);
    default:
      return (
        isValidSsn(digits) ||
        isValidEin(digits) ||
        isValidItin(digits) ||
        isValidAtin(digits)
      );
  }
}

function isSsn(value: unknown): boolean {
  const digits = extractDigits(value);
  return digits.length === 9 && isValidSsn(digits);
}

function isEin(value: unknown): boolean {
  const digits = extractDigits(value);
  return digits.length === 9 && isValidEin(digits);
}

function isItin(value: unknown): boolean {
  const digits = extractDigits(value);
  return digits.length === 9 && isValidItin(digits);
}

function isAtin(value: unknown): boolean {
  const digits = extractDigits(value);
  return digits.length === 9 && isValidAtin(digits);
}

function isValidSsn(digits: string): boolean {
  const area =
    (digits.charCodeAt(0) - 48) * 100 +
    (digits.charCodeAt(1) - 48) * 10 +
    (digits.charCodeAt(2) - 48);
  const group = (digits.charCodeAt(3) - 48) * 10 + (digits.charCodeAt(4) - 48);
  const serial =
    (digits.charCodeAt(5) - 48) * 1000 +
    (digits.charCodeAt(6) - 48) * 100 +
    (digits.charCodeAt(7) - 48) * 10 +
    (digits.charCodeAt(8) - 48);

  if (area === 0 || area === 666 || area >= 900) return false;
  if (group === 0 || serial === 0) return false;
  return true;
}

function isValidEin(digits: string): boolean {
  const campus = (digits.charCodeAt(0) - 48) * 10 + (digits.charCodeAt(1) - 48);
  return VALID_EIN_CAMPUSES.has(campus);
}

function isValidItin(digits: string): boolean {
  return digits[0] === "9" && (digits[3] === "7" || digits[3] === "8");
}

function isValidAtin(digits: string): boolean {
  return (
    digits.charCodeAt(0) === 57 &&
    digits.charCodeAt(3) === 57 &&
    digits.charCodeAt(4) === 51
  );
}

function luhn(value: unknown): boolean {
  const digits = extractDigits(value);
  return luhnDigits(digits);
}

function luhnDigits(digits: string): boolean {
  if (digits.length === 0) return false;
  let sum = 0;
  let double = false;

  for (let i = digits.length - 1; i >= 0; i--) {
    let d = digits.charCodeAt(i) - 48;
    if (double) {
      d *= 2;
      if (d > 9) d -= 9;
    }
    sum += d;
    double = !double;
  }

  return sum % 10 === 0;
}

// Financial predicates
function isCusip(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toUpperCase();
  if (s.length !== 9) return false;
  if (!/^[A-Z0-9]+$/.test(s)) return false;

  const validateChecksum =
    typeof args === "object" && args !== null
      ? ((args as { validate_checksum?: boolean }).validate_checksum ?? true)
      : true;

  if (!validateChecksum) return true;

  // CUSIP checksum validation
  let sum = 0;
  for (let i = 0; i < 8; i++) {
    const c = s[i];
    if (!c) return false;
    let val: number;
    if (c >= "0" && c <= "9") {
      val = Number.parseInt(c, 10);
    } else {
      val = c.charCodeAt(0) - 55; // A=10, B=11, etc.
    }
    if (i % 2 === 1) val *= 2;
    sum += Math.floor(val / 10) + (val % 10);
  }

  const checkDigit = (10 - (sum % 10)) % 10;
  return s[8] === checkDigit.toString();
}

function isAbaRouting(value: unknown): boolean {
  const digits = extractDigits(value);
  if (digits.length !== 9) return false;

  const prefix = (digits.charCodeAt(0) - 48) * 10 + (digits.charCodeAt(1) - 48);
  const validPrefixes =
    (prefix >= 0 && prefix <= 12) ||
    (prefix >= 21 && prefix <= 32) ||
    (prefix >= 61 && prefix <= 72) ||
    prefix === 80;
  if (!validPrefixes) return false;

  const d0 = digits.charCodeAt(0) - 48;
  const d1 = digits.charCodeAt(1) - 48;
  const d2 = digits.charCodeAt(2) - 48;
  const d3 = digits.charCodeAt(3) - 48;
  const d4 = digits.charCodeAt(4) - 48;
  const d5 = digits.charCodeAt(5) - 48;
  const d6 = digits.charCodeAt(6) - 48;
  const d7 = digits.charCodeAt(7) - 48;
  const d8 = digits.charCodeAt(8) - 48;
  const checksum = 3 * (d0 + d3 + d6) + 7 * (d1 + d4 + d7) + (d2 + d5 + d8);

  return checksum % 10 === 0;
}

function isMcc(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  return s.length === 4 && isAsciiDigitsOnly(s);
}

function isAccountNumber(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  const minLen =
    typeof args === "object" && args !== null
      ? ((args as { min_len?: number }).min_len ?? 1)
      : 1;
  const maxLen =
    typeof args === "object" && args !== null
      ? ((args as { max_len?: number }).max_len ?? 30)
      : 30;
  const allowHyphens =
    typeof args === "object" && args !== null
      ? ((args as { allow_hyphens?: boolean }).allow_hyphens ?? true)
      : true;

  if (s.length < minLen || s.length > maxLen) return false;

  const pattern = allowHyphens ? /^[A-Za-z0-9-]+$/ : /^[A-Za-z0-9]+$/;
  return pattern.test(s);
}

// Date predicates
function isDate(value: unknown): boolean {
  if (typeof value !== "string") return false;
  return parseDate(value) !== null;
}

function isTime(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();

  const format =
    typeof args === "object" && args !== null
      ? (args as { format?: string }).format
      : undefined;

  switch (format) {
    case "24h":
      return parseTime24h(s);
    case "12h":
      return parseTime12h(s);
    default:
      return parseTime24h(s) || parseTime12h(s);
  }
}

function parseTime24h(s: string): boolean {
  const match = s.match(/^(\d{2}):(\d{2})(?::(\d{2}))?$/);
  if (!match) return false;
  const [, hourPart, minutePart, secondPart] = match;
  if (hourPart === undefined || minutePart === undefined) return false;
  const hour = Number.parseInt(hourPart, 10);
  const minute = Number.parseInt(minutePart, 10);
  if (hour > 23 || minute > 59) return false;
  if (secondPart !== undefined) {
    const second = Number.parseInt(secondPart, 10);
    if (second > 59) return false;
  }
  return true;
}

function parseTime12h(s: string): boolean {
  const match = s.match(/^(\d{1,2}):(\d{2})(?::(\d{2}))?\s*(AM|PM|am|pm)$/);
  if (!match) return false;
  const [, hourPart, minutePart, secondPart] = match;
  if (hourPart === undefined || minutePart === undefined) return false;
  const hour = Number.parseInt(hourPart, 10);
  const minute = Number.parseInt(minutePart, 10);
  if (hour < 1 || hour > 12 || minute > 59) return false;
  if (secondPart !== undefined) {
    const second = Number.parseInt(secondPart, 10);
    if (second > 59) return false;
  }
  return true;
}

function isIsoDatetime(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();

  const tPos = s.indexOf("T");
  if (tPos === -1) return false;

  // Validate date part (YYYY-MM-DD)
  const datePart = s.slice(0, tPos);
  const dMatch = datePart.match(/^(\d{4})-(\d{2})-(\d{2})$/);
  if (!dMatch) return false;
  const [, yearPart, monthPart, dayPart] = dMatch;
  if (
    yearPart === undefined ||
    monthPart === undefined ||
    dayPart === undefined
  ) {
    return false;
  }
  const year = Number.parseInt(yearPart, 10);
  const month = Number.parseInt(monthPart, 10);
  const day = Number.parseInt(dayPart, 10);
  if (!validateDate(year, month, day)) return false;

  // Time + timezone
  let timeRest = s.slice(tPos + 1);

  // Separate timezone
  let tzValid = true;
  if (timeRest.endsWith("Z")) {
    timeRest = timeRest.slice(0, -1);
  } else {
    // Check for +HH:MM or -HH:MM at end
    const tzMatch = timeRest.match(/([+-])(\d{2}):(\d{2})$/);
    if (tzMatch) {
      const [, , tzHourPart, tzMinPart] = tzMatch;
      if (tzHourPart === undefined || tzMinPart === undefined) return false;
      const tzHour = Number.parseInt(tzHourPart, 10);
      const tzMin = Number.parseInt(tzMinPart, 10);
      if (tzHour > 23 || tzMin > 59) tzValid = false;
      timeRest = timeRest.slice(0, -6);
    }
  }

  if (!tzValid) return false;

  // Strip fractional seconds
  const dotPos = timeRest.indexOf(".");
  if (dotPos !== -1) {
    const frac = timeRest.slice(dotPos + 1);
    if (frac.length === 0 || !/^\d+$/.test(frac)) return false;
    timeRest = timeRest.slice(0, dotPos);
  }

  // Validate HH:MM or HH:MM:SS
  return parseTime24h(timeRest);
}

function isTaxYear(value: unknown, args: unknown): boolean {
  let year: number | null = null;
  if (typeof value === "number") year = value;
  else if (typeof value === "string") year = Number.parseInt(value.trim(), 10);

  if (year === null || Number.isNaN(year)) return false;

  const min =
    typeof args === "object" && args !== null
      ? ((args as { min?: number }).min ?? 2020)
      : 2020;
  const max =
    typeof args === "object" && args !== null
      ? ((args as { max?: number }).max ?? 2100)
      : 2100;

  return year >= min && year <= max;
}

function dateInRange(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const date = parseDate(value);
  if (!date) return false;

  if (typeof args !== "object" || args === null) return true;
  const a = args as {
    min_year?: number;
    max_year?: number;
    min?: string;
    max?: string;
  };

  if (a.min_year !== undefined && date.year < a.min_year) return false;
  if (a.max_year !== undefined && date.year > a.max_year) return false;

  if (a.min !== undefined) {
    const minDate = parseDate(a.min);
    if (minDate && compareDates(date, minDate) < 0) return false;
  }
  if (a.max !== undefined) {
    const maxDate = parseDate(a.max);
    if (maxDate && compareDates(date, maxDate) > 0) return false;
  }

  return true;
}

function dateBefore(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const date = parseDate(value);
  if (!date) return false;

  if (typeof args !== "object" || args === null) return true;
  const a = args as { date?: string; allow_equal?: boolean };
  if (!a.date) return true;

  const other = parseDate(a.date);
  if (!other) return true;

  const cmp = compareDates(date, other);
  const allowEqual = a.allow_equal ?? true;
  return allowEqual ? cmp <= 0 : cmp < 0;
}

function dateAfter(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const date = parseDate(value);
  if (!date) return false;

  if (typeof args !== "object" || args === null) return true;
  const a = args as { date?: string; allow_equal?: boolean };
  if (!a.date) return true;

  const other = parseDate(a.date);
  if (!other) return true;

  const cmp = compareDates(date, other);
  const allowEqual = a.allow_equal ?? true;
  return allowEqual ? cmp >= 0 : cmp > 0;
}

function timeBefore(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const time = parseTimeToSeconds(value.trim());
  if (time === null) return false;

  if (typeof args !== "object" || args === null) return true;
  const a = args as { time?: string; allow_equal?: boolean };
  if (!a.time) return true;

  const other = parseTimeToSeconds(a.time.trim());
  if (other === null) return true;

  const allowEqual = a.allow_equal ?? true;
  return allowEqual ? time <= other : time < other;
}

function timeAfter(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const time = parseTimeToSeconds(value.trim());
  if (time === null) return false;

  if (typeof args !== "object" || args === null) return true;
  const a = args as { time?: string; allow_equal?: boolean };
  if (!a.time) return true;

  const other = parseTimeToSeconds(a.time.trim());
  if (other === null) return true;

  const allowEqual = a.allow_equal ?? true;
  return allowEqual ? time >= other : time > other;
}

function timeInRange(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const time = parseTimeToSeconds(value.trim());
  if (time === null) return false;

  if (typeof args !== "object" || args === null) return true;
  const a = args as { min?: string; max?: string };

  const min = a.min ? parseTimeToSeconds(a.min.trim()) : null;
  const max = a.max ? parseTimeToSeconds(a.max.trim()) : null;

  if (min !== null && max !== null) {
    if (min <= max) {
      // Normal range
      return time >= min && time <= max;
    }
    // Overnight range (wraps around midnight)
    return time >= min || time <= max;
  }
  if (min !== null) return time >= min;
  if (max !== null) return time <= max;
  return true;
}

/**
 * Parse any time string (24h or 12h) to total seconds from midnight.
 */
function parseTimeToSeconds(s: string): number | null {
  return parseTimeToSeconds24h(s) ?? parseTimeToSeconds12h(s);
}

function parseTimeToSeconds24h(s: string): number | null {
  const match = s.match(/^(\d{2}):(\d{2})(?::(\d{2}))?$/);
  if (!match) return null;
  const [, hourPart, minutePart, secondPart] = match;
  if (hourPart === undefined || minutePart === undefined) return null;
  const hour = Number.parseInt(hourPart, 10);
  const minute = Number.parseInt(minutePart, 10);
  if (hour > 23 || minute > 59) return null;
  let seconds = hour * 3600 + minute * 60;
  if (secondPart !== undefined) {
    const sec = Number.parseInt(secondPart, 10);
    if (sec > 59) return null;
    seconds += sec;
  }
  return seconds;
}

function parseTimeToSeconds12h(s: string): number | null {
  const match = s.match(/^(\d{1,2}):(\d{2})(?::(\d{2}))?\s*(AM|PM|am|pm)$/);
  if (!match) return null;
  const [, hourPart, minutePart, secondPart, meridiem] = match;
  if (
    hourPart === undefined ||
    minutePart === undefined ||
    meridiem === undefined
  ) {
    return null;
  }
  let hour = Number.parseInt(hourPart, 10);
  const minute = Number.parseInt(minutePart, 10);
  if (hour < 1 || hour > 12 || minute > 59) return null;

  let seconds = 0;
  if (secondPart !== undefined) {
    const sec = Number.parseInt(secondPart, 10);
    if (sec > 59) return null;
    seconds = sec;
  }

  const isPm = meridiem.toUpperCase() === "PM";
  if (isPm) {
    if (hour !== 12) hour += 12;
  } else {
    if (hour === 12) hour = 0;
  }

  return hour * 3600 + minute * 60 + seconds;
}

interface ParsedDate {
  year: number;
  month: number;
  day: number;
}

function parseDate(s: string): ParsedDate | null {
  const trimmed = s.trim();

  // MM/DD/YYYY or MM-DD-YYYY
  let match = trimmed.match(/^(\d{1,2})[/-](\d{1,2})[/-](\d{4})$/);
  if (match) {
    const [, month, day, year] = match;
    return validateDate(Number(year), Number(month), Number(day));
  }

  // YYYY-MM-DD
  match = trimmed.match(/^(\d{4})-(\d{1,2})-(\d{1,2})$/);
  if (match) {
    const [, year, month, day] = match;
    return validateDate(Number(year), Number(month), Number(day));
  }

  // MMDDYYYY
  if (/^\d{8}$/.test(trimmed)) {
    const month = Number(trimmed.slice(0, 2));
    const day = Number(trimmed.slice(2, 4));
    const year = Number(trimmed.slice(4, 8));
    return validateDate(year, month, day);
  }

  return null;
}

function validateDate(
  year: number,
  month: number,
  day: number,
): ParsedDate | null {
  if (month < 1 || month > 12) return null;
  if (day < 1) return null;

  const daysInMonth = [
    31,
    isLeapYear(year) ? 29 : 28,
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
  const maxDays = daysInMonth[month - 1];
  if (maxDays === undefined || day > maxDays) return null;

  return { year, month, day };
}

function isLeapYear(year: number): boolean {
  return (year % 4 === 0 && year % 100 !== 0) || year % 400 === 0;
}

function compareDates(a: ParsedDate, b: ParsedDate): number {
  if (a.year !== b.year) return a.year - b.year;
  if (a.month !== b.month) return a.month - b.month;
  return a.day - b.day;
}

// Amount predicates
function isNonNegative(value: unknown): boolean {
  const n = extractNumber(value);
  return n !== null && n >= 0;
}

function isPositive(value: unknown): boolean {
  const n = extractNumber(value);
  return n !== null && n > 0;
}

function isNegative(value: unknown): boolean {
  const n = extractNumber(value);
  return n !== null && n < 0;
}

function isNonPositive(value: unknown): boolean {
  const n = extractNumber(value);
  return n !== null && n <= 0;
}

function isPercentage(value: unknown, args: unknown): boolean {
  const n = extractNumber(value);
  if (n === null || Number.isNaN(n)) return false;

  const format =
    typeof args === "object" && args !== null
      ? ((args as { format?: string }).format ?? "percent")
      : "percent";
  const allowOver100 =
    typeof args === "object" && args !== null
      ? ((args as { allow_over_100?: boolean }).allow_over_100 ?? false)
      : false;

  const max =
    format === "decimal"
      ? allowOver100
        ? Number.POSITIVE_INFINITY
        : 1
      : allowOver100
        ? Number.POSITIVE_INFINITY
        : 100;
  return n >= 0 && n <= max;
}

function isMoneyFormat(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;

  const cleaned = s.replace(/[^0-9.-]/g, "");
  const n = parseStrictFiniteNumber(cleaned);
  if (n === null) return false;

  const maxDecimals =
    typeof args === "object" && args !== null
      ? ((args as { max_decimals?: number }).max_decimals ?? 2)
      : 2;
  const allowNegative =
    typeof args === "object" && args !== null
      ? ((args as { allow_negative?: boolean }).allow_negative ?? true)
      : true;

  if (!allowNegative && n < 0) return false;

  const dotPos = cleaned.indexOf(".");
  if (dotPos !== -1) {
    const decimals = cleaned.length - dotPos - 1;
    if (decimals > maxDecimals) return false;
  }

  return true;
}

function isMoneyNoSymbol(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;

  const maxDecimals =
    typeof args === "object" && args !== null
      ? ((args as { max_decimals?: number }).max_decimals ?? 2)
      : 2;
  const allowNegative =
    typeof args === "object" && args !== null
      ? ((args as { allow_negative?: boolean }).allow_negative ?? true)
      : true;

  let i = 0;
  // Optional leading minus
  if (s.charCodeAt(i) === 45) {
    // '-'
    if (!allowNegative) return false;
    i++;
    if (i >= s.length) return false;
  }

  // Must start with a digit
  if (s.charCodeAt(i) < 48 || s.charCodeAt(i) > 57) return false;

  // Integer part: digits with optional comma grouping
  let digitsSinceComma = 0;
  let hasComma = false;
  let intDigits = 0;
  for (; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (c >= 48 && c <= 57) {
      // '0'-'9'
      digitsSinceComma++;
      intDigits++;
    } else if (c === 44) {
      // ','
      if (hasComma && digitsSinceComma !== 3) return false;
      if (!hasComma && intDigits > 3) return false;
      hasComma = true;
      digitsSinceComma = 0;
    } else if (c === 46) {
      // '.'
      break;
    } else {
      return false;
    }
  }
  if (hasComma && digitsSinceComma !== 3) return false;
  if (intDigits === 0) return false;

  // Decimal part
  if (i < s.length && s.charCodeAt(i) === 46) {
    i++;
    const decStart = i;
    for (; i < s.length; i++) {
      const c = s.charCodeAt(i);
      if (c < 48 || c > 57) return false;
    }
    const decimals = i - decStart;
    if (decimals === 0 || decimals > maxDecimals) return false;
  }

  return i === s.length;
}

function isMultipleOf(value: unknown, args: unknown): boolean {
  const n = extractNumber(value);
  const step =
    typeof args === "object" && args !== null
      ? extractNumber((args as { value?: unknown }).value)
      : null;

  if (n === null || step === null || step === 0) return false;
  const ratio = n / step;
  return Math.abs(ratio - Math.round(ratio)) < 1e-9;
}

function lessThanOrEqual(value: unknown, args: unknown): boolean {
  const n = extractNumber(value);
  const max =
    typeof args === "object" && args !== null
      ? extractNumber((args as { value?: unknown }).value)
      : null;
  return n !== null && max !== null && n <= max;
}

function greaterThanOrEqual(value: unknown, args: unknown): boolean {
  const n = extractNumber(value);
  const min =
    typeof args === "object" && args !== null
      ? extractNumber((args as { value?: unknown }).value)
      : null;
  return n !== null && min !== null && n >= min;
}

// Reference predicates
const US_STATE_CODES = new Set([
  "AL",
  "AK",
  "AZ",
  "AR",
  "CA",
  "CO",
  "CT",
  "DE",
  "FL",
  "GA",
  "HI",
  "ID",
  "IL",
  "IN",
  "IA",
  "KS",
  "KY",
  "LA",
  "ME",
  "MD",
  "MA",
  "MI",
  "MN",
  "MS",
  "MO",
  "MT",
  "NE",
  "NV",
  "NH",
  "NJ",
  "NM",
  "NY",
  "NC",
  "ND",
  "OH",
  "OK",
  "OR",
  "PA",
  "RI",
  "SC",
  "SD",
  "TN",
  "TX",
  "UT",
  "VT",
  "VA",
  "WA",
  "WV",
  "WI",
  "WY",
  "DC",
  "AS",
  "GU",
  "MP",
  "PR",
  "VI",
  "UM",
  "AA",
  "AE",
  "AP",
]);

const COUNTRY_NAMES = new Set([
  "united states",
  "canada",
  "mexico",
  "united kingdom",
  "germany",
  "france",
  "italy",
  "spain",
  "japan",
  "china",
  "india",
  "australia",
  "brazil",
  "south korea",
  "russia",
  "netherlands",
  "switzerland",
  "ireland",
  "singapore",
  "hong kong",
]);

const US_STATE_NAMES = new Set([
  "alabama",
  "alaska",
  "arizona",
  "arkansas",
  "california",
  "colorado",
  "connecticut",
  "delaware",
  "district of columbia",
  "florida",
  "georgia",
  "hawaii",
  "idaho",
  "illinois",
  "indiana",
  "iowa",
  "kansas",
  "kentucky",
  "louisiana",
  "maine",
  "maryland",
  "massachusetts",
  "michigan",
  "minnesota",
  "mississippi",
  "missouri",
  "montana",
  "nebraska",
  "nevada",
  "new hampshire",
  "new jersey",
  "new mexico",
  "new york",
  "north carolina",
  "north dakota",
  "ohio",
  "oklahoma",
  "oregon",
  "pennsylvania",
  "rhode island",
  "south carolina",
  "south dakota",
  "tennessee",
  "texas",
  "utah",
  "vermont",
  "virginia",
  "washington",
  "west virginia",
  "wisconsin",
  "wyoming",
]);

const US_TERRITORY_NAMES = new Set([
  "american samoa",
  "guam",
  "northern mariana islands",
  "puerto rico",
  "u.s. virgin islands",
  "united states minor outlying islands",
]);

const FILING_STATUSES = new Set([
  "S",
  "MFJ",
  "MFS",
  "HOH",
  "QW",
  "1",
  "2",
  "3",
  "4",
  "5",
]);

const ISO_COUNTRY_CODES = new Set([
  // Major countries
  "US",
  "CA",
  "MX",
  "GB",
  "DE",
  "FR",
  "IT",
  "ES",
  "JP",
  "CN",
  "IN",
  "AU",
  "BR",
  "KR",
  "RU",
  // European Union
  "AT",
  "BE",
  "BG",
  "HR",
  "CY",
  "CZ",
  "DK",
  "EE",
  "FI",
  "GR",
  "HU",
  "IE",
  "LV",
  "LT",
  "LU",
  "MT",
  "NL",
  "PL",
  "PT",
  "RO",
  "SK",
  "SI",
  "SE",
  // Other common
  "CH",
  "NO",
  "IL",
  "SG",
  "HK",
  "TW",
  "NZ",
  "ZA",
  "AE",
  "SA",
  "AR",
  "CL",
  "CO",
  "PE",
  "VE",
  "PH",
  "TH",
  "VN",
  "MY",
  "ID",
  "PK",
  "BD",
  "EG",
  "NG",
  "KE",
  "TR",
  "UA",
  // Tax havens / financial centers
  "BM",
  "KY",
  "VG",
  "BS",
  "PA",
  "LI",
  "MC",
  "AD",
  "GI",
  "JE",
  "GG",
  "IM",
  // Caribbean
  "JM",
  "TT",
  "BB",
  "PR",
  "VI",
  "CU",
  "DO",
  "HT",
  // Central America
  "GT",
  "HN",
  "SV",
  "NI",
  "CR",
  "BZ",
  // Africa, Middle East, Asia, etc.
  "AF",
  "AL",
  "DZ",
  "AO",
  "AM",
  "AZ",
  "BH",
  "BY",
  "BJ",
  "BT",
  "BO",
  "BA",
  "BW",
  "BN",
  "BF",
  "BI",
  "KH",
  "CM",
  "CV",
  "CF",
  "TD",
  "CI",
  "CD",
  "CG",
  "DJ",
  "EC",
  "GQ",
  "ER",
  "ET",
  "FJ",
  "GA",
  "GM",
  "GE",
  "GH",
  "GN",
  "GW",
  "GY",
  "IQ",
  "IR",
  "IS",
  "JO",
  "KZ",
  "KW",
  "KG",
  "LA",
  "LB",
  "LS",
  "LR",
  "LY",
  "MK",
  "MG",
  "MW",
  "MV",
  "ML",
  "MR",
  "MU",
  "MD",
  "MN",
  "ME",
  "MA",
  "MZ",
  "MM",
  "NA",
  "NP",
  "NE",
  "KP",
  "OM",
  "QA",
  "RW",
  "RS",
  "SL",
  "SO",
  "SS",
  "SD",
  "SR",
  "SZ",
  "SY",
  "TJ",
  "TZ",
  "TG",
  "TO",
  "TN",
  "TM",
  "UG",
  "UY",
  "UZ",
  "VU",
  "YE",
  "ZM",
  "ZW",
]);

const W2_BOX12_CODES = new Set([
  "A",
  "B",
  "C",
  "D",
  "E",
  "F",
  "G",
  "H",
  "J",
  "K",
  "L",
  "M",
  "N",
  "P",
  "Q",
  "R",
  "S",
  "T",
  "V",
  "W",
  "Y",
  "Z",
  "AA",
  "BB",
  "DD",
  "EE",
  "FF",
  "GG",
  "HH",
]);

const CODE_1099B_TERM = new Set(["A", "B", "C", "D", "E", "F", "X"]);
const CODE_1099B_BASIS = new Set(["1", "2", "3"]);
const CODE_1099B_TYPE = new Set(["P", "S", "C", "O", "W"]);
const IATA_AIRPORT_CODES = new Set([
  "SFO",
  "LAX",
  "JFK",
  "ORD",
  "DFW",
  "DEN",
  "SEA",
  "BOS",
  "MIA",
  "ATL",
  "PHX",
  "IAH",
  "EWR",
  "LHR",
  "LGW",
  "CDG",
  "FRA",
  "AMS",
  "MAD",
  "BCN",
  "NRT",
  "HND",
  "KIX",
  "SYD",
  "MEL",
  "YYZ",
  "YVR",
  "DXB",
  "SIN",
  "HKG",
]);
const ICAO_AIRPORT_CODES = new Set([
  "KSFO",
  "KLAX",
  "KJFK",
  "KORD",
  "KDFW",
  "KDEN",
  "KSEA",
  "KBOS",
  "KMIA",
  "KATL",
  "KPHX",
  "KIAH",
  "KEWR",
  "EGLL",
  "EGKK",
  "LFPG",
  "EDDF",
  "EHAM",
  "LEMD",
  "LEBL",
  "RJAA",
  "RJTT",
  "RJBB",
  "YSSY",
  "YMML",
  "CYYZ",
  "CYVR",
  "OMDB",
  "WSSS",
  "VHHH",
]);
const IATA_AIRLINE_CODES = new Set([
  "UA",
  "AA",
  "DL",
  "WN",
  "B6",
  "AS",
  "NK",
  "F9",
  "AC",
  "BA",
  "LH",
  "AF",
  "KL",
  "EK",
  "SQ",
  "NH",
  "JL",
  "QF",
]);
const ICAO_AIRLINE_CODES = new Set([
  "UAL",
  "AAL",
  "DAL",
  "SWA",
  "JBU",
  "ASA",
  "NKS",
  "FFT",
  "ACA",
  "BAW",
  "DLH",
  "AFR",
  "KLM",
  "UAE",
  "SIA",
  "ANA",
  "JAL",
  "QFA",
]);

function isCountryCode(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toUpperCase();
  if (s.length !== 2) return false;

  const excludeUs =
    typeof args === "object" && args !== null
      ? ((args as { exclude_us?: boolean }).exclude_us ?? false)
      : false;
  if (excludeUs && s === "US") return false;

  return ISO_COUNTRY_CODES.has(s);
}

function isCountryName(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toLowerCase();
  return s.length > 0 && COUNTRY_NAMES.has(s);
}

function isCurrencyCode(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toUpperCase();
  if (s.length !== 3) return false;
  return ISO_CURRENCY_CODES.has(s);
}

const ISO_CURRENCY_CODES = new Set([
  "USD",
  "EUR",
  "GBP",
  "JPY",
  "CNY",
  "CAD",
  "AUD",
  "CHF",
  "HKD",
  "SGD",
  "SEK",
  "NOK",
  "DKK",
  "NZD",
  "MXN",
  "BRL",
  "INR",
  "RUB",
  "ZAR",
  "KRW",
  "TWD",
  "THB",
  "MYR",
  "IDR",
  "PHP",
  "VND",
  "AED",
  "SAR",
  "ILS",
  "TRY",
  "PLN",
  "CZK",
  "HUF",
  "RON",
  "BGN",
  "HRK",
  "ISK",
  "CLP",
  "COP",
  "PEN",
  "ARS",
  "EGP",
  "NGN",
  "KES",
  "PKR",
  "BDT",
]);

function isUsState(value: unknown): boolean {
  if (typeof value !== "string") return false;
  return US_STATE_CODES.has(value.trim().toUpperCase());
}

function isStateName(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toLowerCase();
  if (s.length === 0) return false;
  if (US_STATE_NAMES.has(s)) return true;

  const includeTerritories =
    typeof args === "object" && args !== null
      ? ((args as { include_territories?: boolean }).include_territories ??
        true)
      : true;
  return includeTerritories && US_TERRITORY_NAMES.has(s);
}

function isUsZip(value: unknown): boolean {
  const digits = extractDigits(value);
  if (digits.length !== 5 && digits.length !== 9) return false;
  if (isAllSameDigit(digits, 48)) return false;
  return true;
}

function isStreetAddress(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length < 5) return false;
  // Must start with a digit (street number) and contain at least one letter (street name)
  if (!/^\d/.test(s)) return false;
  if (!/[A-Za-z]/.test(s)) return false;
  return true;
}

function isFilingStatus(value: unknown): boolean {
  if (typeof value !== "string") return false;
  return FILING_STATUSES.has(value.trim().toUpperCase());
}

// Text character class predicates (no regex — pure charCode scanning)

function isAlpha(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (!((c >= 65 && c <= 90) || (c >= 97 && c <= 122))) return false;
  }
  return true;
}

function isDigits(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (c < 48 || c > 57) return false;
  }
  return true;
}

function isAlphanumeric(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (
      !((c >= 65 && c <= 90) || (c >= 97 && c <= 122) || (c >= 48 && c <= 57))
    )
      return false;
  }
  return true;
}

function isAlphaSpaces(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;
  let hasLetter = false;
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if ((c >= 65 && c <= 90) || (c >= 97 && c <= 122)) {
      hasLetter = true;
    } else if (c !== 32) {
      return false;
    }
  }
  return hasLetter;
}

function isAlphanumericSpaces(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;
  let hasAlnum = false;
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if ((c >= 65 && c <= 90) || (c >= 97 && c <= 122) || (c >= 48 && c <= 57)) {
      hasAlnum = true;
    } else if (c !== 32) {
      return false;
    }
  }
  return hasAlnum;
}

function isNameChars(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;
  let hasLetter = false;
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if ((c >= 65 && c <= 90) || (c >= 97 && c <= 122)) {
      hasLetter = true;
    } else if (c !== 45 && c !== 39) {
      // 45 = '-', 39 = "'"
      return false;
    }
  }
  return hasLetter;
}

function isUppercase(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (c < 65 || c > 90) return false;
  }
  return true;
}

function isLowercase(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (c < 97 || c > 122) return false;
  }
  return true;
}

function isTitleCase(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length < 2) return false;
  const first = s.charCodeAt(0);
  if (first < 65 || first > 90) return false;
  for (let i = 1; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (c < 97 || c > 122) return false;
  }
  return true;
}

function isW2Box12Code(value: unknown): boolean {
  if (typeof value !== "string") return false;
  return W2_BOX12_CODES.has(value.trim().toUpperCase());
}

function is1099BCode(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toUpperCase();

  const codeType =
    typeof args === "object" && args !== null
      ? (args as { type?: string }).type
      : undefined;

  switch (codeType) {
    case "term":
      return CODE_1099B_TERM.has(s);
    case "basis":
      return CODE_1099B_BASIS.has(s);
    case "type":
      return CODE_1099B_TYPE.has(s);
    default:
      return (
        CODE_1099B_TERM.has(s) ||
        CODE_1099B_BASIS.has(s) ||
        CODE_1099B_TYPE.has(s)
      );
  }
}

// Aviation predicates

function isIataAirportCode(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toUpperCase();
  if (s.length !== 3 || !/^[A-Z]{3}$/.test(s)) return false;

  const knownOnly =
    typeof args === "object" && args !== null
      ? ((args as { known_only?: boolean }).known_only ?? false)
      : false;

  return knownOnly ? IATA_AIRPORT_CODES.has(s) : true;
}

function isIcaoAirportCode(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toUpperCase();
  if (s.length !== 4 || !/^[A-Z]{4}$/.test(s)) return false;

  const knownOnly =
    typeof args === "object" && args !== null
      ? ((args as { known_only?: boolean }).known_only ?? false)
      : false;

  return knownOnly ? ICAO_AIRPORT_CODES.has(s) : true;
}

function isIataAirlineCode(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toUpperCase();
  if (s.length !== 2 || !/^[A-Z0-9]{2}$/.test(s)) return false;

  const knownOnly =
    typeof args === "object" && args !== null
      ? ((args as { known_only?: boolean }).known_only ?? false)
      : false;

  return knownOnly ? IATA_AIRLINE_CODES.has(s) : true;
}

function isIcaoAirlineCode(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toUpperCase();
  if (s.length !== 3 || !/^[A-Z]{3}$/.test(s)) return false;

  const knownOnly =
    typeof args === "object" && args !== null
      ? ((args as { known_only?: boolean }).known_only ?? false)
      : false;

  return knownOnly ? ICAO_AIRLINE_CODES.has(s) : true;
}

function isAirportCode(value: unknown, args: unknown): boolean {
  const a = args as { system?: string; known_only?: boolean } | undefined;
  const system = a?.system?.toUpperCase() ?? "ANY";
  const knownOnly = a?.known_only ?? false;
  const knownArgs = { known_only: knownOnly };

  switch (system) {
    case "IATA":
      return isIataAirportCode(value, knownArgs);
    case "ICAO":
      return isIcaoAirportCode(value, knownArgs);
    case "ANY":
      return (
        isIataAirportCode(value, knownArgs) ||
        isIcaoAirportCode(value, knownArgs)
      );
    default:
      return false;
  }
}

function isAirlineCode(value: unknown, args: unknown): boolean {
  const a = args as { system?: string; known_only?: boolean } | undefined;
  const system = a?.system?.toUpperCase() ?? "ANY";
  const knownOnly = a?.known_only ?? false;
  const knownArgs = { known_only: knownOnly };

  switch (system) {
    case "IATA":
      return isIataAirlineCode(value, knownArgs);
    case "ICAO":
      return isIcaoAirlineCode(value, knownArgs);
    case "ANY":
      return (
        isIataAirlineCode(value, knownArgs) ||
        isIcaoAirlineCode(value, knownArgs)
      );
    default:
      return false;
  }
}

function isFlightNumber(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;

  const compact = value.trim().toUpperCase().replace(/\s+/g, "");
  if (compact.length < 3) return false;

  const a = args as
    | {
        carrier_format?: string;
        known_carrier?: boolean;
        allow_suffix?: boolean;
      }
    | undefined;
  const carrierFormat = a?.carrier_format?.toUpperCase() ?? "ANY";
  const knownCarrier = a?.known_carrier ?? false;
  const allowSuffix = a?.allow_suffix ?? true;

  switch (carrierFormat) {
    case "IATA":
      return validateFlightNumber(compact, 2, knownCarrier, allowSuffix);
    case "ICAO":
      return validateFlightNumber(compact, 3, knownCarrier, allowSuffix);
    default:
      return (
        validateFlightNumber(compact, 2, knownCarrier, allowSuffix) ||
        validateFlightNumber(compact, 3, knownCarrier, allowSuffix)
      );
  }
}

function validateFlightNumber(
  s: string,
  carrierLen: 2 | 3,
  knownCarrier: boolean,
  allowSuffix: boolean,
): boolean {
  if (s.length <= carrierLen) return false;

  const carrier = s.slice(0, carrierLen);
  const rest = s.slice(carrierLen);

  if (carrierLen === 2) {
    if (!/^[A-Z0-9]{2}$/.test(carrier)) return false;
    if (knownCarrier && !IATA_AIRLINE_CODES.has(carrier)) return false;
  } else {
    if (!/^[A-Z]{3}$/.test(carrier)) return false;
    if (knownCarrier && !ICAO_AIRLINE_CODES.has(carrier)) return false;
  }

  if (allowSuffix) {
    if (/^\d{1,4}[A-Z]$/.test(rest)) return true;
  }
  return /^\d{1,4}$/.test(rest);
}

// Contact predicates

/**
 * Strict US phone number validation.
 * Rejects international prefix (+).
 */
function isPhoneNumberUS(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;

  // US format — reject international prefix
  if (value.trim().startsWith("+")) return false;

  const digits = extractDigits(value);

  if (digits.length === 10) {
    const area = digits[0];
    const exchange = digits[3];
    return area !== "0" && area !== "1" && exchange !== "0" && exchange !== "1";
  }
  if (digits.length === 11 && digits[0] === "1") {
    const area = digits[1];
    const exchange = digits[4];
    return area !== "0" && area !== "1" && exchange !== "0" && exchange !== "1";
  }

  const requireAreaCode =
    typeof args === "object" && args !== null
      ? ((args as { require_area_code?: boolean }).require_area_code ?? true)
      : true;

  if (!requireAreaCode && digits.length === 7) {
    return digits[0] !== "0" && digits[0] !== "1";
  }

  return false;
}

/**
 * General phone number validation.
 * Accepts US format OR international format (+ prefix, 7-15 digits).
 */
function isPhoneNumber(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;

  const trimmed = value.trim();

  // International format: starts with +, 7-15 digits
  if (trimmed.startsWith("+")) {
    const digits = extractDigits(value);
    return digits.length >= 7 && digits.length <= 15;
  }

  // Otherwise, validate as US phone
  return isPhoneNumberUS(value, args);
}

function isEmail(value: unknown): boolean {
  if (typeof value !== "string") return false;

  let start = 0;
  let end = value.length;
  while (start < end && isTrimWhitespace(value.charCodeAt(start))) start++;
  while (end > start && isTrimWhitespace(value.charCodeAt(end - 1))) end--;
  if (start === end) return false;

  let at = -1;
  for (let i = start; i < end; i++) {
    if (value.charCodeAt(i) === 64) {
      if (at !== -1) return false;
      at = i;
    }
  }
  if (at <= start || at >= end - 1) return false;

  const localStart = start;
  const localEnd = at;
  const domainStart = at + 1;
  const domainEnd = end;
  const localLength = localEnd - localStart;
  const domainLength = domainEnd - domainStart;

  if (localLength > 64 || domainLength > 253) return false;

  if (
    value.charCodeAt(localStart) === 46 ||
    value.charCodeAt(localEnd - 1) === 46
  ) {
    return false;
  }

  for (let i = localStart; i < localEnd; i++) {
    if (!isEmailLocalChar(value.charCodeAt(i))) return false;
  }

  const firstDomainChar = value.charCodeAt(domainStart);
  const lastDomainChar = value.charCodeAt(domainEnd - 1);
  if (
    firstDomainChar === 46 ||
    firstDomainChar === 45 ||
    lastDomainChar === 46 ||
    lastDomainChar === 45
  ) {
    return false;
  }

  let lastDot = -1;
  for (let i = domainStart; i < domainEnd; i++) {
    const code = value.charCodeAt(i);
    if (code === 46) {
      lastDot = i;
      continue;
    }
    if (!isAsciiAlphanumeric(code) && code !== 45) return false;
  }

  if (lastDot === -1 || domainEnd - lastDot - 1 < 2) return false;
  for (let i = lastDot + 1; i < domainEnd; i++) {
    if (!isAsciiAlpha(value.charCodeAt(i))) return false;
  }

  return true;
}

function isEmailLocalChar(code: number): boolean {
  return (
    isAsciiAlphanumeric(code) ||
    code === 46 ||
    code === 95 ||
    code === 37 ||
    code === 43 ||
    code === 45
  );
}

function isAsciiAlphanumeric(code: number): boolean {
  return isAsciiAlpha(code) || (code >= 48 && code <= 57);
}

function isAsciiAlpha(code: number): boolean {
  return (code >= 65 && code <= 90) || (code >= 97 && code <= 122);
}

function isAsciiDigit(code: number): boolean {
  return code >= 48 && code <= 57;
}

function isAsciiHexDigit(code: number): boolean {
  return (
    isAsciiDigit(code) ||
    (code >= 65 && code <= 70) ||
    (code >= 97 && code <= 102)
  );
}

function isAsciiDigitsOnly(value: string): boolean {
  if (value.length === 0) return false;
  for (let i = 0; i < value.length; i++) {
    if (!isAsciiDigit(value.charCodeAt(i))) return false;
  }
  return true;
}

function isAsciiHexDigitsOnlyRange(
  value: string,
  start: number,
  end: number,
): boolean {
  if (start >= end) return false;
  for (let i = start; i < end; i++) {
    if (!isAsciiHexDigit(value.charCodeAt(i))) return false;
  }
  return true;
}

function isAllSameDigit(value: string, digitCode: number): boolean {
  if (value.length === 0) return false;
  for (let i = 0; i < value.length; i++) {
    if (value.charCodeAt(i) !== digitCode) return false;
  }
  return true;
}

function toAsciiUpperCode(code: number): number {
  return code >= 97 && code <= 122 ? code - 32 : code;
}

function toAsciiLowerCode(code: number): number {
  return code >= 65 && code <= 90 ? code + 32 : code;
}

function isTrimWhitespace(code: number): boolean {
  return (
    code === 32 || (code >= 9 && code <= 13) || code === 160 || code === 65279
  );
}

function isRegexWhitespace(code: number): boolean {
  return (
    code === 32 ||
    (code >= 9 && code <= 13) ||
    code === 160 ||
    code === 5760 ||
    (code >= 8192 && code <= 8202) ||
    code === 8232 ||
    code === 8233 ||
    code === 8239 ||
    code === 8287 ||
    code === 12288 ||
    code === 65279
  );
}

function collectDigitsIgnoringSpaceHyphen(value: string): string | null {
  let digits = "";
  let changed = false;
  for (let i = 0; i < value.length; i++) {
    const code = value.charCodeAt(i);
    if (isAsciiDigit(code)) {
      if (changed) digits += value.charAt(i);
    } else if (code === 45 || isRegexWhitespace(code)) {
      if (!changed) {
        digits = value.slice(0, i);
        changed = true;
      }
    } else {
      return null;
    }
  }
  return changed ? digits : value;
}

function isUrl(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;

  const requireHttps =
    typeof args === "object" && args !== null
      ? ((args as { require_https?: boolean }).require_https ?? false)
      : false;

  const hasHttp = s.startsWith("http://");
  const hasHttps = s.startsWith("https://");

  if (requireHttps && !hasHttps) return false;
  if (!hasHttp && !hasHttps) return false;

  // Extract everything after the scheme
  const rest = hasHttps ? s.slice(8) : s.slice(7);
  if (rest.length === 0) return false;

  // Must have a domain part
  const domainPart = rest.split("/")[0];
  if (!domainPart || domainPart.length === 0) return false;

  // Strip port if present
  const domainWithoutPort = domainPart.split(":")[0];
  if (!domainWithoutPort || domainWithoutPort.length === 0) return false;

  // Basic domain validation
  return /^[a-zA-Z0-9.-]+$/.test(domainWithoutPort);
}

function isUuid(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toLowerCase();

  // Must be 8-4-4-4-12 hex with dashes
  if (
    !/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(s)
  ) {
    return false;
  }

  // Optional version filter (version digit is at position 14)
  const version =
    typeof args === "object" && args !== null
      ? (args as { version?: number }).version
      : undefined;

  if (version !== undefined) {
    const actualVersion = Number.parseInt(s.charAt(14), 16);
    if (actualVersion !== version) return false;
  }

  return true;
}

function isIp(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  const version =
    typeof args === "object" && args !== null
      ? (args as { version?: string }).version?.toLowerCase()
      : undefined;

  const v4 =
    /^(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}$/;
  const v6 =
    /^(([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|(([0-9a-fA-F]{1,4}:){1,7}:)|(:([0-9a-fA-F]{1,4}:){1,7})|(([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4})|(([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2})|(([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3})|(([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4})|(([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5})|([0-9a-fA-F]{1,4}:)((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:))$/;

  if (version === "v4") return v4.test(s);
  if (version === "v6") return v6.test(s);
  return v4.test(s) || v6.test(s);
}

function isCidr(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  const slash = s.indexOf("/");
  if (slash <= 0 || slash === s.length - 1) return false;

  const ip = s.slice(0, slash);
  const prefixStr = s.slice(slash + 1);
  if (!/^\d+$/.test(prefixStr)) return false;
  const prefix = Number.parseInt(prefixStr, 10);

  const version =
    typeof args === "object" && args !== null
      ? (args as { version?: string }).version?.toLowerCase()
      : undefined;

  if (version === "v4") {
    return isIp(ip, { version: "v4" }) && prefix >= 0 && prefix <= 32;
  }
  if (version === "v6") {
    return isIp(ip, { version: "v6" }) && prefix >= 0 && prefix <= 128;
  }

  if (isIp(ip, { version: "v4" })) return prefix >= 0 && prefix <= 32;
  if (isIp(ip, { version: "v6" })) return prefix >= 0 && prefix <= 128;
  return false;
}

function isMacAddress(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  const colonOrHyphen = /^([0-9A-Fa-f]{2}([:-])){5}[0-9A-Fa-f]{2}$/;
  const cisco = /^[0-9A-Fa-f]{4}(\.[0-9A-Fa-f]{4}){2}$/;
  return colonOrHyphen.test(s) || cisco.test(s);
}

// Payment card predicates

function isCreditCard(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const digits = extractDigits(value);

  // Luhn check
  if (!luhnDigits(digits)) return false;

  const network =
    typeof args === "object" && args !== null
      ? (args as { network?: string }).network
      : undefined;

  switch (network) {
    case "visa":
      return isVisa(digits);
    case "mastercard":
      return isMastercard(digits);
    case "amex":
      return isAmex(digits);
    case "discover":
      return isDiscover(digits);
    default:
      return (
        isVisa(digits) ||
        isMastercard(digits) ||
        isAmex(digits) ||
        isDiscover(digits)
      );
  }
}

function isVisa(digits: string): boolean {
  return digits.length === 16 && digits.startsWith("4");
}

function isMastercard(digits: string): boolean {
  if (digits.length !== 16) return false;
  const prefix2 = Number.parseInt(digits.slice(0, 2), 10);
  if (prefix2 >= 51 && prefix2 <= 55) return true;
  const prefix4 = Number.parseInt(digits.slice(0, 4), 10);
  if (prefix4 >= 2221 && prefix4 <= 2720) return true;
  return false;
}

function isAmex(digits: string): boolean {
  return (
    digits.length === 15 && (digits.startsWith("34") || digits.startsWith("37"))
  );
}

function isDiscover(digits: string): boolean {
  if (digits.length !== 16) return false;
  if (digits.startsWith("6011") || digits.startsWith("65")) return true;
  const prefix3 = Number.parseInt(digits.slice(0, 3), 10);
  if (prefix3 >= 644 && prefix3 <= 649) return true;
  return false;
}

function isCvv(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (!isAsciiDigitsOnly(s)) return false;

  const network =
    typeof args === "object" && args !== null
      ? (args as { network?: string }).network
      : undefined;

  switch (network) {
    case "amex":
      return s.length === 4;
    case undefined:
      return s.length === 3 || s.length === 4;
    default:
      return s.length === 3;
  }
}

function isCardExpiry(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();

  const parts = s.split("/");
  if (parts.length !== 2) return false;

  const [monthStr, yearStr] = parts;
  if (!monthStr || !yearStr) return false;

  // Month must be exactly 2 digits
  if (monthStr.length !== 2 || !/^\d{2}$/.test(monthStr)) return false;

  // Year must be 2 or 4 digits
  if ((yearStr.length !== 2 && yearStr.length !== 4) || !/^\d+$/.test(yearStr))
    return false;

  const month = Number.parseInt(monthStr, 10);
  if (month < 1 || month > 12) return false;

  const rejectExpired =
    typeof args === "object" && args !== null
      ? ((args as { reject_expired?: boolean }).reject_expired ?? false)
      : false;

  if (rejectExpired) {
    let year = Number.parseInt(yearStr, 10);
    if (year < 100) year += 2000;

    const now = new Date();
    const curYear = now.getFullYear();
    const curMonth = now.getMonth() + 1; // getMonth is 0-based

    if (year < curYear || (year === curYear && month < curMonth)) {
      return false;
    }
  }

  return true;
}

// International banking predicates

const IBAN_COUNTRY_LENGTHS: Record<string, number> = {
  AL: 28,
  AD: 24,
  AT: 20,
  AZ: 28,
  BH: 22,
  BY: 28,
  BE: 16,
  BA: 20,
  BR: 29,
  BG: 22,
  CR: 22,
  HR: 21,
  CY: 28,
  CZ: 24,
  DK: 18,
  DO: 28,
  EE: 20,
  FO: 18,
  FI: 18,
  FR: 27,
  GE: 22,
  DE: 22,
  GI: 23,
  GR: 27,
  GL: 18,
  GT: 28,
  HU: 28,
  IS: 26,
  IQ: 23,
  IE: 22,
  IL: 23,
  IT: 27,
  JO: 30,
  KZ: 20,
  XK: 20,
  KW: 30,
  LV: 21,
  LB: 28,
  LI: 21,
  LT: 20,
  LU: 20,
  MK: 19,
  MT: 31,
  MR: 27,
  MU: 30,
  MC: 27,
  MD: 24,
  ME: 22,
  NL: 18,
  NO: 15,
  PK: 24,
  PS: 29,
  PL: 28,
  PT: 25,
  QA: 29,
  RO: 24,
  LC: 32,
  SM: 27,
  SA: 24,
  RS: 22,
  SC: 31,
  SK: 24,
  SI: 19,
  ES: 24,
  SE: 24,
  CH: 21,
  TN: 24,
  TR: 26,
  AE: 23,
  GB: 22,
  VA: 22,
  VG: 24,
  UA: 29,
};

function isIban(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;

  const iban: number[] = [];
  for (let i = 0; i < value.length; i++) {
    const code = value.charCodeAt(i);
    if (isRegexWhitespace(code)) continue;

    const upper = toAsciiUpperCode(code);
    if (!isAsciiDigit(upper) && !isAsciiAlpha(upper)) return false;
    iban.push(upper);
  }

  if (iban.length < 5 || iban.length > 34) return false;

  const country0 = iban[0] ?? 0;
  const country1 = iban[1] ?? 0;
  const check0 = iban[2] ?? 0;
  const check1 = iban[3] ?? 0;

  if (!isAsciiAlpha(country0) || !isAsciiAlpha(country1)) return false;
  if (!isAsciiDigit(check0) || !isAsciiDigit(check1)) return false;

  for (let i = 4; i < iban.length; i++) {
    const code = iban[i] ?? 0;
    if (!isAsciiAlphanumeric(code)) return false;
  }

  // Optional country filter
  const country = String.fromCharCode(country0, country1);
  const requiredCountry =
    typeof args === "object" && args !== null
      ? (args as { country?: string }).country
      : undefined;
  if (requiredCountry && country !== requiredCountry.toUpperCase())
    return false;

  // Country-specific length
  const expectedLen = IBAN_COUNTRY_LENGTHS[country];
  if (expectedLen !== undefined && iban.length !== expectedLen) return false;

  // Mod-97 checksum
  return ibanMod97(iban);
}

function ibanMod97(iban: number[]): boolean {
  let remainder = 0;

  for (let i = 4; i < iban.length; i++) {
    remainder = ibanMod97Step(iban[i] ?? 0, remainder);
  }
  for (let i = 0; i < 4; i++) {
    remainder = ibanMod97Step(iban[i] ?? 0, remainder);
  }

  return remainder === 1;
}

function ibanMod97Step(code: number, remainder: number): number {
  if (code >= 48 && code <= 57) {
    return (remainder * 10 + (code - 48)) % 97;
  }

  // Letter: A=10, B=11, ..., Z=35 (two digits)
  return (remainder * 100 + (code - 55)) % 97;
}

function isBic(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();

  // Must be 8 or 11 characters
  if (s.length !== 8 && s.length !== 11) return false;

  // Bank code: first 4 chars must be letters
  for (let i = 0; i < 4; i++) {
    if (!isAsciiAlpha(toAsciiUpperCode(s.charCodeAt(i)))) return false;
  }

  // Country code: chars 4-5 must be letters
  if (
    !isAsciiAlpha(toAsciiUpperCode(s.charCodeAt(4))) ||
    !isAsciiAlpha(toAsciiUpperCode(s.charCodeAt(5)))
  ) {
    return false;
  }

  // Location code: chars 6-7 must be alphanumeric
  const loc0 = toAsciiUpperCode(s.charCodeAt(6));
  const loc1 = toAsciiUpperCode(s.charCodeAt(7));
  if (!isAsciiAlphanumeric(loc0) || !isAsciiAlphanumeric(loc1)) return false;

  // Branch code (if present): chars 8-10 must be alphanumeric
  if (s.length === 11) {
    for (let i = 8; i < 11; i++) {
      if (!isAsciiAlphanumeric(toAsciiUpperCode(s.charCodeAt(i)))) {
        return false;
      }
    }
  }

  return true;
}

// Product / ecommerce predicates

const VIN_WEIGHTS = [8, 7, 6, 5, 4, 3, 2, 10, 0, 9, 8, 7, 6, 5, 4, 3, 2];

function vinTransliterate(c: string): number | null {
  if (c >= "0" && c <= "9") return c.charCodeAt(0) - 48;
  switch (c) {
    case "A":
    case "J":
      return 1;
    case "B":
    case "K":
    case "S":
      return 2;
    case "C":
    case "L":
    case "T":
      return 3;
    case "D":
    case "M":
    case "U":
      return 4;
    case "E":
    case "N":
    case "V":
      return 5;
    case "F":
    case "W":
      return 6;
    case "G":
    case "P":
    case "X":
      return 7;
    case "H":
    case "Y":
      return 8;
    case "R":
    case "Z":
      return 9;
    default:
      return null; // I, O, Q are invalid
  }
}

function isVin(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toUpperCase();

  if (s.length !== 17) return false;

  // VIN charset: 0-9, A-Z except I, O, Q
  if (!/^[A-HJ-NPR-Z0-9]{17}$/.test(s)) return false;

  const validateChecksum =
    typeof args === "object" && args !== null
      ? ((args as { validate_checksum?: boolean }).validate_checksum ?? true)
      : true;

  if (!validateChecksum) return true;

  // Check digit at position 9 (0-indexed 8)
  let sum = 0;
  for (let i = 0; i < 17; i++) {
    if (i === 8) continue;
    const val = vinTransliterate(s.charAt(i));
    if (val === null) return false;
    const weight = VIN_WEIGHTS[i];
    if (weight === undefined) return false;
    sum += val * weight;
  }
  const remainder = sum % 11;
  const expected = remainder === 10 ? "X" : String(remainder);
  return s[8] === expected;
}

function isUpc(value: unknown): boolean {
  if (typeof value !== "string") return false;

  const digits = collectDigitsIgnoringSpaceHyphen(value);
  if (digits === null || digits.length !== 12) return false;

  // UPC check: odd positions (0-indexed even) × 3, even × 1
  let sum = 0;
  for (let i = 0; i < 12; i++) {
    const d = digits.charCodeAt(i) - 48;
    sum += i % 2 === 0 ? d * 3 : d;
  }
  return sum % 10 === 0;
}

function isEan(value: unknown): boolean {
  if (typeof value !== "string") return false;

  const digits = collectDigitsIgnoringSpaceHyphen(value);
  if (digits === null || digits.length !== 13) return false;

  // EAN-13 check: alternating weights 1, 3
  let sum = 0;
  for (let i = 0; i < 13; i++) {
    const d = digits.charCodeAt(i) - 48;
    sum += i % 2 === 0 ? d : d * 3;
  }
  return sum % 10 === 0;
}

function isIsbn(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();

  const version =
    typeof args === "object" && args !== null
      ? (args as { version?: number }).version
      : undefined;

  const cleaned = normalizeIsbn(s);
  if (cleaned === null) return false;

  switch (version) {
    case 10:
      return isbn10Check(cleaned);
    case 13:
      return isbn13Check(cleaned);
    default:
      return isbn10Check(cleaned) || isbn13Check(cleaned);
  }
}

function isbn10Check(s: string): boolean {
  if (s.length !== 10) return false;

  // First 9 must be digits, last can be digit or X
  let sum = 0;
  for (let i = 0; i < 10; i++) {
    const code = s.charCodeAt(i);
    let val: number;
    if (i === 9 && (code === 88 || code === 120)) {
      val = 10;
    } else if (isAsciiDigit(code)) {
      val = code - 48;
    } else {
      return false;
    }
    sum += val * (10 - i);
  }
  return sum % 11 === 0;
}

function isbn13Check(s: string): boolean {
  if (s.length !== 13) return false;

  // Must start with 978 or 979
  if (!s.startsWith("978") && !s.startsWith("979")) return false;

  // Same as EAN-13
  let sum = 0;
  for (let i = 0; i < 13; i++) {
    const code = s.charCodeAt(i);
    if (!isAsciiDigit(code)) return false;
    const d = code - 48;
    sum += i % 2 === 0 ? d : d * 3;
  }
  return sum % 10 === 0;
}

function normalizeIsbn(value: string): string | null {
  let result = "";
  let changed = false;
  for (let i = 0; i < value.length; i++) {
    const code = value.charCodeAt(i);
    if (isAsciiDigit(code) || code === 88 || code === 120) {
      if (changed) result += value.charAt(i);
    } else if (code === 45 || isRegexWhitespace(code)) {
      if (!changed) {
        result = value.slice(0, i);
        changed = true;
      }
    } else {
      return null;
    }
  }
  return changed ? result : value;
}

// ============================================================================
// Text Analysis Predicates
// ============================================================================

/** Check if a character code point is in an RTL Unicode script range. */
function isRtlChar(cp: number): boolean {
  return (
    // Hebrew
    (cp >= 0x0590 && cp <= 0x05ff) ||
    (cp >= 0xfb1d && cp <= 0xfb4f) ||
    // Arabic
    (cp >= 0x0600 && cp <= 0x06ff) ||
    (cp >= 0x0750 && cp <= 0x077f) ||
    (cp >= 0x08a0 && cp <= 0x08ff) ||
    (cp >= 0xfb50 && cp <= 0xfdff) ||
    (cp >= 0xfe70 && cp <= 0xfeff) ||
    // Syriac
    (cp >= 0x0700 && cp <= 0x074f) ||
    // Thaana
    (cp >= 0x0780 && cp <= 0x07bf) ||
    // N'Ko
    (cp >= 0x07c0 && cp <= 0x07ff) ||
    // Samaritan
    (cp >= 0x0800 && cp <= 0x083f) ||
    // Mandaic
    (cp >= 0x0840 && cp <= 0x085f) ||
    // RTL marks
    cp === 0x200f ||
    cp === 0x202b ||
    cp === 0x202e ||
    cp === 0x2067
  );
}

function isRtl(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;

  const a = args as Record<string, unknown> | undefined;
  const threshold = typeof a?.threshold === "number" ? a.threshold : 0;

  if (threshold <= 0) {
    for (const char of s) {
      const cp = char.codePointAt(0);
      if (cp !== undefined && isRtlChar(cp)) return true;
    }
    return false;
  }

  let rtlCount = 0;
  let ltrCount = 0;
  for (const char of s) {
    const cp = char.codePointAt(0);
    if (cp === undefined) continue;
    if (isRtlChar(cp)) {
      rtlCount++;
    } else if (
      (cp >= 0x41 && cp <= 0x5a) ||
      (cp >= 0x61 && cp <= 0x7a) ||
      (cp >= 0xc0 && cp <= 0x024f)
    ) {
      ltrCount++;
    }
  }

  const total = rtlCount + ltrCount;
  if (total === 0) return false;
  return rtlCount / total >= threshold;
}

function isLtr(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;

  for (const char of s) {
    const cp = char.codePointAt(0);
    if (cp !== undefined && isRtlChar(cp)) return false;
  }
  return true;
}

function startsWith(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const prefix =
    typeof args === "object" && args !== null
      ? (args as { value?: unknown }).value
      : undefined;
  return typeof prefix === "string" && value.startsWith(prefix);
}

function endsWith(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const suffix =
    typeof args === "object" && args !== null
      ? (args as { value?: unknown }).value
      : undefined;
  return typeof suffix === "string" && value.endsWith(suffix);
}

function containsSubstring(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const needle =
    typeof args === "object" && args !== null
      ? (args as { value?: unknown }).value
      : undefined;
  return typeof needle === "string" && value.includes(needle);
}

// ============================================================================
// Color Predicates
// ============================================================================

function isHexColor(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();

  const a = args as Record<string, unknown> | undefined;
  const allowAlpha = a?.allow_alpha !== false;
  const requireHash = a?.require_hash !== false;

  const start = s.charCodeAt(0) === 35 ? 1 : 0;
  if (start === 0 && requireHash) {
    return false;
  }

  const hexLen = s.length - start;
  const validLen = allowAlpha
    ? hexLen === 3 || hexLen === 4 || hexLen === 6 || hexLen === 8
    : hexLen === 3 || hexLen === 6;

  if (!validLen) return false;

  return isAsciiHexDigitsOnlyRange(s, start, s.length);
}

function isRgbColor(value: unknown, _args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();

  let inner: string;
  let hasAlphaPrefix: boolean;

  if (s.startsWith("rgba(")) {
    inner = s.slice(5);
    hasAlphaPrefix = true;
  } else if (s.startsWith("rgb(")) {
    inner = s.slice(4);
    hasAlphaPrefix = false;
  } else {
    return false;
  }

  if (!inner.endsWith(")")) return false;
  inner = inner.slice(0, -1).trim();

  // Slash separator: "R G B / A"
  if (inner.includes("/")) {
    const [rgbPart, alphaPart] = inner.split("/");
    if (!rgbPart || !alphaPart) return false;
    const parts = rgbPart.trim().split(/\s+/);
    if (parts.length !== 3) return false;
    const allPct = parts.every((p) => p.endsWith("%"));
    const allNum = parts.every((p) => !p.endsWith("%"));
    if (!allPct && !allNum) return false;
    return (
      parts.every(validateRgbComponent) &&
      validateAlphaComponent(alphaPart.trim())
    );
  }

  // Comma-separated
  if (inner.includes(",")) {
    const parts = inner.split(",").map((p) => p.trim());
    if (parts.length === 3) {
      const allPct = parts.every((p) => p.endsWith("%"));
      const allNum = parts.every((p) => !p.endsWith("%"));
      if (!allPct && !allNum) return false;
      return parts.every(validateRgbComponent);
    }
    if (parts.length === 4) {
      const rgb = parts.slice(0, 3);
      const allPct = rgb.every((p) => p.endsWith("%"));
      const allNum = rgb.every((p) => !p.endsWith("%"));
      if (!allPct && !allNum) return false;
      return (
        rgb.every(validateRgbComponent) &&
        validateAlphaComponent(parts[3] ?? "")
      );
    }
    return false;
  }

  // Space-separated
  const parts = inner.split(/\s+/);
  if (parts.length !== 3 && !(hasAlphaPrefix && parts.length === 4))
    return false;
  const rgb = parts.slice(0, 3);
  const allPct = rgb.every((p) => p.endsWith("%"));
  const allNum = rgb.every((p) => !p.endsWith("%"));
  if (!allPct && !allNum) return false;
  const rgbValid = rgb.every(validateRgbComponent);
  if (parts.length === 4) {
    return rgbValid && validateAlphaComponent(parts[3] ?? "");
  }
  return rgbValid;
}

function validateRgbComponent(s: string): boolean {
  if (s.endsWith("%")) {
    const v = Number.parseFloat(s.slice(0, -1));
    return !Number.isNaN(v) && v >= 0 && v <= 100;
  }
  const v = Number.parseFloat(s);
  return !Number.isNaN(v) && v >= 0 && v <= 255 && v === Math.floor(v);
}

function validateAlphaComponent(s: string): boolean {
  if (s.endsWith("%")) {
    const v = Number.parseFloat(s.slice(0, -1));
    return !Number.isNaN(v) && v >= 0 && v <= 100;
  }
  const v = Number.parseFloat(s);
  return !Number.isNaN(v) && v >= 0 && v <= 1;
}

function isHslColor(value: unknown, _args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();

  let inner: string;
  let hasAlphaPrefix: boolean;

  if (s.startsWith("hsla(")) {
    inner = s.slice(5);
    hasAlphaPrefix = true;
  } else if (s.startsWith("hsl(")) {
    inner = s.slice(4);
    hasAlphaPrefix = false;
  } else {
    return false;
  }

  if (!inner.endsWith(")")) return false;
  inner = inner.slice(0, -1).trim();

  // Slash separator: "H S% L% / A"
  if (inner.includes("/")) {
    const [hslPart, alphaPart] = inner.split("/");
    if (!hslPart || !alphaPart) return false;
    const parts = hslPart.trim().split(/\s+/);
    if (parts.length !== 3) return false;
    return (
      validateHslComponents(parts) && validateAlphaComponent(alphaPart.trim())
    );
  }

  // Comma-separated
  if (inner.includes(",")) {
    const parts = inner.split(",").map((p) => p.trim());
    if (parts.length === 3) {
      return validateHslComponents(parts);
    }
    if (parts.length === 4) {
      return (
        validateHslComponents(parts) && validateAlphaComponent(parts[3] ?? "")
      );
    }
    return false;
  }

  // Space-separated
  const parts = inner.split(/\s+/);
  if (parts.length !== 3 && !(hasAlphaPrefix && parts.length === 4))
    return false;
  const hslValid = validateHslComponents(parts);
  if (parts.length === 4) {
    return hslValid && validateAlphaComponent(parts[3] ?? "");
  }
  return hslValid;
}

function validateHslComponents(parts: string[]): boolean {
  const [hue, saturation, lightness] = parts;
  if (
    hue === undefined ||
    saturation === undefined ||
    lightness === undefined
  ) {
    return false;
  }
  return (
    validateHue(hue) &&
    validatePercentageComponent(saturation) &&
    validatePercentageComponent(lightness)
  );
}

function validateHue(s: string): boolean {
  const cleaned = s.endsWith("deg") ? s.slice(0, -3) : s;
  const v = Number.parseFloat(cleaned);
  return !Number.isNaN(v) && v >= 0 && v <= 360;
}

function validatePercentageComponent(s: string): boolean {
  if (!s.endsWith("%")) return false;
  const v = Number.parseFloat(s.slice(0, -1));
  return !Number.isNaN(v) && v >= 0 && v <= 100;
}

// ============================================================================
// Numeric Type Predicates
// ============================================================================

function isInteger(value: unknown): boolean {
  if (typeof value === "number") {
    return Number.isFinite(value) && value === Math.floor(value);
  }
  if (typeof value === "string") {
    const s = value.trim();
    // Try integer parse
    if (/^-?\d+$/.test(s)) return true;
    // Try float that's whole
    const f = parseStrictFiniteNumber(s);
    return f !== null && f === Math.floor(f);
  }
  return false;
}

function isFloat(value: unknown): boolean {
  if (typeof value === "number") return Number.isFinite(value);
  if (typeof value === "string") {
    return parseStrictFiniteNumber(value) !== null;
  }
  return false;
}

function makeIntRangeCheck(
  min: bigint,
  max: bigint,
): (value: unknown) => boolean {
  return (value: unknown): boolean => {
    let bi: bigint;
    if (typeof value === "number") {
      if (!Number.isFinite(value) || value !== Math.floor(value)) return false;
      bi = BigInt(Math.trunc(value));
    } else if (typeof value === "string") {
      const s = value.trim();
      // Try direct integer parse
      try {
        bi = BigInt(s);
      } catch {
        // Try float
        const f = parseStrictFiniteNumber(s);
        if (f === null || f !== Math.floor(f)) return false;
        bi = BigInt(Math.trunc(f));
      }
    } else {
      return false;
    }
    return bi >= min && bi <= max;
  };
}

// ============================================================================
// Decimal Places Predicate
// ============================================================================

function formatDecimal2(value: unknown, _args: unknown): boolean {
  if (typeof value !== "string") return true;

  const s = value.trim();
  if (s.length === 0) return true;

  if (!s.includes(".")) {
    return /^[+-]?\d+$/.test(s);
  }

  const dotIdx = s.lastIndexOf(".");
  const decimals = s.length - dotIdx - 1;
  return decimals === 2 && Number.isFinite(Number(s));
}

function isDecimalPlaces(value: unknown, args: unknown): boolean {
  const a = args as Record<string, unknown> | undefined;
  const places = typeof a?.places === "number" ? a.places : 2;
  const isMax = a?.max === true;

  let s: string;
  if (typeof value === "string") {
    s = value.trim();
  } else if (typeof value === "number") {
    s = String(value);
  } else {
    return false;
  }

  if (s.length === 0) return false;

  // Remove leading minus
  const abs = s.replace(/^-/, "");
  if (Number.isNaN(Number.parseFloat(abs))) return false;

  const dotIdx = abs.indexOf(".");
  const actualPlaces = dotIdx >= 0 ? abs.length - dotIdx - 1 : 0;

  return isMax ? actualPlaces <= places : actualPlaces === places;
}

// ============================================================================
// Insurance Predicates
// ============================================================================

function isNpi(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const digits = collectDigitsIgnoringSpaceHyphen(value);

  if (digits === null || digits.length !== 10) return false;

  // NPI Luhn: prefix with "80840" then run standard Luhn
  const prefixed = `80840${digits}`;
  return luhnCheck(prefixed);
}

/** Raw Luhn check on a digit string (no extraction). */
function luhnCheck(digits: string): boolean {
  let sum = 0;
  let double = false;
  for (let i = digits.length - 1; i >= 0; i--) {
    let d = digits.charCodeAt(i) - 48;
    if (double) {
      d *= 2;
      if (d > 9) d -= 9;
    }
    sum += d;
    double = !double;
  }
  return sum % 10 === 0;
}

function isDeaNumber(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();

  if (s.length !== 9) return false;

  // First char: valid type code
  if (!isDeaTypeCode(toAsciiUpperCode(s.charCodeAt(0)))) return false;

  // Second char: letter
  if (!isAsciiAlpha(toAsciiUpperCode(s.charCodeAt(1)))) return false;

  // Remaining 7: digits
  const d1 = s.charCodeAt(2) - 48;
  const d2 = s.charCodeAt(3) - 48;
  const d3 = s.charCodeAt(4) - 48;
  const d4 = s.charCodeAt(5) - 48;
  const d5 = s.charCodeAt(6) - 48;
  const d6 = s.charCodeAt(7) - 48;
  const check = s.charCodeAt(8) - 48;
  if (
    d1 < 0 ||
    d1 > 9 ||
    d2 < 0 ||
    d2 > 9 ||
    d3 < 0 ||
    d3 > 9 ||
    d4 < 0 ||
    d4 > 9 ||
    d5 < 0 ||
    d5 > 9 ||
    d6 < 0 ||
    d6 > 9 ||
    check < 0 ||
    check > 9
  ) {
    return false;
  }

  // Check digit: (d1 + d3 + d5 + 2*(d2 + d4 + d6)) mod 10 = d7
  const sum = d1 + d3 + d5 + 2 * (d2 + d4 + d6);
  return sum % 10 === check;
}

function isDeaTypeCode(code: number): boolean {
  switch (code) {
    case 65:
    case 66:
    case 67:
    case 68:
    case 69:
    case 70:
    case 71:
    case 72:
    case 74:
    case 75:
    case 76:
    case 77:
    case 80:
    case 82:
    case 83:
    case 84:
    case 85:
    case 88:
      return true;
    default:
      return false;
  }
}

function isIcd10Code(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toUpperCase();
  if (!isIcd10Syntax(s)) return false;

  const strictFormat =
    typeof args === "object" && args !== null
      ? ((args as { strict_format?: boolean }).strict_format ?? false)
      : false;

  if (strictFormat) {
    const plainLen = s.replace(".", "").length;
    if (plainLen > 3 && !s.includes(".")) return false;
  }
  return true;
}

function isIcd10Syntax(s: string): boolean {
  if (s.length < 3) return false;
  if (!/^[A-Z][A-Z0-9][A-Z0-9]/.test(s)) return false;
  if (s.length === 3) return true;

  if (s[3] === ".") {
    const suffix = s.slice(4);
    return (
      suffix.length >= 1 && suffix.length <= 4 && /^[A-Z0-9]+$/.test(suffix)
    );
  }

  const suffix = s.slice(3);
  return suffix.length >= 1 && suffix.length <= 4 && /^[A-Z0-9]+$/.test(suffix);
}

function isCptCode(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toUpperCase();
  if (s.length !== 5) return false;
  if (/^\d{5}$/.test(s)) return true;

  const allowCategoryIi =
    typeof args === "object" && args !== null
      ? ((args as { allow_category_ii?: boolean }).allow_category_ii ?? true)
      : true;
  const allowCategoryIii =
    typeof args === "object" && args !== null
      ? ((args as { allow_category_iii?: boolean }).allow_category_iii ?? true)
      : true;

  if (!/^\d{4}[A-Z]$/.test(s)) return false;
  const last = s[4];
  return (
    (allowCategoryIi && last === "F") || (allowCategoryIii && last === "T")
  );
}

function isHcpcsCode(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim().toUpperCase();
  return /^[A-V]\d{4}$/.test(s);
}

function isNdcCode(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  const format =
    typeof args === "object" && args !== null
      ? ((args as { format?: string }).format ?? "ANY").toUpperCase()
      : "ANY";

  if (format === "10") return isNdc10(s);
  if (format === "11") return isNdc11(s);
  return isNdc10(s) || isNdc11(s);
}

function isNdc11(s: string): boolean {
  return extractDigits(s).length === 11;
}

function isNdc10(s: string): boolean {
  const parts = s.split("-");
  if (parts.length === 3) {
    const lens = [
      parts[0]?.length ?? 0,
      parts[1]?.length ?? 0,
      parts[2]?.length ?? 0,
    ];
    const validShape =
      (lens[0] === 4 && lens[1] === 4 && lens[2] === 2) ||
      (lens[0] === 5 && lens[1] === 3 && lens[2] === 2) ||
      (lens[0] === 5 && lens[1] === 4 && lens[2] === 1);
    return validShape && parts.every(isAsciiDigitsOnly);
  }

  return extractDigits(s).length === 10;
}

// ============================================================================
// Encoding / Crypto Predicates
// ============================================================================

function isBase58Str(s: string): boolean {
  if (s.length === 0) return false;
  for (let i = 0; i < s.length; i++) {
    if (!isBase58Code(s.charCodeAt(i))) return false;
  }
  return true;
}

function isBase58Code(code: number): boolean {
  return (
    (code >= 49 && code <= 57) ||
    (code >= 65 && code <= 72) ||
    (code >= 74 && code <= 78) ||
    (code >= 80 && code <= 90) ||
    (code >= 97 && code <= 107) ||
    (code >= 109 && code <= 122)
  );
}

function isBase58(value: unknown): boolean {
  if (typeof value !== "string") return false;
  return isBase58Str(value.trim());
}

function isBase64(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;

  const a = args as Record<string, unknown> | undefined;
  const urlSafe = a?.url_safe === true;

  // Strip padding
  const withoutPad = s.replace(/=+$/, "");
  const padCount = s.length - withoutPad.length;

  // Check valid characters
  const pattern = urlSafe ? /^[A-Za-z0-9\-_]+$/ : /^[A-Za-z0-9+/]+$/;

  if (!pattern.test(withoutPad)) return false;

  // Check padding
  if (padCount > 2) return false;
  if (padCount > 0 && s.length % 4 !== 0) return false;

  return true;
}

function isBitcoinAddress(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (s.length === 0) return false;

  // P2PKH (1...) or P2SH (3...)
  if (s[0] === "1" || s[0] === "3") {
    return s.length >= 25 && s.length <= 34 && isBase58Str(s);
  }

  // Bech32/Bech32m (bc1...)
  if (
    toAsciiLowerCode(s.charCodeAt(0)) === 98 &&
    toAsciiLowerCode(s.charCodeAt(1)) === 99 &&
    s.charCodeAt(2) === 49
  ) {
    const rest0 = toAsciiLowerCode(s.charCodeAt(3));
    // SegWit v0: bc1q..., 42 chars total
    if (rest0 === 113 && s.length === 42) {
      return isBech32Payload(s, 3);
    }
    // Taproot: bc1p..., 62 chars total
    if (rest0 === 112 && s.length === 62) {
      return isBech32Payload(s, 3);
    }
    return false;
  }

  return false;
}

function isBech32Payload(s: string, start: number): boolean {
  for (let i = start; i < s.length; i++) {
    if (!isBech32Code(toAsciiLowerCode(s.charCodeAt(i)))) return false;
  }
  return true;
}

function isBech32Code(code: number): boolean {
  return (
    code === 48 ||
    (code >= 50 && code <= 57) ||
    code === 97 ||
    (code >= 99 && code <= 104) ||
    (code >= 106 && code <= 110) ||
    (code >= 112 && code <= 122)
  );
}

function isEthereumAddress(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();

  if (!s.startsWith("0x") && !s.startsWith("0X")) return false;

  return s.length === 42 && isAsciiHexDigitsOnlyRange(s, 2, s.length);
}

function isSolanaAddress(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();

  return s.length >= 32 && s.length <= 44 && isBase58Str(s);
}

function isJwt(value: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  const parts = s.split(".");
  if (parts.length !== 3 || parts.some((p) => p.length === 0)) return false;
  return parts.every((p) => /^[A-Za-z0-9\-_]+$/.test(p));
}

function isHash(value: unknown, args: unknown): boolean {
  if (typeof value !== "string") return false;
  const s = value.trim();
  if (!isAsciiHexDigitsOnlyRange(s, 0, s.length)) return false;

  const algorithm =
    typeof args === "object" && args !== null
      ? (args as { algorithm?: string }).algorithm?.toLowerCase()
      : undefined;

  const expected = (() => {
    switch (algorithm) {
      case undefined:
        return null;
      case "md5":
        return 32;
      case "sha1":
        return 40;
      case "sha224":
        return 56;
      case "sha256":
        return 64;
      case "sha384":
        return 96;
      case "sha512":
        return 128;
      default:
        return -1;
    }
  })();

  if (expected === -1) return false;
  if (expected !== null) return s.length === expected;
  return [32, 40, 56, 64, 96, 128].includes(s.length);
}
