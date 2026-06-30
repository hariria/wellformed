import { describe, expect, it } from "vitest";
import { optional, w } from "./w.js";

describe("w.string()", () => {
  it("creates basic string schema", () => {
    const schema = w.string().toTypeSchema();
    expect(schema.type).toBe("string");
  });

  it("chains transforms", () => {
    const schema = w.string().trim().upper().toTypeSchema();
    expect(schema.type).toBe("string");
    expect(schema.transforms).toEqual([{ fn: "trim" }, { fn: "upper" }]);
  });

  it("chains constraints", () => {
    const schema = w.string().minLen(1).maxLen(100).toTypeSchema();
    expect(schema.constraints).toHaveLength(2);
    expect(schema.constraints?.[0]?.pred).toEqual({ type: "min_len", len: 1 });
    expect(schema.constraints?.[1]?.pred).toEqual({
      type: "max_len",
      len: 100,
    });
  });

  it("supports TIN predicates", () => {
    const schema = w.string().trim().digitsOnly().tin().toTypeSchema();
    expect(schema.transforms).toHaveLength(2);
    expect(schema.constraints).toHaveLength(1);
    expect(schema.constraints?.[0]?.pred).toEqual({
      type: "call",
      name: "is_tin",
    });
  });

  it("supports email validation", () => {
    const schema = w.string().email().toTypeSchema();
    expect(schema.constraints?.[0]?.pred).toEqual({
      type: "call",
      name: "is_email",
    });
    expect(schema.constraints?.[0]?.error.code).toBe("INVALID_EMAIL");
  });

  it("supports aviation predicates", () => {
    const schema = w
      .string()
      .iataAirportCode()
      .icaoAirportCode()
      .airportCode({ system: "ANY" })
      .iataAirlineCode()
      .icaoAirlineCode()
      .airlineCode({ system: "IATA" })
      .flightNumber()
      .toTypeSchema();

    expect(schema.constraints).toHaveLength(7);
    expect(schema.constraints?.[0]?.pred).toEqual({
      type: "call",
      name: "is_iata_airport_code",
      args: { known_only: undefined },
    });
    expect(schema.constraints?.[2]?.pred).toEqual({
      type: "call",
      name: "is_airport_code",
      args: { system: "ANY", known_only: undefined },
    });
    expect(schema.constraints?.[5]?.pred).toEqual({
      type: "call",
      name: "is_airline_code",
      args: { system: "IATA", known_only: undefined },
    });
    expect(schema.constraints?.[6]?.pred).toEqual({
      type: "call",
      name: "is_flight_number",
      args: {
        carrier_format: undefined,
        known_carrier: undefined,
        allow_suffix: undefined,
      },
    });
  });

  it("supports normalizeFlightNumber transform", () => {
    const schema = w.string().normalizeFlightNumber().toTypeSchema();
    expect(schema.transforms).toEqual([{ fn: "normalize_flight_number" }]);
  });

  it("supports healthcare transforms and predicates", () => {
    const schema = w
      .string()
      .normalizeIcd10()
      .normalizeCpt()
      .normalizeHcpcs()
      .normalizeNdc11()
      .icd10Code({ strictFormat: true })
      .cptCode({ allowCategoryIi: false })
      .hcpcsCode()
      .ndcCode({ format: "11" })
      .toTypeSchema();

    expect(schema.transforms).toEqual([
      { fn: "normalize_icd10" },
      { fn: "normalize_cpt" },
      { fn: "normalize_hcpcs" },
      { fn: "normalize_ndc11" },
    ]);
    expect(schema.constraints?.[0]?.pred).toEqual({
      type: "call",
      name: "is_icd10_code",
      args: { strict_format: true },
    });
    expect(schema.constraints?.[3]?.pred).toEqual({
      type: "call",
      name: "is_ndc_code",
      args: { format: "11" },
    });
  });

  it("supports Zod-style string format predicates", () => {
    const schema = w
      .string()
      .startsWith("foo")
      .endsWith("bar")
      .includes("mid")
      .ip({ version: "v4" })
      .cidr()
      .macAddress()
      .jwt()
      .hash({ algorithm: "sha256" })
      .toTypeSchema();

    expect(schema.constraints?.[0]?.pred).toEqual({
      type: "call",
      name: "starts_with",
      args: { value: "foo" },
    });
    expect(schema.constraints?.[7]?.pred).toEqual({
      type: "call",
      name: "is_hash",
      args: { algorithm: "sha256" },
    });
  });

  it("supports custom error messages", () => {
    const schema = w
      .string()
      .minLen(5, { message: "Too short!" })
      .toTypeSchema();
    expect(schema.constraints?.[0]?.error.message).toBe("Too short!");
  });

  it("supports regex validation", () => {
    const schema = w.string().regex("^[A-Z]+$", { flags: "i" }).toTypeSchema();
    expect(schema.constraints?.[0]?.pred).toEqual({
      type: "regex",
      pattern: "^[A-Z]+$",
      flags: "i",
    });
  });

  it("supports template literal validation", () => {
    const schema = w
      .string()
      .templateLiteral([
        "SFO-",
        { kind: "digits", min: 3, max: 4 },
        "-",
        { kind: "uppercase", min: 2, max: 2 },
      ])
      .toTypeSchema();
    expect(schema.constraints?.[0]?.pred).toEqual({
      type: "template_literal",
      parts: [
        { kind: "literal", value: "SFO-" },
        { kind: "digits", min: 3, max: 4 },
        { kind: "literal", value: "-" },
        { kind: "uppercase", min: 2, max: 2 },
      ],
    });
  });
});

