// biome-ignore-all lint/complexity/noExcessiveCognitiveComplexity: Schema parsing validates nested IR shapes inline so error paths stay precise.
/**
 * JSON serialization and deserialization for wellformed schemas.
 */

import type { Schema } from "./ir/types.js";

/**
 * Error thrown when schema parsing fails.
 */
export class SchemaParseError extends Error {
  constructor(
    message: string,
    public path?: string,
  ) {
    super(message);
    this.name = "SchemaParseError";
  }
}

/**
 * Serialize a Schema to a JSON string.
 */
export function schemaToJSON(schema: Schema, pretty = false): string {
  return JSON.stringify(schema, null, pretty ? 2 : undefined);
}

/**
 * Parse a JSON string to a Schema.
 * Validates the structure matches the expected Schema format.
 */
export function parseSchema(json: string): Schema {
  let parsed: unknown;
  try {
    parsed = JSON.parse(json);
  } catch (e) {
    throw new SchemaParseError(`Invalid JSON: ${(e as Error).message}`);
  }

  return validateSchema(parsed);
}

/**
 * Validate that an object conforms to the Schema interface.
 */
export function validateSchema(value: unknown): Schema {
  if (typeof value !== "object" || value === null) {
    throw new SchemaParseError("Schema must be an object");
  }

  const obj = value as Record<string, unknown>;

  if (typeof obj.version !== "string") {
    throw new SchemaParseError("Schema.version must be a string");
  }

  for (const key of ["id", "title", "description"] as const) {
    if (obj[key] !== undefined && typeof obj[key] !== "string") {
      throw new SchemaParseError(`Schema.${key} must be a string`, `/${key}`);
    }
  }

  if (!obj.root) {
    throw new SchemaParseError("Schema.root is required");
  }

  validateTypeSchema(obj.root, "/root");

  if (obj.irs_form !== undefined) {
    validateIrsFormMetadata(obj.irs_form, "/irs_form");
  }
  if (obj.pdf_template !== undefined) {
    validatePdfTemplate(obj.pdf_template, "/pdf_template");
  }
  if (obj.import !== undefined) {
    validateImportConfig(obj.import, "/import");
  }
  if (obj.sections !== undefined) {
    validateSections(obj.sections, "/sections");
  }

  if (obj.definitions !== undefined) {
    if (
      typeof obj.definitions !== "object" ||
      obj.definitions === null ||
      Array.isArray(obj.definitions)
    ) {
      throw new SchemaParseError("Schema.definitions must be an object");
    }

    for (const [name, definition] of Object.entries(
      obj.definitions as Record<string, unknown>,
    )) {
      if (typeof definition !== "object" || definition === null) {
        throw new SchemaParseError(
          `Schema.definitions.${name} must be an object`,
          `/definitions/${name}`,
        );
      }

      validateTypeSchema(definition, `/definitions/${name}`);
    }
  }

  return value as Schema;
}

function validateIrsFormMetadata(value: unknown, path: string): void {
  const obj = requireObject(value, "IrsFormMetadata", path);
  requireStringField(obj, "name", `${path}/name`);
  requireStringField(obj, "title", `${path}/title`);
  for (const key of [
    "revision",
    "revision_date",
    "omb_number",
    "cat_number",
  ] as const) {
    if (obj[key] !== undefined && typeof obj[key] !== "string") {
      throw new SchemaParseError(
        `IrsFormMetadata.${key} must be a string`,
        `${path}/${key}`,
      );
    }
  }
}

function validatePdfTemplate(value: unknown, path: string): void {
  const obj = requireObject(value, "PdfTemplate", path);
  for (const key of ["hash", "filename", "path", "source_uri"] as const) {
    if (obj[key] !== undefined && typeof obj[key] !== "string") {
      throw new SchemaParseError(
        `PdfTemplate.${key} must be a string`,
        `${path}/${key}`,
      );
    }
  }
}

function validateImportConfig(value: unknown, path: string): void {
  const obj = requireObject(value, "ImportConfig", path);
  if (obj.enabled !== undefined && typeof obj.enabled !== "boolean") {
    throw new SchemaParseError(
      "ImportConfig.enabled must be a boolean",
      `${path}/enabled`,
    );
  }
  if (obj.max_rows !== undefined) {
    validateNonNegativeInteger(
      obj.max_rows,
      "ImportConfig.max_rows",
      `${path}/max_rows`,
    );
  }
  if (obj.max_file_size !== undefined) {
    validateNonNegativeInteger(
      obj.max_file_size,
      "ImportConfig.max_file_size",
      `${path}/max_file_size`,
    );
  }
  if (obj.column_mappings !== undefined) {
    validateStringRecord(
      obj.column_mappings,
      "ImportConfig.column_mappings",
      `${path}/column_mappings`,
    );
  }
  if (obj.required_columns !== undefined) {
    validateStringArray(
      obj.required_columns,
      "ImportConfig.required_columns",
      `${path}/required_columns`,
    );
  }
  if (obj.description !== undefined && typeof obj.description !== "string") {
    throw new SchemaParseError(
      "ImportConfig.description must be a string",
      `${path}/description`,
    );
  }
}

