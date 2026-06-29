import type { PropertySchema, Schema, TypeSchema } from "wellformed-ts/ir";

const NUMERIC_SAMPLE_TYPES = new Set<string>([
  "number",
  "integer",
  "int32",
  "int64",
  "uint32",
  "uint64",
  "money",
  "currency",
  "decimal",
  "percentage",
]);

/**
 * Generates a self-contained TSX component string that demonstrates
 * how to use wellformed + react-hook-form for the given schema.
 */
export function generateCode(schema: Schema): string {
  const componentName = toComponentName(schema.title ?? "MyForm");
  const hasArrays = schemaHasType(schema.root, "array");

  const lines: string[] = [];

  // Imports
  lines.push(
    `import { useForm${hasArrays ? ", useFieldArray" : ""} } from "react-hook-form";`,
  );
  lines.push(`import type { Schema } from "wellformed-ts/ir";`);
  lines.push(`import { validate } from "wellformed-ts/runtime";`);
  lines.push(``);

  // Inline schema
  lines.push(`const schema: Schema = ${JSON.stringify(schema, null, 2)};`);
  lines.push(``);

  // Resolver
  lines.push(`function wellformedResolver(schema: Schema) {`);
  lines.push(`  return async (values: Record<string, unknown>) => {`);
  lines.push(`    const result = validate(schema, values);`);
  lines.push(`    if (result.valid) {`);
  lines.push(
    `      return { values: result.value as Record<string, unknown>, errors: {} };`,
  );
  lines.push(`    }`);
  lines.push(
    `    const errors: Record<string, { type: string; message: string }> = {};`,
  );
  lines.push(`    for (const err of result.errors) {`);
  lines.push(
    `      const path = err.path.replace(/^\\//, "").replace(/\\//g, ".");`,
  );
  lines.push(`      if (path && !errors[path]) {`);
  lines.push(
    `        errors[path] = { type: err.code, message: err.message };`,
  );
  lines.push(`      }`);
  lines.push(`    }`);
  lines.push(`    return { values: {}, errors };`);
  lines.push(`  };`);
  lines.push(`}`);
  lines.push(``);

  // Component
  lines.push(`export default function ${componentName}() {`);
  lines.push(`  const {`);
  lines.push(`    register,`);
  lines.push(`    handleSubmit,`);
  if (hasArrays) lines.push(`    control,`);
  lines.push(`    formState: { errors },`);
  lines.push(`  } = useForm({`);
  lines.push(`    resolver: wellformedResolver(schema),`);
  lines.push(`    mode: "onBlur",`);
  lines.push(`  });`);
  lines.push(``);

  // Declare useFieldArray hooks at the top level for any root-level arrays
  appendRootArrayHooks(lines, schema);

  lines.push(`  const onSubmit = (data: Record<string, unknown>) => {`);
  lines.push(`    console.log("Form submitted:", data);`);
  lines.push(`  };`);
  lines.push(``);
  lines.push(`  return (`);
  lines.push(
    `    <form onSubmit={handleSubmit(onSubmit)} style={{ maxWidth: 480, margin: "0 auto" }}>`,
  );

  if (schema.title) {
    lines.push(`      <h1>${schema.title}</h1>`);
  }
  if (schema.description) {
    lines.push(`      <p style={{ color: "#666" }}>${schema.description}</p>`);
  }

  // Render fields
  if (schema.root.type === "object" && schema.root.properties) {
    for (const [key, prop] of Object.entries(schema.root.properties)) {
      lines.push(``);
      renderField(lines, key, key, prop as TypeSchema, 6);
    }
  }

  lines.push(``);
  lines.push(
    `      <button type="submit" style={{ marginTop: 16 }}>Submit</button>`,
  );
  lines.push(`    </form>`);
  lines.push(`  );`);
  lines.push(`}`);

  return lines.join("\n");
}

