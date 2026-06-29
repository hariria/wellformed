import { describe, expectTypeOf, it } from "vitest";
import { optional, w } from "./builder/index.js";
import type { Infer } from "./infer.js";

describe("type inference", () => {
  describe("primitive types", () => {
    it("infers string", () => {
      const schema = w.string();
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<string>();
    });

    it("infers number", () => {
      const schema = w.number();
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<number>();
    });

    it("infers integer as number", () => {
      const schema = w.integer();
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<number>();
    });

    it("infers boolean", () => {
      const schema = w.boolean();
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<boolean>();
    });

    it("infers money as number", () => {
      const schema = w.money();
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<number>();
    });

    it("infers date as string", () => {
      const schema = w.date();
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<string>();
    });
  });

  describe("enum types", () => {
    it("infers enum as union of literals", () => {
      const schema = w.enum(["active", "pending", "closed"] as const);
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<"active" | "pending" | "closed">();
    });
  });

  describe("literal types", () => {
    it("infers literal value", () => {
      const schema = w.literal("active");
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<"active">();
    });
  });

  describe("nullable/nullish wrappers", () => {
    it("infers nullable builder", () => {
      const schema = w.string().nullable();
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<string | null>();
    });

    it("infers nullish helper wrapper", () => {
      const schema = w.nullish(w.number());
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<number | null>();
    });
  });

  describe("never type", () => {
    it("infers never", () => {
      const schema = w.never();
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<never>();
    });
  });

  describe("array types", () => {
    it("infers array of strings", () => {
      const schema = w.array(w.string());
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<string[]>();
    });

    it("infers array of numbers", () => {
      const schema = w.array(w.number());
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<number[]>();
    });
  });

  describe("object types", () => {
    it("infers simple object", () => {
      const schema = w.object({
        name: w.string(),
        age: w.number(),
      });
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<{ name: string; age: number }>();
    });

    it("infers object with optional fields using optional()", () => {
      const schema = w.object({
        name: w.string(),
        bio: optional(w.string()),
      });
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<{ name: string; bio?: string }>();
    });

    it("infers object with optional fields using .optional()", () => {
      const schema = w.object({
        name: w.string(),
        bio: w.string().optional(),
      });
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<{ name: string; bio?: string }>();
    });

    it("infers object with nullish field", () => {
      const schema = w.object({
        name: w.string(),
        bio: w.string().nullish(),
      });
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<{
        name: string;
        bio?: string | null;
      }>();
    });

    it("infers nested objects", () => {
      const schema = w.object({
        user: w.object({
          name: w.string(),
          email: w.string(),
        }),
      });
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<{
        user: { name: string; email: string };
      }>();
    });

    it("infers object with array field", () => {
      const schema = w.object({
        tags: w.array(w.string()),
      });
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<{ tags: string[] }>();
    });

    it("infers object with enum field", () => {
      const schema = w.object({
        status: w.enum(["active", "inactive"] as const),
      });
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<{ status: "active" | "inactive" }>();
    });
  });

  describe("union types", () => {
    it("infers union of primitives", () => {
      const schema = w.union([w.string(), w.number()] as const);
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<string | number>();
    });
  });

  describe("tuple types", () => {
    it("infers tuple items in order", () => {
      const schema = w.tuple([w.string(), w.number(), w.boolean()] as const);
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<[string, number, boolean]>();
    });
  });

  describe("record types", () => {
    it("infers record value type", () => {
      const schema = w.record(w.integer());
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<Record<string, number>>();
    });
  });

  describe("intersection types", () => {
    it("infers intersection of object variants", () => {
      const schema = w.intersection([
        w.object({ id: w.string() }),
        w.object({ name: w.string() }),
      ] as const);
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<{ id: string } & { name: string }>();
    });
  });

  describe("preprocess/catch wrappers", () => {
    it("infers preprocess inner type", () => {
      const schema = w.preprocess({ fn: "trim" }, w.string());
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<string>();
    });

    it("infers catch inner type", () => {
      const schema = w.catch(w.integer(), 0);
      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<number>();
    });
  });

  describe("complex schemas", () => {
    it("infers W-2 payee schema", () => {
      const addressSchema = w.object({
        street: w.string(),
        city: w.string(),
        state: w.string(),
        zip: w.string(),
      });

      const payeeSchema = w.object({
        tin: w.string(),
        name: w.string(),
        address: addressSchema,
      });

      type Result = Infer<typeof payeeSchema>;
      expectTypeOf<Result>().toEqualTypeOf<{
        tin: string;
        name: string;
        address: {
          street: string;
          city: string;
          state: string;
          zip: string;
        };
      }>();
    });

    it("infers schema with mixed optional/required", () => {
      const schema = w.object({
        id: w.string(),
        name: w.string(),
        email: optional(w.string()),
        phone: optional(w.string()),
        age: w.integer(),
      });

      type Result = Infer<typeof schema>;
      expectTypeOf<Result>().toEqualTypeOf<{
        id: string;
        name: string;
        email?: string;
        phone?: string;
        age: number;
      }>();
    });
  });
});