function validateSections(value: unknown, path: string): void {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new SchemaParseError("Schema.sections must be an object", path);
  }

  for (const [name, section] of Object.entries(
    value as Record<string, unknown>,
  )) {
    const sectionPath = `${path}/${name}`;
    const obj = requireObject(section, "SectionDefinition", sectionPath);
    requireStringField(obj, "title", `${sectionPath}/title`);
    validateNonNegativeInteger(
      obj.order,
      "SectionDefinition.order",
      `${sectionPath}/order`,
    );
    if (obj.description !== undefined && typeof obj.description !== "string") {
      throw new SchemaParseError(
        "SectionDefinition.description must be a string",
        `${sectionPath}/description`,
      );
    }
  }
}

/**
 * Validate a TypeSchema structure.
 */
function validateTypeSchema(value: unknown, path: string): void {
  if (typeof value !== "object" || value === null) {
    throw new SchemaParseError("TypeSchema must be an object", path);
  }

  const obj = value as Record<string, unknown>;

  if (typeof obj.type !== "string") {
    throw new SchemaParseError("TypeSchema.type must be a string", path);
  }

  const validTypes = [
    "string",
    "number",
    "integer",
    "int32",
    "int64",
    "uint32",
    "uint64",
    "boolean",
    "money",
    "currency",
    "decimal",
    "percentage",
    "date",
    "object",
    "array",
    "tuple",
    "enum",
    "literal",
    "never",
    "union",
    "intersection",
    "record",
    "preprocess",
    "catch",
    "ref",
    "any",
  ];

  if (!validTypes.includes(obj.type)) {
    throw new SchemaParseError(`Invalid type: ${obj.type}`, path);
  }

  validateCommonSchemaFields(obj, path);

  // Validate type-specific schema properties (flattened format)
  switch (obj.type) {
    case "string":
      validateStringSchema(obj, path);
      break;
    case "money":
      validateOptionalU8Field(obj, "scale", `${path}/scale`);
      break;
    case "currency":
      validateCurrencySchema(obj, path);
      break;
    case "decimal":
      validateOptionalU8Field(obj, "precision", `${path}/precision`);
      validateOptionalU8Field(obj, "scale", `${path}/scale`);
      break;
    case "percentage":
      validatePercentageSchema(obj, path);
      break;
    case "date":
      if (obj.format !== undefined && typeof obj.format !== "string") {
        throw new SchemaParseError(
          "DateSchema.format must be a string",
          `${path}/format`,
        );
      }
      break;
    case "object":
      validateObjectSchema(obj, path);
      break;
    case "array":
      validateArraySchema(obj, path);
      break;
    case "tuple":
      validateTupleSchema(obj, path);
      break;
    case "enum":
      validateEnumSchema(obj, path);
      break;
    case "literal":
      validateLiteralSchema(obj, path);
      break;
    case "union":
      validateUnionSchema(obj, path);
      break;
    case "intersection":
      validateIntersectionSchema(obj, path);
      break;
    case "record":
      validateRecordSchema(obj, path);
      break;
    case "preprocess":
      validatePreprocessSchema(obj, path);
      break;
    case "catch":
      validateCatchSchema(obj, path);
      break;
    case "ref":
      if (typeof obj.$ref !== "string") {
        throw new SchemaParseError("ref.$ref must be a string", path);
      }
      break;
  }
}

function validateStringSchema(
  obj: Record<string, unknown>,
  path: string,
): void {
  if (obj.example !== undefined && typeof obj.example !== "string") {
    throw new SchemaParseError(
      "StringSchema.example must be a string",
      `${path}/example`,
    );
  }
}

function validateCurrencySchema(
  obj: Record<string, unknown>,
  path: string,
): void {
  if (obj.code !== undefined && typeof obj.code !== "string") {
    throw new SchemaParseError(
      "CurrencySchema.code must be a string",
      `${path}/code`,
    );
  }
  validateOptionalU8Field(obj, "scale", `${path}/scale`);
}