describe("w.number()", () => {
  it("creates basic number schema", () => {
    const schema = w.number().toTypeSchema();
    expect(schema.type).toBe("number");
  });

  it("supports range constraints", () => {
    const schema = w.number().min(0).max(100).toTypeSchema();
    expect(schema.constraints).toHaveLength(2);
  });

  it("supports nonNegative", () => {
    const schema = w.number().nonNegative().toTypeSchema();
    expect(schema.constraints?.[0]?.pred).toEqual({
      type: "call",
      name: "is_non_negative",
    });
  });

  it("supports negative/nonPositive/multipleOf", () => {
    const schema = w
      .number()
      .negative()
      .nonPositive()
      .multipleOf(0.5)
      .toTypeSchema();
    expect(schema.constraints?.[0]?.pred).toEqual({
      type: "call",
      name: "is_negative",
    });
    expect(schema.constraints?.[1]?.pred).toEqual({
      type: "call",
      name: "is_non_positive",
    });
    expect(schema.constraints?.[2]?.pred).toEqual({
      type: "call",
      name: "is_multiple_of",
      args: { value: 0.5 },
    });
  });

  it("supports percentage", () => {
    const schema = w.number().percentage({ format: "decimal" }).toTypeSchema();
    expect(schema.constraints?.[0]?.pred).toEqual({
      type: "call",
      name: "is_percentage",
      args: { format: "decimal", allow_over_100: undefined },
    });
  });
});

describe("w.integer()", () => {
  it("creates integer schema", () => {
    const schema = w.integer().toTypeSchema();
    expect(schema.type).toBe("integer");
  });

  it("supports taxYear validation", () => {
    const schema = w.integer().taxYear({ min: 2020, max: 2030 }).toTypeSchema();
    expect(schema.constraints?.[0]?.pred).toEqual({
      type: "call",
      name: "is_tax_year",
      args: { min: 2020, max: 2030 },
    });
  });
});

describe("w.money()", () => {
  it("creates money schema", () => {
    const schema = w.money().toTypeSchema();
    expect(schema.type).toBe("money");
  });

  it("supports scale", () => {
    const schema = w.money().scale(4).toTypeSchema();
    expect(schema.scale).toBe(4);
  });

  it("supports nonNegative", () => {
    const schema = w.money().nonNegative().toTypeSchema();
    expect(schema.constraints?.[0]?.error.code).toBe("NEGATIVE_AMOUNT");
  });
});

describe("w.date()", () => {
  it("creates date schema", () => {
    const schema = w.date().toTypeSchema();
    expect(schema.type).toBe("date");
  });

  it("supports format", () => {
    const schema = w.date().format("MM/DD/YYYY").toTypeSchema();
    expect(schema.format).toBe("MM/DD/YYYY");
  });

  it("supports range constraints", () => {
    const schema = w
      .date()
      .inRange({ minYear: 2020, maxYear: 2030 })
      .toTypeSchema();
    expect(schema.constraints?.[0]?.pred).toEqual({
      type: "call",
      name: "date_in_range",
      args: { min_year: 2020, max_year: 2030, min: undefined, max: undefined },
    });
  });
});

