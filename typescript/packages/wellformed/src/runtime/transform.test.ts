import { describe, expect, it } from "vitest";
import { applyTransform, applyTransforms } from "./transform.js";

describe("applyTransform", () => {
  describe("trim", () => {
    it("trims whitespace from strings", () => {
      expect(applyTransform("  hello  ", { fn: "trim" })).toBe("hello");
      expect(applyTransform("\t\nhello\t\n", { fn: "trim" })).toBe("hello");
    });

    it("returns non-strings unchanged", () => {
      expect(applyTransform(42, { fn: "trim" })).toBe(42);
      expect(applyTransform(null, { fn: "trim" })).toBe(null);
    });
  });

  describe("collapse_whitespace", () => {
    it("collapses multiple whitespace to single space", () => {
      expect(
        applyTransform("hello   world", { fn: "collapse_whitespace" }),
      ).toBe("hello world");
      expect(
        applyTransform("  hello  \n  world  ", { fn: "collapse_whitespace" }),
      ).toBe("hello world");
    });
  });

  describe("digits_only", () => {
    it("removes non-digit characters", () => {
      expect(applyTransform("123-456-7890", { fn: "digits_only" })).toBe(
        "1234567890",
      );
      expect(applyTransform("(555) 123-4567", { fn: "digits_only" })).toBe(
        "5551234567",
      );
      expect(applyTransform("abc123def", { fn: "digits_only" })).toBe("123");
    });
  });

  describe("upper", () => {
    it("converts to uppercase", () => {
      expect(applyTransform("hello", { fn: "upper" })).toBe("HELLO");
      expect(applyTransform("Hello World", { fn: "upper" })).toBe(
        "HELLO WORLD",
      );
    });
  });

  describe("lower", () => {
    it("converts to lowercase", () => {
      expect(applyTransform("HELLO", { fn: "lower" })).toBe("hello");
      expect(applyTransform("Hello World", { fn: "lower" })).toBe(
        "hello world",
      );
    });
  });

  describe("money_to_cents", () => {
    it("converts string dollar amounts to cents", () => {
      expect(applyTransform("$1.00", { fn: "money_to_cents" })).toBe(100);
      expect(applyTransform("$123.45", { fn: "money_to_cents" })).toBe(12345);
      expect(applyTransform("1,234.56", { fn: "money_to_cents" })).toBe(123456);
    });

    it("converts number values to cents", () => {
      expect(applyTransform(1.0, { fn: "money_to_cents" })).toBe(100);
      expect(applyTransform(123.45, { fn: "money_to_cents" })).toBe(12345);
    });

    it("uses custom scale", () => {
      expect(applyTransform("1.00", { fn: "money_to_cents", scale: 3 })).toBe(
        1000,
      );
      expect(applyTransform(1.0, { fn: "money_to_cents", scale: 3 })).toBe(
        1000,
      );
    });

    it("returns invalid strings as-is", () => {
      expect(applyTransform("not a number", { fn: "money_to_cents" })).toBe(
        "not a number",
      );
      expect(applyTransform("12-3", { fn: "money_to_cents" })).toBe("12-3");
    });
  });

  describe("date_parse", () => {
    it("parses strftime-style MM/DD/YYYY format", () => {
      expect(
        applyTransform("12/31/2024", {
          fn: "date_parse",
          format: "%m/%d/%Y",
        }),
      ).toBe("2024-12-31");
    });

    it("parses MM/DD/YYYY format", () => {
      expect(
        applyTransform("12/31/2024", {
          fn: "date_parse",
          format: "MM/DD/YYYY",
        }),
      ).toBe("2024-12-31");
      expect(
        applyTransform("1/5/2024", { fn: "date_parse", format: "MM/DD/YYYY" }),
      ).toBe("2024-01-05");
    });

    it("parses MM-DD-YYYY format", () => {
      expect(
        applyTransform("12-31-2024", {
          fn: "date_parse",
          format: "MM-DD-YYYY",
        }),
      ).toBe("2024-12-31");
    });

    it("parses YYYY-MM-DD format", () => {
      expect(
        applyTransform("2024-12-31", {
          fn: "date_parse",
          format: "YYYY-MM-DD",
        }),
      ).toBe("2024-12-31");
      expect(
        applyTransform("2024-1-5", {
          fn: "date_parse",
          format: "YYYY-MM-DD",
        }),
      ).toBe("2024-01-05");
    });

    it("parses MMDDYYYY format", () => {
      expect(
        applyTransform("12312024", { fn: "date_parse", format: "MMDDYYYY" }),
      ).toBe("2024-12-31");
    });

    it("returns invalid dates as-is", () => {
      expect(
        applyTransform("not a date", {
          fn: "date_parse",
          format: "MM/DD/YYYY",
        }),
      ).toBe("not a date");
      expect(
        applyTransform("13/31/2024", {
          fn: "date_parse",
          format: "MM/DD/YYYY",
        }),
      ).toBe("13/31/2024");
      expect(
        applyTransform("02/31/2024", {
          fn: "date_parse",
          format: "MM/DD/YYYY",
        }),
      ).toBe("02/31/2024");
    });
  });

  describe("replace", () => {
    it("replaces all literal occurrences of pattern", () => {
      expect(
        applyTransform("hello world world", {
          fn: "replace",
          pattern: "world",
          replacement: "there",
        }),
      ).toBe("hello there there");
    });

    it("treats pattern as a literal string, not a regex", () => {
      expect(
        applyTransform("hello123world", {
          fn: "replace",
          pattern: "\\d+",
          replacement: "-",
        }),
      ).toBe("hello123world");

      expect(
        applyTransform("a.a.a", {
          fn: "replace",
          pattern: ".",
          replacement: "X",
        }),
      ).toBe("aXaXa");
    });
  });

  describe("default", () => {
    it("returns default value for null/undefined", () => {
      expect(applyTransform(null, { fn: "default", value: "default" })).toBe(
        "default",
      );
      expect(
        applyTransform(undefined, { fn: "default", value: "default" }),
      ).toBe("default");
    });

    it("returns original value if not null/undefined", () => {
      expect(applyTransform("hello", { fn: "default", value: "default" })).toBe(
        "hello",
      );
      expect(applyTransform(42, { fn: "default", value: 0 })).toBe(42);
    });
  });

  describe("normalize_flight_number", () => {
    it("uppercases and removes spaces/hyphens", () => {
      expect(applyTransform("ua 123", { fn: "normalize_flight_number" })).toBe(
        "UA123",
      );
      expect(
        applyTransform("ual-1234a", { fn: "normalize_flight_number" }),
      ).toBe("UAL1234A");
    });

    it("returns non-strings unchanged", () => {
      expect(applyTransform(42, { fn: "normalize_flight_number" })).toBe(42);
      expect(applyTransform(null, { fn: "normalize_flight_number" })).toBe(
        null,
      );
    });
  });

  describe("normalize_icd10", () => {
    it("normalizes to canonical dotted uppercase format", () => {
      expect(applyTransform("s72.001a", { fn: "normalize_icd10" })).toBe(
        "S72.001A",
      );
      expect(applyTransform("u071", { fn: "normalize_icd10" })).toBe("U07.1");
    });

    it("returns value as-is when invalid", () => {
      expect(applyTransform("bad$", { fn: "normalize_icd10" })).toBe("BAD");
    });
  });

  describe("normalize_cpt", () => {
    it("normalizes cpt forms", () => {
      expect(applyTransform(" 99213 ", { fn: "normalize_cpt" })).toBe("99213");
      expect(applyTransform("1234f", { fn: "normalize_cpt" })).toBe("1234F");
    });

    it("returns value as-is when invalid", () => {
      expect(applyTransform("12A3F", { fn: "normalize_cpt" })).toBe("12A3F");
    });
  });

  describe("normalize_hcpcs", () => {
    it("normalizes hcpcs forms", () => {
      expect(applyTransform(" a0428 ", { fn: "normalize_hcpcs" })).toBe(
        "A0428",
      );
    });

    it("returns value as-is when invalid", () => {
      expect(applyTransform("W1234", { fn: "normalize_hcpcs" })).toBe("W1234");
    });
  });

  describe("normalize_ndc11", () => {
    it("normalizes hyphenated and unhyphenated ndc to 11 when possible", () => {
      expect(applyTransform("12345-6789-01", { fn: "normalize_ndc11" })).toBe(
        "12345678901",
      );
      expect(applyTransform("1234-5678-90", { fn: "normalize_ndc11" })).toBe(
        "01234567890",
      );
    });

    it("returns value as-is when not normalizable", () => {
      expect(applyTransform("1234567890", { fn: "normalize_ndc11" })).toBe(
        "1234567890",
      );
    });
  });

  describe("phone_us", () => {
    it("formats 10-digit phone number", () => {
      expect(applyTransform("6501234567", { fn: "phone_us" })).toBe(
        "(650) 123-4567",
      );
    });

    it("strips country code 1 from 11-digit phone number", () => {
      expect(applyTransform("16501234567", { fn: "phone_us" })).toBe(
        "(650) 123-4567",
      );
    });

    it("is idempotent on already formatted phone", () => {
      expect(applyTransform("(650) 123-4567", { fn: "phone_us" })).toBe(
        "(650) 123-4567",
      );
    });

    it("returns value as-is if too few digits", () => {
      expect(applyTransform("123", { fn: "phone_us" })).toBe("123");
    });

    it("returns value as-is if wrong digit count", () => {
      expect(applyTransform("12345678901234", { fn: "phone_us" })).toBe(
        "12345678901234",
      );
    });

    it("handles phone with formatting characters", () => {
      expect(applyTransform("(650) 123-4567", { fn: "phone_us" })).toBe(
        "(650) 123-4567",
      );
      expect(applyTransform("650-123-4567", { fn: "phone_us" })).toBe(
        "(650) 123-4567",
      );
      expect(applyTransform("650.123.4567", { fn: "phone_us" })).toBe(
        "(650) 123-4567",
      );
    });
  });

  describe("phone_e164", () => {
    it("normalizes 10-digit US number to +1", () => {
      expect(applyTransform("6501234567", { fn: "phone_e164" })).toBe(
        "+16501234567",
      );
    });

    it("normalizes 11-digit US number with leading country code", () => {
      expect(applyTransform("1 (650) 123-4567", { fn: "phone_e164" })).toBe(
        "+16501234567",
      );
    });

    it("normalizes explicit international number", () => {
      expect(applyTransform("+44 20 7946 0958", { fn: "phone_e164" })).toBe(
        "+442079460958",
      );
    });

    it("returns value as-is when it cannot normalize", () => {
      expect(applyTransform("1234", { fn: "phone_e164" })).toBe("1234");
    });
  });

  describe("card_mask_last4", () => {
    it("masks all but last 4 digits", () => {
      expect(
        applyTransform("4111111111111111", { fn: "card_mask_last4" }),
      ).toBe("************1111");
    });

    it("handles card number with spaces", () => {
      expect(
        applyTransform("4111 1111 1111 1111", { fn: "card_mask_last4" }),
      ).toBe("************1111");
    });

    it("returns digits as-is if 4 or fewer", () => {
      expect(applyTransform("1234", { fn: "card_mask_last4" })).toBe("1234");
      expect(applyTransform("12", { fn: "card_mask_last4" })).toBe("12");
    });

    it("returns non-strings unchanged", () => {
      expect(applyTransform(42, { fn: "card_mask_last4" })).toBe(42);
      expect(applyTransform(null, { fn: "card_mask_last4" })).toBe(null);
    });
  });
  describe("format_ssn", () => {
    it("formats 9 digits as SSN", () => {
      expect(applyTransform("123456789", { fn: "format_ssn" })).toBe(
        "123-45-6789",
      );
    });

    it("extracts digits and formats", () => {
      expect(applyTransform("123-45-6789", { fn: "format_ssn" })).toBe(
        "123-45-6789",
      );
    });

    it("returns value as-is if not 9 digits", () => {
      expect(applyTransform("12345", { fn: "format_ssn" })).toBe("12345");
    });
  });

  describe("format_ein", () => {
    it("formats 9 digits as EIN", () => {
      expect(applyTransform("123456789", { fn: "format_ein" })).toBe(
        "12-3456789",
      );
    });

    it("extracts digits and formats", () => {
      expect(applyTransform("12-3456789", { fn: "format_ein" })).toBe(
        "12-3456789",
      );
    });

    it("returns value as-is if not 9 digits", () => {
      expect(applyTransform("12345", { fn: "format_ein" })).toBe("12345");
    });
  });

  describe("mask_ssn", () => {
    it("masks SSN showing only last 4", () => {
      expect(applyTransform("123-45-6789", { fn: "mask_ssn" })).toBe(
        "***-**-6789",
      );
      expect(applyTransform("123456789", { fn: "mask_ssn" })).toBe(
        "***-**-6789",
      );
    });

    it("returns value as-is if not 9 digits", () => {
      expect(applyTransform("12345", { fn: "mask_ssn" })).toBe("12345");
    });
  });

  describe("mask_ein", () => {
    it("masks EIN showing only last 4", () => {
      expect(applyTransform("12-3456789", { fn: "mask_ein" })).toBe(
        "**-***6789",
      );
      expect(applyTransform("123456789", { fn: "mask_ein" })).toBe(
        "**-***6789",
      );
    });

    it("returns value as-is if not 9 digits", () => {
      expect(applyTransform("12345", { fn: "mask_ein" })).toBe("12345");
    });
  });

  describe("format_iban", () => {
    it("formats IBAN with spaces every 4 chars", () => {
      expect(
        applyTransform("GB29NWBK60161331926819", { fn: "format_iban" }),
      ).toBe("GB29 NWBK 6016 1331 9268 19");
    });

    it("strips existing spaces and re-formats", () => {
      expect(
        applyTransform("GB29 NWBK 6016 1331 9268 19", { fn: "format_iban" }),
      ).toBe("GB29 NWBK 6016 1331 9268 19");
    });

    it("uppercases lowercase input", () => {
      expect(
        applyTransform("gb29nwbk60161331926819", { fn: "format_iban" }),
      ).toBe("GB29 NWBK 6016 1331 9268 19");
    });

    it("returns value as-is if too short", () => {
      expect(applyTransform("GB29", { fn: "format_iban" })).toBe("GB29");
    });
  });

  describe("format_credit_card", () => {
    it("formats card number with spaces every 4 digits", () => {
      expect(
        applyTransform("4111111111111111", { fn: "format_credit_card" }),
      ).toBe("4111 1111 1111 1111");
    });

    it("strips existing formatting and re-formats", () => {
      expect(
        applyTransform("4111-1111-1111-1111", { fn: "format_credit_card" }),
      ).toBe("4111 1111 1111 1111");
    });

    it("handles Amex (15 digits)", () => {
      expect(
        applyTransform("371449635398431", { fn: "format_credit_card" }),
      ).toBe("3714 4963 5398 431");
    });

    it("returns value as-is if too short", () => {
      expect(applyTransform("1234", { fn: "format_credit_card" })).toBe("1234");
    });
  });

  describe("format_thousands", () => {
    it("adds comma separators to integers", () => {
      expect(applyTransform("1234567", { fn: "format_thousands" })).toBe(
        "1,234,567",
      );
    });

    it("adds comma separators to decimals", () => {
      expect(applyTransform("1234567.89", { fn: "format_thousands" })).toBe(
        "1,234,567.89",
      );
    });

    it("leaves small numbers unchanged", () => {
      expect(applyTransform("999", { fn: "format_thousands" })).toBe("999");
    });

    it("handles negative numbers", () => {
      expect(applyTransform("-1234567.89", { fn: "format_thousands" })).toBe(
        "-1,234,567.89",
      );
    });

    it("supports custom separator", () => {
      expect(
        applyTransform("1234567", { fn: "format_thousands", separator: "." }),
      ).toBe("1.234.567");
    });

    it("handles number type", () => {
      expect(applyTransform(1234567.89, { fn: "format_thousands" })).toBe(
        "1,234,567.89",
      );
    });
  });

  describe("format_decimal", () => {
    it("pads to fixed decimal places", () => {
      expect(applyTransform("3.1", { fn: "format_decimal", places: 2 })).toBe(
        "3.10",
      );
      expect(applyTransform("3", { fn: "format_decimal", places: 2 })).toBe(
        "3.00",
      );
    });

    it("truncates/rounds to fixed decimal places", () => {
      expect(
        applyTransform("3.14159", { fn: "format_decimal", places: 2 }),
      ).toBe("3.14");
      expect(applyTransform("3.145", { fn: "format_decimal", places: 2 })).toBe(
        "3.15",
      );
    });

    it("handles negative numbers", () => {
      expect(applyTransform("-1.5", { fn: "format_decimal", places: 3 })).toBe(
        "-1.500",
      );
    });

    it("leaves malformed cleaned numbers unchanged", () => {
      expect(applyTransform("12-3", { fn: "format_decimal", places: 2 })).toBe(
        "12-3",
      );
    });

    it("handles number type", () => {
      expect(applyTransform(3.1, { fn: "format_decimal", places: 2 })).toBe(
        "3.10",
      );
    });
  });
});

describe("applyTransforms", () => {
  it("applies transforms in sequence", () => {
    const result = applyTransforms("  $1,234.56  ", [
      { fn: "trim" },
      { fn: "money_to_cents" },
    ]);
    expect(result).toBe(123456);
  });

  it("returns value unchanged if no transforms", () => {
    expect(applyTransforms("hello", [])).toBe("hello");
  });
});