function validatePercentageSchema(
  obj: Record<string, unknown>,
  path: string,
): void {
  if (
    obj.format !== undefined &&
    obj.format !== "decimal" &&
    obj.format !== "whole"
  ) {
    throw new SchemaParseError(
      "PercentageSchema.format must be decimal or whole",
      `${path}/format`,
    );
  }
  if (
    obj.allow_over_100 !== undefined &&
    typeof obj.allow_over_100 !== "boolean"
  ) {
    throw new SchemaParseError(
      "PercentageSchema.allow_over_100 must be a boolean",
      `${path}/allow_over_100`,
    );
  }
  validateOptionalU8Field(obj, "scale", `${path}/scale`);
}

function validateCommonSchemaFields(
  obj: Record<string, unknown>,
  path: string,
): void {
  if (
    typeof obj.description !== "undefined" &&
    typeof obj.description !== "string"
  ) {
    throw new SchemaParseError(
      "description must be a string",
      `${path}/description`,
    );
  }

  if (typeof obj.constraints !== "undefined") {
    if (!schemaTypeSupportsConstraints(obj.type)) {
      throw new SchemaParseError(
        `${obj.type}.constraints is not supported`,
        `${path}/constraints`,
      );
    }
    validateConstraintArray(obj, "constraints", `${path}/constraints`);
  }

  if (obj.transforms !== undefined) {
    if (!schemaTypeSupportsTransforms(obj.type)) {
      throw new SchemaParseError(
        `${obj.type}.transforms is not supported`,
        `${path}/transforms`,
      );
    }
    validateTransformArray(obj.transforms, `${path}/transforms`);
  }
}

function schemaTypeSupportsTransforms(type: unknown): boolean {
  return (
    type === "string" ||
    type === "number" ||
    type === "integer" ||
    type === "int32" ||
    type === "int64" ||
    type === "uint32" ||
    type === "uint64" ||
    type === "money" ||
    type === "currency" ||
    type === "decimal" ||
    type === "percentage" ||
    type === "date" ||
    type === "preprocess"
  );
}

function schemaTypeSupportsConstraints(type: unknown): boolean {
  return (
    type === "string" ||
    type === "number" ||
    type === "integer" ||
    type === "int32" ||
    type === "int64" ||
    type === "uint32" ||
    type === "uint64" ||
    type === "money" ||
    type === "currency" ||
    type === "decimal" ||
    type === "percentage" ||
    type === "date" ||
    type === "array"
  );
}

function validateObjectSchema(value: unknown, path: string): void {
  if (typeof value !== "object" || value === null) {
    throw new SchemaParseError("ObjectSchema must be an object", path);
  }

  const obj = value as Record<string, unknown>;

  // Properties are optional (empty object is valid). `fields` is accepted as
  // a legacy alias for older Rust-emitted IR.
  if (obj.properties !== undefined && obj.fields !== undefined) {
    throw new SchemaParseError(
      "ObjectSchema cannot define both properties and fields",
      path,
    );
  }

  const propertyEntries = [
    ["properties", obj.properties],
    ["fields", obj.fields],
  ] as const;

  for (const [propertyKey, propertyValue] of propertyEntries) {
    if (propertyValue === undefined) continue;

    if (typeof propertyValue !== "object" || propertyValue === null) {
      throw new SchemaParseError(
        `ObjectSchema.${propertyKey} must be an object`,
        path,
      );
    }

    // Each property is a flattened PropertySchema (TypeSchema + optional required field)
    for (const [key, prop] of Object.entries(
      propertyValue as Record<string, unknown>,
    )) {
      if (typeof prop !== "object" || prop === null) {
        throw new SchemaParseError(
          `Property ${key} must be an object`,
          `${path}/${propertyKey}/${key}`,
        );
      }
      // PropertySchema is a flattened TypeSchema, validate it directly
      validateTypeSchema(prop, `${path}/${propertyKey}/${key}`);
      const propObj = prop as Record<string, unknown>;
      if (
        propObj.required !== undefined &&
        typeof propObj.required !== "boolean"
      ) {
        throw new SchemaParseError(
          "Property.required must be a boolean",
          `${path}/${propertyKey}/${key}/required`,
        );
      }
      validatePropertyMetadata(propObj, `${path}/${propertyKey}/${key}`);
    }
  }

  if (obj.unknown_keys !== undefined) {
    if (
      obj.unknown_keys !== "strict" &&
      obj.unknown_keys !== "passthrough" &&
      obj.unknown_keys !== "strip"
    ) {
      throw new SchemaParseError(
        "ObjectSchema.unknown_keys must be one of strict|passthrough|strip",
        path,
      );
    }
  }

  if (
    obj.additional_properties !== undefined &&
    typeof obj.additional_properties !== "boolean"
  ) {
    throw new SchemaParseError(
      "ObjectSchema.additional_properties must be a boolean",
      `${path}/additional_properties`,
    );
  }

  if (obj.catchall !== undefined) {
    validateTypeSchema(obj.catchall, `${path}/catchall`);
  }

  if (obj.rules !== undefined) {
    validateConstraintArray(obj, "rules", `${path}/rules`);
  }

  if (obj.pages !== undefined) {
    validatePageMap(obj.pages, `${path}/pages`);
  }
  if (obj.acroform_mappings !== undefined) {
    validateAcroFormMappings(
      obj.acroform_mappings,
      `${path}/acroform_mappings`,
    );
  }
}

