import { describe, expect, it } from "vitest";
import { validate } from "../runtime/validate.js";
import { optional, w } from "./w.js";

describe("fluent conditional validation", () => {
  describe("when().equals().require()", () => {
    it("requires field when condition is met", () => {
      const schema = w
        .object({
          type: w.enum(["individual", "business"]),
          ssn: optional(w.string()),
          ein: optional(w.string()),
        })
        .when("type")
        .equals("individual")
        .require("ssn", { message: "SSN required for individuals" });

      // Individual without SSN - should fail
      const invalid = validate(schema.toTypeSchema(), { type: "individual" });
      expect(invalid.valid).toBe(false);
      expect(invalid.errors[0]?.message).toBe("SSN required for individuals");

      // Individual with SSN - should pass
      const valid = validate(schema.toTypeSchema(), {
        type: "individual",
        ssn: "123-45-6789",
      });
      expect(valid.valid).toBe(true);

      // Business without SSN - should pass (condition not met)
      const business = validate(schema.toTypeSchema(), { type: "business" });
      expect(business.valid).toBe(true);
    });

    it("chains multiple conditions", () => {
      const schema = w
        .object({
          type: w.enum(["individual", "business"]),
          ssn: optional(w.string()),
          ein: optional(w.string()),
        })
        .when("type")
        .equals("individual")
        .require("ssn")
        .when("type")
        .equals("business")
        .require("ein");

      expect(
        validate(schema.toTypeSchema(), { type: "individual" }).valid,
      ).toBe(false);
      expect(
        validate(schema.toTypeSchema(), { type: "individual", ssn: "123" })
          .valid,
      ).toBe(true);
      expect(validate(schema.toTypeSchema(), { type: "business" }).valid).toBe(
        false,
      );
      expect(
        validate(schema.toTypeSchema(), { type: "business", ein: "12-3456789" })
          .valid,
      ).toBe(true);
    });
  });

  describe("when().equals().and().equals().require()", () => {
    it("requires field when multiple conditions are met", () => {
      const schema = w
        .object({
          country: w.string(),
          type: w.enum(["individual", "business"]),
          ssn: optional(w.string()),
        })
        .when("country")
        .equals("US")
        .and("type")
        .equals("individual")
        .require("ssn", { message: "SSN required for US individuals" });

      // US individual without SSN - should fail
      const invalid = validate(schema.toTypeSchema(), {
        country: "US",
        type: "individual",
      });
      expect(invalid.valid).toBe(false);
      expect(invalid.errors[0]?.message).toBe(
        "SSN required for US individuals",
      );

      // US business without SSN - should pass (second condition not met)
      const usBusiness = validate(schema.toTypeSchema(), {
        country: "US",
        type: "business",
      });
      expect(usBusiness.valid).toBe(true);

      // Canadian individual without SSN - should pass (first condition not met)
      const canadian = validate(schema.toTypeSchema(), {
        country: "CA",
        type: "individual",
      });
      expect(canadian.valid).toBe(true);

      // US individual with SSN - should pass
      const valid = validate(schema.toTypeSchema(), {
        country: "US",
        type: "individual",
        ssn: "123-45-6789",
      });
      expect(valid.valid).toBe(true);
    });
  });

  describe("when().equals().or().equals().require()", () => {
    it("requires field when any condition is met", () => {
      const schema = w
        .object({
          type: w.enum(["individual", "sole_proprietor", "business"]),
          ssn: optional(w.string()),
        })
        .when("type")
        .equals("individual")
        .or("type")
        .equals("sole_proprietor")
        .require("ssn", {
          message: "SSN required for individuals or sole proprietors",
        });

      // Individual without SSN - should fail
      expect(
        validate(schema.toTypeSchema(), { type: "individual" }).valid,
      ).toBe(false);

      // Sole proprietor without SSN - should fail
      expect(
        validate(schema.toTypeSchema(), { type: "sole_proprietor" }).valid,
      ).toBe(false);

      // Business without SSN - should pass
      expect(validate(schema.toTypeSchema(), { type: "business" }).valid).toBe(
        true,
      );

      // Individual with SSN - should pass
      expect(
        validate(schema.toTypeSchema(), { type: "individual", ssn: "123" })
          .valid,
      ).toBe(true);
    });
  });

  describe("when().in().require()", () => {
    it("requires field when value is in list", () => {
      const schema = w
        .object({
          status: w.enum(["active", "pending", "suspended", "closed"]),
          reason: optional(w.string()),
        })
        .when("status")
        .in(["suspended", "closed"])
        .require("reason", {
          message: "Reason required for suspended/closed accounts",
        });

      expect(validate(schema.toTypeSchema(), { status: "active" }).valid).toBe(
        true,
      );
      expect(
        validate(schema.toTypeSchema(), { status: "suspended" }).valid,
      ).toBe(false);
      expect(
        validate(schema.toTypeSchema(), {
          status: "suspended",
          reason: "Fraud",
        }).valid,
      ).toBe(true);
    });
  });

  describe("when().exists().require()", () => {
    it("requires field when another field exists", () => {
      const schema = w
        .object({
          startDate: optional(w.string()),
          endDate: optional(w.string()),
        })
        .when("startDate")
        .exists()
        .require("endDate");

      expect(validate(schema.toTypeSchema(), {}).valid).toBe(true);
      expect(
        validate(schema.toTypeSchema(), { startDate: "2024-01-01" }).valid,
      ).toBe(false);
      expect(
        validate(schema.toTypeSchema(), {
          startDate: "2024-01-01",
          endDate: "2024-12-31",
        }).valid,
      ).toBe(true);
    });
  });

  describe("when().equals().forbid()", () => {
    it("forbids field when condition is met", () => {
      const schema = w
        .object({
          type: w.enum(["personal", "business"]),
          ein: optional(w.string()),
        })
        .when("type")
        .equals("personal")
        .forbid("ein", { message: "EIN not allowed for personal accounts" });

      // Personal with EIN - should fail
      const invalid = validate(schema.toTypeSchema(), {
        type: "personal",
        ein: "12-3456789",
      });
      expect(invalid.valid).toBe(false);
      expect(invalid.errors[0]?.message).toBe(
        "EIN not allowed for personal accounts",
      );

      // Personal without EIN - should pass
      expect(validate(schema.toTypeSchema(), { type: "personal" }).valid).toBe(
        true,
      );

      // Business with EIN - should pass
      expect(
        validate(schema.toTypeSchema(), { type: "business", ein: "12-3456789" })
          .valid,
      ).toBe(true);
    });
  });

  describe("when().equals().requireEquals()", () => {
    it("requires field to have specific value", () => {
      const schema = w
        .object({
          premium: w.boolean(),
          tier: optional(w.string()),
        })
        .when("premium")
        .equals(true)
        .requireEquals("tier", "gold", {
          message: "Premium accounts must have gold tier",
        });

      expect(
        validate(schema.toTypeSchema(), { premium: true, tier: "gold" }).valid,
      ).toBe(true);
      expect(
        validate(schema.toTypeSchema(), { premium: true, tier: "silver" })
          .valid,
      ).toBe(false);
      expect(
        validate(schema.toTypeSchema(), { premium: false, tier: "silver" })
          .valid,
      ).toBe(true);
    });
  });

  describe("when().equals().requireIn()", () => {
    it("requires field to be one of values", () => {
      const schema = w
        .object({
          country: w.string(),
          state: optional(w.string()),
        })
        .when("country")
        .equals("US")
        .requireIn("state", ["CA", "NY", "TX"], {
          message: "Must select a supported US state",
        });

      expect(
        validate(schema.toTypeSchema(), { country: "US", state: "CA" }).valid,
      ).toBe(true);
      expect(
        validate(schema.toTypeSchema(), { country: "US", state: "FL" }).valid,
      ).toBe(false);
      expect(
        validate(schema.toTypeSchema(), { country: "CA", state: "ON" }).valid,
      ).toBe(true);
    });
  });

  describe("when().notEquals().require()", () => {
    it("requires field when value is not equal", () => {
      const schema = w
        .object({
          status: w.string(),
          details: optional(w.string()),
        })
        .when("status")
        .notEquals("ok")
        .require("details", {
          message: "Details required when status is not OK",
        });

      expect(validate(schema.toTypeSchema(), { status: "ok" }).valid).toBe(
        true,
      );
      expect(validate(schema.toTypeSchema(), { status: "error" }).valid).toBe(
        false,
      );
      expect(
        validate(schema.toTypeSchema(), {
          status: "error",
          details: "Something went wrong",
        }).valid,
      ).toBe(true);
    });
  });

  describe("complex conditions", () => {
    it("handles real-world tax form scenario", () => {
      const schema = w
        .object({
          formType: w.enum(["W2", "1099-NEC", "1099-INT"]),
          employerEin: optional(w.string()),
          payerTin: optional(w.string()),
          wages: optional(w.number()),
          nonEmployeeComp: optional(w.number()),
          interestIncome: optional(w.number()),
        })
        // W2 requires employerEin and wages
        .when("formType")
        .equals("W2")
        .require("employerEin")
        .when("formType")
        .equals("W2")
        .require("wages")
        // 1099-NEC requires payerTin and nonEmployeeComp
        .when("formType")
        .equals("1099-NEC")
        .require("payerTin")
        .when("formType")
        .equals("1099-NEC")
        .require("nonEmployeeComp")
        // 1099-INT requires payerTin and interestIncome
        .when("formType")
        .equals("1099-INT")
        .require("payerTin")
        .when("formType")
        .equals("1099-INT")
        .require("interestIncome");

      // Valid W2
      expect(
        validate(schema.toTypeSchema(), {
          formType: "W2",
          employerEin: "12-3456789",
          wages: 50000,
        }).valid,
      ).toBe(true);

      // Invalid W2 (missing wages)
      expect(
        validate(schema.toTypeSchema(), {
          formType: "W2",
          employerEin: "12-3456789",
        }).valid,
      ).toBe(false);

      // Valid 1099-NEC
      expect(
        validate(schema.toTypeSchema(), {
          formType: "1099-NEC",
          payerTin: "123-45-6789",
          nonEmployeeComp: 5000,
        }).valid,
      ).toBe(true);

      // Invalid 1099-INT (missing interestIncome)
      expect(
        validate(schema.toTypeSchema(), {
          formType: "1099-INT",
          payerTin: "123-45-6789",
        }).valid,
      ).toBe(false);
    });
  });
});