function appendRootArrayHooks(lines: string[], schema: Schema): void {
  if (schema.root.type !== "object" || !schema.root.properties) return;

  for (const [key, prop] of Object.entries(schema.root.properties)) {
    const ts = prop as TypeSchema;
    if (ts.type !== "array") continue;

    const fieldVar = `${key}Fields`;
    lines.push(
      `  const { fields: ${fieldVar}, append: append${capitalize(key)}, remove: remove${capitalize(key)} } = useFieldArray({`,
    );
    lines.push(`    control,`);
    lines.push(`    name: "${key}",`);
    lines.push(`  });`);
    lines.push(``);
  }
}

/**
 * Generates a self-contained Rust example that validates data using
 * the runtime schema engine.
 */
export function generateRustCode(schema: Schema): string {
  const schemaJson = JSON.stringify(schema, null, 2);
  const sampleInput = JSON.stringify(buildSampleValue(schema.root), null, 2);
  const schemaPathHint = `schemas/${toSnakeCase(schema.id ?? schema.title ?? "schema")}.json`;

  const schemaLiteral = toRustRawString(schemaJson);
  const inputLiteral = toRustRawString(sampleInput);

  const lines: string[] = [];
  lines.push(`// Recommended for static schemas (compile-time codegen):`);
  lines.push(`// use wellformed_macros::wel_schema;`);
  lines.push(`// wel_schema!("${schemaPathHint}");`);
  lines.push(`//`);
  lines.push(
    `// This playground example uses runtime schema loading for portability.`,
  );
  lines.push(``);
  lines.push(`use serde_json::Value;`);
  lines.push(`use wellformed::{validate, Schema};`);
  lines.push(``);
  lines.push(`fn main() -> Result<(), Box<dyn std::error::Error>> {`);
  lines.push(`    let schema_json = ${schemaLiteral};`);
  lines.push(`    let schema: Schema = serde_json::from_str(schema_json)?;`);
  lines.push(``);
  lines.push(
    `    let mut input: Value = serde_json::from_str(${inputLiteral})?;`,
  );
  lines.push(``);
  lines.push(`    let result = validate(&schema, &mut input)?;`);
  lines.push(``);
  lines.push(`    if result.is_valid() {`);
  lines.push(`        println!("Validation passed");`);
  lines.push(`        println!("Normalized value: {input}");`);
  lines.push(`    } else {`);
  lines.push(`        println!("Validation failed:");`);
  lines.push(`        for err in &result.errors {`);
  lines.push(
    `            println!("- {} at {}: {}", err.code, err.path, err.message);`,
  );
  lines.push(`        }`);
  lines.push(`    }`);
  lines.push(``);
  lines.push(`    Ok(())`);
  lines.push(`}`);

  return lines.join("\n");
}

