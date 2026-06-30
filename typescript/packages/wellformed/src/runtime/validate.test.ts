import { describe, expect, it } from "vitest";
import { optional, w } from "../builder/index.js";
import type { Schema, TypeSchema } from "../ir/types.js";
import { ValidationError, validate, validateOrThrow } from "./validate.js";

describe("validate", () => {
  describe("string validation", () => {
    it("validates valid string", () => {
      const schema = w.string().minLen(1).maxLen(10);
      const result = validate(schema.toTypeSchema(), "hello");

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
      expect(result.value).toBe("hello");
    });

    it("applies transforms", () => {
      const schema = w.string().trim().upper();
      const result = validate(schema.toTypeSchema(), "  hello  ");

      expect(result.valid).toBe(true);
      expect(result.value).toBe("HELLO");
    });

    it("collects constraint errors", () => {
      const schema = w.string().minLen(5);
      const result = validate(schema.toTypeSchema(), "hi");

      expect(result.valid).toBe(false);
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0]?.code).toBe("TOO_SHORT");
      expect(result.errors[0]?.message).toContain("at least 5");
    });

    it("validates email", () => {
      const schema = w.string().email();

      expect(validate(schema.toTypeSchema(), "test@example.com").valid).toBe(
        true,
      );
      expect(validate(schema.toTypeSchema(), "not-an-email").valid).toBe(false);
    });

    it("keeps repeated default-context regex validation stable", () => {
      const schema = w.string().regex("^a$", { flags: "g" }).toTypeSchema();

      expect(validate(schema, "a").valid).toBe(true);
      expect(validate(schema, "a").valid).toBe(true);
    });

    it("validates TIN", () => {
      const schema = w.string().trim().digitsOnly().ssn();

      const valid = validate(schema.toTypeSchema(), "123-45-6789");
      expect(valid.valid).toBe(true);
      expect(valid.value).toBe("123456789");

      const invalid = validate(schema.toTypeSchema(), "000-00-0000");
      expect(invalid.valid).toBe(false);
      expect(invalid.errors[0]?.code).toBe("INVALID_SSN");
    });

    it("includes custom error message", () => {
      const schema = w.string().minLen(5, { message: "Name too short!" });
      const result = validate(schema.toTypeSchema(), "hi");

      expect(result.errors[0]?.message).toBe("Name too short!");
    });
  });

  describe("number validation", () => {
    it("validates valid number", () => {
      const schema = w.number().min(0).max(100);
      const result = validate(schema.toTypeSchema(), 50);

      expect(result.valid).toBe(true);
    });

    it("validates range", () => {
      const schema = w.number().min(0).max(100);

      expect(validate(schema.toTypeSchema(), -1).valid).toBe(false);
      expect(validate(schema.toTypeSchema(), 101).valid).toBe(false);
    });

    it("validates nonNegative", () => {
      const schema = w.number().nonNegative();

      expect(validate(schema.toTypeSchema(), 0).valid).toBe(true);
      expect(validate(schema.toTypeSchema(), -1).valid).toBe(false);
    });

    it("reports type errors", () => {
      const schema = w.number();
      const result = validate(schema.toTypeSchema(), "not a number");

      expect(result.valid).toBe(false);
      expect(result.errors[0]?.code).toBe("TYPE_ERROR");
    });
  });

  describe("integer validation", () => {
    it("validates integers", () => {
      const schema = w.integer();

      expect(validate(schema.toTypeSchema(), 42).valid).toBe(true);
      expect(validate(schema.toTypeSchema(), 3.14).valid).toBe(false);
    });

    it("validates tax year", () => {
      const schema = w.integer().taxYear({ min: 2020, max: 2030 });

      expect(validate(schema.toTypeSchema(), 2024).valid).toBe(true);
      expect(validate(schema.toTypeSchema(), 2019).valid).toBe(false);
    });
  });

  describe("object validation", () => {
    it("validates object properties", () => {
      const schema = w.object({
        name: w.string().minLen(1),
        age: w.integer().min(0),
      });

      const result = validate(schema.toTypeSchema(), {
        name: "Alice",
        age: 30,
      });
      expect(result.valid).toBe(true);
    });

    it("validates object fields alias from legacy Rust IR", () => {
      const schema = {
        type: "object",
        fields: {
          name: { type: "string" },
        },
      } as const;

      expect(validate(schema, { name: "Alice" }).valid).toBe(true);

      const missing = validate(schema, {});
      expect(missing.valid).toBe(false);
      expect(missing.errors).toMatchObject([
        { code: "REQUIRED", path: "/name" },
      ]);
    });

    it("reports required field errors", () => {
      const schema = w.object({
        name: w.string(),
        email: w.string().email(),
      });

      const result = validate(schema.toTypeSchema(), { name: "Alice" });
      expect(result.valid).toBe(false);
      expect(result.errors[0]?.code).toBe("REQUIRED");
      expect(result.errors[0]?.path).toBe("/email");
    });

    it("allows optional fields with optional()", () => {
      const schema = w.object({
        name: w.string(),
        nickname: optional(w.string()),
      });

      const result = validate(schema.toTypeSchema(), { name: "Alice" });
      expect(result.valid).toBe(true);
    });

    it("allows optional fields with .optional()", () => {
      const schema = w.object({
        name: w.string(),
        nickname: w.string().optional(),
      });

      const result = validate(schema.toTypeSchema(), { name: "Alice" });
      expect(result.valid).toBe(true);

      // With the optional field provided
      const result2 = validate(schema.toTypeSchema(), {
        name: "Alice",
        nickname: "Ali",
      });
      expect(result2.valid).toBe(true);
    });

    it("validates nested objects", () => {
      const schema = w.object({
        user: w.object({
          name: w.string().minLen(1),
        }),
      });

      const result = validate(schema.toTypeSchema(), { user: { name: "" } });
      expect(result.valid).toBe(false);
      expect(result.errors[0]?.path).toBe("/user/name");
    });

    it("escapes object property names in error paths", () => {
      const schema = {
        type: "object",
        properties: {
          "a/b~c": { type: "string" },
        },
      } as const;

      const result = validate(schema, {});
      expect(result.valid).toBe(false);
      expect(result.errors).toMatchObject([
        { code: "REQUIRED", path: "/a~1b~0c" },
      ]);
    });

    it("preserves empty object keys in nested error paths", () => {
      const schema: TypeSchema = {
        type: "object",
        properties: {
          "": {
            type: "tuple",
            items: [{ type: "number" }, { type: "number" }],
          },
        },
      };

      const result = validate(schema, { "": [false, "0"] });

      expect(result.valid).toBe(false);
      expect(result.errors).toMatchObject([
        { code: "TYPE_ERROR", path: "//0" },
        { code: "TYPE_ERROR", path: "//1" },
      ]);
    });

    it("evaluates cross-field rules", () => {
      const schema = w
        .object({
          password: optional(w.string()),
          confirmPassword: optional(w.string()),
        })
        .requireWith("password", "confirmPassword");

      // When password is provided, confirmPassword should be required
      const result = validate(schema.toTypeSchema(), { password: "secret" });
      expect(result.valid).toBe(false);
      expect(result.errors[0]?.code).toBe("MISSING_REQUIRED_FIELD");

      // When both provided, should pass
      const valid = validate(schema.toTypeSchema(), {
        password: "secret",
        confirmPassword: "secret",
      });
      expect(valid.valid).toBe(true);

      // When neither provided, should pass
      const empty = validate(schema.toTypeSchema(), {});
      expect(empty.valid).toBe(true);
    });

    it("evaluates requireWithout rules", () => {
      const schema = w
        .object({
          taxId: optional(w.string()),
          ssn: optional(w.string()),
        })
        .requireWithout("taxId", "ssn");

      expect(validate(schema.toTypeSchema(), {}).valid).toBe(false);
      expect(
        validate(schema.toTypeSchema(), { ssn: "123-45-6789" }).valid,
      ).toBe(true);
      expect(
        validate(schema.toTypeSchema(), { taxId: "12-3456789" }).valid,
      ).toBe(true);
    });

    it("evaluates requireExactlyOneOf rules", () => {
      const schema = w
        .object({
          ssn: optional(w.string()),
          ein: optional(w.string()),
        })
        .requireExactlyOneOf(["ssn", "ein"]);

      expect(
        validate(schema.toTypeSchema(), { ssn: "123-45-6789" }).valid,
      ).toBe(true);
      expect(validate(schema.toTypeSchema(), { ein: "12-3456789" }).valid).toBe(
        true,
      );
      expect(
        validate(schema.toTypeSchema(), {
          ssn: "123-45-6789",
          ein: "12-3456789",
        }).valid,
      ).toBe(false);
      expect(validate(schema.toTypeSchema(), {}).valid).toBe(false);
    });

    it("applies transforms to nested values", () => {
      const schema = w.object({
        email: w.string().trim().lower(),
      });

      const result = validate(schema.toTypeSchema(), {
        email: "  TEST@EXAMPLE.COM  ",
      });
      expect(result.valid).toBe(true);
      expect((result.value as { email: string }).email).toBe(
        "test@example.com",
      );
    });

    it("rejects unknown keys in strict mode", () => {
      const schema = w.object({ name: w.string() }).strict();
      const result = validate(schema.toTypeSchema(), {
        name: "Alice",
        extra: "x",
      });
      expect(result.valid).toBe(false);
      expect(result.errors[0]?.code).toBe("ADDITIONAL_PROPERTY_NOT_ALLOWED");
      expect((result.value as { extra: string }).extra).toBe("x");
    });

    it("keeps unknown keys in passthrough mode", () => {
      const schema = w.object({ name: w.string() }).passthrough();
      const result = validate(schema.toTypeSchema(), {
        name: "Alice",
        extra: "x",
      });
      expect(result.valid).toBe(true);
      expect((result.value as { extra: string }).extra).toBe("x");
    });

    it("strips unknown keys in strip mode", () => {
      const schema = w.object({ name: w.string() }).strip();
      const result = validate(schema.toTypeSchema(), {
        name: "Alice",
        extra: "x",
      });
      expect(result.valid).toBe(true);
      expect("extra" in (result.value as Record<string, unknown>)).toBe(false);
    });

    it("validates unknown keys with catchall", () => {
      const schema = w.object({ name: w.string() }).catchall(w.integer());
      const valid = validate(schema.toTypeSchema(), {
        name: "Alice",
        age: 30,
      });
      expect(valid.valid).toBe(true);

      const invalid = validate(schema.toTypeSchema(), {
        name: "Alice",
        age: "thirty",
      });
      expect(invalid.valid).toBe(false);
      expect(invalid.errors[0]?.path).toBe("/age");
    });
  });

  describe("array validation", () => {
    it("validates array items", () => {
      const schema = w.array(w.string().minLen(1));

      const valid = validate(schema.toTypeSchema(), ["a", "b", "c"]);
      expect(valid.valid).toBe(true);

      const invalid = validate(schema.toTypeSchema(), ["a", "", "c"]);
      expect(invalid.valid).toBe(false);
      expect(invalid.errors[0]?.path).toBe("/1");
    });

    it("validates array length", () => {
      const schema = w.array(w.string()).minItems(1).maxItems(3);

      const tooShort = validate(schema.toTypeSchema(), []);
      expect(tooShort.valid).toBe(false);
      expect(tooShort.errors).toHaveLength(1);
      expect(tooShort.errors[0]?.code).toBe("TOO_FEW_ITEMS");

      const tooLong = validate(schema.toTypeSchema(), ["a", "b", "c", "d"]);
      expect(tooLong.valid).toBe(false);
      expect(tooLong.errors).toHaveLength(1);
      expect(tooLong.errors[0]?.code).toBe("TOO_MANY_ITEMS");

      expect(validate(schema.toTypeSchema(), ["a", "b"]).valid).toBe(true);
    });

    it("ignores array-level transforms to match the Rust IR", () => {
      const schema = {
        type: "array",
        items: { type: "string" },
        transforms: [{ fn: "default", value: [] }],
      } as unknown as TypeSchema;

      const result = validate(schema, undefined);
      expect(result.valid).toBe(true);
      expect(result.value).toBeUndefined();
    });

    it("does not treat array-level default transforms as required-field defaults", () => {
      const schema = {
        type: "object",
        properties: {
          items: {
            type: "array",
            items: { type: "string" },
            transforms: [{ fn: "default", value: [] }],
          },
        },
      } as unknown as TypeSchema;

      const result = validate(schema, {});
      expect(result.valid).toBe(false);
      expect(result.errors).toMatchObject([
        { code: "REQUIRED", path: "/items" },
      ]);
    });

    it("validates first-class array min_items and max_items without constraints", () => {
      const schema = {
        type: "array",
        items: { type: "string" },
        min_items: 1,
        max_items: 2,
      } as const;

      expect(validate(schema, []).errors).toMatchObject([
        { code: "ARRAY_TOO_SHORT", path: "" },
      ]);
      expect(validate(schema, ["a", "b", "c"]).errors).toMatchObject([
        { code: "ARRAY_TOO_LONG", path: "" },
      ]);
      expect(validate(schema, ["a"]).valid).toBe(true);
    });

    it("evaluates array constraints after item transforms", () => {
      const schema: TypeSchema = {
        type: "array",
        items: { type: "string", transforms: [{ fn: "trim" }] },
        constraints: [
          {
            pred: { type: "eq", path: "/0", value: "x" },
            error: {
              code: "FIRST_ITEM",
              message: "first item must be transformed",
            },
          },
        ],
      };

      const result = validate(schema, [" x "]);

      expect(result.valid).toBe(true);
      expect(result.value).toEqual(["x"]);
    });

    it("validates nonEmpty", () => {
      const schema = w.array(w.string()).nonEmpty();

      expect(validate(schema.toTypeSchema(), []).valid).toBe(false);
      expect(validate(schema.toTypeSchema(), ["a"]).valid).toBe(true);
    });
  });

  describe("enum validation", () => {
    it("validates enum values", () => {
      const schema = w.enum(["active", "pending", "closed"]);

      expect(validate(schema.toTypeSchema(), "active").valid).toBe(true);
      expect(validate(schema.toTypeSchema(), "invalid").valid).toBe(false);
    });

    it("reports invalid enum error", () => {
      const schema = w.enum(["A", "B", "C"]);
      const result = validate(schema.toTypeSchema(), "D");

      expect(result.errors[0]?.code).toBe("INVALID_ENUM");
      expect(result.errors[0]?.message).toContain("A, B, C");
    });

    it("compares enum JSON values structurally", () => {
      const schema: TypeSchema = {
        type: "enum",
        values: [{ kind: "business", code: 1 }],
      };

      expect(validate(schema, { kind: "business", code: 1 }).valid).toBe(true);
      expect(validate(schema, { kind: "business", code: 2 }).valid).toBe(false);
    });

    it("skips enum validation for empty string form values", () => {
      const schema = w.enum(["A", "B", "C"]);

      expect(validate(schema.toTypeSchema(), "").valid).toBe(true);
    });
  });

  describe("literal validation", () => {
    it("accepts matching literal", () => {
      const schema = w.literal("active");
      expect(validate(schema.toTypeSchema(), "active").valid).toBe(true);
    });

    it("rejects non-matching literal", () => {
      const schema = w.literal("active");
      const result = validate(schema.toTypeSchema(), "inactive");
      expect(result.valid).toBe(false);
      expect(result.errors[0]?.code).toBe("INVALID_LITERAL");
    });
  });

  describe("never validation", () => {
    it("always rejects", () => {
      const schema = w.never();
      const result = validate(schema.toTypeSchema(), "anything");
      expect(result.valid).toBe(false);
      expect(result.errors[0]?.code).toBe("TYPE_ERROR");
    });
  });

  describe("tuple validation", () => {
    it("validates tuple items and length", () => {
      const schema = w.tuple([w.string().minLen(1), w.number().min(0)]);

      expect(validate(schema.toTypeSchema(), ["ok", 1]).valid).toBe(true);
      expect(validate(schema.toTypeSchema(), ["ok"]).valid).toBe(false);
    });

    it("reports nested tuple item path", () => {
      const schema = w.tuple([w.string(), w.number()]);
      const result = validate(schema.toTypeSchema(), ["ok", "bad"]);
      expect(result.valid).toBe(false);
      expect(result.errors[0]?.path).toBe("/1");
    });
  });

  describe("union validation", () => {
    it("validates union variants", () => {
      const schema = w.union([w.string().minLen(1), w.number().min(0)]);

      expect(validate(schema.toTypeSchema(), "hello").valid).toBe(true);
      expect(validate(schema.toTypeSchema(), 42).valid).toBe(true);
    });

    it("fails when no variant matches", () => {
      const schema = w.union([w.string().minLen(5), w.number().min(100)]);

      const result = validate(schema.toTypeSchema(), "hi");
      expect(result.valid).toBe(false);
      expect(result.errors[0]?.code).toBe("INVALID_UNION");
    });

    it("allows null through unions during validation", () => {
      const schema = w.object({
        status: optional(w.union([w.literal("active")])),
      });

      const result = validate(schema.toTypeSchema(), { status: null });

      expect(result.valid).toBe(true);
      expect(result.value).toEqual({ status: null });
    });
  });

  describe("intersection validation", () => {
    it("requires all variants to pass", () => {
      const schema = w.intersection([
        w.string().minLen(2),
        w.string().maxLen(5),
      ]);

      expect(validate(schema.toTypeSchema(), "abcd").valid).toBe(true);
      expect(validate(schema.toTypeSchema(), "a").valid).toBe(false);
    });
  });

  describe("record validation", () => {
    it("validates record values", () => {
      const schema = w.record(w.integer());
      expect(validate(schema.toTypeSchema(), { a: 1, b: 2 }).valid).toBe(true);
      expect(validate(schema.toTypeSchema(), { a: 1, b: "x" }).valid).toBe(
        false,
      );
    });

    it("escapes record keys in error paths", () => {
      const schema = w.record(w.integer());
      const result = validate(schema.toTypeSchema(), { "a/b~c": "x" });

      expect(result.valid).toBe(false);
      expect(result.errors).toMatchObject([
        { code: "TYPE_ERROR", path: "/a~1b~0c" },
      ]);
    });

    it("preserves empty record keys in key validation paths", () => {
      const schema: TypeSchema = {
        type: "record",
        key: {
          type: "string",
          constraints: [
            {
              pred: { type: "min_len", len: 1 },
              error: { code: "KEY", message: "key required" },
            },
          ],
        },
        value: { type: "number" },
      };

      const result = validate(schema, { "": 1 });

      expect(result.valid).toBe(false);
      expect(result.errors).toMatchObject([{ code: "KEY", path: "//$key" }]);
    });
  });

  describe("preprocess/catch validation", () => {
    it("applies preprocess before validation", () => {
      const schema = w.preprocess({ fn: "trim" }, w.string().minLen(1));
      const result = validate(schema.toTypeSchema(), "  ok  ");
      expect(result.valid).toBe(true);
      expect(result.value).toBe("ok");
    });

    it("returns catch fallback on failure", () => {
      const schema = w.catch(w.integer(), 0);
      const result = validate(schema.toTypeSchema(), "nope");
      expect(result.valid).toBe(true);
      expect(result.value).toBe(0);
    });
  });

  describe("null/undefined handling", () => {
    it("allows null for optional fields", () => {
      const schema = w.object({
        name: w.string(),
        bio: optional(w.string()),
      });

      const result = validate(schema.toTypeSchema(), {
        name: "Alice",
        bio: null,
      });
      expect(result.valid).toBe(true);
    });

    it("skips scalar constraints for null optional fields", () => {
      const schema = w.object({
        name: w.string(),
        bio: optional(w.string().minLen(5)),
        score: optional(w.number().min(10)),
      });

      const result = validate(schema.toTypeSchema(), {
        name: "Alice",
        bio: null,
        score: null,
      });

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it("supports nullable wrapper", () => {
      const schema = w.string().nullable();
      expect(validate(schema.toTypeSchema(), "bio").valid).toBe(true);
      expect(validate(schema.toTypeSchema(), null).valid).toBe(true);
    });

    it("rejects null for required non-nullable object fields", () => {
      const schema = w.object({
        name: w.string(),
      });

      const result = validate(schema.toTypeSchema(), { name: null });
      expect(result.valid).toBe(false);
      expect(result.errors).toMatchObject([
        { code: "REQUIRED", path: "/name" },
      ]);
      expect(result.value).toEqual({ name: null });
    });

    it("allows null for required nullable object fields", () => {
      const schema = w.object({
        bio: w.string().nullable(),
      });

      const result = validate(schema.toTypeSchema(), { bio: null });
      expect(result.valid).toBe(true);
      expect(result.value).toEqual({ bio: null });
    });

    it("materializes default transforms for missing object fields", () => {
      const schema = w.object({
        name: w.string().default("Anonymous"),
      });

      const result = validate(schema.toTypeSchema(), {});
      expect(result.valid).toBe(true);
      expect(result.value).toEqual({ name: "Anonymous" });
    });

    it("supports nullish wrapper in object fields", () => {
      const schema = w.object({
        name: w.string(),
        bio: w.string().nullish(),
      });

      expect(validate(schema.toTypeSchema(), { name: "Alice" }).valid).toBe(
        true,
      );
      expect(
        validate(schema.toTypeSchema(), { name: "Alice", bio: null }).valid,
      ).toBe(true);
    });
  });

  describe("full schema validation", () => {
    it("validates complete schema with version", () => {
      const schema = w
        .object({
          name: w.string().trim().minLen(1),
          email: w.string().email(),
        })
        .toSchema("1.0");

      const result = validate(schema, {
        name: "Alice",
        email: "alice@example.com",
      });
      expect(result.valid).toBe(true);
    });
  });
});