describe("w.boolean()", () => {
  it("creates boolean schema", () => {
    const schema = w.boolean().toTypeSchema();
    expect(schema.type).toBe("boolean");
  });
});

describe("w.object()", () => {
  it("creates object schema from shape", () => {
    const schema = w
      .object({
        name: w.string(),
        age: w.integer(),
      })
      .toTypeSchema();

    expect(schema.type).toBe("object");
    // Properties are now flattened - type is directly on the property
    expect(schema.properties?.name).toEqual({ type: "string" });
    expect(schema.properties?.age).toEqual({ type: "integer" });
  });

  it("supports optional properties", () => {
    const schema = w
      .object({
        name: w.string(),
        nickname: optional(w.string()),
      })
      .toTypeSchema();

    // Required fields don't have 'required' property (defaults to true)
    expect(schema.properties?.name).toEqual({ type: "string" });
    // Optional fields have required: false
    expect(schema.properties?.nickname).toEqual({
      type: "string",
      required: false,
    });
  });

  it("supports additionalProperties", () => {
    const schema = w.object({}).additionalProperties(true).toTypeSchema();
    expect(schema.additional_properties).toBe(true);
  });

  it("supports strict/passthrough/strip unknown key modes", () => {
    expect(w.object({}).strict().toTypeSchema().unknown_keys).toBe("strict");
    expect(w.object({}).passthrough().toTypeSchema().unknown_keys).toBe(
      "passthrough",
    );
    expect(w.object({}).strip().toTypeSchema().unknown_keys).toBe("strip");
  });

  it("supports catchall", () => {
    const schema = w.object({}).catchall(w.integer()).toTypeSchema();
    expect(schema.catchall).toEqual({ type: "integer" });
    expect(schema.unknown_keys).toBe("passthrough");
  });

  it("supports cross-field rules", () => {
    const schema = w
      .object({
        startDate: w.string(),
        endDate: w.string(),
      })
      .requireWith("startDate", "endDate")
      .toTypeSchema();

    expect(schema.rules).toHaveLength(1);
    expect(schema.rules?.[0]?.pred.type).toBe("required_with");
  });

  it("supports mutually exclusive fields", () => {
    const schema = w
      .object({
        ssn: w.string(),
        ein: w.string(),
      })
      .mutuallyExclusive("ssn", "ein")
      .toTypeSchema();

    expect(schema.rules?.[0]?.error.code).toBe("MUTUALLY_EXCLUSIVE");
  });

  it("supports requireOneOf", () => {
    const schema = w
      .object({
        email: w.string(),
        phone: w.string(),
      })
      .requireOneOf(["email", "phone"])
      .toTypeSchema();

    expect(schema.rules?.[0]?.pred.type).toBe("or");
  });

  it("supports requireWithout", () => {
    const schema = w
      .object({
        taxId: w.string(),
        ssn: w.string(),
      })
      .requireWithout("taxId", "ssn")
      .toTypeSchema();

    expect(schema.rules?.[0]?.pred.type).toBe("required_without");
  });

  it("supports requireExactlyOneOf", () => {
    const schema = w
      .object({
        ssn: w.string(),
        ein: w.string(),
      })
      .requireExactlyOneOf(["ssn", "ein"])
      .toTypeSchema();

    expect(schema.rules?.[0]?.pred.type).toBe("exactly_one_of");
  });
});

describe("w.array()", () => {
  it("creates array schema", () => {
    const schema = w.array(w.string()).toTypeSchema();
    expect(schema.type).toBe("array");
    expect(schema.items).toEqual({ type: "string" });
  });

  it("supports minItems/maxItems", () => {
    const schema = w.array(w.string()).minItems(1).maxItems(10).toTypeSchema();
    expect(schema.constraints).toHaveLength(2);
  });

  it("supports nonEmpty", () => {
    const schema = w.array(w.string()).nonEmpty().toTypeSchema();
    expect(schema.constraints?.[0]?.pred).toEqual({ type: "min_len", len: 1 });
  });
});

describe("w.record()", () => {
  it("creates record schema", () => {
    const schema = w.record(w.integer()).toTypeSchema();
    expect(schema.type).toBe("record");
    expect(schema.value).toEqual({ type: "integer" });
  });

  it("supports key schema and partial", () => {
    const schema = w.record(w.number(), w.string()).partial().toTypeSchema();
    expect(schema.key).toEqual({ type: "string" });
    expect(schema.partial).toBe(true);
  });
});

