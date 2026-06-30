import { describe, expect, it } from "vitest";
import { validate } from "../runtime/validate.js";
import { w } from "./w.js";

describe("cross-field validation", () => {
  describe("requireFieldsMatch", () => {
    it("validates equal fields", () => {
      const schema = w
        .object({
          password: w.string(),
          confirmPassword: w.string(),
        })
        .requireFieldsMatch("password", "confirmPassword");

      // Matching passwords - valid
      expect(
        validate(schema.toTypeSchema(), {
          password: "secret",
          confirmPassword: "secret",
        }).valid,
      ).toBe(true);

      // Mismatched passwords - invalid
      const result = validate(schema.toTypeSchema(), {
        password: "secret",
        confirmPassword: "different",
      });
      expect(result.valid).toBe(false);
      expect(result.errors[0]?.code).toBe("FIELDS_MISMATCH");
    });

    it("validates nested field equality", () => {
      const schema = w
        .object({
          scheduleB: w.object({ total: w.number() }),
          form941: w.object({ liability: w.number() }),
        })
        .requireFieldsMatch("scheduleB/total", "form941/liability");

      // Matching totals
      expect(
        validate(schema.toTypeSchema(), {
          scheduleB: { total: 5000 },
          form941: { liability: 5000 },
        }).valid,
      ).toBe(true);

      // Mismatched totals
      const result = validate(schema.toTypeSchema(), {
        scheduleB: { total: 5000 },
        form941: { liability: 4500 },
      });
      expect(result.valid).toBe(false);
    });
  });

  describe("requireSum", () => {
    it("validates sum equals target field", () => {
      const schema = w
        .object({
          line1: w.number(),
          line2: w.number(),
          line3: w.number(),
          total: w.number(),
        })
        .requireSum(["line1", "line2", "line3"], "total");

      // Correct sum
      expect(
        validate(schema.toTypeSchema(), {
          line1: 100,
          line2: 200,
          line3: 300,
          total: 600,
        }).valid,
      ).toBe(true);

      // Incorrect sum
      const result = validate(schema.toTypeSchema(), {
        line1: 100,
        line2: 200,
        line3: 300,
        total: 500,
      });
      expect(result.valid).toBe(false);
      expect(result.errors[0]?.code).toBe("SUM_MISMATCH");
    });

    it("handles decimal values", () => {
      const schema = w
        .object({
          amount1: w.number(),
          amount2: w.number(),
          total: w.number(),
        })
        .requireSum(["amount1", "amount2"], "total");

      expect(
        validate(schema.toTypeSchema(), {
          amount1: 10.5,
          amount2: 20.25,
          total: 30.75,
        }).valid,
      ).toBe(true);
    });
  });

  describe("requireSumEquals", () => {
    it("validates sum equals specific value", () => {
      const schema = w
        .object({
          percent1: w.number(),
          percent2: w.number(),
          percent3: w.number(),
        })
        .requireSumEquals(["percent1", "percent2", "percent3"], 100);

      // Correct sum
      expect(
        validate(schema.toTypeSchema(), {
          percent1: 50,
          percent2: 30,
          percent3: 20,
        }).valid,
      ).toBe(true);

      // Incorrect sum
      const result = validate(schema.toTypeSchema(), {
        percent1: 50,
        percent2: 30,
        percent3: 25,
      });
      expect(result.valid).toBe(false);
    });
  });

  describe("field comparisons", () => {
    it("requireFieldGreaterThan", () => {
      const schema = w
        .object({
          endDate: w.number(),
          startDate: w.number(),
        })
        .requireFieldGreaterThan("endDate", "startDate");

      expect(
        validate(schema.toTypeSchema(), { startDate: 100, endDate: 200 }).valid,
      ).toBe(true);
      expect(
        validate(schema.toTypeSchema(), { startDate: 200, endDate: 100 }).valid,
      ).toBe(false);
      expect(
        validate(schema.toTypeSchema(), { startDate: 100, endDate: 100 }).valid,
      ).toBe(false);
    });

    it("requireFieldGreaterOrEqual", () => {
      const schema = w
        .object({
          max: w.number(),
          min: w.number(),
        })
        .requireFieldGreaterOrEqual("max", "min");

      expect(
        validate(schema.toTypeSchema(), { min: 100, max: 200 }).valid,
      ).toBe(true);
      expect(
        validate(schema.toTypeSchema(), { min: 100, max: 100 }).valid,
      ).toBe(true);
      expect(
        validate(schema.toTypeSchema(), { min: 200, max: 100 }).valid,
      ).toBe(false);
    });

    it("requireFieldLessThan", () => {
      const schema = w
        .object({
          startDate: w.number(),
          endDate: w.number(),
        })
        .requireFieldLessThan("startDate", "endDate");

      expect(
        validate(schema.toTypeSchema(), { startDate: 100, endDate: 200 }).valid,
      ).toBe(true);
      expect(
        validate(schema.toTypeSchema(), { startDate: 200, endDate: 100 }).valid,
      ).toBe(false);
    });

    it("requireFieldLessOrEqual", () => {
      const schema = w
        .object({
          min: w.number(),
          max: w.number(),
        })
        .requireFieldLessOrEqual("min", "max");

      expect(
        validate(schema.toTypeSchema(), { min: 100, max: 200 }).valid,
      ).toBe(true);
      expect(
        validate(schema.toTypeSchema(), { min: 100, max: 100 }).valid,
      ).toBe(true);
      expect(
        validate(schema.toTypeSchema(), { min: 200, max: 100 }).valid,
      ).toBe(false);
    });
  });

  describe("IRS form examples", () => {
    it("Form 1095-C: code controls required fields", () => {
      const schema = w
        .object({
          line14: w.string(),
          line15: w.string().optional(),
          line16: w.string().optional(),
        })
        // If NOT code 1G, require lines 15 and 16
        .when("line14")
        .notEquals("1G")
        .require("line15")
        .when("line14")
        .notEquals("1G")
        .require("line16");

      // Code 1G - lines 15/16 optional
      expect(validate(schema.toTypeSchema(), { line14: "1G" }).valid).toBe(
        true,
      );

      // Code 1A - lines 15/16 required
      expect(validate(schema.toTypeSchema(), { line14: "1A" }).valid).toBe(
        false,
      );
      expect(
        validate(schema.toTypeSchema(), {
          line14: "1A",
          line15: "1",
          line16: "A",
        }).valid,
      ).toBe(true);
    });

    it("Form 1099-B: checkbox toggles optional fields", () => {
      const schema = w
        .object({
          box5: w.boolean(), // noncovered security
          box1b: w.string().optional(),
          box1e: w.string().optional(),
        })
        // If box5 is false (covered security), require boxes
        .when("box5")
        .equals(false)
        .require("box1b")
        .when("box5")
        .equals(false)
        .require("box1e");

      // Noncovered security - boxes optional
      expect(validate(schema.toTypeSchema(), { box5: true }).valid).toBe(true);

      // Covered security - boxes required
      expect(validate(schema.toTypeSchema(), { box5: false }).valid).toBe(
        false,
      );
      expect(
        validate(schema.toTypeSchema(), {
          box5: false,
          box1b: "2024-01-15",
          box1e: "1000",
        }).valid,
      ).toBe(true);
    });

    it("Form 941/Schedule B: totals must match", () => {
      const schema = w
        .object({
          scheduleB: w.object({
            month1: w.number(),
            month2: w.number(),
            month3: w.number(),
            totalLiability: w.number(),
          }),
          form941: w.object({
            totalTaxLiability: w.number(),
          }),
        })
        // Schedule B months must sum to its total
        .requireSum(
          ["scheduleB/month1", "scheduleB/month2", "scheduleB/month3"],
          "scheduleB/totalLiability",
        )
        // Schedule B total must match Form 941 total
        .requireFieldsMatch(
          "scheduleB/totalLiability",
          "form941/totalTaxLiability",
        );

      // Valid - sums match
      const valid = validate(schema.toTypeSchema(), {
        scheduleB: {
          month1: 1000,
          month2: 1500,
          month3: 2000,
          totalLiability: 4500,
        },
        form941: { totalTaxLiability: 4500 },
      });
      expect(valid.valid).toBe(true);

      // Invalid - Schedule B months don't sum to total
      const invalid1 = validate(schema.toTypeSchema(), {
        scheduleB: {
          month1: 1000,
          month2: 1500,
          month3: 2000,
          totalLiability: 5000,
        },
        form941: { totalTaxLiability: 5000 },
      });
      expect(invalid1.valid).toBe(false);
      expect(invalid1.errors[0]?.code).toBe("SUM_MISMATCH");

      // Invalid - Schedule B total doesn't match Form 941
      const invalid2 = validate(schema.toTypeSchema(), {
        scheduleB: {
          month1: 1000,
          month2: 1500,
          month3: 2000,
          totalLiability: 4500,
        },
        form941: { totalTaxLiability: 4000 },
      });
      expect(invalid2.valid).toBe(false);
      expect(invalid2.errors[0]?.code).toBe("FIELDS_MISMATCH");
    });

    it("Form 940: credit reduction state requires schedule", () => {
      const schema = w
        .object({
          creditReductionState: w.boolean(),
          scheduleA: w
            .object({
              state: w.string(),
              extraTax: w.number(),
            })
            .optional(),
        })
        .when("creditReductionState")
        .equals(true)
        .require("scheduleA");

      // No credit reduction - schedule optional
      expect(
        validate(schema.toTypeSchema(), { creditReductionState: false }).valid,
      ).toBe(true);

      // Credit reduction - schedule required
      expect(
        validate(schema.toTypeSchema(), { creditReductionState: true }).valid,
      ).toBe(false);
      expect(
        validate(schema.toTypeSchema(), {
          creditReductionState: true,
          scheduleA: { state: "CA", extraTax: 500 },
        }).valid,
      ).toBe(true);
    });
  });
});
