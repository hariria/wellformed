import { describe, expect, expectTypeOf, it } from "vitest";
import type { Infer } from "../infer.js";
import { validate } from "../runtime/validate.js";
import { optional, w } from "./w.js";

describe("schema composition", () => {
  describe("extend", () => {
    it("adds properties to an existing schema", () => {
      const base = w.object({ id: w.string() });
      const extended = base.extend({ name: w.string(), age: w.integer() });

      const schema = extended.toTypeSchema();
      expect(schema.properties?.id).toBeDefined();
      expect(schema.properties?.name).toBeDefined();
      expect(schema.properties?.age).toBeDefined();
    });

    it("validates extended schema", () => {
      const base = w.object({ id: w.string() });
      const extended = base.extend({ name: w.string() });

      expect(
        validate(extended.toTypeSchema(), { id: "1", name: "Alice" }).valid,
      ).toBe(true);
      expect(validate(extended.toTypeSchema(), { id: "1" }).valid).toBe(false);
    });

    it("preserves original rules", () => {
      const base = w
        .object({ password: w.string(), confirm: optional(w.string()) })
        .requireWith("password", "confirm");
      const extended = base.extend({ name: w.string() });

      expect(
        validate(extended.toTypeSchema(), { password: "secret", name: "Alice" })
          .valid,
      ).toBe(false);
      expect(
        validate(extended.toTypeSchema(), {
          password: "secret",
          confirm: "secret",
          name: "Alice",
        }).valid,
      ).toBe(true);
    });

    it("preserves type inference", () => {
      const base = w.object({ id: w.string() });
      const extended = base.extend({ name: w.string(), age: w.integer() });

      type Result = Infer<typeof extended>;
      expectTypeOf<Result>().toEqualTypeOf<{
        id: string;
        name: string;
        age: number;
      }>();
    });
  });

  describe("merge", () => {
    it("combines two schemas", () => {
      const a = w.object({ name: w.string() });
      const b = w.object({ age: w.integer() });
      const merged = a.merge(b);

      const schema = merged.toTypeSchema();
      expect(schema.properties?.name).toBeDefined();
      expect(schema.properties?.age).toBeDefined();
    });

    it("validates merged schema", () => {
      const a = w.object({ name: w.string() });
      const b = w.object({ age: w.integer() });
      const merged = a.merge(b);

      expect(
        validate(merged.toTypeSchema(), { name: "Alice", age: 30 }).valid,
      ).toBe(true);
      expect(validate(merged.toTypeSchema(), { name: "Alice" }).valid).toBe(
        false,
      );
    });

    it("combines rules from both schemas", () => {
      const a = w
        .object({ name: w.string(), nickname: optional(w.string()) })
        .requireWith("name", "nickname");
      const b = w.object({ age: w.integer() });
      const merged = a.merge(b);

      // Rule from a should still apply
      expect(
        validate(merged.toTypeSchema(), { name: "Alice", age: 30 }).valid,
      ).toBe(false);
    });

    it("preserves type inference", () => {
      const a = w.object({ name: w.string() });
      const b = w.object({ age: w.integer() });
      const merged = a.merge(b);

      type Result = Infer<typeof merged>;
      expectTypeOf<Result>().toEqualTypeOf<{ name: string; age: number }>();
    });
  });

  describe("pick", () => {
    it("selects specified keys", () => {
      const user = w.object({
        id: w.string(),
        name: w.string(),
        email: w.string(),
        password: w.string(),
      });
      const picked = user.pick("id", "name");

      const schema = picked.toTypeSchema();
      expect(schema.properties?.id).toBeDefined();
      expect(schema.properties?.name).toBeDefined();
      expect(schema.properties?.email).toBeUndefined();
      expect(schema.properties?.password).toBeUndefined();
    });

    it("validates picked schema", () => {
      const user = w.object({
        id: w.string(),
        name: w.string(),
        email: w.string(),
      });
      const picked = user.pick("id", "name");

      expect(
        validate(picked.toTypeSchema(), { id: "1", name: "Alice" }).valid,
      ).toBe(true);
      expect(
        validate(picked.toTypeSchema(), {
          id: "1",
          name: "Alice",
          email: "a@b.com",
        }).valid,
      ).toBe(false);

      const passthroughPicked = user.passthrough().pick("id", "name");
      expect(
        validate(passthroughPicked.toTypeSchema(), {
          id: "1",
          name: "Alice",
          email: "a@b.com",
        }).valid,
      ).toBe(true);
    });

    it("preserves type inference", () => {
      const user = w.object({
        id: w.string(),
        name: w.string(),
        email: w.string(),
      });
      const picked = user.pick("id", "name");

      type Result = Infer<typeof picked>;
      expectTypeOf<Result>().toEqualTypeOf<{ id: string; name: string }>();
    });
  });

  describe("omit", () => {
    it("excludes specified keys", () => {
      const user = w.object({
        id: w.string(),
        name: w.string(),
        password: w.string(),
      });
      const safe = user.omit("password");

      const schema = safe.toTypeSchema();
      expect(schema.properties?.id).toBeDefined();
      expect(schema.properties?.name).toBeDefined();
      expect(schema.properties?.password).toBeUndefined();
    });

    it("validates omitted schema", () => {
      const user = w.object({
        id: w.string(),
        name: w.string(),
        password: w.string(),
      });
      const safe = user.omit("password");

      expect(
        validate(safe.toTypeSchema(), { id: "1", name: "Alice" }).valid,
      ).toBe(true);
    });

    it("preserves type inference", () => {
      const user = w.object({
        id: w.string(),
        name: w.string(),
        password: w.string(),
      });
      const safe = user.omit("password");

      type Result = Infer<typeof safe>;
      expectTypeOf<Result>().toEqualTypeOf<{ id: string; name: string }>();
    });
  });

  describe("partial", () => {
    it("makes all properties optional", () => {
      const user = w.object({ name: w.string(), age: w.integer() });
      const partialUser = user.partial();

      const schema = partialUser.toTypeSchema();
      // In flattened format, optional properties have required: false
      expect(
        (schema.properties?.name as { required?: boolean })?.required,
      ).toBe(false);
      expect((schema.properties?.age as { required?: boolean })?.required).toBe(
        false,
      );
    });

    it("validates partial schema", () => {
      const user = w.object({ name: w.string(), age: w.integer() });
      const partialUser = user.partial();

      expect(validate(partialUser.toTypeSchema(), {}).valid).toBe(true);
      expect(
        validate(partialUser.toTypeSchema(), { name: "Alice" }).valid,
      ).toBe(true);
      expect(
        validate(partialUser.toTypeSchema(), { name: "Alice", age: 30 }).valid,
      ).toBe(true);
    });

    it("preserves type inference", () => {
      const user = w.object({ name: w.string(), age: w.integer() });
      const partialUser = user.partial();

      type Result = Infer<typeof partialUser>;
      expectTypeOf<Result>().toEqualTypeOf<{ name?: string; age?: number }>();
    });
  });

  describe("required", () => {
    it("makes all properties required", () => {
      const user = w.object({
        name: optional(w.string()),
        age: optional(w.integer()),
      });
      const requiredUser = user.required();

      const schema = requiredUser.toTypeSchema();
      // Required properties don't have 'required' field (defaults to true)
      expect(
        (schema.properties?.name as { required?: boolean })?.required,
      ).toBeUndefined();
      expect(
        (schema.properties?.age as { required?: boolean })?.required,
      ).toBeUndefined();
    });

    it("validates required schema", () => {
      const user = w.object({
        name: optional(w.string()),
        age: optional(w.integer()),
      });
      const requiredUser = user.required();

      expect(validate(requiredUser.toTypeSchema(), {}).valid).toBe(false);
      expect(
        validate(requiredUser.toTypeSchema(), { name: "Alice" }).valid,
      ).toBe(false);
      expect(
        validate(requiredUser.toTypeSchema(), { name: "Alice", age: 30 }).valid,
      ).toBe(true);
    });

    it("preserves type inference", () => {
      const user = w.object({
        name: optional(w.string()),
        age: optional(w.integer()),
      });
      const requiredUser = user.required();

      type Result = Infer<typeof requiredUser>;
      expectTypeOf<Result>().toEqualTypeOf<{ name: string; age: number }>();
    });
  });

  describe("chained composition", () => {
    it("supports chaining multiple operations", () => {
      const user = w.object({
        id: w.string(),
        name: w.string(),
        email: w.string(),
        password: w.string(),
      });

      const safePartialUser = user.omit("password").partial();

      const schema = safePartialUser.toTypeSchema();
      expect(schema.properties?.password).toBeUndefined();
      expect((schema.properties?.id as { required?: boolean })?.required).toBe(
        false,
      );

      expect(validate(safePartialUser.toTypeSchema(), {}).valid).toBe(true);
      expect(validate(safePartialUser.toTypeSchema(), { id: "1" }).valid).toBe(
        true,
      );
    });

    it("preserves type inference through chain", () => {
      const user = w.object({
        id: w.string(),
        name: w.string(),
        password: w.string(),
      });

      const safePartialUser = user.omit("password").partial();

      type Result = Infer<typeof safePartialUser>;
      expectTypeOf<Result>().toEqualTypeOf<{ id?: string; name?: string }>();
    });
  });
});