function validateArraySchema(value: unknown, path: string): void {
  // Value is already the TypeSchema object with type: "array"
  const obj = value as Record<string, unknown>;

  if (!obj.items) {
    throw new SchemaParseError("ArraySchema.items is required", path);
  }

  validateTypeSchema(obj.items, `${path}/items`);

  if (obj.min_items !== undefined) {
    validateNonNegativeInteger(
      obj.min_items,
      "ArraySchema.min_items",
      `${path}/min_items`,
    );
  }
  if (obj.max_items !== undefined) {
    validateNonNegativeInteger(
      obj.max_items,
      "ArraySchema.max_items",
      `${path}/max_items`,
    );
  }
}

function validateEnumSchema(value: unknown, path: string): void {
  // Value is already the TypeSchema object with type: "enum"
  const obj = value as Record<string, unknown>;

  if (!Array.isArray(obj.values)) {
    throw new SchemaParseError("EnumSchema.values must be an array", path);
  }
}

function validateTupleSchema(value: unknown, path: string): void {
  const obj = value as Record<string, unknown>;

  if (!Array.isArray(obj.items)) {
    throw new SchemaParseError("TupleSchema.items must be an array", path);
  }

  for (let i = 0; i < obj.items.length; i++) {
    validateTypeSchema(obj.items[i], `${path}/items/${i}`);
  }
}

function validateLiteralSchema(value: unknown, path: string): void {
  const obj = value as Record<string, unknown>;

  if (!("value" in obj)) {
    throw new SchemaParseError("LiteralSchema.value is required", path);
  }
}

function validateUnionSchema(value: unknown, path: string): void {
  // Value is already the TypeSchema object with type: "union"
  const obj = value as Record<string, unknown>;

  // Rust uses "oneOf" for union variants
  if (!Array.isArray(obj.oneOf)) {
    throw new SchemaParseError("UnionSchema.oneOf must be an array", path);
  }

  for (let i = 0; i < obj.oneOf.length; i++) {
    validateTypeSchema(obj.oneOf[i], `${path}/oneOf/${i}`);
  }

  if (
    obj.discriminator !== undefined &&
    typeof obj.discriminator !== "string"
  ) {
    throw new SchemaParseError(
      "UnionSchema.discriminator must be a string",
      `${path}/discriminator`,
    );
  }
}

function validateIntersectionSchema(value: unknown, path: string): void {
  const obj = value as Record<string, unknown>;

  if (!Array.isArray(obj.allOf)) {
    throw new SchemaParseError(
      "IntersectionSchema.allOf must be an array",
      path,
    );
  }

  for (let i = 0; i < obj.allOf.length; i++) {
    validateTypeSchema(obj.allOf[i], `${path}/allOf/${i}`);
  }
}

function validateRecordSchema(value: unknown, path: string): void {
  const obj = value as Record<string, unknown>;

  if (!obj.value) {
    throw new SchemaParseError("RecordSchema.value is required", path);
  }
  validateTypeSchema(obj.value, `${path}/value`);

  if (obj.key !== undefined) {
    validateTypeSchema(obj.key, `${path}/key`);
  }

  if (obj.partial !== undefined && typeof obj.partial !== "boolean") {
    throw new SchemaParseError(
      "RecordSchema.partial must be a boolean",
      `${path}/partial`,
    );
  }
}

function validatePreprocessSchema(value: unknown, path: string): void {
  const obj = value as Record<string, unknown>;

  if (!obj.schema) {
    throw new SchemaParseError("PreprocessSchema.schema is required", path);
  }
  validateTypeSchema(obj.schema, `${path}/schema`);

  if (obj.transforms !== undefined && !Array.isArray(obj.transforms)) {
    throw new SchemaParseError(
      "PreprocessSchema.transforms must be an array",
      path,
    );
  }
}

