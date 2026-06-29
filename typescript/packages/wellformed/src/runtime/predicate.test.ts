import { describe, expect, it } from "vitest";
import type { Predicate } from "../ir/types.js";
import { createEvalContext, evaluate, PredicateRegistry } from "./predicate.js";

describe("evaluate", () => {
  const ctx = createEvalContext();

  describe("constant predicates", () => {
    it("true always returns true", () => {
      expect(evaluate({ type: "true" }, "anything", ctx)).toBe(true);
    });

    it("false always returns false", () => {
      expect(evaluate({ type: "false" }, "anything", ctx)).toBe(false);
    });
  });

  describe("regex", () => {
    it("matches regex patterns", () => {
      expect(evaluate({ type: "regex", pattern: "^\\d+$" }, "123", ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "regex", pattern: "^\\d+$" }, "abc", ctx)).toBe(
        false,
      );
    });

    it("supports regex flags", () => {
      expect(
        evaluate({ type: "regex", pattern: "hello", flags: "i" }, "HELLO", ctx),
      ).toBe(true);
    });

    it("resets cached regex state for global flags", () => {
      const pred = { type: "regex", pattern: "^a$", flags: "g" } as const;

      expect(evaluate(pred, "a", ctx)).toBe(true);
      expect(evaluate(pred, "a", ctx)).toBe(true);
    });

    it("passes non-strings as not applicable", () => {
      expect(evaluate({ type: "regex", pattern: ".*" }, 123, ctx)).toBe(true);
    });
  });

  describe("format:decimal-2", () => {
    const pred: Predicate = { type: "call", name: "format:decimal-2" };

    it("matches Rust decimal-2 format semantics", () => {
      expect(evaluate(pred, "", ctx)).toBe(true);
      expect(evaluate(pred, "123", ctx)).toBe(true);
      expect(evaluate(pred, "123.45", ctx)).toBe(true);
      expect(evaluate(pred, "123.4", ctx)).toBe(false);
      expect(evaluate(pred, "123.456", ctx)).toBe(false);
      expect(evaluate(pred, "123.45x", ctx)).toBe(false);
      expect(evaluate(pred, 123, ctx)).toBe(true);
    });
  });

  describe("reference named predicates", () => {
    it("matches Rust default registry reference names", () => {
      expect(
        evaluate({ type: "call", name: "is_country_name" }, "canada", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_state_name" }, "Puerto Rico", ctx),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "is_state_name",
            args: { include_territories: false },
          },
          "Puerto Rico",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_filing_status" }, "mfj", ctx),
      ).toBe(true);
    });
  });

  describe("template_literal", () => {
    const pred: Predicate = {
      type: "template_literal",
      parts: [
        { kind: "literal", value: "SFO-" },
        { kind: "digits", min: 3, max: 4 },
        { kind: "literal", value: "-" },
        { kind: "uppercase", min: 2, max: 2 },
      ],
    };

    it("matches ordered literal and class segments", () => {
      expect(evaluate(pred, "SFO-123-AB", ctx)).toBe(true);
      expect(evaluate(pred, "SFO-1234-ZZ", ctx)).toBe(true);
      expect(evaluate(pred, "SFO-12-AB", ctx)).toBe(false);
      expect(evaluate(pred, "SFO-123-ab", ctx)).toBe(false);
    });

    it("supports bounded variable segment before a literal", () => {
      const variablePrefix: Predicate = {
        type: "template_literal",
        parts: [
          { kind: "ascii_letters", min: 1, max: 3 },
          { kind: "literal", value: "-" },
          { kind: "digits", min: 2, max: 2 },
        ],
      };
      expect(evaluate(variablePrefix, "AB-12", ctx)).toBe(true);
      expect(evaluate(variablePrefix, "A-12", ctx)).toBe(true);
      expect(evaluate(variablePrefix, "ABCD-12", ctx)).toBe(false);
    });

    it("passes non-strings as not applicable", () => {
      expect(evaluate(pred, 123, ctx)).toBe(true);
    });
  });

  describe("min_len", () => {
    it("checks minimum length for strings", () => {
      expect(evaluate({ type: "min_len", len: 3 }, "abc", ctx)).toBe(true);
      expect(evaluate({ type: "min_len", len: 3 }, "ab", ctx)).toBe(false);
    });

    it("counts Unicode code points for string length like Rust chars", () => {
      expect(evaluate({ type: "min_len", len: 1 }, "😀", ctx)).toBe(true);
      expect(evaluate({ type: "max_len", len: 1 }, "😀", ctx)).toBe(true);
    });

    it("checks minimum length for arrays", () => {
      expect(evaluate({ type: "min_len", len: 2 }, [1, 2, 3], ctx)).toBe(true);
      expect(evaluate({ type: "min_len", len: 2 }, [1], ctx)).toBe(false);
    });
  });

  describe("max_len", () => {
    it("checks maximum length for strings", () => {
      expect(evaluate({ type: "max_len", len: 3 }, "abc", ctx)).toBe(true);
      expect(evaluate({ type: "max_len", len: 3 }, "abcd", ctx)).toBe(false);
    });

    it("checks maximum length for arrays", () => {
      expect(evaluate({ type: "max_len", len: 2 }, [1, 2], ctx)).toBe(true);
      expect(evaluate({ type: "max_len", len: 2 }, [1, 2, 3], ctx)).toBe(false);
    });
  });

  describe("range", () => {
    it("checks numeric range", () => {
      expect(evaluate({ type: "range", min: 0, max: 100 }, 50, ctx)).toBe(true);
      expect(evaluate({ type: "range", min: 0, max: 100 }, 0, ctx)).toBe(true);
      expect(evaluate({ type: "range", min: 0, max: 100 }, 100, ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "range", min: 0, max: 100 }, -1, ctx)).toBe(
        false,
      );
      expect(evaluate({ type: "range", min: 0, max: 100 }, 101, ctx)).toBe(
        false,
      );
    });

    it("supports open ranges", () => {
      expect(evaluate({ type: "range", min: 0 }, 1000, ctx)).toBe(true);
      expect(evaluate({ type: "range", max: 100 }, -1000, ctx)).toBe(true);
    });

    it("returns false for non-numbers", () => {
      expect(evaluate({ type: "range", min: 0 }, "50", ctx)).toBe(false);
    });
  });

  describe("exists", () => {
    it("checks path existence", () => {
      expect(evaluate({ type: "exists", path: "/foo" }, { foo: 42 }, ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "exists", path: "/foo" }, { bar: 42 }, ctx)).toBe(
        false,
      );
    });

    it("returns false for null/undefined values", () => {
      expect(
        evaluate({ type: "exists", path: "/foo" }, { foo: null }, ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "exists", path: "/foo" }, { foo: undefined }, ctx),
      ).toBe(false);
    });
  });

  describe("eq", () => {
    it("checks path equals value", () => {
      expect(
        evaluate({ type: "eq", path: "/foo", value: 42 }, { foo: 42 }, ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "eq", path: "/foo", value: 42 }, { foo: 43 }, ctx),
      ).toBe(false);
    });

    it("handles nested paths", () => {
      expect(
        evaluate(
          { type: "eq", path: "/a/b", value: "x" },
          { a: { b: "x" } },
          ctx,
        ),
      ).toBe(true);
    });
  });

  describe("in", () => {
    it("checks path value in list", () => {
      expect(
        evaluate(
          { type: "in", path: "/status", values: ["active", "pending"] },
          { status: "active" },
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "in", path: "/status", values: ["active", "pending"] },
          { status: "inactive" },
          ctx,
        ),
      ).toBe(false);
    });
  });

  describe("presence predicates", () => {
    it("required_with enforces a dependent field", () => {
      expect(
        evaluate(
          {
            type: "required_with",
            field: "/confirmPassword",
            with: "/password",
          },
          { password: "secret" },
          ctx,
        ),
      ).toBe(false);

      expect(
        evaluate(
          {
            type: "required_with",
            field: "/confirmPassword",
            with: "/password",
          },
          { password: "secret", confirmPassword: "secret" },
          ctx,
        ),
      ).toBe(true);

      expect(
        evaluate(
          {
            type: "required_with",
            field: "/confirmPassword",
            with: "/password",
          },
          {},
          ctx,
        ),
      ).toBe(true);
    });

    it("required_without enforces a fallback field", () => {
      const pred: Predicate = {
        type: "required_without",
        field: "/taxId",
        without: "/ssn",
      };

      expect(evaluate(pred, {}, ctx)).toBe(false);
      expect(evaluate(pred, { taxId: "12-3456789" }, ctx)).toBe(true);
      expect(evaluate(pred, { ssn: "123-45-6789" }, ctx)).toBe(true);
    });

    it("exactly_one_of requires only one field", () => {
      const pred: Predicate = {
        type: "exactly_one_of",
        paths: ["/ssn", "/ein"],
      };

      expect(evaluate(pred, { ssn: "123-45-6789" }, ctx)).toBe(true);
      expect(evaluate(pred, { ein: "12-3456789" }, ctx)).toBe(true);
      expect(
        evaluate(pred, { ssn: "123-45-6789", ein: "12-3456789" }, ctx),
      ).toBe(false);
      expect(evaluate(pred, {}, ctx)).toBe(false);
    });
  });

  describe("boolean combinators", () => {
    it("and requires all predicates true", () => {
      const pred: Predicate = {
        type: "and",
        predicates: [
          { type: "min_len", len: 1 },
          { type: "max_len", len: 10 },
        ],
      };
      expect(evaluate(pred, "hello", ctx)).toBe(true);
      expect(evaluate(pred, "", ctx)).toBe(false);
      expect(evaluate(pred, "hello world!", ctx)).toBe(false);
    });

    it("or requires at least one predicate true", () => {
      const pred: Predicate = {
        type: "or",
        predicates: [
          { type: "eq", path: "/type", value: "A" },
          { type: "eq", path: "/type", value: "B" },
        ],
      };
      expect(evaluate(pred, { type: "A" }, ctx)).toBe(true);
      expect(evaluate(pred, { type: "B" }, ctx)).toBe(true);
      expect(evaluate(pred, { type: "C" }, ctx)).toBe(false);
    });

    it("not negates predicate", () => {
      const pred: Predicate = {
        type: "not",
        predicate: { type: "regex", pattern: "^test" },
      };
      expect(evaluate(pred, "hello", ctx)).toBe(true);
      expect(evaluate(pred, "test123", ctx)).toBe(false);
    });

    it("implies (if-then) logic", () => {
      // If type is "required", then value must exist
      const pred: Predicate = {
        type: "implies",
        if: { type: "eq", path: "/type", value: "required" },
        // biome-ignore lint/suspicious/noThenProperty: `then` is a legitimate property in our Predicate IR
        then: { type: "exists", path: "/value" },
      };
      expect(evaluate(pred, { type: "required", value: 42 }, ctx)).toBe(true);
      expect(evaluate(pred, { type: "required" }, ctx)).toBe(false);
      expect(evaluate(pred, { type: "optional" }, ctx)).toBe(true); // antecedent false
    });
  });

  describe("numeric cross-field predicates", () => {
    it("requires wildcard eq_fields result sets to have equal length and values", () => {
      const pred: Predicate = {
        type: "eq_fields",
        left: "/left/*/id",
        right: "/right/*/id",
      };

      expect(
        evaluate(
          pred,
          { left: [{ id: 1 }, { id: 2 }], right: [{ id: 1 }, { id: 2 }] },
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          pred,
          { left: [{ id: 1 }, { id: 2 }], right: [{ id: 1 }] },
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate(
          pred,
          { left: [{ id: 1 }, { id: 2 }], right: [{ id: 1 }, { id: 3 }] },
          ctx,
        ),
      ).toBe(false);
    });

    it("does not coerce strings for numeric comparisons or sums", () => {
      expect(
        evaluate(
          { type: "gt_field", left: "/a", right: "/b" },
          { a: "20", b: "10" },
          ctx,
        ),
      ).toBe(false);

      expect(
        evaluate(
          { type: "sum_equals", paths: ["/a", "/b"], target: "/total" },
          { a: "1", b: "2", total: "3" },
          ctx,
        ),
      ).toBe(false);
    });
  });

  describe("call (named predicates)", () => {
    it("throws for unknown predicates", () => {
      expect(() =>
        evaluate({ type: "call", name: "unknown_pred" }, "x", ctx),
      ).toThrow(/Unknown predicate/);
    });
  });
});