describe("validateOrThrow", () => {
  it("returns value on success", () => {
    const schema = w.string().minLen(1);
    const result = validateOrThrow(schema.toTypeSchema(), "hello");
    expect(result).toBe("hello");
  });

  it("returns transformed value", () => {
    const schema = w.string().trim().upper();
    const result = validateOrThrow(schema.toTypeSchema(), "  hello  ");
    expect(result).toBe("HELLO");
  });

  it("throws ValidationError on failure", () => {
    const schema = w.string().minLen(10);

    expect(() => validateOrThrow(schema.toTypeSchema(), "hi")).toThrow(
      ValidationError,
    );
  });

  it("includes errors in exception", () => {
    const schema = w.string().minLen(10);

    try {
      validateOrThrow(schema.toTypeSchema(), "hi");
    } catch (e) {
      expect(e).toBeInstanceOf(ValidationError);
      expect((e as ValidationError).errors).toHaveLength(1);
    }
  });
});

describe("schema references", () => {
  it("resolves named definitions in a full schema", () => {
    const schema = {
      version: "1.0",
      definitions: {
        Name: {
          type: "string",
          constraints: [
            {
              pred: { type: "min_len", len: 1 },
              error: { code: "REQUIRED", message: "name is required" },
            },
          ],
        },
      },
      root: {
        type: "object",
        properties: {
          name: { type: "ref", $ref: "Name" },
        },
      },
    } satisfies Schema;

    expect(validate(schema, { name: "Alice" }).valid).toBe(true);

    const invalid = validate(schema, { name: "" });
    expect(invalid.valid).toBe(false);
    expect(invalid.errors).toMatchObject([{ code: "REQUIRED", path: "/name" }]);
  });

  it("reports unresolved references", () => {
    const schema = {
      version: "1.0",
      root: { type: "ref", $ref: "Missing" },
    } satisfies Schema;

    const result = validate(schema, "value");

    expect(result.valid).toBe(false);
    expect(result.errors).toMatchObject([{ code: "REF_NOT_FOUND", path: "" }]);
  });

  it("reports cyclic references", () => {
    const schema = {
      version: "1.0",
      definitions: {
        A: { type: "ref", $ref: "B" },
        B: { type: "ref", $ref: "A" },
      },
      root: { type: "ref", $ref: "A" },
    } satisfies Schema;

    const result = validate(schema, "value");

    expect(result.valid).toBe(false);
    expect(result.errors).toMatchObject([{ code: "REF_CYCLE", path: "" }]);
  });
});