function validateCatchSchema(value: unknown, path: string): void {
  const obj = value as Record<string, unknown>;

  if (!obj.schema) {
    throw new SchemaParseError("CatchSchema.schema is required", path);
  }
  if (!("value" in obj)) {
    throw new SchemaParseError("CatchSchema.value is required", path);
  }
  validateTypeSchema(obj.schema, `${path}/schema`);
}

function validatePropertyMetadata(
  obj: Record<string, unknown>,
  path: string,
): void {
  for (const key of ["label", "section"] as const) {
    if (obj[key] !== undefined && typeof obj[key] !== "string") {
      throw new SchemaParseError(
        `Property.${key} must be a string`,
        `${path}/${key}`,
      );
    }
  }

  if (obj.render !== undefined) {
    validateRenderMetadata(obj.render, `${path}/render`);
  }
  if (obj.acroform !== undefined) {
    validateAcroFormMetadata(obj.acroform, `${path}/acroform`);
  }
}

function validatePageMap(value: unknown, path: string): void {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new SchemaParseError("ObjectSchema.pages must be an object", path);
  }

  for (const [name, page] of Object.entries(value as Record<string, unknown>)) {
    const pagePath = `${path}/${name}`;
    const obj = requireObject(page, "PageSchema", pagePath);
    if (obj.name !== undefined && typeof obj.name !== "string") {
      throw new SchemaParseError(
        "PageSchema.name must be a string",
        `${pagePath}/name`,
      );
    }
    if (obj.fields !== undefined) {
      if (
        typeof obj.fields !== "object" ||
        obj.fields === null ||
        Array.isArray(obj.fields)
      ) {
        throw new SchemaParseError(
          "PageSchema.fields must be an object",
          `${pagePath}/fields`,
        );
      }
      for (const [fieldName, render] of Object.entries(
        obj.fields as Record<string, unknown>,
      )) {
        validateRenderMetadata(render, `${pagePath}/fields/${fieldName}`);
      }
    }
  }
}

function validateAcroFormMappings(value: unknown, path: string): void {
  if (!Array.isArray(value)) {
    throw new SchemaParseError(
      "ObjectSchema.acroform_mappings must be an array",
      path,
    );
  }

  for (let i = 0; i < value.length; i++) {
    const mappingPath = `${path}/${i}`;
    const obj = requireObject(
      value[i],
      "AcroFormCompositionMetadata",
      mappingPath,
    );
    requireStringField(obj, "field_id", `${mappingPath}/field_id`);
    validateNonNegativeInteger(
      obj.page,
      "AcroFormCompositionMetadata.page",
      `${mappingPath}/page`,
    );
    if (obj.copy !== undefined && typeof obj.copy !== "string") {
      throw new SchemaParseError(
        "AcroFormCompositionMetadata.copy must be a string",
        `${mappingPath}/copy`,
      );
    }
    if (obj.compose !== undefined) {
      validateStringArray(
        obj.compose,
        "AcroFormCompositionMetadata.compose",
        `${mappingPath}/compose`,
      );
    }
    for (const key of ["separator", "format"] as const) {
      if (obj[key] !== undefined && typeof obj[key] !== "string") {
        throw new SchemaParseError(
          `AcroFormCompositionMetadata.${key} must be a string`,
          `${mappingPath}/${key}`,
        );
      }
    }
  }
}

function validateRenderMetadata(value: unknown, path: string): void {
  const obj = requireObject(value, "RenderMetadata", path);
  requireStringField(obj, "type", `${path}/type`);
  requireNumberField(obj, "x", `${path}/x`);
  requireNumberField(obj, "y", `${path}/y`);

  for (const key of [
    "font",
    "color",
    "align",
    "v_align",
    "box_number",
  ] as const) {
    if (obj[key] !== undefined && typeof obj[key] !== "string") {
      throw new SchemaParseError(
        `RenderMetadata.${key} must be a string`,
        `${path}/${key}`,
      );
    }
  }

  if (obj.page !== undefined) {
    validateNonNegativeInteger(obj.page, "RenderMetadata.page", `${path}/page`);
  }

  for (const key of [
    "font_size",
    "h_scale",
    "width",
    "height",
    "max_width",
    "line_height",
  ] as const) {
    if (obj[key] !== undefined && !isFiniteNumber(obj[key])) {
      throw new SchemaParseError(
        `RenderMetadata.${key} must be a number`,
        `${path}/${key}`,
      );
    }
  }

  if (obj.multiline !== undefined && typeof obj.multiline !== "boolean") {
    throw new SchemaParseError(
      "RenderMetadata.multiline must be a boolean",
      `${path}/multiline`,
    );
  }
}