describe("w.enum()", () => {
  it("creates enum schema", () => {
    const schema = w.enum(["A", "B", "C"]).toTypeSchema();
    expect(schema.type).toBe("enum");
    expect(schema.values).toEqual(["A", "B", "C"]);
  });
});

describe("w.literal()", () => {
  it("creates literal schema", () => {
    const schema = w.literal("A").toTypeSchema();
    expect(schema.type).toBe("literal");
    expect(schema.value).toBe("A");
  });
});

describe("w.never()", () => {
  it("creates never schema", () => {
    const schema = w.never().toTypeSchema();
    expect(schema.type).toBe("never");
  });
});

describe("w.tuple()", () => {
  it("creates tuple schema", () => {
    const schema = w.tuple([w.string(), w.number()]).toTypeSchema();
    expect(schema.type).toBe("tuple");
    expect(schema.items).toEqual([{ type: "string" }, { type: "number" }]);
  });
});

describe("w.union()", () => {
  it("creates union schema", () => {
    const schema = w.union([w.string(), w.number()]).toTypeSchema();
    expect(schema.type).toBe("union");
    expect(schema.oneOf).toHaveLength(2);
  });

  it("supports discriminator", () => {
    const schema = w
      .union([w.object({}), w.object({})])
      .discriminator("type")
      .toTypeSchema();
    expect(schema.discriminator).toBe("type");
  });
});

describe("w.intersection()", () => {
  it("creates intersection schema", () => {
    const schema = w
      .intersection([w.string(), w.string().minLen(1)])
      .toTypeSchema();
    expect(schema.type).toBe("intersection");
    expect(schema.allOf).toHaveLength(2);
  });
});

describe("nullable/nullish wrappers", () => {
  it("supports builder .nullable()", () => {
    const schema = w.string().nullable().toTypeSchema();
    expect(schema.type).toBe("union");
    expect(schema.oneOf).toEqual([
      { type: "string" },
      { type: "literal", value: null },
    ]);
  });

  it("supports w.nullable()", () => {
    const schema = w.nullable(w.number()).toTypeSchema();
    expect(schema.type).toBe("union");
  });

  it("supports w.nullish()", () => {
    const schema = w.nullish(w.string()).toTypeSchema();
    expect(schema.type).toBe("union");
  });

  it("supports preprocess wrapper", () => {
    const schema = w.preprocess({ fn: "trim" }, w.string()).toTypeSchema();
    expect(schema.type).toBe("preprocess");
    expect(schema.transforms).toEqual([{ fn: "trim" }]);
    expect(schema.schema).toEqual({ type: "string" });
  });

  it("supports catch wrapper", () => {
    const schema = w.catch(w.integer(), 0).toTypeSchema();
    expect(schema.type).toBe("catch");
    expect(schema.schema).toEqual({ type: "integer" });
    expect(schema.value).toBe(0);
  });
});

describe("toSchema()", () => {
  it("creates complete schema with version", () => {
    const schema = w.string().toSchema("2.0");
    expect(schema.version).toBe("2.0");
    expect(schema.root.type).toBe("string");
  });

  it("defaults to version 1.0", () => {
    const schema = w.string().toSchema();
    expect(schema.version).toBe("1.0");
  });
});

describe("complex schema example", () => {
  it("creates W-2 payee schema", () => {
    const addressSchema = w.object({
      street: w.string().trim().minLen(1),
      city: w.string().trim().minLen(1),
      state: w.string().usState(),
      zip: w.string().usZip(),
    });

    const payeeSchema = w.object({
      tin: w.string().trim().digitsOnly().tin(),
      name: w.string().trim().minLen(1).maxLen(100),
      address: addressSchema,
    });

    const schema = payeeSchema.toSchema();

    expect(schema.version).toBe("1.0");
    expect(schema.root.type).toBe("object");

    const rootSchema = schema.root as {
      type: "object";
      properties: Record<string, unknown>;
    };
    expect(rootSchema.properties).toHaveProperty("tin");
    expect(rootSchema.properties).toHaveProperty("name");
    expect(rootSchema.properties).toHaveProperty("address");
  });
});