describe("PredicateRegistry", () => {
  it("registers and retrieves predicates", () => {
    const registry = new PredicateRegistry();
    registry.register("is_foo", (value) => value === "foo");
    const isFoo = registry.get("is_foo");
    expect(isFoo).toBeDefined();
    expect(isFoo?.("foo", undefined)).toBe(true);
    expect(isFoo?.("bar", undefined)).toBe(false);
  });

  it("withBuiltins includes all built-in predicates", () => {
    const registry = PredicateRegistry.withBuiltins();
    expect(registry.get("is_tin")).toBeDefined();
    expect(registry.get("is_ssn")).toBeDefined();
    expect(registry.get("is_ein")).toBeDefined();
    expect(registry.get("is_email")).toBeDefined();
    expect(registry.get("is_phone")).toBeDefined();
    expect(registry.get("phone_number")).toBeDefined();
    expect(registry.get("phone_number_us")).toBeDefined();
    expect(registry.get("is_icd10_code")).toBeDefined();
    expect(registry.get("is_jwt")).toBeDefined();
    expect(registry.get("is_ip")).toBeDefined();
    expect(registry.get("is_country_name")).toBeDefined();
    expect(registry.get("is_state_name")).toBeDefined();
    expect(registry.get("is_filing_status")).toBeDefined();
  });
});