function validateAcroFormMetadata(value: unknown, path: string): void {
  const obj = requireObject(value, "AcroFormMetadata", path);
  requireStringField(obj, "field_id", `${path}/field_id`);
  requireStringField(obj, "field_type", `${path}/field_type`);
  if (obj.copy_suffix !== undefined && typeof obj.copy_suffix !== "string") {
    throw new SchemaParseError(
      "AcroFormMetadata.copy_suffix must be a string",
      `${path}/copy_suffix`,
    );
  }
}

function validateTransformArray(value: unknown, path: string): void {
  if (!Array.isArray(value)) {
    throw new SchemaParseError("transforms must be an array", path);
  }

  for (let i = 0; i < value.length; i++) {
    validateTransform(value[i], `${path}/${i}`);
  }
}

function validateTransform(value: unknown, path: string): void {
  const obj = requireObject(value, "Transform", path);

  if (typeof obj.fn !== "string") {
    throw new SchemaParseError("Transform.fn must be a string", `${path}/fn`);
  }

  switch (obj.fn) {
    case "trim":
    case "collapse_whitespace":
    case "digits_only":
    case "upper":
    case "lower":
    case "normalize_flight_number":
    case "normalize_icd10":
    case "normalize_cpt":
    case "normalize_hcpcs":
    case "normalize_ndc11":
    case "phone_us":
    case "phone_e164":
    case "card_mask_last4":
    case "format_ssn":
    case "format_ein":
    case "mask_ssn":
    case "mask_ein":
    case "format_iban":
    case "format_credit_card":
      return;
    case "money_to_cents":
      if (obj.scale !== undefined) {
        validateU8(
          obj.scale,
          "Transform.money_to_cents.scale",
          `${path}/scale`,
        );
      }
      return;
    case "date_parse":
      requireStringField(obj, "format", `${path}/format`);
      return;
    case "replace":
      requireStringField(obj, "pattern", `${path}/pattern`);
      requireStringField(obj, "replacement", `${path}/replacement`);
      return;
    case "default":
      if (!("value" in obj)) {
        throw new SchemaParseError(
          "Transform.default.value is required",
          `${path}/value`,
        );
      }
      return;
    case "format_thousands":
      if (obj.separator !== undefined && typeof obj.separator !== "string") {
        throw new SchemaParseError(
          "Transform.format_thousands.separator must be a string",
          `${path}/separator`,
        );
      }
      return;
    case "format_decimal":
      validateU8(
        obj.places,
        "Transform.format_decimal.places",
        `${path}/places`,
      );
      return;
    default:
      throw new SchemaParseError(`Invalid transform: ${obj.fn}`, path);
  }
}

function validateConstraintArray(
  owner: Record<string, unknown>,
  key: string,
  path: string,
): void {
  const constraints = owner[key];
  if (!Array.isArray(constraints)) {
    throw new SchemaParseError(`${key} must be an array`, path);
  }

  for (let i = 0; i < constraints.length; i++) {
    constraints[i] = normalizeAndValidateConstraint(
      constraints[i],
      `${path}/${i}`,
    );
  }
}

function normalizeAndValidateConstraint(value: unknown, path: string): unknown {
  const obj = requireObject(value, "Constraint", path);

  if (obj.pred !== undefined) {
    if (obj.id !== undefined && typeof obj.id !== "string") {
      throw new SchemaParseError(
        "Constraint.id must be a string",
        `${path}/id`,
      );
    }
    validatePredicate(obj.pred, `${path}/pred`);
    validateErrorMeta(obj.error, `${path}/error`);
    return value;
  }

  if (typeof obj.type === "string") {
    return normalizeTemplateConstraint(obj, path);
  }

  throw new SchemaParseError(
    "Constraint must have either pred or template type",
    path,
  );
}