function renderField(
  lines: string[],
  name: string,
  dotPath: string,
  schema: TypeSchema,
  indent: number,
): void {
  const pad = " ".repeat(indent);
  const label = formatLabel(leafName(name));
  const errPath = dotPathToErrorAccess(dotPath);

  switch (schema.type) {
    case "string":
      lines.push(`${pad}<div style={{ marginBottom: 12 }}>`);
      lines.push(`${pad}  <label>${label}</label><br />`);
      lines.push(`${pad}  <input {...register("${dotPath}")} />`);
      lines.push(
        `${pad}  {${errPath} && <p style={{ color: "red" }}>{${errPath}.message}</p>}`,
      );
      lines.push(`${pad}</div>`);
      break;

    case "number":
    case "integer":
    case "int32":
    case "int64":
    case "uint32":
    case "uint64":
    case "money":
    case "decimal":
    case "currency":
    case "percentage":
      lines.push(`${pad}<div style={{ marginBottom: 12 }}>`);
      lines.push(`${pad}  <label>${label}</label><br />`);
      lines.push(
        `${pad}  <input type="number" {...register("${dotPath}", { valueAsNumber: true })} />`,
      );
      lines.push(
        `${pad}  {${errPath} && <p style={{ color: "red" }}>{${errPath}.message}</p>}`,
      );
      lines.push(`${pad}</div>`);
      break;

    case "boolean":
      lines.push(`${pad}<div style={{ marginBottom: 12 }}>`);
      lines.push(`${pad}  <label>`);
      lines.push(
        `${pad}    <input type="checkbox" {...register("${dotPath}")} />`,
      );
      lines.push(`${pad}    {" "}${label}`);
      lines.push(`${pad}  </label>`);
      lines.push(`${pad}</div>`);
      break;

    case "enum":
      lines.push(`${pad}<div style={{ marginBottom: 12 }}>`);
      lines.push(`${pad}  <label>${label}</label><br />`);
      lines.push(`${pad}  <select {...register("${dotPath}")}>`);
      lines.push(`${pad}    <option value="">Select...</option>`);
      if ("values" in schema) {
        for (const v of schema.values) {
          lines.push(
            `${pad}    <option value="${String(v)}">${String(v)}</option>`,
          );
        }
      }
      lines.push(`${pad}  </select>`);
      lines.push(
        `${pad}  {${errPath} && <p style={{ color: "red" }}>{${errPath}.message}</p>}`,
      );
      lines.push(`${pad}</div>`);
      break;

    case "date":
      lines.push(`${pad}<div style={{ marginBottom: 12 }}>`);
      lines.push(`${pad}  <label>${label}</label><br />`);
      lines.push(`${pad}  <input type="date" {...register("${dotPath}")} />`);
      lines.push(
        `${pad}  {${errPath} && <p style={{ color: "red" }}>{${errPath}.message}</p>}`,
      );
      lines.push(`${pad}</div>`);
      break;

    case "object":
      lines.push(
        `${pad}<fieldset style={{ marginBottom: 12, padding: 12, border: "1px solid #ddd" }}>`,
      );
      lines.push(`${pad}  <legend>${label}</legend>`);
      if ("properties" in schema && schema.properties) {
        for (const [key, prop] of Object.entries(schema.properties)) {
          renderField(
            lines,
            key,
            `${dotPath}.${key}`,
            prop as TypeSchema,
            indent + 2,
          );
        }
      }
      lines.push(`${pad}</fieldset>`);
      break;

    case "array":
      renderArrayField(lines, name, dotPath, schema, indent);
      break;

    default:
      // Fallback to text input
      lines.push(`${pad}<div style={{ marginBottom: 12 }}>`);
      lines.push(`${pad}  <label>${label}</label><br />`);
      lines.push(`${pad}  <input {...register("${dotPath}")} />`);
      lines.push(
        `${pad}  {${errPath} && <p style={{ color: "red" }}>{${errPath}.message}</p>}`,
      );
      lines.push(`${pad}</div>`);
      break;
  }
}

function renderArrayField(
  lines: string[],
  name: string,
  dotPath: string,
  schema: TypeSchema & { type: "array" },
  indent: number,
): void {
  const pad = " ".repeat(indent);
  const fieldName = leafName(name);
  const label = formatLabel(fieldName);
  const fieldVar = `${dotPath.replace(/\./g, "_")}Fields`;
  const appendFn = `append${capitalize(fieldName)}`;
  const removeFn = `remove${capitalize(fieldName)}`;
  const itemSchema = schema.items;

  lines.push(
    `${pad}<fieldset style={{ marginBottom: 12, padding: 12, border: "1px solid #ddd" }}>`,
  );
  lines.push(`${pad}  <legend>${label}</legend>`);
  lines.push(`${pad}  {${fieldVar}.map((field, index) => (`);
  lines.push(
    `${pad}    <div key={field.id} style={{ marginBottom: 8, padding: 8, border: "1px solid #eee" }}>`,
  );

  if (
    itemSchema.type === "object" &&
    "properties" in itemSchema &&
    itemSchema.properties
  ) {
    for (const [key, prop] of Object.entries(itemSchema.properties)) {
      renderField(
        lines,
        key,
        `${dotPath}.\${index}.${key}`,
        prop as TypeSchema,
        indent + 6,
      );
    }
  } else {
    renderField(lines, "item", `${dotPath}.\${index}`, itemSchema, indent + 6);
  }

  lines.push(
    `${pad}      <button type="button" onClick={() => ${removeFn}(index)}>Remove</button>`,
  );
  lines.push(`${pad}    </div>`);
  lines.push(`${pad}  ))}`);

  // Build the default item for append
  let defaultItem: string;
  if (
    itemSchema.type === "object" &&
    "properties" in itemSchema &&
    itemSchema.properties
  ) {
    const entries = Object.keys(itemSchema.properties)
      .map((k) => `${k}: ""`)
      .join(", ");
    defaultItem = `{ ${entries} }`;
  } else {
    defaultItem = `""`;
  }

  lines.push(
    `${pad}  <button type="button" onClick={() => ${appendFn}(${defaultItem})}>Add ${label}</button>`,
  );
  lines.push(`${pad}</fieldset>`);
}

