import { describe, expect, it } from "vitest";
import { optional, w } from "./builder/index.js";
import {
  parseSchema,
  SchemaParseError,
  schemaToJSON,
  validateSchema,
} from "./serialize.js";

describe("JSON serialization", () => {
  describe("toJSON", () => {
    it("serializes a simple schema", () => {
      const schema = w.string().minLen(1).maxLen(100);
      const json = schema.toJSON();

      expect(json).toContain('"version":"1.0"');
      expect(json).toContain('"type":"string"');
    });

    it("serializes with custom version", () => {
      const schema = w.string();
      const json = schema.toJSON("2.0");

      expect(json).toContain('"version":"2.0"');
    });

    it("serializes with pretty printing", () => {
      const schema = w.string();
      const json = schema.toJSON("1.0", true);

      expect(json).toContain("\n");
      expect(json).toContain("  ");
    });

    it("serializes complex object schema", () => {
      const schema = w.object({
        name: w.string().trim().minLen(1),
        age: w.integer().min(0),
        tags: w.array(w.string()),
      });

      const json = schema.toJSON();
      const parsed = JSON.parse(json);

      expect(parsed.version).toBe("1.0");
      expect(parsed.root.type).toBe("object");
      // Flattened format: properties.name.type, not properties.name.schema.type
      expect(parsed.root.properties.name.type).toBe("string");
      expect(parsed.root.properties.age.type).toBe("integer");
      expect(parsed.root.properties.tags.type).toBe("array");
    });
  });

  describe("schemaToJSON", () => {
    it("serializes a Schema object", () => {
      const schema = w.object({ name: w.string() }).toSchema("1.0");
      const json = schemaToJSON(schema);

      expect(JSON.parse(json)).toEqual(schema);
    });

    it("serializes with pretty printing", () => {
      const schema = w.string().toSchema();
      const json = schemaToJSON(schema, true);

      expect(json).toContain("\n");
    });
  });

  describe("parseSchema", () => {
    it("parses valid JSON schema", () => {
      const original = w
        .object({
          name: w.string(),
          email: w.string().email(),
        })
        .toSchema("1.0");

      const json = JSON.stringify(original);
      const parsed = parseSchema(json);

      expect(parsed).toEqual(original);
    });

    it("throws on invalid JSON", () => {
      expect(() => parseSchema("not json")).toThrow(SchemaParseError);
      expect(() => parseSchema("not json")).toThrow(/Invalid JSON/);
    });

    it("throws on non-object", () => {
      expect(() => parseSchema('"string"')).toThrow(SchemaParseError);
      expect(() => parseSchema("123")).toThrow(SchemaParseError);
      expect(() => parseSchema("null")).toThrow(SchemaParseError);
    });

    it("throws on missing version", () => {
      expect(() => parseSchema('{"root":{"type":"string"}}')).toThrow(
        /version/,
      );
    });

    it("throws on missing root", () => {
      expect(() => parseSchema('{"version":"1.0"}')).toThrow(/root/);
    });

    it("throws on invalid type", () => {
      expect(() =>
        parseSchema('{"version":"1.0","root":{"type":"invalid"}}'),
      ).toThrow(/Invalid type/);
    });

    it("parses array schema", () => {
      const original = w.array(w.string()).toSchema();
      const json = JSON.stringify(original);
      const parsed = parseSchema(json);

      expect(parsed.root.type).toBe("array");
    });

    it("parses enum schema", () => {
      const original = w.enum(["a", "b", "c"]).toSchema();
      const json = JSON.stringify(original);
      const parsed = parseSchema(json);

      expect(parsed.root.type).toBe("enum");
    });

    it("parses union schema", () => {
      const original = w.union([w.string(), w.number()]).toSchema();
      const json = JSON.stringify(original);
      const parsed = parseSchema(json);

      expect(parsed.root.type).toBe("union");
    });

    it("parses literal schema", () => {
      const original = w.literal("active").toSchema();
      const json = JSON.stringify(original);
      const parsed = parseSchema(json);

      expect(parsed.root.type).toBe("literal");
    });

    it("parses never schema", () => {
      const original = w.never().toSchema();
      const json = JSON.stringify(original);
      const parsed = parseSchema(json);

      expect(parsed.root.type).toBe("never");
    });

    it("parses tuple schema", () => {
      const original = w.tuple([w.string(), w.number()]).toSchema();
      const json = JSON.stringify(original);
      const parsed = parseSchema(json);

      expect(parsed.root.type).toBe("tuple");
    });

    it("parses object unknown-key behavior and catchall", () => {
      const original = w
        .object({ name: w.string() })
        .strip()
        .catchall(w.integer())
        .toSchema();
      const json = JSON.stringify(original);
      const parsed = parseSchema(json);

      expect(parsed.root.type).toBe("object");
      expect((parsed.root as { unknown_keys?: string }).unknown_keys).toBe(
        "passthrough",
      );
      expect(
        (parsed.root as { catchall?: { type: string } }).catchall?.type,
      ).toBe("integer");
    });

    it("parses intersection schema", () => {
      const original = w
        .intersection([w.string(), w.string().minLen(1)])
        .toSchema();
      const json = JSON.stringify(original);
      const parsed = parseSchema(json);

      expect(parsed.root.type).toBe("intersection");
    });

    it("parses record schema", () => {
      const original = w.record(w.integer(), w.string()).toSchema();
      const json = JSON.stringify(original);
      const parsed = parseSchema(json);

      expect(parsed.root.type).toBe("record");
    });

    it("parses preprocess and catch wrappers", () => {
      const preprocessSchema = w
        .preprocess({ fn: "trim" }, w.string())
        .toSchema();
      const catchSchema = w.catch(w.integer(), 0).toSchema();

      expect(parseSchema(JSON.stringify(preprocessSchema)).root.type).toBe(
        "preprocess",
      );
      expect(parseSchema(JSON.stringify(catchSchema)).root.type).toBe("catch");
    });

    it("parses named definitions as a map", () => {
      const original = {
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
        root: { type: "ref", $ref: "Name" },
      };

      expect(parseSchema(JSON.stringify(original))).toEqual(original);
    });

    it("normalizes legacy template constraints to canonical constraints", () => {
      const parsed = parseSchema(
        JSON.stringify({
          version: "1.0",
          root: {
            type: "string",
            constraints: [
              { type: "minLength", value: 2, source: "iris" },
              { type: "format", value: "decimal-2" },
              { type: "enum", value: ["A", "B"] },
            ],
          },
        }),
      );

      expect((parsed.root as { constraints?: unknown[] }).constraints).toEqual([
        {
          pred: { type: "min_len", len: 2 },
          error: {
            code: "MIN_LENGTH_NOT_MET",
            message: "Must be at least 2 characters",
            severity: "error",
            source: "iris",
          },
        },
        {
          pred: { type: "call", name: "format:decimal-2" },
          error: {
            code: "FORMAT_INVALID",
            message: "Must be in decimal-2 format",
            severity: "error",
          },
        },
        {
          pred: { type: "in", path: "", values: ["A", "B"] },
          error: {
            code: "INVALID_ENUM",
            message: 'Must be one of: ["A","B"]',
            severity: "error",
          },
        },
      ]);
    });

    it("rejects legacy array-shaped definitions", () => {
      expect(() =>
        parseSchema(
          JSON.stringify({
            version: "1.0",
            definitions: [{ name: "Name", schema: { type: "string" } }],
            root: { type: "ref", $ref: "Name" },
          }),
        ),
      ).toThrow(/Schema\.definitions must be an object/);
    });
  });

  describe("validateSchema", () => {
    it("validates and returns Schema", () => {
      const schema = {
        version: "1.0",
        root: { type: "string" },
      };

      const result = validateSchema(schema);
      expect(result).toBe(schema);
    });

    it("validates object with optional fields", () => {
      const schema = w
        .object({
          name: w.string(),
          bio: optional(w.string()),
        })
        .toSchema();

      const result = validateSchema(schema);
      expect(result.root.type).toBe("object");
    });

    it("validates nested object schemas", () => {
      // Flattened format: properties are TypeSchema objects with optional required field
      const schema = {
        version: "1.0",
        root: {
          type: "object",
          properties: {
            user: {
              type: "object",
              properties: {
                name: { type: "string" },
              },
            },
          },
        },
      };

      const result = validateSchema(schema);
      expect(result).toBe(schema);
    });

    it("validates top-level and object rendering metadata", () => {
      const schema = {
        version: "1.0",
        irs_form: {
          name: "1099-INT",
          title: "Interest Income",
          revision: "2026",
        },
        pdf_template: {
          filename: "f1099int.pdf",
        },
        import: {
          enabled: true,
          max_rows: 100,
          column_mappings: { payer: "Payer Name" },
          required_columns: ["payer"],
        },
        sections: {
          payer: { title: "Payer", order: 1 },
        },
        root: {
          type: "object",
          pages: {
            copy_a: {
              name: "Copy A",
              fields: {
                payer: { type: "text", page: 0, x: 12, y: 24 },
              },
            },
          },
          acroform_mappings: [
            { field_id: "f1_1[0]", page: 1, compose: ["payer"] },
          ],
          properties: {
            payer: {
              type: "string",
              render: { type: "text", page: 0, x: 12, y: 24 },
              acroform: { field_id: "f1_1[0]", field_type: "text" },
              section: "payer",
            },
          },
        },
      };

      expect(validateSchema(schema)).toBe(schema);
    });

    it("accepts legacy object fields alias and validates nested schemas", () => {
      const schema = {
        version: "1.0",
        root: {
          type: "object",
          fields: {
            name: { type: "string" },
          },
        },
      };

      expect(validateSchema(schema)).toBe(schema);

      expect(() =>
        validateSchema({
          version: "1.0",
          root: {
            type: "object",
            fields: {
              name: { type: "not_a_type" },
            },
          },
        }),
      ).toThrow(/Invalid type/);
    });

    it("rejects objects with both properties and legacy fields", () => {
      expect(() =>
        validateSchema({
          version: "1.0",
          root: {
            type: "object",
            properties: {},
            fields: {
              name: { type: "string" },
            },
          },
        }),
      ).toThrow(/both properties and fields/);
    });

    it("rejects malformed transforms before runtime validation", () => {
      expect(() =>
        validateSchema({
          version: "1.0",
          root: {
            type: "string",
            transforms: [{ fn: "money_to_cents", scale: -1 }],
          },
        }),
      ).toThrow(/integer from 0 to 255/);
    });

    it("rejects malformed predicates before runtime validation", () => {
      expect(() =>
        validateSchema({
          version: "1.0",
          root: {
            type: "string",
            constraints: [
              {
                pred: { type: "min_len", len: -1 },
                error: { code: "TOO_SHORT", message: "too short" },
              },
            ],
          },
        }),
      ).toThrow(/non-negative integer/);
    });

    it("rejects transforms on schema variants that Rust does not transform", () => {
      expect(() =>
        validateSchema({
          version: "1.0",
          root: {
            type: "array",
            items: { type: "string" },
            transforms: [{ fn: "default", value: [] }],
          },
        }),
      ).toThrow(/array\.transforms is not supported/);
    });

    it("rejects constraints on schema variants that Rust does not constrain", () => {
      expect(() =>
        validateSchema({
          version: "1.0",
          root: {
            type: "enum",
            values: ["A", "B"],
            constraints: [
              {
                pred: { type: "true" },
                error: { code: "BAD", message: "bad" },
              },
            ],
          },
        }),
      ).toThrow(/enum\.constraints is not supported/);
    });

    it("rejects unsigned metadata fields with invalid numbers", () => {
      expect(() =>
        validateSchema({
          version: "1.0",
          sections: {
            payer: { title: "Payer", order: -1 },
          },
          root: { type: "string" },
        }),
      ).toThrow(/non-negative integer/);

      expect(() =>
        validateSchema({
          version: "1.0",
          root: {
            type: "object",
            pages: {
              copy_a: {
                fields: {
                  payer: { type: "text", page: 0.5, x: 12, y: 24 },
                },
              },
            },
          },
        }),
      ).toThrow(/RenderMetadata\.page/);
    });
  });

  describe("round-trip", () => {
    it("preserves schema through serialize/deserialize", () => {
      const original = w
        .object({
          type: w.enum(["individual", "business"]),
          name: w.string().trim().minLen(1),
          ssn: optional(w.string().ssn()),
          address: w.object({
            street: w.string(),
            city: w.string(),
            state: w.string().usState(),
            zip: w.string().usZip(),
          }),
        })
        .when("type")
        .equals("individual")
        .require("ssn")
        .toSchema("1.0");

      const json = JSON.stringify(original);
      const parsed = parseSchema(json);

      expect(parsed).toEqual(original);
    });
  });
});