function normalizeTemplateConstraint(
  obj: Record<string, unknown>,
  path: string,
): unknown {
  const message =
    obj.message === undefined
      ? undefined
      : requireString(obj.message, `${path}/message`);
  const source =
    obj.source === undefined
      ? undefined
      : requireString(obj.source, `${path}/source`);

  switch (obj.type) {
    case "pattern": {
      const pattern = requireString(obj.value, `${path}/value`);
      return {
        pred: { type: "regex", pattern },
        error: {
          code: "PATTERN_MISMATCH",
          message: message ?? `Must match pattern ${pattern}`,
          severity: "error",
          source,
        },
      };
    }
    case "maxLength": {
      validateNonNegativeInteger(
        obj.value,
        "TemplateConstraint.maxLength.value",
        `${path}/value`,
      );
      return {
        pred: { type: "max_len", len: obj.value },
        error: {
          code: "MAX_LENGTH_EXCEEDED",
          message: message ?? `Must be at most ${obj.value} characters`,
          severity: "error",
          source,
        },
      };
    }
    case "minLength": {
      validateNonNegativeInteger(
        obj.value,
        "TemplateConstraint.minLength.value",
        `${path}/value`,
      );
      return {
        pred: { type: "min_len", len: obj.value },
        error: {
          code: "MIN_LENGTH_NOT_MET",
          message: message ?? `Must be at least ${obj.value} characters`,
          severity: "error",
          source,
        },
      };
    }
    case "format": {
      const format = requireString(obj.value, `${path}/value`);
      return {
        pred: { type: "call", name: `format:${format}` },
        error: {
          code: "FORMAT_INVALID",
          message: message ?? `Must be in ${format} format`,
          severity: "error",
          source,
        },
      };
    }
    case "enum": {
      const values = obj.values ?? obj.value;
      if (!Array.isArray(values)) {
        throw new SchemaParseError(
          "TemplateConstraint.enum.values must be an array",
          `${path}/values`,
        );
      }
      return {
        pred: { type: "in", path: "", values },
        error: {
          code: "INVALID_ENUM_VALUE",
          message: message ?? `Must be one of: ${JSON.stringify(values)}`,
          severity: "error",
          source,
        },
      };
    }
    default:
      throw new SchemaParseError(
        `Invalid template constraint: ${obj.type}`,
        path,
      );
  }
}

function validatePredicate(value: unknown, path: string): void {
  const obj = requireObject(value, "Predicate", path);

  if (typeof obj.type !== "string") {
    throw new SchemaParseError(
      "Predicate.type must be a string",
      `${path}/type`,
    );
  }

  switch (obj.type) {
    case "true":
    case "false":
      return;
    case "regex":
      requireStringField(obj, "pattern", `${path}/pattern`);
      if (obj.flags !== undefined && typeof obj.flags !== "string") {
        throw new SchemaParseError(
          "Predicate.regex.flags must be a string",
          `${path}/flags`,
        );
      }
      return;
    case "template_literal":
      validateTemplateLiteralParts(obj.parts, `${path}/parts`);
      return;
    case "min_len":
    case "max_len":
      validateNonNegativeInteger(
        obj.len,
        `Predicate.${obj.type}.len`,
        `${path}/len`,
      );
      return;
    case "range":
      if (obj.min !== undefined && !isFiniteNumber(obj.min)) {
        throw new SchemaParseError(
          "Predicate.range.min must be a number",
          `${path}/min`,
        );
      }
      if (obj.max !== undefined && !isFiniteNumber(obj.max)) {
        throw new SchemaParseError(
          "Predicate.range.max must be a number",
          `${path}/max`,
        );
      }
      return;
    case "exists":
      requireStringField(obj, "path", `${path}/path`);
      return;
    case "eq":
      requireStringField(obj, "path", `${path}/path`);
      if (!("value" in obj)) {
        throw new SchemaParseError(
          "Predicate.eq.value is required",
          `${path}/value`,
        );
      }
      return;
    case "in":
      requireStringField(obj, "path", `${path}/path`);
      if (!Array.isArray(obj.values)) {
        throw new SchemaParseError(
          "Predicate.in.values must be an array",
          `${path}/values`,
        );
      }
      return;
    case "required_with":
      requireStringField(obj, "field", `${path}/field`);
      requireStringField(obj, "with", `${path}/with`);
      return;
    case "required_without":
      requireStringField(obj, "field", `${path}/field`);
      requireStringField(obj, "without", `${path}/without`);
      return;
    case "exactly_one_of":
      validateStringArray(
        obj.paths,
        "Predicate.exactly_one_of.paths",
        `${path}/paths`,
      );
      return;
    case "eq_fields":
    case "gt_field":
    case "gte_field":
    case "lt_field":
    case "lte_field":
      requireStringField(obj, "left", `${path}/left`);
      requireStringField(obj, "right", `${path}/right`);
      return;
    case "sum_equals":
      validateStringArray(
        obj.paths,
        "Predicate.sum_equals.paths",
        `${path}/paths`,
      );
      requireStringField(obj, "target", `${path}/target`);
      return;
    case "sum_equals_value":
      validateStringArray(
        obj.paths,
        "Predicate.sum_equals_value.paths",
        `${path}/paths`,
      );
      requireNumberField(obj, "value", `${path}/value`);
      return;
    case "and":
    case "or":
      validatePredicateArray(obj.predicates, `${path}/predicates`);
      return;
    case "not":
      validatePredicate(obj.predicate, `${path}/predicate`);
      return;
    case "implies":
      validatePredicate(obj.if, `${path}/if`);
      validatePredicate(obj.then, `${path}/then`);
      return;
    case "call":
      requireStringField(obj, "name", `${path}/name`);
      return;
    default:
      throw new SchemaParseError(`Invalid predicate: ${obj.type}`, path);
  }
}

