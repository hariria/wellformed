/**
 * Builder helpers for IRS form schemas.
 */

import type {
  AcroFormFieldComposition,
  IrsFieldSchema,
  IrsFieldType,
  IrsFormMeta,
  IrsFormSchema,
  PageFields,
  PdfFieldPosition,
  PdfTemplateMeta,
  SectionMeta,
} from "./irs-types.js";

// ============================================================================
// Field Builder
// ============================================================================

export class FieldBuilder {
  private schema: IrsFieldSchema;

  constructor(type: IrsFieldType, description: string) {
    this.schema = { type, description };
  }

  required(value = true): this {
    this.schema.required = value;
    return this;
  }

  optional(): this {
    this.schema.required = false;
    return this;
  }

  example(value: string): this {
    this.schema.example = value;
    return this;
  }

  section(name: string): this {
    this.schema.section = name;
    return this;
  }

  label(text: string): this {
    this.schema.label = text;
    return this;
  }

  values(vals: string[]): this {
    this.schema.values = vals;
    return this;
  }

  build(): IrsFieldSchema {
    return this.schema;
  }
}

// Field factory functions
export const f = {
  string: (description: string) => new FieldBuilder("string", description),
  uint32: (description: string) => new FieldBuilder("uint32", description),
  int32: (description: string) => new FieldBuilder("int32", description),
  money: (description: string) => new FieldBuilder("money", description),
  boolean: (description: string) => new FieldBuilder("boolean", description),
  date: (description: string) => new FieldBuilder("date", description),
  enum: (description: string, values: string[]) =>
    new FieldBuilder("enum", description).values(values),
};

// ============================================================================
// Form Builder
// ============================================================================

export class IrsFormBuilder {
  private schema: IrsFormSchema;

  constructor() {
    this.schema = {
      version: "1.0.0",
      irs_form: {
        name: "",
        title: "",
        revision: "",
        revision_date: "",
        omb_number: "",
        cat_number: "",
      },
      pdf_template: {
        hash: null,
        path: "",
        source_uri: "",
      },
      id: "",
      title: "",
      description: "",
      sections: {},
      root: {
        type: "object",
        description: "",
        fields: {},
      },
    };
  }

  version(v: string): this {
    this.schema.version = v;
    return this;
  }

  irsForm(meta: IrsFormMeta): this {
    this.schema.irs_form = meta;
    return this;
  }

  pdfTemplate(meta: PdfTemplateMeta): this {
    this.schema.pdf_template = meta;
    return this;
  }

  id(id: string): this {
    this.schema.id = id;
    return this;
  }

  title(title: string): this {
    this.schema.title = title;
    return this;
  }

  description(desc: string): this {
    this.schema.description = desc;
    this.schema.root.description = desc;
    return this;
  }

  section(name: string, meta: SectionMeta): this {
    this.schema.sections[name] = meta;
    return this;
  }

  field(name: string, builder: FieldBuilder): this {
    this.schema.root.fields[name] = builder.build();
    return this;
  }

  fields(fields: Record<string, FieldBuilder>): this {
    for (const [name, builder] of Object.entries(fields)) {
      this.schema.root.fields[name] = builder.build();
    }
    return this;
  }

  page(pageNum: number, page: PageFields): this {
    if (!this.schema.root.pages) {
      this.schema.root.pages = {};
    }
    this.schema.root.pages[String(pageNum)] = page;
    return this;
  }

  /**
   * Define how IRIS fields compose into an AcroForm PDF field.
   * @param fieldId - The AcroForm field ID (e.g., "f1_1[0]")
   * @param mapping - Composition details
   */
  acroformField(
    fieldId: string,
    mapping: Omit<AcroFormFieldComposition, "field_id">,
  ): this {
    if (!this.schema.root.acroform_mappings) {
      this.schema.root.acroform_mappings = [];
    }
    this.schema.root.acroform_mappings.push({
      field_id: fieldId,
      ...mapping,
    });
    return this;
  }

  build(): IrsFormSchema {
    return this.schema;
  }

  toJSON(pretty = true): string {
    return JSON.stringify(this.schema, null, pretty ? 2 : undefined);
  }
}

export function defineForm(): IrsFormBuilder {
  return new IrsFormBuilder();
}

// ============================================================================
// Helpers for PDF positioning
// ============================================================================

export function textField(
  x: number,
  y: number,
  width: number,
  height: number,
  opts?: {
    font_size?: number;
    align?: "left" | "center" | "right";
    multiline?: boolean;
  },
): PdfFieldPosition {
  return {
    type: "text",
    x,
    y,
    width,
    height,
    ...opts,
  };
}

export function checkbox(x: number, y: number, size = 10): PdfFieldPosition {
  return {
    type: "checkbox",
    x,
    y,
    width: size,
    height: size,
  };
}