describe("built-in predicates", () => {
  const ctx = createEvalContext();

  describe("TIN predicates", () => {
    it("is_ssn validates SSN format", () => {
      expect(
        evaluate({ type: "call", name: "is_ssn" }, "123-45-6789", ctx),
      ).toBe(true);
      expect(evaluate({ type: "call", name: "is_ssn" }, "123456789", ctx)).toBe(
        true,
      );
      expect(
        evaluate({ type: "call", name: "is_ssn" }, "000-00-0000", ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_ssn" }, "666-12-3456", ctx),
      ).toBe(false);
    });

    it("is_ein validates EIN format", () => {
      expect(
        evaluate({ type: "call", name: "is_ein" }, "12-3456789", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_ein" }, "00-0000000", ctx),
      ).toBe(false);
    });

    it("luhn validates Luhn checksum", () => {
      expect(evaluate({ type: "call", name: "luhn" }, "79927398713", ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "luhn" }, "79927398710", ctx)).toBe(
        false,
      );
    });
  });

  describe("financial predicates", () => {
    it("is_cusip validates CUSIP", () => {
      expect(
        evaluate({ type: "call", name: "is_cusip" }, "037833100", ctx),
      ).toBe(true);
    });

    it("is_aba_routing validates ABA routing numbers", () => {
      expect(
        evaluate({ type: "call", name: "is_aba_routing" }, "021000021", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_aba_routing" }, "123456789", ctx),
      ).toBe(false);
    });

    it("is_mcc validates MCC codes", () => {
      expect(evaluate({ type: "call", name: "is_mcc" }, "5411", ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_mcc" }, "541", ctx)).toBe(
        false,
      );
    });
  });

  describe("date and time predicates", () => {
    it("is_time validates time formats", () => {
      // 24-hour
      expect(evaluate({ type: "call", name: "is_time" }, "14:30", ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_time" }, "00:00", ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_time" }, "23:59:59", ctx)).toBe(
        true,
      );

      // 12-hour
      expect(evaluate({ type: "call", name: "is_time" }, "2:30 PM", ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_time" }, "12:00 AM", ctx)).toBe(
        true,
      );

      // Format filter
      expect(
        evaluate(
          { type: "call", name: "is_time", args: { format: "24h" } },
          "14:30",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_time", args: { format: "24h" } },
          "2:30 PM",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_time", args: { format: "12h" } },
          "2:30 PM",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_time", args: { format: "12h" } },
          "14:30",
          ctx,
        ),
      ).toBe(false);

      // Invalid
      expect(evaluate({ type: "call", name: "is_time" }, "24:00", ctx)).toBe(
        false,
      );
      expect(evaluate({ type: "call", name: "is_time" }, "12:60", ctx)).toBe(
        false,
      );
      expect(evaluate({ type: "call", name: "is_time" }, "13:00 PM", ctx)).toBe(
        false,
      );
    });

    it("is_iso_datetime validates ISO 8601 datetimes", () => {
      expect(
        evaluate(
          { type: "call", name: "is_iso_datetime" },
          "2024-01-15T10:30:00Z",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_iso_datetime" },
          "2024-01-15T10:30:00+05:30",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_iso_datetime" },
          "2024-01-15T10:30:00.123Z",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_iso_datetime" },
          "2024-01-15T10:30:00",
          ctx,
        ),
      ).toBe(true);

      // Invalid
      expect(
        evaluate(
          { type: "call", name: "is_iso_datetime" },
          "2024-01-15 10:30:00",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_iso_datetime" },
          "2024-13-15T10:30:00Z",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_iso_datetime" },
          "not-a-datetime",
          ctx,
        ),
      ).toBe(false);
    });

    it("time_before validates time is before a given time", () => {
      // Before
      expect(
        evaluate(
          { type: "call", name: "time_before", args: { time: "12:00" } },
          "09:00",
          ctx,
        ),
      ).toBe(true);

      // Equal (allowed by default)
      expect(
        evaluate(
          { type: "call", name: "time_before", args: { time: "12:00" } },
          "12:00",
          ctx,
        ),
      ).toBe(true);

      // Equal (not allowed)
      expect(
        evaluate(
          {
            type: "call",
            name: "time_before",
            args: { time: "12:00", allow_equal: false },
          },
          "12:00",
          ctx,
        ),
      ).toBe(false);

      // After (fail)
      expect(
        evaluate(
          { type: "call", name: "time_before", args: { time: "12:00" } },
          "17:00",
          ctx,
        ),
      ).toBe(false);

      // 12h format
      expect(
        evaluate(
          { type: "call", name: "time_before", args: { time: "12:00" } },
          "9:00 AM",
          ctx,
        ),
      ).toBe(true);

      // With seconds
      expect(
        evaluate(
          { type: "call", name: "time_before", args: { time: "12:00:01" } },
          "12:00:00",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "time_before", args: { time: "12:00:00" } },
          "12:00:01",
          ctx,
        ),
      ).toBe(false);
    });

    it("time_after validates time is after a given time", () => {
      // After
      expect(
        evaluate(
          { type: "call", name: "time_after", args: { time: "12:00" } },
          "17:00",
          ctx,
        ),
      ).toBe(true);

      // Equal (allowed by default)
      expect(
        evaluate(
          { type: "call", name: "time_after", args: { time: "12:00" } },
          "12:00",
          ctx,
        ),
      ).toBe(true);

      // Equal (not allowed)
      expect(
        evaluate(
          {
            type: "call",
            name: "time_after",
            args: { time: "12:00", allow_equal: false },
          },
          "12:00",
          ctx,
        ),
      ).toBe(false);

      // Before (fail)
      expect(
        evaluate(
          { type: "call", name: "time_after", args: { time: "12:00" } },
          "09:00",
          ctx,
        ),
      ).toBe(false);

      // 12h format
      expect(
        evaluate(
          { type: "call", name: "time_after", args: { time: "12:00" } },
          "1:00 PM",
          ctx,
        ),
      ).toBe(true);
    });

    it("time_in_range validates time within a range", () => {
      // Normal range: business hours
      expect(
        evaluate(
          {
            type: "call",
            name: "time_in_range",
            args: { min: "09:00", max: "17:00" },
          },
          "12:00",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "time_in_range",
            args: { min: "09:00", max: "17:00" },
          },
          "09:00",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "time_in_range",
            args: { min: "09:00", max: "17:00" },
          },
          "17:00",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "time_in_range",
            args: { min: "09:00", max: "17:00" },
          },
          "08:59",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate(
          {
            type: "call",
            name: "time_in_range",
            args: { min: "09:00", max: "17:00" },
          },
          "17:01",
          ctx,
        ),
      ).toBe(false);

      // Overnight range: night shift
      expect(
        evaluate(
          {
            type: "call",
            name: "time_in_range",
            args: { min: "22:00", max: "06:00" },
          },
          "23:00",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "time_in_range",
            args: { min: "22:00", max: "06:00" },
          },
          "02:00",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "time_in_range",
            args: { min: "22:00", max: "06:00" },
          },
          "12:00",
          ctx,
        ),
      ).toBe(false);

      // Open-ended
      expect(
        evaluate(
          { type: "call", name: "time_in_range", args: { min: "09:00" } },
          "14:00",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "time_in_range", args: { min: "09:00" } },
          "08:00",
          ctx,
        ),
      ).toBe(false);

      // 12h format
      expect(
        evaluate(
          {
            type: "call",
            name: "time_in_range",
            args: { min: "09:00", max: "17:00" },
          },
          "10:00 AM",
          ctx,
        ),
      ).toBe(true);
    });

    it("is_date validates date formats", () => {
      expect(
        evaluate({ type: "call", name: "is_date" }, "2024-12-31", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_date" }, "12/31/2024", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_date" }, "not a date", ctx),
      ).toBe(false);
    });

    it("is_tax_year validates tax years", () => {
      expect(evaluate({ type: "call", name: "is_tax_year" }, "2024", ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_tax_year" }, 2024, ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_tax_year" }, "1900", ctx)).toBe(
        false,
      );
    });
  });

  describe("amount predicates", () => {
    it("is_non_negative validates non-negative amounts", () => {
      expect(evaluate({ type: "call", name: "is_non_negative" }, 0, ctx)).toBe(
        true,
      );
      expect(
        evaluate({ type: "call", name: "is_non_negative" }, 100, ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_non_negative" }, "abc123", ctx),
      ).toBe(true);
      expect(evaluate({ type: "call", name: "is_non_negative" }, -1, ctx)).toBe(
        false,
      );
    });

    it("is_positive validates positive amounts", () => {
      expect(evaluate({ type: "call", name: "is_positive" }, 1, ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_positive" }, 0, ctx)).toBe(
        false,
      );
    });

    it("is_negative and is_non_positive validate sign constraints", () => {
      expect(evaluate({ type: "call", name: "is_negative" }, -1, ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_negative" }, 0, ctx)).toBe(
        false,
      );

      expect(evaluate({ type: "call", name: "is_non_positive" }, 0, ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_non_positive" }, 1, ctx)).toBe(
        false,
      );
    });

    it("is_percentage validates percentage values", () => {
      expect(evaluate({ type: "call", name: "is_percentage" }, 50, ctx)).toBe(
        true,
      );
      expect(
        evaluate({ type: "call", name: "is_percentage" }, "50%", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_percentage" }, "12-3%", ctx),
      ).toBe(false);
      expect(evaluate({ type: "call", name: "is_percentage" }, 150, ctx)).toBe(
        false,
      );
    });

    it("is_money_format validates money format", () => {
      expect(
        evaluate({ type: "call", name: "is_money_format" }, "$1,234.56", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_money_format" }, "1234.56", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_money_format" }, "12-3", ctx),
      ).toBe(false);
    });

    it("is_multiple_of validates numeric step divisibility", () => {
      expect(
        evaluate(
          { type: "call", name: "is_multiple_of", args: { value: 5 } },
          10,
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_multiple_of", args: { value: 2.5 } },
          "12.5",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_multiple_of", args: { value: 5 } },
          11,
          ctx,
        ),
      ).toBe(false);
    });
  });

  describe("reference predicates", () => {
    it("is_country_code validates ISO country codes", () => {
      expect(
        evaluate({ type: "call", name: "is_country_code" }, "US", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_country_code" }, "CA", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_country_code" }, "XX", ctx),
      ).toBe(false);
    });

    it("is_currency_code validates ISO 4217 currency codes", () => {
      expect(
        evaluate({ type: "call", name: "is_currency_code" }, "USD", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_currency_code" }, "EUR", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_currency_code" }, "GBP", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_currency_code" }, "JPY", ctx),
      ).toBe(true);
      // Case insensitive
      expect(
        evaluate({ type: "call", name: "is_currency_code" }, "usd", ctx),
      ).toBe(true);
      // Invalid
      expect(
        evaluate({ type: "call", name: "is_currency_code" }, "XYZ", ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_currency_code" }, "US", ctx),
      ).toBe(false);
    });

    it("is_us_state validates US state codes", () => {
      expect(evaluate({ type: "call", name: "is_us_state" }, "CA", ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_us_state" }, "NY", ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_us_state" }, "XX", ctx)).toBe(
        false,
      );
    });

    it("is_us_zip validates US ZIP codes", () => {
      expect(evaluate({ type: "call", name: "is_us_zip" }, "12345", ctx)).toBe(
        true,
      );
      expect(
        evaluate({ type: "call", name: "is_us_zip" }, "12345-6789", ctx),
      ).toBe(true);
      expect(evaluate({ type: "call", name: "is_us_zip" }, "1234", ctx)).toBe(
        false,
      );
    });

    it("is_w2_box12_code validates W-2 box 12 codes", () => {
      expect(
        evaluate({ type: "call", name: "is_w2_box12_code" }, "D", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_w2_box12_code" }, "DD", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_w2_box12_code" }, "X", ctx),
      ).toBe(false);
    });
  });

  describe("aviation predicates", () => {
    it("is_iata_airport_code validates airport codes like SFO", () => {
      expect(
        evaluate({ type: "call", name: "is_iata_airport_code" }, "SFO", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_iata_airport_code" }, "sfo", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_iata_airport_code" }, "SF", ctx),
      ).toBe(false);

      expect(
        evaluate(
          {
            type: "call",
            name: "is_iata_airport_code",
            args: { known_only: true },
          },
          "SFO",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "is_iata_airport_code",
            args: { known_only: true },
          },
          "ZZZ",
          ctx,
        ),
      ).toBe(false);
    });

    it("is_icao_airport_code validates airport codes like KSFO", () => {
      expect(
        evaluate({ type: "call", name: "is_icao_airport_code" }, "KSFO", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_icao_airport_code" }, "ksfo", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_icao_airport_code" }, "SFO", ctx),
      ).toBe(false);
    });

    it("validates airline codes", () => {
      expect(
        evaluate({ type: "call", name: "is_iata_airline_code" }, "UA", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_iata_airline_code" }, "B6", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_icao_airline_code" }, "UAL", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_icao_airline_code" }, "UA", ctx),
      ).toBe(false);
    });

    it("is_airport_code supports IATA/ICAO/ANY modes", () => {
      expect(
        evaluate({ type: "call", name: "is_airport_code" }, "SFO", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_airport_code" }, "KSFO", ctx),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_airport_code", args: { system: "IATA" } },
          "KSFO",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_airport_code", args: { system: "ICAO" } },
          "KSFO",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "is_airport_code",
            args: { system: "IATA", known_only: true },
          },
          "ZZZ",
          ctx,
        ),
      ).toBe(false);
    });

    it("is_airline_code supports IATA/ICAO/ANY modes", () => {
      expect(
        evaluate({ type: "call", name: "is_airline_code" }, "UA", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_airline_code" }, "UAL", ctx),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_airline_code", args: { system: "IATA" } },
          "UAL",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_airline_code", args: { system: "ICAO" } },
          "UAL",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "is_airline_code",
            args: { system: "IATA", known_only: true },
          },
          "ZZ",
          ctx,
        ),
      ).toBe(false);
    });

    it("is_flight_number validates common formats", () => {
      expect(
        evaluate({ type: "call", name: "is_flight_number" }, "UA123", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_flight_number" }, "UAL1234A", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_flight_number" }, "UA 123", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_flight_number" }, "UA12345", ctx),
      ).toBe(false);

      expect(
        evaluate(
          {
            type: "call",
            name: "is_flight_number",
            args: { carrier_format: "ICAO" },
          },
          "UAL123",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "is_flight_number",
            args: { carrier_format: "ICAO" },
          },
          "UA123",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate(
          {
            type: "call",
            name: "is_flight_number",
            args: { carrier_format: "ICAO", allow_suffix: false },
          },
          "UAL123A",
          ctx,
        ),
      ).toBe(false);
    });
  });

  describe("payment card predicates", () => {
    it("is_credit_card validates card numbers with Luhn + network", () => {
      // Valid Visa
      expect(
        evaluate(
          { type: "call", name: "is_credit_card" },
          "4532015112830366",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_credit_card", args: { network: "visa" } },
          "4532015112830366",
          ctx,
        ),
      ).toBe(true);

      // Valid Amex
      expect(
        evaluate(
          { type: "call", name: "is_credit_card" },
          "371449635398431",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_credit_card", args: { network: "amex" } },
          "371449635398431",
          ctx,
        ),
      ).toBe(true);

      // Valid Mastercard
      expect(
        evaluate(
          { type: "call", name: "is_credit_card" },
          "5425233430109903",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "is_credit_card",
            args: { network: "mastercard" },
          },
          "5425233430109903",
          ctx,
        ),
      ).toBe(true);

      // Valid Discover
      expect(
        evaluate(
          { type: "call", name: "is_credit_card" },
          "6011111111111117",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "is_credit_card",
            args: { network: "discover" },
          },
          "6011111111111117",
          ctx,
        ),
      ).toBe(true);

      // With spaces
      expect(
        evaluate(
          { type: "call", name: "is_credit_card" },
          "4532 0151 1283 0366",
          ctx,
        ),
      ).toBe(true);

      // Wrong network
      expect(
        evaluate(
          { type: "call", name: "is_credit_card", args: { network: "amex" } },
          "4532015112830366",
          ctx,
        ),
      ).toBe(false);

      // Invalid Luhn
      expect(
        evaluate(
          { type: "call", name: "is_credit_card" },
          "4532015112830367",
          ctx,
        ),
      ).toBe(false);
    });

    it("is_cvv validates CVV codes", () => {
      // 3 digits (no network)
      expect(evaluate({ type: "call", name: "is_cvv" }, "123", ctx)).toBe(true);
      // 4 digits (no network)
      expect(evaluate({ type: "call", name: "is_cvv" }, "1234", ctx)).toBe(
        true,
      );
      // Amex: exactly 4
      expect(
        evaluate(
          { type: "call", name: "is_cvv", args: { network: "amex" } },
          "1234",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_cvv", args: { network: "amex" } },
          "123",
          ctx,
        ),
      ).toBe(false);
      // Visa: exactly 3
      expect(
        evaluate(
          { type: "call", name: "is_cvv", args: { network: "visa" } },
          "123",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_cvv", args: { network: "visa" } },
          "1234",
          ctx,
        ),
      ).toBe(false);
      // Invalid: non-digits
      expect(evaluate({ type: "call", name: "is_cvv" }, "12a", ctx)).toBe(
        false,
      );
      // Invalid: too short
      expect(evaluate({ type: "call", name: "is_cvv" }, "12", ctx)).toBe(false);
    });

    it("is_card_expiry validates expiry dates", () => {
      // Valid MM/YY
      expect(
        evaluate({ type: "call", name: "is_card_expiry" }, "12/25", ctx),
      ).toBe(true);
      // Valid MM/YYYY
      expect(
        evaluate({ type: "call", name: "is_card_expiry" }, "12/2025", ctx),
      ).toBe(true);
      // Invalid month
      expect(
        evaluate({ type: "call", name: "is_card_expiry" }, "13/25", ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_card_expiry" }, "00/25", ctx),
      ).toBe(false);
      // Invalid format
      expect(
        evaluate({ type: "call", name: "is_card_expiry" }, "12-25", ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_card_expiry" }, "1/25", ctx),
      ).toBe(false);
      // Expired with reject_expired
      expect(
        evaluate(
          {
            type: "call",
            name: "is_card_expiry",
            args: { reject_expired: true },
          },
          "01/20",
          ctx,
        ),
      ).toBe(false);
      // Far future with reject_expired
      expect(
        evaluate(
          {
            type: "call",
            name: "is_card_expiry",
            args: { reject_expired: true },
          },
          "12/99",
          ctx,
        ),
      ).toBe(true);
    });
  });

  describe("international banking predicates", () => {
    it("is_iban validates IBAN numbers", () => {
      // Valid IBANs
      expect(
        evaluate(
          { type: "call", name: "is_iban" },
          "GB29NWBK60161331926819",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_iban" },
          "DE89370400440532013000",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_iban" },
          "FR7630006000011234567890189",
          ctx,
        ),
      ).toBe(true);

      // With spaces
      expect(
        evaluate(
          { type: "call", name: "is_iban" },
          "GB29 NWBK 6016 1331 9268 19",
          ctx,
        ),
      ).toBe(true);

      // Country filter
      expect(
        evaluate(
          { type: "call", name: "is_iban", args: { country: "DE" } },
          "DE89370400440532013000",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_iban", args: { country: "GB" } },
          "DE89370400440532013000",
          ctx,
        ),
      ).toBe(false);

      // Invalid checksum
      expect(
        evaluate(
          { type: "call", name: "is_iban" },
          "GB29NWBK60161331926818",
          ctx,
        ),
      ).toBe(false);

      // Too short
      expect(evaluate({ type: "call", name: "is_iban" }, "GB29", ctx)).toBe(
        false,
      );
    });

    it("is_bic validates BIC/SWIFT codes", () => {
      // Valid 8-char
      expect(evaluate({ type: "call", name: "is_bic" }, "DEUTDEFF", ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_bic" }, "CHASUS33", ctx)).toBe(
        true,
      );

      // Valid 11-char
      expect(
        evaluate({ type: "call", name: "is_bic" }, "DEUTDEFF500", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_bic" }, "COBADEFFXXX", ctx),
      ).toBe(true);

      // Case insensitive
      expect(evaluate({ type: "call", name: "is_bic" }, "deutdeff", ctx)).toBe(
        true,
      );

      // Invalid: wrong length
      expect(evaluate({ type: "call", name: "is_bic" }, "DEUTDE", ctx)).toBe(
        false,
      );
      expect(
        evaluate({ type: "call", name: "is_bic" }, "DEUTDEFF50", ctx),
      ).toBe(false);

      // Invalid: digits in bank code
      expect(evaluate({ type: "call", name: "is_bic" }, "D3UTDEFF", ctx)).toBe(
        false,
      );
    });
  });

  describe("product / ecommerce predicates", () => {
    it("is_vin validates Vehicle Identification Numbers", () => {
      // Valid VINs
      expect(
        evaluate({ type: "call", name: "is_vin" }, "11111111111111111", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_vin" }, "1M8GDM9AXKP042788", ctx),
      ).toBe(true);

      // Without checksum
      expect(
        evaluate(
          { type: "call", name: "is_vin", args: { validate_checksum: false } },
          "12345678901234567",
          ctx,
        ),
      ).toBe(true);

      // Invalid: contains I, O, or Q
      expect(
        evaluate({ type: "call", name: "is_vin" }, "1M8GDM9AXKPI42788", ctx),
      ).toBe(false);

      // Invalid: wrong length
      expect(
        evaluate({ type: "call", name: "is_vin" }, "1234567890", ctx),
      ).toBe(false);

      // Invalid: bad check digit
      expect(
        evaluate({ type: "call", name: "is_vin" }, "1M8GDM9AXKP042789", ctx),
      ).toBe(false);
    });

    it("is_upc validates UPC-A barcodes", () => {
      // Valid
      expect(
        evaluate({ type: "call", name: "is_upc" }, "036000291452", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_upc" }, "012345678905", ctx),
      ).toBe(true);

      // Invalid: bad check digit
      expect(
        evaluate({ type: "call", name: "is_upc" }, "036000291453", ctx),
      ).toBe(false);

      // Invalid: wrong length
      expect(evaluate({ type: "call", name: "is_upc" }, "12345", ctx)).toBe(
        false,
      );
    });

    it("is_ean validates EAN-13 barcodes", () => {
      // Valid
      expect(
        evaluate({ type: "call", name: "is_ean" }, "4006381333931", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_ean" }, "5901234123457", ctx),
      ).toBe(true);

      // Invalid: bad check digit
      expect(
        evaluate({ type: "call", name: "is_ean" }, "4006381333932", ctx),
      ).toBe(false);

      // Invalid: wrong length
      expect(evaluate({ type: "call", name: "is_ean" }, "12345", ctx)).toBe(
        false,
      );
    });

    it("is_isbn validates ISBN-10 and ISBN-13", () => {
      // Valid ISBN-10
      expect(
        evaluate({ type: "call", name: "is_isbn" }, "0306406152", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_isbn" }, "0-306-40615-2", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_isbn" }, "007462542X", ctx),
      ).toBe(true);

      // Valid ISBN-13
      expect(
        evaluate({ type: "call", name: "is_isbn" }, "9780306406157", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_isbn" }, "978-0-306-40615-7", ctx),
      ).toBe(true);

      // Version filter
      expect(
        evaluate(
          { type: "call", name: "is_isbn", args: { version: 10 } },
          "0306406152",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_isbn", args: { version: 10 } },
          "9780306406157",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_isbn", args: { version: 13 } },
          "9780306406157",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_isbn", args: { version: 13 } },
          "0306406152",
          ctx,
        ),
      ).toBe(false);

      // Invalid
      expect(
        evaluate({ type: "call", name: "is_isbn" }, "0306406153", ctx),
      ).toBe(false);
      expect(evaluate({ type: "call", name: "is_isbn" }, "12345", ctx)).toBe(
        false,
      );
    });
  });

  describe("text analysis predicates", () => {
    it("is_rtl detects RTL text", () => {
      // Arabic
      expect(evaluate({ type: "call", name: "is_rtl" }, "مرحبا", ctx)).toBe(
        true,
      );
      // Hebrew
      expect(evaluate({ type: "call", name: "is_rtl" }, "שלום", ctx)).toBe(
        true,
      );
      // Mixed
      expect(
        evaluate({ type: "call", name: "is_rtl" }, "Hello مرحبا", ctx),
      ).toBe(true);
      // English only
      expect(
        evaluate({ type: "call", name: "is_rtl" }, "Hello world", ctx),
      ).toBe(false);
      // Numbers only
      expect(evaluate({ type: "call", name: "is_rtl" }, "12345", ctx)).toBe(
        false,
      );
      // Empty
      expect(evaluate({ type: "call", name: "is_rtl" }, "", ctx)).toBe(false);
    });

    it("is_rtl supports threshold", () => {
      // "مرحبا Hello" - ~50% RTL
      expect(
        evaluate(
          { type: "call", name: "is_rtl", args: { threshold: 0.5 } },
          "مرحبا Hello",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_rtl", args: { threshold: 0.8 } },
          "مرحبا Hello World Test",
          ctx,
        ),
      ).toBe(false);
    });

    it("is_ltr validates LTR-only text", () => {
      expect(
        evaluate({ type: "call", name: "is_ltr" }, "Hello world", ctx),
      ).toBe(true);
      expect(evaluate({ type: "call", name: "is_ltr" }, "12345", ctx)).toBe(
        true,
      );
      // Contains RTL → false
      expect(
        evaluate({ type: "call", name: "is_ltr" }, "Hello مرحبا", ctx),
      ).toBe(false);
      expect(evaluate({ type: "call", name: "is_ltr" }, "שלום", ctx)).toBe(
        false,
      );
      // Empty
      expect(evaluate({ type: "call", name: "is_ltr" }, "", ctx)).toBe(false);
    });

    it("starts_with / ends_with / contains validate substring constraints", () => {
      expect(
        evaluate(
          { type: "call", name: "starts_with", args: { value: "foo" } },
          "foobar",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "ends_with", args: { value: "bar" } },
          "foobar",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "contains", args: { value: "oob" } },
          "foobar",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "contains", args: { value: "baz" } },
          "foobar",
          ctx,
        ),
      ).toBe(false);
    });
  });

  describe("color predicates", () => {
    it("is_hex_color validates hex color strings", () => {
      // 3-digit
      expect(
        evaluate({ type: "call", name: "is_hex_color" }, "#fff", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_hex_color" }, "#F0A", ctx),
      ).toBe(true);
      // 6-digit
      expect(
        evaluate({ type: "call", name: "is_hex_color" }, "#FF00AA", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_hex_color" }, "#000000", ctx),
      ).toBe(true);
      // 4-digit (alpha)
      expect(
        evaluate({ type: "call", name: "is_hex_color" }, "#F0AF", ctx),
      ).toBe(true);
      // 8-digit (alpha)
      expect(
        evaluate({ type: "call", name: "is_hex_color" }, "#FF00AAFF", ctx),
      ).toBe(true);
      // No alpha
      expect(
        evaluate(
          { type: "call", name: "is_hex_color", args: { allow_alpha: false } },
          "#FFF",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_hex_color", args: { allow_alpha: false } },
          "#FFFF",
          ctx,
        ),
      ).toBe(false);
      // No hash required
      expect(
        evaluate({ type: "call", name: "is_hex_color" }, "FF00AA", ctx),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_hex_color", args: { require_hash: false } },
          "FF00AA",
          ctx,
        ),
      ).toBe(true);
      // Invalid
      expect(
        evaluate({ type: "call", name: "is_hex_color" }, "#GGG", ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_hex_color" }, "#12345", ctx),
      ).toBe(false);
      expect(evaluate({ type: "call", name: "is_hex_color" }, "red", ctx)).toBe(
        false,
      );
    });

    it("is_rgb_color validates RGB/RGBA color strings", () => {
      // Comma-separated
      expect(
        evaluate(
          { type: "call", name: "is_rgb_color" },
          "rgb(255, 0, 170)",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_rgb_color" }, "rgb(0, 0, 0)", ctx),
      ).toBe(true);
      // Space-separated
      expect(
        evaluate({ type: "call", name: "is_rgb_color" }, "rgb(255 0 170)", ctx),
      ).toBe(true);
      // Percentages
      expect(
        evaluate(
          { type: "call", name: "is_rgb_color" },
          "rgb(100%, 0%, 50%)",
          ctx,
        ),
      ).toBe(true);
      // RGBA
      expect(
        evaluate(
          { type: "call", name: "is_rgb_color" },
          "rgba(255, 0, 170, 0.5)",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_rgb_color" },
          "rgba(255, 0, 170, 50%)",
          ctx,
        ),
      ).toBe(true);
      // Slash alpha
      expect(
        evaluate(
          { type: "call", name: "is_rgb_color" },
          "rgb(255 0 170 / 0.5)",
          ctx,
        ),
      ).toBe(true);
      // Invalid
      expect(
        evaluate({ type: "call", name: "is_rgb_color" }, "rgb(256, 0, 0)", ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_rgb_color" }, "rgb(-1, 0, 0)", ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_rgb_color" }, "rgb(255, 0)", ctx),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_rgb_color" },
          "rgb(255, 0%, 0)",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_rgb_color" }, "hsl(0, 0%, 0%)", ctx),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_rgb_color" },
          "rgba(255, 0, 0, 1.5)",
          ctx,
        ),
      ).toBe(false);
    });

    it("is_hsl_color validates HSL/HSLA color strings", () => {
      // Comma-separated
      expect(
        evaluate(
          { type: "call", name: "is_hsl_color" },
          "hsl(360, 100%, 50%)",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_hsl_color" }, "hsl(0, 0%, 0%)", ctx),
      ).toBe(true);
      // Space-separated
      expect(
        evaluate(
          { type: "call", name: "is_hsl_color" },
          "hsl(360 100% 50%)",
          ctx,
        ),
      ).toBe(true);
      // HSLA
      expect(
        evaluate(
          { type: "call", name: "is_hsl_color" },
          "hsla(360, 100%, 50%, 0.5)",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_hsl_color" },
          "hsla(360, 100%, 50%, 50%)",
          ctx,
        ),
      ).toBe(true);
      // Slash alpha
      expect(
        evaluate(
          { type: "call", name: "is_hsl_color" },
          "hsl(360 100% 50% / 0.5)",
          ctx,
        ),
      ).toBe(true);
      // deg suffix
      expect(
        evaluate(
          { type: "call", name: "is_hsl_color" },
          "hsl(360deg, 100%, 50%)",
          ctx,
        ),
      ).toBe(true);
      // Invalid
      expect(
        evaluate(
          { type: "call", name: "is_hsl_color" },
          "hsl(361, 100%, 50%)",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_hsl_color" },
          "hsl(360, 100, 50)",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_hsl_color" },
          "hsl(360, 101%, 50%)",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_hsl_color" }, "rgb(255, 0, 0)", ctx),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_hsl_color" },
          "hsla(360, 100%, 50%, 1.5)",
          ctx,
        ),
      ).toBe(false);
    });
  });

  describe("numeric type predicates", () => {
    it("is_integer validates whole numbers", () => {
      expect(evaluate({ type: "call", name: "is_integer" }, 42, ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_integer" }, 0, ctx)).toBe(true);
      expect(evaluate({ type: "call", name: "is_integer" }, -100, ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_integer" }, "42", ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_integer" }, 42.0, ctx)).toBe(
        true,
      );

      expect(evaluate({ type: "call", name: "is_integer" }, 3.14, ctx)).toBe(
        false,
      );
      expect(evaluate({ type: "call", name: "is_integer" }, "3.14", ctx)).toBe(
        false,
      );
      expect(
        evaluate({ type: "call", name: "is_integer" }, "123abc", ctx),
      ).toBe(false);
      expect(evaluate({ type: "call", name: "is_integer" }, "abc", ctx)).toBe(
        false,
      );
    });

    it("is_float validates numbers", () => {
      expect(evaluate({ type: "call", name: "is_float" }, 3.14, ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_float" }, 42, ctx)).toBe(true);
      expect(evaluate({ type: "call", name: "is_float" }, "3.14", ctx)).toBe(
        true,
      );

      expect(evaluate({ type: "call", name: "is_float" }, "123abc", ctx)).toBe(
        false,
      );
      expect(evaluate({ type: "call", name: "is_float" }, "abc", ctx)).toBe(
        false,
      );
    });

    it("is_u8 validates 0-255", () => {
      expect(evaluate({ type: "call", name: "is_u8" }, 0, ctx)).toBe(true);
      expect(evaluate({ type: "call", name: "is_u8" }, 255, ctx)).toBe(true);
      expect(evaluate({ type: "call", name: "is_u8" }, "128", ctx)).toBe(true);

      expect(evaluate({ type: "call", name: "is_u8" }, -1, ctx)).toBe(false);
      expect(evaluate({ type: "call", name: "is_u8" }, 256, ctx)).toBe(false);
      expect(evaluate({ type: "call", name: "is_u8" }, 3.14, ctx)).toBe(false);
    });

    it("is_u16 validates 0-65535", () => {
      expect(evaluate({ type: "call", name: "is_u16" }, 0, ctx)).toBe(true);
      expect(evaluate({ type: "call", name: "is_u16" }, 65535, ctx)).toBe(true);

      expect(evaluate({ type: "call", name: "is_u16" }, -1, ctx)).toBe(false);
      expect(evaluate({ type: "call", name: "is_u16" }, 65536, ctx)).toBe(
        false,
      );
    });

    it("is_u32 validates 0-4294967295", () => {
      expect(evaluate({ type: "call", name: "is_u32" }, 0, ctx)).toBe(true);
      expect(evaluate({ type: "call", name: "is_u32" }, 4294967295, ctx)).toBe(
        true,
      );

      expect(evaluate({ type: "call", name: "is_u32" }, -1, ctx)).toBe(false);
      expect(evaluate({ type: "call", name: "is_u32" }, 4294967296, ctx)).toBe(
        false,
      );
    });

    it("is_u64 validates non-negative 64-bit integers", () => {
      expect(evaluate({ type: "call", name: "is_u64" }, 0, ctx)).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_u64" }, "18446744073709551615", ctx),
      ).toBe(true);

      expect(evaluate({ type: "call", name: "is_u64" }, -1, ctx)).toBe(false);
      expect(evaluate({ type: "call", name: "is_u64" }, 3.14, ctx)).toBe(false);
    });

    it("is_i8 validates -128 to 127", () => {
      expect(evaluate({ type: "call", name: "is_i8" }, -128, ctx)).toBe(true);
      expect(evaluate({ type: "call", name: "is_i8" }, 127, ctx)).toBe(true);

      expect(evaluate({ type: "call", name: "is_i8" }, -129, ctx)).toBe(false);
      expect(evaluate({ type: "call", name: "is_i8" }, 128, ctx)).toBe(false);
    });

    it("is_i16 validates -32768 to 32767", () => {
      expect(evaluate({ type: "call", name: "is_i16" }, -32768, ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_i16" }, 32767, ctx)).toBe(true);

      expect(evaluate({ type: "call", name: "is_i16" }, -32769, ctx)).toBe(
        false,
      );
      expect(evaluate({ type: "call", name: "is_i16" }, 32768, ctx)).toBe(
        false,
      );
    });

    it("is_i32 validates -2147483648 to 2147483647", () => {
      expect(evaluate({ type: "call", name: "is_i32" }, -2147483648, ctx)).toBe(
        true,
      );
      expect(evaluate({ type: "call", name: "is_i32" }, 2147483647, ctx)).toBe(
        true,
      );

      expect(evaluate({ type: "call", name: "is_i32" }, -2147483649, ctx)).toBe(
        false,
      );
      expect(evaluate({ type: "call", name: "is_i32" }, 2147483648, ctx)).toBe(
        false,
      );
    });

    it("is_i64 validates 64-bit signed integers", () => {
      expect(evaluate({ type: "call", name: "is_i64" }, 0, ctx)).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_i64" }, "9223372036854775807", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_i64" }, "-9223372036854775808", ctx),
      ).toBe(true);

      expect(
        evaluate({ type: "call", name: "is_i64" }, "18446744073709551615", ctx),
      ).toBe(false);
      expect(evaluate({ type: "call", name: "is_i64" }, 3.14, ctx)).toBe(false);
    });
  });

  describe("decimal places predicate", () => {
    it("is_decimal_places validates exact decimal places", () => {
      expect(
        evaluate(
          { type: "call", name: "is_decimal_places", args: { places: 2 } },
          "123.45",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_decimal_places", args: { places: 2 } },
          "0.99",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_decimal_places", args: { places: 2 } },
          "123.4",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_decimal_places", args: { places: 2 } },
          "123.456",
          ctx,
        ),
      ).toBe(false);

      // 0 decimal places
      expect(
        evaluate(
          { type: "call", name: "is_decimal_places", args: { places: 0 } },
          "123",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_decimal_places", args: { places: 0 } },
          "123.4",
          ctx,
        ),
      ).toBe(false);

      // 3 decimal places
      expect(
        evaluate(
          { type: "call", name: "is_decimal_places", args: { places: 3 } },
          "1.234",
          ctx,
        ),
      ).toBe(true);
    });

    it("is_decimal_places supports max mode", () => {
      expect(
        evaluate(
          {
            type: "call",
            name: "is_decimal_places",
            args: { places: 2, max: true },
          },
          "123",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "is_decimal_places",
            args: { places: 2, max: true },
          },
          "123.4",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "is_decimal_places",
            args: { places: 2, max: true },
          },
          "123.45",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "is_decimal_places",
            args: { places: 2, max: true },
          },
          "123.456",
          ctx,
        ),
      ).toBe(false);
    });

    it("is_decimal_places handles negatives and numbers", () => {
      expect(
        evaluate(
          { type: "call", name: "is_decimal_places", args: { places: 2 } },
          "-123.45",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_decimal_places", args: { places: 2 } },
          123.45,
          ctx,
        ),
      ).toBe(true);
    });
  });

  describe("insurance predicates", () => {
    it("is_npi validates National Provider Identifiers", () => {
      // Valid NPIs (Luhn with 80840 prefix)
      expect(
        evaluate({ type: "call", name: "is_npi" }, "1234567893", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_npi" }, "1245319599", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_npi" }, "1123456786", ctx),
      ).toBe(true);

      // Invalid: wrong check digit
      expect(
        evaluate({ type: "call", name: "is_npi" }, "1234567890", ctx),
      ).toBe(false);
      // Invalid: too short
      expect(evaluate({ type: "call", name: "is_npi" }, "123456789", ctx)).toBe(
        false,
      );
      // Invalid: non-digit
      expect(
        evaluate({ type: "call", name: "is_npi" }, "123456789A", ctx),
      ).toBe(false);
    });

    it("is_dea_number validates DEA registration numbers", () => {
      // Valid: (1+3+5) + 2*(2+4+6) = 9 + 24 = 33, 33 % 10 = 3
      expect(
        evaluate({ type: "call", name: "is_dea_number" }, "AB1234563", ctx),
      ).toBe(true);
      // Case insensitive
      expect(
        evaluate({ type: "call", name: "is_dea_number" }, "ab1234563", ctx),
      ).toBe(true);

      // Invalid: wrong check digit
      expect(
        evaluate({ type: "call", name: "is_dea_number" }, "AB1234560", ctx),
      ).toBe(false);
      // Invalid: bad type code
      expect(
        evaluate({ type: "call", name: "is_dea_number" }, "ZB1234563", ctx),
      ).toBe(false);
      // Invalid: too short
      expect(
        evaluate({ type: "call", name: "is_dea_number" }, "AB123456", ctx),
      ).toBe(false);
    });

    it("healthcare code predicates validate ICD-10/CPT/HCPCS/NDC", () => {
      expect(
        evaluate({ type: "call", name: "is_icd10_code" }, "E11.9", ctx),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "is_icd10_code",
            args: { strict_format: true },
          },
          "S72001A",
          ctx,
        ),
      ).toBe(false);

      expect(
        evaluate({ type: "call", name: "is_cpt_code" }, "99213", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_cpt_code" }, "1234F", ctx),
      ).toBe(true);
      expect(
        evaluate(
          {
            type: "call",
            name: "is_cpt_code",
            args: { allow_category_ii: false },
          },
          "1234F",
          ctx,
        ),
      ).toBe(false);

      expect(
        evaluate({ type: "call", name: "is_hcpcs_code" }, "A0428", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_hcpcs_code" }, "W1234", ctx),
      ).toBe(false);

      expect(
        evaluate({ type: "call", name: "is_ndc_code" }, "12345-6789-01", ctx),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_ndc_code", args: { format: "11" } },
          "1234567890",
          ctx,
        ),
      ).toBe(false);
    });
  });

  describe("encoding / crypto predicates", () => {
    it("is_base58 validates base58 strings", () => {
      expect(
        evaluate(
          { type: "call", name: "is_base58" },
          "5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyTJ",
          ctx,
        ),
      ).toBe(true);

      // Invalid: contains 0
      expect(evaluate({ type: "call", name: "is_base58" }, "0ABC", ctx)).toBe(
        false,
      );
      // Invalid: contains O
      expect(evaluate({ type: "call", name: "is_base58" }, "OABC", ctx)).toBe(
        false,
      );
      // Invalid: contains I
      expect(evaluate({ type: "call", name: "is_base58" }, "IABC", ctx)).toBe(
        false,
      );
      // Invalid: contains l
      expect(evaluate({ type: "call", name: "is_base58" }, "lABC", ctx)).toBe(
        false,
      );
      // Invalid: empty
      expect(evaluate({ type: "call", name: "is_base58" }, "", ctx)).toBe(
        false,
      );
    });

    it("is_base64 validates base64 strings", () => {
      expect(
        evaluate({ type: "call", name: "is_base64" }, "SGVsbG8gV29ybGQ=", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_base64" }, "dGVzdA==", ctx),
      ).toBe(true);
      expect(evaluate({ type: "call", name: "is_base64" }, "AQID", ctx)).toBe(
        true,
      );

      // URL-safe
      expect(
        evaluate(
          { type: "call", name: "is_base64", args: { url_safe: true } },
          "abc-def_ghi",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_base64", args: { url_safe: true } },
          "abc+def/ghi",
          ctx,
        ),
      ).toBe(false);

      // Invalid
      expect(evaluate({ type: "call", name: "is_base64" }, "", ctx)).toBe(
        false,
      );
      expect(
        evaluate({ type: "call", name: "is_base64" }, "Hello World!", ctx),
      ).toBe(false);
      expect(evaluate({ type: "call", name: "is_base64" }, "A===", ctx)).toBe(
        false,
      );
    });

    it("is_bitcoin_address validates Bitcoin addresses", () => {
      // P2PKH
      expect(
        evaluate(
          { type: "call", name: "is_bitcoin_address" },
          "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
          ctx,
        ),
      ).toBe(true);
      // P2SH
      expect(
        evaluate(
          { type: "call", name: "is_bitcoin_address" },
          "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy",
          ctx,
        ),
      ).toBe(true);
      // Bech32 SegWit
      expect(
        evaluate(
          { type: "call", name: "is_bitcoin_address" },
          "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
          ctx,
        ),
      ).toBe(true);
      // Taproot
      expect(
        evaluate(
          { type: "call", name: "is_bitcoin_address" },
          "bc1p5d7rjq7g6rdk2yhzks9smlaqtedr4dekq08ge8ztwac72sfr9rusxg3s7a",
          ctx,
        ),
      ).toBe(true);

      // Invalid
      expect(
        evaluate({ type: "call", name: "is_bitcoin_address" }, "1A1z", ctx),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_bitcoin_address" },
          "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_bitcoin_address" }, "", ctx),
      ).toBe(false);
    });

    it("is_ethereum_address validates Ethereum addresses", () => {
      expect(
        evaluate(
          { type: "call", name: "is_ethereum_address" },
          "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_ethereum_address" },
          "0x0000000000000000000000000000000000000000",
          ctx,
        ),
      ).toBe(true);

      // Missing 0x
      expect(
        evaluate(
          { type: "call", name: "is_ethereum_address" },
          "742d35Cc6634C0532925a3b844Bc9e7595f2bD18",
          ctx,
        ),
      ).toBe(false);
      // Too short
      expect(
        evaluate(
          { type: "call", name: "is_ethereum_address" },
          "0x742d35Cc",
          ctx,
        ),
      ).toBe(false);
    });

    it("is_solana_address validates Solana addresses", () => {
      expect(
        evaluate(
          { type: "call", name: "is_solana_address" },
          "7EcDhSYGxXyscszYEp35KHN8vvw3svAuLKTzXwCFLtV",
          ctx,
        ),
      ).toBe(true);

      // Too short
      expect(
        evaluate(
          { type: "call", name: "is_solana_address" },
          "7EcDhSYGxXyscszYEp35KHN8vvw",
          ctx,
        ),
      ).toBe(false);
      // Contains invalid base58 (0)
      expect(
        evaluate(
          { type: "call", name: "is_solana_address" },
          "0EcDhSYGxXyscszYEp35KHN8vvw3svAuLKTzXwCFLtV",
          ctx,
        ),
      ).toBe(false);
    });

    it("is_jwt validates compact jwt shape", () => {
      expect(
        evaluate(
          { type: "call", name: "is_jwt" },
          "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjMifQ.dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk",
          ctx,
        ),
      ).toBe(true);
      expect(evaluate({ type: "call", name: "is_jwt" }, "abc.def", ctx)).toBe(
        false,
      );
    });

    it("is_hash validates digest lengths", () => {
      expect(
        evaluate(
          { type: "call", name: "is_hash", args: { algorithm: "md5" } },
          "d41d8cd98f00b204e9800998ecf8427e",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_hash", args: { algorithm: "sha256" } },
          "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_hash", args: { algorithm: "sha256" } },
          "d41d8cd98f00b204e9800998ecf8427e",
          ctx,
        ),
      ).toBe(false);
    });
  });

  describe("contact predicates", () => {
    it("phone_number_us validates US phone numbers only", () => {
      // US formats pass
      expect(
        evaluate(
          { type: "call", name: "phone_number_us" },
          "(555) 234-5678",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "phone_number_us" },
          "555-234-5678",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "phone_number_us" }, "5552345678", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "phone_number_us" }, "15552345678", ctx),
      ).toBe(true);
      // International/short numbers fail
      expect(
        evaluate(
          { type: "call", name: "phone_number_us" },
          "+1 555-234-5678",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "phone_number_us" },
          "+62 34234233",
          ctx,
        ),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "phone_number_us" }, "123", ctx),
      ).toBe(false);
    });

    it("phone_number validates US and international phone numbers", () => {
      // US formats pass
      expect(
        evaluate({ type: "call", name: "phone_number" }, "(555) 234-5678", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "phone_number" }, "555-234-5678", ctx),
      ).toBe(true);
      // International formats pass
      expect(
        evaluate(
          { type: "call", name: "phone_number" },
          "+1 555-234-5678",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "phone_number" }, "+62 34234233", ctx),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "phone_number" },
          "+44 20 7946 0958",
          ctx,
        ),
      ).toBe(true);
      // Junk fails
      expect(evaluate({ type: "call", name: "phone_number" }, "123", ctx)).toBe(
        false,
      );
      expect(
        evaluate({ type: "call", name: "phone_number" }, "not-a-phone", ctx),
      ).toBe(false);
    });

    it("is_phone is backwards-compat alias for phone_number", () => {
      // US formats pass
      expect(
        evaluate({ type: "call", name: "is_phone" }, "(555) 234-5678", ctx),
      ).toBe(true);
      // International formats now pass (changed behavior)
      expect(
        evaluate({ type: "call", name: "is_phone" }, "+1 555-234-5678", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_phone" }, "+62 34234233", ctx),
      ).toBe(true);
    });

    it("is_uuid validates UUID format", () => {
      expect(
        evaluate(
          { type: "call", name: "is_uuid" },
          "550e8400-e29b-41d4-a716-446655440000",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_uuid" },
          "f47ac10b-58cc-4372-a567-0e02b2c3d479",
          ctx,
        ),
      ).toBe(true);
      // Case insensitive
      expect(
        evaluate(
          { type: "call", name: "is_uuid" },
          "550E8400-E29B-41D4-A716-446655440000",
          ctx,
        ),
      ).toBe(true);
      // Version filter
      expect(
        evaluate(
          { type: "call", name: "is_uuid", args: { version: 4 } },
          "f47ac10b-58cc-4372-a567-0e02b2c3d479",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_uuid", args: { version: 4 } },
          "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
          ctx,
        ),
      ).toBe(false);
      // Invalid
      expect(
        evaluate({ type: "call", name: "is_uuid" }, "not-a-uuid", ctx),
      ).toBe(false);
      expect(
        evaluate(
          { type: "call", name: "is_uuid" },
          "550e8400e29b41d4a716446655440000",
          ctx,
        ),
      ).toBe(false);
    });

    it("is_url validates URLs", () => {
      // Valid
      expect(
        evaluate({ type: "call", name: "is_url" }, "http://example.com", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_url" }, "https://example.com", ctx),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_url" },
          "https://example.com/path",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_url" },
          "https://sub.example.com",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_url" },
          "https://example.com:8080/path",
          ctx,
        ),
      ).toBe(true);

      // Require HTTPS
      expect(
        evaluate(
          { type: "call", name: "is_url", args: { require_https: true } },
          "https://example.com",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_url", args: { require_https: true } },
          "http://example.com",
          ctx,
        ),
      ).toBe(false);

      // Invalid
      expect(
        evaluate({ type: "call", name: "is_url" }, "example.com", ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_url" }, "ftp://example.com", ctx),
      ).toBe(false);
      expect(evaluate({ type: "call", name: "is_url" }, "http://", ctx)).toBe(
        false,
      );
      expect(evaluate({ type: "call", name: "is_url" }, "not a url", ctx)).toBe(
        false,
      );
    });

    it("is_email validates email addresses", () => {
      expect(
        evaluate({ type: "call", name: "is_email" }, "test@example.com", ctx),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_email" },
          "user.name+tag@domain.org",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_email" },
          "user@sub.example.com",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_email" }, " test@example.com ", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_email" }, "not-an-email", ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_email" }, "missing@domain", ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_email" }, "user@@example.com", ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_email" }, ".user@example.com", ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_email" }, "user.@example.com", ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_email" }, "user@example.", ctx),
      ).toBe(false);
      expect(
        evaluate({ type: "call", name: "is_email" }, "user@example.c0", ctx),
      ).toBe(false);
    });

    it("is_ip / is_cidr / is_mac_address validate network formats", () => {
      expect(
        evaluate({ type: "call", name: "is_ip" }, "192.168.1.1", ctx),
      ).toBe(true);
      expect(
        evaluate(
          { type: "call", name: "is_ip", args: { version: "v4" } },
          "2001:db8::1",
          ctx,
        ),
      ).toBe(false);

      expect(
        evaluate({ type: "call", name: "is_cidr" }, "10.0.0.0/8", ctx),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_cidr" }, "10.0.0.0/33", ctx),
      ).toBe(false);

      expect(
        evaluate(
          { type: "call", name: "is_mac_address" },
          "aa:bb:cc:dd:ee:ff",
          ctx,
        ),
      ).toBe(true);
      expect(
        evaluate({ type: "call", name: "is_mac_address" }, "not-a-mac", ctx),
      ).toBe(false);
    });
  });
});