function validateTemplateLiteralParts(value: unknown, path: string): void {
  if (!Array.isArray(value)) {
    throw new SchemaParseError(
      "Predicate.template_literal.parts must be an array",
      path,
    );
  }

  const kinds = new Set([
    "literal",
    "digits",
    "ascii_letters",
    "ascii_alphanumeric",
    "uppercase",
    "lowercase",
    "hex",
  ]);

  for (let i = 0; i < value.length; i++) {
    const partPath = `${path}/${i}`;
    const part = requireObject(value[i], "TemplateLiteralPart", partPath);
    if (typeof part.kind !== "string" || !kinds.has(part.kind)) {
      throw new SchemaParseError(
        "Invalid template literal part kind",
        `${partPath}/kind`,
      );
    }
    if (part.kind === "literal") {
      requireStringField(part, "value", `${partPath}/value`);
      continue;
    }
    if (part.min !== undefined) {
      validateNonNegativeInteger(
        part.min,
        "TemplateLiteralPart.min",
        `${partPath}/min`,
      );
    }
    if (part.max !== undefined) {
      validateNonNegativeInteger(
        part.max,
        "TemplateLiteralPart.max",
        `${partPath}/max`,
      );
    }
  }
}

function validatePredicateArray(value: unknown, path: string): void {
  if (!Array.isArray(value)) {
    throw new SchemaParseError("Predicate list must be an array", path);
  }
  for (let i = 0; i < value.length; i++) {
    validatePredicate(value[i], `${path}/${i}`);
  }
}

function validateErrorMeta(value: unknown, path: string): void {
  const obj = requireObject(value, "ErrorMeta", path);
  requireStringField(obj, "code", `${path}/code`);
  requireStringField(obj, "message", `${path}/message`);

  if (
    obj.severity !== undefined &&
    obj.severity !== "error" &&
    obj.severity !== "warning"
  ) {
    throw new SchemaParseError(
      "ErrorMeta.severity must be error or warning",
      `${path}/severity`,
    );
  }

  for (const key of ["path", "help", "source"] as const) {
    if (obj[key] !== undefined && typeof obj[key] !== "string") {
      throw new SchemaParseError(
        `ErrorMeta.${key} must be a string`,
        `${path}/${key}`,
      );
    }
  }
}

function requireObject(
  value: unknown,
  name: string,
  path: string,
): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new SchemaParseError(`${name} must be an object`, path);
  }
  return value as Record<string, unknown>;
}

function requireString(value: unknown, path: string): string {
  if (typeof value !== "string") {
    throw new SchemaParseError("value must be a string", path);
  }
  return value;
}

function requireStringField(
  obj: Record<string, unknown>,
  key: string,
  path: string,
): void {
  if (typeof obj[key] !== "string") {
    throw new SchemaParseError(`${key} must be a string`, path);
  }
}

function requireNumberField(
  obj: Record<string, unknown>,
  key: string,
  path: string,
): void {
  if (!isFiniteNumber(obj[key])) {
    throw new SchemaParseError(`${key} must be a number`, path);
  }
}

function validateStringArray(value: unknown, name: string, path: string): void {
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string")) {
    throw new SchemaParseError(`${name} must be an array of strings`, path);
  }
}

function validateStringRecord(
  value: unknown,
  name: string,
  path: string,
): void {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new SchemaParseError(`${name} must be an object`, path);
  }
  for (const [key, item] of Object.entries(value as Record<string, unknown>)) {
    if (typeof item !== "string") {
      throw new SchemaParseError(
        `${name}.${key} must be a string`,
        `${path}/${key}`,
      );
    }
  }
}

function validateNonNegativeInteger(
  value: unknown,
  name: string,
  path: string,
): void {
  if (!Number.isSafeInteger(value) || (value as number) < 0) {
    throw new SchemaParseError(`${name} must be a non-negative integer`, path);
  }
}

function validateU8(value: unknown, name: string, path: string): void {
  if (
    !Number.isSafeInteger(value) ||
    (value as number) < 0 ||
    (value as number) > 255
  ) {
    throw new SchemaParseError(
      `${name} must be an integer from 0 to 255`,
      path,
    );
  }
}

function validateOptionalU8Field(
  obj: Record<string, unknown>,
  key: string,
  path: string,
): void {
  if (obj[key] !== undefined) {
    validateU8(obj[key], key, path);
  }
}

function isFiniteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}