/** Convert "Contact Form" to "ContactForm" */
function toComponentName(title: string): string {
  const cleaned = title
    .replace(/[^a-zA-Z0-9\s]/g, "")
    .trim()
    .split(/\s+/)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join("");
  // Ensure it starts with a letter
  if (!cleaned || !/^[A-Z]/.test(cleaned)) return "MyForm";
  return cleaned;
}

/** "firstName" -> "First Name" */
function formatLabel(key: string): string {
  return key
    .replace(/([A-Z])/g, " $1")
    .replace(/^./, (s) => s.toUpperCase())
    .trim();
}

/** "foo.bar" -> "errors.foo?.bar" */
function dotPathToErrorAccess(dotPath: string): string {
  const parts = dotPath.split(".");
  if (parts.length === 1) return `errors.${parts[0]}`;
  return (
    `errors.${parts[0]}` +
    parts
      .slice(1)
      .map((p) => `?.${p}`)
      .join("")
  );
}

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

function leafName(path: string): string {
  const parts = path.split(".");
  return parts[parts.length - 1] ?? path;
}

/** Check if a TypeSchema tree contains a given type */
function schemaHasType(schema: TypeSchema, targetType: string): boolean {
  if (schema.type === targetType) return true;
  if (schema.type === "object" && "properties" in schema && schema.properties) {
    for (const prop of Object.values(schema.properties)) {
      if (schemaHasType(prop as TypeSchema, targetType)) return true;
    }
  }
  if (schema.type === "array" && "items" in schema) {
    if (schemaHasType(schema.items, targetType)) return true;
  }
  return false;
}

function toRustRawString(value: string): string {
  let hashes = "#";
  while (value.includes(`"${hashes}`)) {
    hashes += "#";
  }
  return `r${hashes}"${value}"${hashes}`;
}

function buildSampleValue(schema: TypeSchema): unknown {
  if (NUMERIC_SAMPLE_TYPES.has(schema.type)) return 0;

  switch (schema.type) {
    case "string":
      return "example";
    case "boolean":
      return false;
    case "date":
      return "2026-01-01";
    case "enum":
      return schema.values[0] ?? null;
    case "literal":
      return schema.value;
    case "array":
      return [buildSampleValue(schema.items)];
    case "tuple":
      return schema.items.map((item) => buildSampleValue(item));
    case "object":
      return buildObjectSampleValue(schema);
    case "union":
      return schema.oneOf[0] ? buildSampleValue(schema.oneOf[0]) : null;
    case "intersection":
      return schema.allOf[0] ? buildSampleValue(schema.allOf[0]) : null;
    case "record":
      return { key: buildSampleValue(schema.value) };
    case "preprocess":
    case "catch":
      return buildSampleValue(schema.schema);
    case "ref":
    case "any":
    case "never":
      return null;
    default:
      return null;
  }
}

function buildObjectSampleValue(
  schema: Extract<TypeSchema, { type: "object" }>,
): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  if (!schema.properties) return out;

  for (const [key, prop] of Object.entries(schema.properties)) {
    const property = prop as PropertySchema;
    const required = property.required ?? true;
    if (required) {
      out[key] = buildSampleValue(property);
    }
  }

  return out;
}

function toSnakeCase(value: string): string {
  return value
    .replace(/[^a-zA-Z0-9]+/g, "_")
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .replace(/^_+|_+$/g, "")
    .toLowerCase();
}
