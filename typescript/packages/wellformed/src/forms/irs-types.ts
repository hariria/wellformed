/**
 * Extended types for IRS form schemas.
 *
 * These extend the base wellformed schema with IRS-specific metadata
 * for form rendering, PDF mapping, and section organization.
 */

import type { Constraint, Transform } from "../ir/types.js";

// ============================================================================
// IRS Form Metadata
// ============================================================================

export interface IrsFormMeta {
  /** Form number (e.g., "1099-INT") */
  name: string;
  /** Form title (e.g., "Interest Income") */
  title: string;
  /** Revision string (e.g., "Rev. January 2024") */
  revision: string;
  /** Revision date in YYYY-MM format */
  revision_date: string;
  /** OMB control number */
  omb_number: string;
  /** IRS catalog number */
  cat_number: string;
}

export interface PdfTemplateMeta {
  /** SHA-256 hash of the template PDF */
  hash: string | null;
  /** Relative path to the template PDF */
  path: string;
  /** Source URI for the original IRS PDF */
  source_uri: string;
}

export interface SectionMeta {
  /** Display title */
  title: string;
  /** Display order (1-based) */
  order: number;
  /** Description of the section */
  description: string;
}

// ============================================================================
// AcroForm Mapping
// ============================================================================

export type AcroFormFieldType = "text" | "checkbox";

export interface AcroFormMapping {
  /** AcroForm field ID in the PDF */
  field_id: string;
  /** Field type */
  field_type: AcroFormFieldType;
  /** Copy suffix for multi-copy forms (e.g., "0", "1") */
  copy_suffix: string;
}

// ============================================================================
// PDF Field Positioning
// ============================================================================

export type PdfFieldType = "text" | "checkbox";
export type TextAlign = "left" | "center" | "right";

export interface PdfFieldPosition {
  type: PdfFieldType;
  x: number;
  y: number;
  width: number;
  height: number;
  font_size?: number;
  align?: TextAlign;
  multiline?: boolean;
}

export interface PageFields {
  name: string;
  fields: Record<string, PdfFieldPosition>;
}

// ============================================================================
// Extended Field Schema
// ============================================================================

export type IrsFieldType =
  | "string"
  | "uint32"
  | "int32"
  | "money"
  | "boolean"
  | "date"
  | "enum";

export interface IrsFieldSchema {
  type: IrsFieldType;
  description: string;
  required?: boolean;
  example?: string;
  section?: string;
  /** Label for display (e.g., "Payer's TIN") */
  label?: string;
  transforms?: Transform[];
  constraints?: Constraint[];
  // For enum types
  values?: string[];
}

// ============================================================================
// AcroForm Field Composition
// ============================================================================

/**
 * Defines how one or more IRIS fields compose into a single AcroForm field.
 * The field ID (key) is the IRIS column letter.
 */
export interface AcroFormFieldComposition {
  /** AcroForm field ID in the PDF (e.g., "f1_1[0]") */
  field_id: string;
  /** Page number (1-indexed) */
  page: number;
  /** Copy identifier (e.g., "A", "B", "C") */
  copy: string;
  /** IRIS columns to compose, in order */
  compose: string[];
  /** Format template (e.g., "{F}\n{G}" or "{O}, {P} {Q}") */
  format?: string;
  /** Default separator if no format specified */
  separator?: string;
}

// ============================================================================
// Full IRS Form Schema
// ============================================================================

export interface IrsFormSchema {
  version: string;
  irs_form: IrsFormMeta;
  pdf_template: PdfTemplateMeta;
  id: string;
  title: string;
  description: string;
  sections: Record<string, SectionMeta>;
  root: {
    type: "object";
    description: string;
    /** Fields keyed by IRIS column letter (e.g., "A", "BL") */
    fields: Record<string, IrsFieldSchema>;
    /** How fields compose into AcroForm PDF fields */
    acroform_mappings?: AcroFormFieldComposition[];
    /** Page-level field positioning for rendering */
    pages?: Record<string, PageFields>;
  };
}