describe("complex validation example", () => {
  it("validates W-2 payee", () => {
    const addressSchema = w.object({
      street: w.string().trim().minLen(1),
      city: w.string().trim().minLen(1),
      state: w.string().usState(),
      zip: w.string().usZip(),
    });

    const payeeSchema = w.object({
      tin: w.string().trim().digitsOnly().ssn(),
      name: w.string().trim().minLen(1).maxLen(100),
      address: addressSchema,
    });

    const validPayee = {
      tin: "123-45-6789",
      name: "  John Doe  ",
      address: {
        street: "123 Main St",
        city: "Springfield",
        state: "IL",
        zip: "62701",
      },
    };

    const result = validate(payeeSchema.toTypeSchema(), validPayee);
    expect(result.valid).toBe(true);

    // Check transforms were applied
    const value = result.value as { tin: string; name: string };
    expect(value.tin).toBe("123456789");
    expect(value.name).toBe("John Doe");
  });

  it("collects multiple errors", () => {
    const schema = w.object({
      name: w.string().minLen(1),
      email: w.string().email(),
      age: w.integer().min(0).max(150),
    });

    const result = validate(schema.toTypeSchema(), {
      name: "",
      email: "not-an-email",
      age: -5,
    });

    expect(result.valid).toBe(false);
    expect(result.errors.length).toBeGreaterThanOrEqual(3);

    const codes = result.errors.map((e) => e.code);
    expect(codes).toContain("TOO_SHORT");
    expect(codes).toContain("INVALID_EMAIL");
  });
});
