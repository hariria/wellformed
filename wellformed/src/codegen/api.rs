//! API code generation from wellformed schemas.
//!
//! Generates storage-agnostic API handlers, request/response DTOs,
//! repository traits, and a clean OpenAPI spec.

use crate::ir::{Schema, TypeSchema};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

use super::util::{derive_struct_name, escape_keyword, to_pascal_case, to_snake_case};

/// Configuration for API metadata used in OpenAPI spec generation.
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// API title (e.g., "wellformed Forms API")
    pub title: String,
    /// API version (e.g., "0.1.0")
    pub version: String,
    /// API description
    pub description: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            title: "wellformed Forms API".to_string(),
            version: "0.1.0".to_string(),
            description: "CRUD API for generated wellformed resources".to_string(),
        }
    }
}

impl ApiConfig {
    /// Create a new API config with custom values.
    #[allow(dead_code)]
    pub fn new(
        title: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            version: version.into(),
            description: description.into(),
        }
    }
}

/// Generate a clean OpenAPI spec JSON from a wellformed schema with default config.
pub fn generate_openapi_spec(schema: &Schema, api_prefix: &str) -> String {
    generate_openapi_spec_with_config(schema, api_prefix, &ApiConfig::default())
}

/// Generate a clean OpenAPI spec JSON from a wellformed schema with custom config.
pub fn generate_openapi_spec_with_config(
    schema: &Schema,
    api_prefix: &str,
    config: &ApiConfig,
) -> String {
    use serde_json::{json, Map};

    let root_name = schema
        .id
        .as_ref()
        .map(|id| derive_struct_name(id))
        .unwrap_or_else(|| "Root".to_string());

    let route_path = schema
        .irs_form
        .as_ref()
        .map(|form| form.name.to_lowercase().replace('-', "/"))
        .or_else(|| schema.id.as_ref().map(|id| id.replace('_', "/")))
        .unwrap_or_else(|| "form".to_string());

    let api_tag = schema
        .irs_form
        .as_ref()
        .map(|form| format!("Form {}: Single", form.name))
        .unwrap_or_else(|| format!("{}: Single", root_name));

    // Separate tag for bulk operations
    let bulk_tag = schema
        .irs_form
        .as_ref()
        .map(|form| format!("Form {}: Bulk", form.name))
        .unwrap_or_else(|| format!("{}: Bulk", root_name));

    let full_path = format!("{}/{}", api_prefix, route_path);
    let snake_name = to_snake_case(&root_name);

    // Check if PDF template is available (filename preferred, path as fallback)
    let has_pdf = schema
        .pdf_template
        .as_ref()
        .and_then(|t| t.filename.as_ref().or(t.path.as_ref()))
        .is_some();

    // Schema refs
    let create_request_ref = format!("#/components/schemas/Create{}Request", root_name);
    let update_request_ref = format!("#/components/schemas/Update{}Request", root_name);
    let response_ref = format!("#/components/schemas/{}Response", root_name);
    let fields_ref = format!("#/components/schemas/{}Fields", root_name);
    let error_ref = "#/components/schemas/ErrorResponse";

    // Build field properties
    let mut field_properties = Map::new();
    if let TypeSchema::Object(obj) = &schema.root {
        for field_name in obj.fields.keys() {
            let rust_field = rust_field_name(field_name);
            let example = generate_field_example(&rust_field);
            field_properties.insert(rust_field, json!({ "type": "string", "example": example }));
        }
    }

    // Helper to create a JSON response
    let json_response = |description: &str, schema_ref: &str| {
        json!({
            "description": description,
            "content": {
                "application/json": {
                    "schema": { "$ref": schema_ref }
                }
            }
        })
    };

    // Helper for PDF response
    let pdf_response = || {
        json!({
            "description": "PDF document",
            "content": {
                "application/pdf": {
                    "schema": { "type": "string", "format": "binary" }
                }
            }
        })
    };

    // Helper for id path parameter
    let id_param = || {
        json!({
            "name": "id",
            "in": "path",
            "required": true,
            "schema": { "type": "string" }
        })
    };

    // Helper for page query parameter
    let page_param = || {
        json!({
            "name": "page",
            "in": "query",
            "required": false,
            "description": "Page number (0-indexed). Omit to render all pages with fields.",
            "schema": { "type": "integer" }
        })
    };

    // Build paths
    let mut paths = Map::new();

    // POST /forms/{type} - Create
    paths.insert(
        full_path.clone(),
        json!({
            "post": {
                "tags": [&api_tag],
                "summary": "Create form",
                "operationId": format!("create_{}", snake_name),
                "requestBody": {
                    "required": true,
                    "content": {
                        "application/json": {
                            "schema": { "$ref": &create_request_ref }
                        }
                    }
                },
                "responses": {
                    "201": json_response("Created", &response_ref),
                    "400": json_response("Bad request", error_ref),
                    "400": json_response("Validation error", error_ref)
                }
            }
        }),
    );

    // GET/PATCH/DELETE /forms/{type}/{id}
    paths.insert(
        format!("{}/{{id}}", full_path),
        json!({
            "get": {
                "tags": [&api_tag],
                "summary": "Get form",
                "operationId": format!("get_{}", snake_name),
                "parameters": [id_param()],
                "responses": {
                    "200": json_response("Found", &response_ref),
                    "404": json_response("Not found", error_ref)
                }
            },
            "patch": {
                "tags": [&api_tag],
                "summary": "Update form",
                "operationId": format!("update_{}", snake_name),
                "parameters": [id_param()],
                "requestBody": {
                    "required": true,
                    "content": {
                        "application/json": {
                            "schema": { "$ref": &update_request_ref }
                        }
                    }
                },
                "responses": {
                    "200": json_response("Updated", &response_ref),
                    "400": json_response("Bad request", error_ref),
                    "404": json_response("Not found", error_ref),
                    "400": json_response("Validation error", error_ref)
                }
            },
            "delete": {
                "tags": [&api_tag],
                "summary": "Delete form",
                "operationId": format!("delete_{}", snake_name),
                "parameters": [id_param()],
                "responses": {
                    "204": { "description": "Deleted" },
                    "404": json_response("Not found", error_ref)
                }
            }
        }),
    );

    // PDF endpoints (only if pdf_template.path is specified)
    if has_pdf {
        // GET /{id}/pdf - render stored form as PDF
        paths.insert(
            format!("{}/{{id}}/pdf", full_path),
            json!({
                "get": {
                    "tags": [&api_tag],
                    "summary": "Render form as PDF",
                    "operationId": format!("render_{}_pdf", snake_name),
                    "parameters": [id_param(), page_param()],
                    "responses": {
                        "200": pdf_response(),
                        "404": json_response("Not found", error_ref)
                    }
                }
            }),
        );

        // POST /pdf - render PDF from request body
        paths.insert(
            format!("{}/pdf", full_path),
            json!({
                "post": {
                    "tags": [&api_tag],
                    "summary": "Render PDF from data",
                    "operationId": format!("render_{}_pdf_from_body", snake_name),
                    "parameters": [page_param()],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": &create_request_ref }
                            }
                        }
                    },
                    "responses": {
                        "200": pdf_response(),
                        "400": json_response("Bad request", error_ref),
                        "400": json_response("Validation error", error_ref)
                    }
                }
            }),
        );

        // POST /pdf/bulk - render multiple forms as PDFs from CSV (returns ZIP)
        paths.insert(
            format!("{}/pdf/bulk", full_path),
            json!({
                "post": {
                    "tags": [&bulk_tag],
                    "summary": "Bulk render PDFs from CSV",
                    "description": "Upload a CSV file containing form data. Returns a ZIP file containing individual PDF files named by row index or optional _filename column.",
                    "operationId": format!("bulk_render_{}_pdf", snake_name),
                    "parameters": [page_param()],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "multipart/form-data": {
                                "schema": {
                                    "type": "object",
                                    "required": ["file"],
                                    "properties": {
                                        "file": {
                                            "type": "string",
                                            "format": "binary",
                                            "description": "CSV file containing form data"
                                        }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "ZIP file containing PDFs",
                            "content": {
                                "application/zip": {
                                    "schema": { "type": "string", "format": "binary" }
                                }
                            }
                        },
                        "400": json_response("Validation error or invalid CSV", error_ref)
                    }
                }
            }),
        );
    }

    // Check if CSV import is enabled
    let has_import = schema.import.as_ref().is_some_and(|i| i.enabled);

    // Import endpoint (only if import.enabled is true)
    if has_import {
        let import_config = schema.import.as_ref().unwrap();
        let import_description = import_config.description.clone().unwrap_or_else(|| {
            format!(
                "Import {} forms from a CSV file. Returns a job ID for tracking progress.",
                root_name
            )
        });

        let mut import_params = vec![json!({
            "name": "idempotency_key",
            "in": "query",
            "required": false,
            "description": "Optional idempotency key for deduplication",
            "schema": { "type": "string" }
        })];

        // Add tax_year parameter when form metadata includes a tax year.
        if schema.irs_form.is_some() {
            import_params.push(json!({
                "name": "tax_year",
                "in": "query",
                "required": false,
                "description": "Tax year for the imported forms (defaults to current year)",
                "schema": { "type": "integer", "example": 2024 }
            }));
        }

        // POST /import - async CSV import
        paths.insert(
            format!("{}/import", full_path),
            json!({
                "post": {
                    "tags": [&bulk_tag],
                    "summary": format!("Import {} forms from CSV", root_name),
                    "description": import_description,
                    "operationId": format!("import_{}", snake_name),
                    "parameters": import_params,
                    "requestBody": {
                        "required": true,
                        "content": {
                            "multipart/form-data": {
                                "schema": {
                                    "type": "object",
                                    "required": ["file"],
                                    "properties": {
                                        "file": {
                                            "type": "string",
                                            "format": "binary",
                                            "description": "CSV file containing form data"
                                        }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "202": json_response("Import job created", "#/components/schemas/ImportJobResponse"),
                        "400": json_response("Invalid CSV format", error_ref),
                        "413": json_response("File too large", error_ref)
                    }
                }
            }),
        );
    }

    // Build schemas map incrementally to avoid recursion limit
    let mut schemas = Map::new();

    // Form-specific schemas
    schemas.insert(
        format!("{}Fields", root_name),
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": field_properties
        }),
    );
    schemas.insert(
        format!("Create{}Request", root_name),
        json!({
            "allOf": [{ "$ref": &fields_ref }]
        }),
    );
    schemas.insert(
        format!("Update{}Request", root_name),
        json!({
            "$ref": &fields_ref
        }),
    );
    schemas.insert(root_name.clone(), json!({
        "allOf": [
            { "$ref": &fields_ref },
            { "type": "object", "required": ["id"], "properties": { "id": { "type": "string" } } }
        ]
    }));
    schemas.insert(
        format!("{}Response", root_name),
        json!({
            "$ref": format!("#/components/schemas/{}", root_name)
        }),
    );

    // Common error schemas
    schemas.insert("ErrorResponse".to_string(), json!({
        "type": "object",
        "required": ["errors"],
        "properties": { "errors": { "type": "array", "items": { "$ref": "#/components/schemas/ApiErrorDetail" } } }
    }));
    schemas.insert(
        "ApiErrorDetail".to_string(),
        json!({
            "type": "object",
            "required": ["status", "title", "detail"],
            "properties": {
                "status": { "type": "string", "example": "400" },
                "title": { "type": "string", "example": "Bad Request" },
                "detail": { "type": "string", "example": "Invalid field value" },
                "source": { "$ref": "#/components/schemas/ErrorSource" }
            }
        }),
    );
    schemas.insert(
        "ErrorSource".to_string(),
        json!({
            "type": "object",
            "properties": { "pointer": { "type": "string", "example": "/payer_tin" } }
        }),
    );

    // Job-related schemas
    schemas.insert("ImportJobResponse".to_string(), json!({
        "type": "object",
        "required": ["job_id", "status", "status_url"],
        "properties": {
            "job_id": { "type": "string", "format": "uuid", "description": "Unique identifier for the import job", "example": "019449a2-5b4e-7c3d-9e2f-1a2b3c4d5e6f" },
            "status": { "type": "string", "enum": ["pending", "processing", "completed", "failed"], "description": "Current status of the import job" },
            "status_url": { "type": "string", "description": "URL to poll for job status updates", "example": "/api/v1/jobs/019449a2-5b4e-7c3d-9e2f-1a2b3c4d5e6f" }
        }
    }));
    schemas.insert("JobStatusResponse".to_string(), json!({
        "type": "object",
        "required": ["job_id", "job_type", "status", "created_at"],
        "properties": {
            "job_id": { "type": "string", "format": "uuid", "description": "Unique identifier for the job" },
            "job_type": { "type": "string", "description": "Type of job (e.g., bulk_create_1099_int)" },
            "status": { "type": "string", "enum": ["pending", "processing", "completed", "failed"], "description": "Current status of the job" },
            "total_rows": { "type": "integer", "nullable": true, "description": "Total number of rows in the import file" },
            "processed_rows": { "type": "integer", "nullable": true, "description": "Number of rows processed so far" },
            "successful_rows": { "type": "integer", "nullable": true, "description": "Number of rows successfully imported" },
            "failed_rows": { "type": "integer", "nullable": true, "description": "Number of rows that failed to import" },
            "error_message": { "type": "string", "nullable": true, "description": "Error message if the job failed" },
            "created_at": { "type": "string", "format": "date-time", "description": "When the job was created" },
            "started_at": { "type": "string", "format": "date-time", "nullable": true, "description": "When processing started" },
            "completed_at": { "type": "string", "format": "date-time", "nullable": true, "description": "When processing completed" }
        }
    }));

    // Build the complete spec
    let spec = json!({
        "openapi": "3.1.0",
        "info": {
            "title": &config.title,
            "version": &config.version,
            "description": &config.description
        },
        "paths": paths,
        "components": { "schemas": schemas }
    });

    spec.to_string()
}

use super::rust::CodegenOptions;

/// Generate all API code from a wellformed schema.
pub fn generate_api(schema: &Schema, api_prefix: &str, options: &CodegenOptions) -> String {
    let mut code = String::new();

    // Get the root struct name
    let root_name = schema
        .id
        .as_ref()
        .map(|id| derive_struct_name(id))
        .unwrap_or_else(|| "Root".to_string());

    // Derive the route path from irs_form metadata (preferred) or schema id (fallback)
    // e.g., irs_form.name="1099-INT" + revision_date="2024-01" -> "1099/int/2024-01"
    let route_path = schema
        .irs_form
        .as_ref()
        .map(|form| {
            // Split form name like "1099-INT" into "1099/int"
            let name_path = form.name.to_lowercase().replace('-', "/");
            match &form.revision_date {
                Some(date) => format!("{}/{}", name_path, date),
                None => name_path,
            }
        })
        .or_else(|| schema.id.as_ref().map(|id| id.replace('_', "/")))
        .unwrap_or_else(|| "form".to_string());

    // Get PDF template path if PDF handler generation is explicitly enabled.
    // If pdf_base_path is set, prefix the filename with it.
    let pdf_template_path = if options.generate_pdf_handlers {
        schema
            .pdf_template
            .as_ref()
            .and_then(|t| t.filename.as_ref().or(t.path.as_ref()))
            .map(|filename| {
                if let Some(base_path) = &options.pdf_base_path {
                    format!("{}/{}", base_path, filename)
                } else {
                    filename.clone()
                }
            })
    } else {
        None
    };

    // Derive the OpenAPI tag from irs_form metadata (e.g., "Form 1099-INT") or fall back to root_name
    let api_tag = schema
        .irs_form
        .as_ref()
        .map(|form| format!("Form {}", form.name))
        .unwrap_or_else(|| root_name.clone());

    // Derive a friendly display name for doc comments (e.g., "1099-INT form")
    let display_name = schema
        .irs_form
        .as_ref()
        .map(|form| format!("{} form", form.name))
        .unwrap_or_else(|| root_name.clone());

    // Generate request types
    code.push_str(&generate_request_types(schema, &root_name));
    code.push('\n');

    // Generate response types
    code.push_str(&generate_response_types(&root_name));
    code.push('\n');

    // Generate repository trait
    code.push_str(&generate_repository_trait(&root_name));
    code.push('\n');

    // Get form_id for template module.
    let form_id = schema.id.as_ref().map(|id| id.to_lowercase());

    // Generate handlers
    code.push_str(&generate_handlers(
        &root_name,
        &route_path,
        api_prefix,
        pdf_template_path.as_deref(),
        &api_tag,
        form_id.as_deref(),
        &display_name,
    ));
    code.push('\n');

    // Generate router
    code.push_str(&generate_router(
        &root_name,
        &route_path,
        api_prefix,
        pdf_template_path.is_some(),
        &api_tag,
        &display_name,
    ));

    code
}

/// Generate request types for the API.
fn generate_request_types(schema: &Schema, root_name: &str) -> String {
    let create_request_name = format_ident!("Create{}Request", root_name);
    let update_request_name = format_ident!("Update{}Request", root_name);
    let bulk_create_request_name = format_ident!("BulkCreate{}Request", root_name);
    let bulk_update_request_name = format_ident!("BulkUpdate{}Request", root_name);
    let bulk_update_item_name = format_ident!("BulkUpdate{}Item", root_name);
    let form_type = Ident::new(root_name, Span::call_site());

    // Build field definitions for create and update requests
    let mut create_fields: Vec<TokenStream> = Vec::new();
    let mut update_fields: Vec<TokenStream> = Vec::new();
    let mut from_field_mappings: Vec<TokenStream> = Vec::new();
    let mut apply_update_mappings: Vec<TokenStream> = Vec::new();

    if let TypeSchema::Object(obj) = &schema.root {
        for (field_name, prop) in &obj.fields {
            let rust_field = rust_field_name(field_name);
            let field_ident = Ident::new(&rust_field, Span::call_site());

            // Accept both generated Rust field names and exact schema keys.
            let serde_attr = if rust_field != *field_name {
                quote! { #[serde(alias = #field_name)] }
            } else {
                quote! {}
            };

            // Create request: use actual required/optional status
            let create_type_str = type_to_api_request_type(&prop.schema, prop.required);
            let create_type: TokenStream = create_type_str.parse().unwrap();
            create_fields.push(quote! { #serde_attr pub #field_ident: #create_type });

            // Update request: all fields optional
            let update_type_str = type_to_api_request_type(&prop.schema, false);
            let update_type: TokenStream = update_type_str.parse().unwrap();
            update_fields.push(quote! { #serde_attr pub #field_ident: #update_type });

            // From impl: direct field mapping
            from_field_mappings.push(quote! { #field_ident: req.#field_ident });

            // Apply updates: update payload fields are always optional.
            // For non-required domain fields, wrap incoming value in Some(...)
            // because the domain type is Option<T>.
            if prop.required {
                apply_update_mappings.push(quote! {
                    if let Some(v) = updates.#field_ident {
                        self.#field_ident = v;
                    }
                });
            } else {
                apply_update_mappings.push(quote! {
                    if let Some(v) = updates.#field_ident {
                        self.#field_ident = Some(v);
                    }
                });
            }
        }
    }

    let tokens = quote! {
        // ============================================================================
        // Request Types
        // ============================================================================

        /// Create request - mirrors the root struct but with API-friendly types.
        #[derive(Debug, Default, Serialize, Deserialize)]
        #[serde(default)]
        pub struct #create_request_name {
            #(#create_fields),*
        }

        /// Update request - all fields optional.
        #[derive(Debug, Default, Serialize, Deserialize)]
        #[serde(default)]
        pub struct #update_request_name {
            #(#update_fields),*
        }

        /// Bulk create request - array of create requests.
        #[derive(Debug, Default, Serialize, Deserialize)]
        pub struct #bulk_create_request_name {
            /// The forms to create.
            pub forms: Vec<#create_request_name>,
        }

        /// Single item in a bulk update request (id + update fields).
        #[derive(Debug, Default, Serialize, Deserialize)]
        #[serde(default)]
        pub struct #bulk_update_item_name {
            /// The ID of the form to update.
            pub id: String,
            /// The fields to update (same as update request).
            #[serde(flatten)]
            pub updates: #update_request_name,
        }

        /// Bulk update request - array of update items with IDs.
        #[derive(Debug, Default, Serialize, Deserialize)]
        pub struct #bulk_update_request_name {
            /// The forms to update.
            pub forms: Vec<#bulk_update_item_name>,
        }

        /// Direct conversion from create request to domain type (no JSON overhead).
        impl From<#create_request_name> for #form_type {
            fn from(req: #create_request_name) -> Self {
                Self {
                    #(#from_field_mappings),*
                }
            }
        }

        /// Extension trait for applying updates to form types.
        impl #form_type {
            /// Apply updates from an update request.
            /// Only non-None fields in the update request will be applied.
            pub fn apply_updates(&mut self, updates: #update_request_name) {
                #(#apply_update_mappings)*
            }
        }
    };

    tokens.to_string()
}

/// Generate response types for the API.
fn generate_response_types(root_name: &str) -> String {
    let response_name = format_ident!("{}Response", root_name);
    let bulk_create_response_name = format_ident!("BulkCreate{}Response", root_name);
    let bulk_create_result_name = format_ident!("BulkCreate{}Result", root_name);
    let bulk_update_response_name = format_ident!("BulkUpdate{}Response", root_name);
    let bulk_update_result_name = format_ident!("BulkUpdate{}Result", root_name);
    let form_type = Ident::new(root_name, Span::call_site());

    let tokens = quote! {
        // ============================================================================
        // Response Types
        // ============================================================================

        /// Single resource response - flattened so response is { id, field1, field2, ... }
        #[derive(Debug, Serialize)]
        pub struct #response_name {
            pub id: String,
            #[serde(flatten)]
            pub form: #form_type,
        }

        /// Result for a single form in a bulk create operation.
        #[derive(Debug, Serialize)]
        #[serde(tag = "status")]
        pub enum #bulk_create_result_name {
            /// Form was created successfully.
            #[serde(rename = "created")]
            Created {
                /// The index of this form in the request array.
                index: usize,
                /// The created form response.
                #[serde(flatten)]
                response: #response_name,
            },
            /// Form creation failed.
            #[serde(rename = "failed")]
            Failed {
                /// The index of this form in the request array.
                index: usize,
                /// Error details.
                errors: Vec<ApiErrorDetail>,
            },
        }

        /// Bulk create response - contains results for each form.
        #[derive(Debug, Serialize)]
        pub struct #bulk_create_response_name {
            /// Number of forms successfully created.
            pub created: usize,
            /// Number of forms that failed to create.
            pub failed: usize,
            /// Results for each form in the request.
            pub results: Vec<#bulk_create_result_name>,
        }

        /// Result for a single form in a bulk update operation.
        #[derive(Debug, Serialize)]
        #[serde(tag = "status")]
        pub enum #bulk_update_result_name {
            /// Form was updated successfully.
            #[serde(rename = "updated")]
            Updated {
                /// The index of this form in the request array.
                index: usize,
                /// The updated form response.
                #[serde(flatten)]
                response: #response_name,
            },
            /// Form update failed.
            #[serde(rename = "failed")]
            Failed {
                /// The index of this form in the request array.
                index: usize,
                /// Error details.
                errors: Vec<ApiErrorDetail>,
            },
        }

        /// Bulk update response - contains results for each form.
        #[derive(Debug, Serialize)]
        pub struct #bulk_update_response_name {
            /// Number of forms successfully updated.
            pub updated: usize,
            /// Number of forms that failed to update.
            pub failed: usize,
            /// Results for each form in the request.
            pub results: Vec<#bulk_update_result_name>,
        }
    };

    tokens.to_string()
}

/// Generate repository trait for storage abstraction.
fn generate_repository_trait(root_name: &str) -> String {
    let trait_name = format_ident!("{}Repository", root_name);
    let form_type = Ident::new(root_name, Span::call_site());

    // Doc comments with interpolation
    let trait_doc = format!("Storage abstraction for {}.", root_name);
    let create_doc = format!("Create a new {} and return its ID.", root_name);
    let create_many_doc = format!(
        "Create multiple {}s and return their IDs. Default implementation calls create() for each.",
        root_name
    );
    let get_doc = format!("Get a {} by ID.", root_name);
    let get_many_doc = format!(
        "Get multiple {}s by ID. Returns a map of ID -> form for found forms.",
        root_name
    );
    let update_doc = format!("Update a {} by ID.", root_name);
    let update_many_doc = format!(
        "Update multiple {}s. Default implementation calls update() for each.",
        root_name
    );

    let tokens = quote! {
        // ============================================================================
        // Repository Trait
        // ============================================================================

        #[doc = #trait_doc]
        pub trait #trait_name: Send + Sync {
            #[doc = #create_doc]
            fn create(&self, form: #form_type) -> impl Future<Output = Result<String, RepositoryError>> + Send;

            #[doc = #create_many_doc]
            fn create_many(&self, forms: Vec<#form_type>) -> impl Future<Output = Result<Vec<Result<String, RepositoryError>>, RepositoryError>> + Send {
                async move {
                    let mut results = Vec::with_capacity(forms.len());
                    for form in forms {
                        results.push(self.create(form).await);
                    }
                    Ok(results)
                }
            }

            #[doc = #get_doc]
            fn get(&self, id: &str) -> impl Future<Output = Result<Option<#form_type>, RepositoryError>> + Send;

            #[doc = #get_many_doc]
            fn get_many(&self, ids: &[String]) -> impl Future<Output = Result<std::collections::HashMap<String, #form_type>, RepositoryError>> + Send {
                async move {
                    let mut result = std::collections::HashMap::new();
                    for id in ids {
                        if let Some(form) = self.get(id).await? {
                            result.insert(id.clone(), form);
                        }
                    }
                    Ok(result)
                }
            }

            #[doc = #update_doc]
            fn update(&self, id: &str, form: #form_type) -> impl Future<Output = Result<(), RepositoryError>> + Send;

            #[doc = #update_many_doc]
            fn update_many(&self, forms: Vec<(String, #form_type)>) -> impl Future<Output = Result<Vec<Result<(), RepositoryError>>, RepositoryError>> + Send {
                async move {
                    let mut results = Vec::with_capacity(forms.len());
                    for (id, form) in forms {
                        results.push(self.update(&id, form).await);
                    }
                    Ok(results)
                }
            }

            /// Delete a form by ID.
            fn delete(&self, id: &str) -> impl Future<Output = Result<(), RepositoryError>> + Send;
        }
    };

    tokens.to_string()
}

/// Generate Axum handlers with utoipa annotations.
fn generate_handlers(
    root_name: &str,
    _route_path: &str,
    _api_prefix: &str,
    pdf_template_path: Option<&str>,
    _api_tag: &str,
    form_id: Option<&str>,
    display_name: &str,
) -> String {
    let snake_name = to_snake_case(root_name);

    // Create identifiers for use in quote!
    let form_type = Ident::new(root_name, Span::call_site());
    let trait_name = format_ident!("{}Repository", root_name);
    let create_request = format_ident!("Create{}Request", root_name);
    let update_request = format_ident!("Update{}Request", root_name);
    let bulk_create_request = format_ident!("BulkCreate{}Request", root_name);
    let bulk_update_request = format_ident!("BulkUpdate{}Request", root_name);
    let response_type = format_ident!("{}Response", root_name);
    let bulk_create_response = format_ident!("BulkCreate{}Response", root_name);
    let bulk_create_result = format_ident!("BulkCreate{}Result", root_name);
    let bulk_update_response = format_ident!("BulkUpdate{}Response", root_name);
    let bulk_update_result = format_ident!("BulkUpdate{}Result", root_name);
    let create_fn = format_ident!("create_{}", snake_name);
    let get_fn = format_ident!("get_{}", snake_name);
    let update_fn = format_ident!("update_{}", snake_name);
    let delete_fn = format_ident!("delete_{}", snake_name);
    let bulk_create_fn = format_ident!("bulk_create_{}", snake_name);
    let bulk_update_fn = format_ident!("bulk_update_{}", snake_name);

    // Doc comments
    let create_doc = format!("Create a new {}.", display_name);
    let get_doc = format!("Get a {} by ID.", display_name);
    let update_doc = format!("Update a {} by ID.", display_name);
    let delete_doc = format!("Delete a {}.", display_name);
    let bulk_create_doc = format!("Create multiple {}s in bulk.", display_name);
    let bulk_update_doc = format!("Update multiple {}s in bulk.", display_name);

    // Generate base handlers
    let base_handlers = quote! {
        // ============================================================================
        // Handlers
        // ============================================================================

        #[doc = #create_doc]
        pub async fn #create_fn<R: #trait_name + 'static>(
            State(repo): State<Arc<R>>,
            Json(req): Json<#create_request>,
        ) -> Result<(StatusCode, Json<#response_type>), ApiError> {
            // Convert request to domain type (direct field mapping, no JSON)
            let form: #form_type = req.into();

            // Validate
            let result = form.validate();
            if !result.is_valid() {
                return Err(ApiError::validation_with_field_map(&result, &field_name_map()));
            }

            // Store
            let id = repo.create(form.clone()).await?;

            // Return response
            Ok((StatusCode::CREATED, Json(#response_type { id, form })))
        }

        #[doc = #get_doc]
        pub async fn #get_fn<R: #trait_name + 'static>(
            State(repo): State<Arc<R>>,
            Path(id): Path<String>,
        ) -> Result<Json<#response_type>, ApiError> {
            let form = repo.get(&id).await?.ok_or_else(|| ApiError::not_found(&id))?;
            Ok(Json(#response_type { id, form }))
        }

        #[doc = #update_doc]
        pub async fn #update_fn<R: #trait_name + 'static>(
            State(repo): State<Arc<R>>,
            Path(id): Path<String>,
            Json(req): Json<#update_request>,
        ) -> Result<Json<#response_type>, ApiError> {
            // Get existing form
            let mut form = repo.get(&id).await?.ok_or_else(|| ApiError::not_found(&id))?;

            // Apply updates directly (no JSON serialization overhead)
            form.apply_updates(req);

            // Validate
            let result = form.validate();
            if !result.is_valid() {
                return Err(ApiError::validation_with_field_map(&result, &field_name_map()));
            }

            // Save
            repo.update(&id, form.clone()).await?;

            // Return response
            Ok(Json(#response_type { id, form }))
        }

        #[doc = #delete_doc]
        pub async fn #delete_fn<R: #trait_name + 'static>(
            State(repo): State<Arc<R>>,
            Path(id): Path<String>,
        ) -> Result<StatusCode, ApiError> {
            repo.delete(&id).await?;
            Ok(StatusCode::NO_CONTENT)
        }

        /// Maximum number of forms allowed in a single bulk create request.
        pub const BULK_CREATE_MAX_FORMS: usize = 1000;

        #[doc = #bulk_create_doc]
        pub async fn #bulk_create_fn<R: #trait_name + 'static>(
            State(repo): State<Arc<R>>,
            body: axum::body::Bytes,
        ) -> Result<axum::response::Response, ApiError> {
            use rayon::prelude::*;
            use std::time::Instant;
            use axum::response::IntoResponse;

            let total_start = Instant::now();

            // Parse JSON from raw bytes (measure this overhead)
            let parse_start = Instant::now();
            let req: #bulk_create_request = serde_json::from_slice(&body)
                .map_err(|e| ApiError::bad_request(&format!("Invalid JSON: {}", e)))?;
            let parse_time = parse_start.elapsed();

            let form_count = req.forms.len();
            tracing::info!(
                "REQUEST_PARSE forms={} bytes={} parse={:.2}ms",
                form_count, body.len(), parse_time.as_secs_f64() * 1000.0
            );

            // Enforce batch size limit
            if form_count > BULK_CREATE_MAX_FORMS {
                return Err(ApiError::bad_request(&format!(
                    "Bulk create request exceeds maximum of {} forms (got {})",
                    BULK_CREATE_MAX_FORMS,
                    form_count
                )));
            }

            let field_map = field_name_map();

            // First pass: validate all forms in parallel
            use std::sync::atomic::{AtomicU64, Ordering};
            let convert_nanos = AtomicU64::new(0);
            let validate_nanos = AtomicU64::new(0);

            let validate_start = Instant::now();
            let validation_results: Vec<_> = req.forms
                .into_par_iter()
                .enumerate()
                .map(|(index, form_req)| {
                    // Convert request to domain type (direct field mapping, no JSON)
                    let t0 = Instant::now();
                    let form: #form_type = form_req.into();
                    convert_nanos.fetch_add(t0.elapsed().as_nanos() as u64, Ordering::Relaxed);

                    // Validate
                    let t1 = Instant::now();
                    let validation_result = form.validate();
                    validate_nanos.fetch_add(t1.elapsed().as_nanos() as u64, Ordering::Relaxed);

                    if !validation_result.is_valid() {
                        let errors = ApiError::validation_with_field_map(&validation_result, &field_map)
                            .to_error_details();
                        return (index, Err(errors));
                    }

                    // Form is valid
                    (index, Ok(form))
                })
                .collect();
            let validate_time = validate_start.elapsed();

            let convert_ms = convert_nanos.load(Ordering::Relaxed) as f64 / 1_000_000.0;
            let val_ms = validate_nanos.load(Ordering::Relaxed) as f64 / 1_000_000.0;
            tracing::info!(
                "VALIDATION_BREAKDOWN forms={} convert={:.2}ms validate={:.2}ms wall={:.2}ms",
                form_count, convert_ms, val_ms, validate_time.as_secs_f64() * 1000.0
            );

            // Separate valid forms from errors
            let collect_start = Instant::now();
            let mut results: Vec<(usize, Result<(String, #form_type), Vec<ApiErrorDetail>>)> = Vec::with_capacity(validation_results.len());
            let mut forms_to_create: Vec<(usize, #form_type)> = Vec::with_capacity(validation_results.len());
            let mut created_count = 0usize;
            let mut failed_count = 0usize;

            for (index, result) in validation_results {
                match result {
                    Ok(form) => forms_to_create.push((index, form)),
                    Err(errors) => {
                        failed_count += 1;
                        results.push((index, Err(errors)));
                    }
                }
            }
            let collect_time = collect_start.elapsed();

            // Second pass: create all valid forms via repository
            let db_start = Instant::now();
            if !forms_to_create.is_empty() {
                let forms: Vec<#form_type> = forms_to_create.iter().map(|(_, f)| f.clone()).collect();
                let create_results = repo.create_many(forms).await?;

                for ((index, form), create_result) in forms_to_create.into_iter().zip(create_results) {
                    match create_result {
                        Ok(id) => {
                            created_count += 1;
                            results.push((index, Ok((id, form))));
                        }
                        Err(e) => {
                            failed_count += 1;
                            results.push((index, Err(vec![ApiErrorDetail {
                                status: "500".to_string(),
                                title: "Internal Error".to_string(),
                                detail: e.to_string(),
                                source: None,
                            }])));
                        }
                    }
                }
            }
            let db_time = db_start.elapsed();

            // Sort results by original index and convert to final format
            let resp_start = Instant::now();
            results.sort_by_key(|(index, _)| *index);
            let final_results: Vec<#bulk_create_result> = results
                .into_iter()
                .map(|(index, result)| match result {
                    Ok((id, form)) => #bulk_create_result::Created {
                        index,
                        response: #response_type { id, form },
                    },
                    Err(errors) => #bulk_create_result::Failed { index, errors },
                })
                .collect();
            let resp_time = resp_start.elapsed();

            // Serialize response manually to measure time
            let response = #bulk_create_response {
                created: created_count,
                failed: failed_count,
                results: final_results,
            };
            let serialize_start = Instant::now();
            let response_bytes = serde_json::to_vec(&response)
                .map_err(|e| ApiError::internal(&e.to_string()))?;
            let serialize_time = serialize_start.elapsed();

            let total_time = total_start.elapsed();

            // Log timing breakdown
            tracing::info!(
                "BULK_CREATE_TIMING forms={} validate={:.2}ms collect={:.2}ms db={:.2}ms resp={:.2}ms serialize={:.2}ms resp_bytes={} total={:.2}ms",
                form_count,
                validate_time.as_secs_f64() * 1000.0,
                collect_time.as_secs_f64() * 1000.0,
                db_time.as_secs_f64() * 1000.0,
                resp_time.as_secs_f64() * 1000.0,
                serialize_time.as_secs_f64() * 1000.0,
                response_bytes.len(),
                total_time.as_secs_f64() * 1000.0,
            );

            Ok((
                [(axum::http::header::CONTENT_TYPE, "application/json")],
                response_bytes
            ).into_response())
        }

        /// Maximum number of forms allowed in a single bulk update request.
        pub const BULK_UPDATE_MAX_FORMS: usize = 1000;

        #[doc = #bulk_update_doc]
        pub async fn #bulk_update_fn<R: #trait_name + 'static>(
            State(repo): State<Arc<R>>,
            body: axum::body::Bytes,
        ) -> Result<axum::response::Response, ApiError> {
            use rayon::prelude::*;
            use std::time::Instant;
            use axum::response::IntoResponse;

            let total_start = Instant::now();

            // Parse JSON from raw bytes
            let parse_start = Instant::now();
            let req: #bulk_update_request = serde_json::from_slice(&body)
                .map_err(|e| ApiError::bad_request(&format!("Invalid JSON: {}", e)))?;
            let parse_time = parse_start.elapsed();

            let form_count = req.forms.len();
            tracing::info!(
                "BULK_UPDATE_PARSE forms={} bytes={} parse={:.2}ms",
                form_count, body.len(), parse_time.as_secs_f64() * 1000.0
            );

            // Enforce batch size limit
            if form_count > BULK_UPDATE_MAX_FORMS {
                return Err(ApiError::bad_request(&format!(
                    "Bulk update request exceeds maximum of {} forms (got {})",
                    BULK_UPDATE_MAX_FORMS,
                    form_count
                )));
            }

            let field_map = field_name_map();

            // Collect all IDs and store id->index mapping
            let ids: Vec<String> = req.forms.iter().map(|f| f.id.clone()).collect();
            let id_to_index: std::collections::HashMap<String, usize> = ids
                .iter()
                .enumerate()
                .map(|(i, id)| (id.clone(), i))
                .collect();

            // Fetch all existing forms in batch
            let fetch_start = Instant::now();
            let existing_forms = repo.get_many(&ids).await?;
            let fetch_time = fetch_start.elapsed();

            tracing::info!(
                "BULK_UPDATE_FETCH ids={} found={} fetch={:.2}ms",
                ids.len(), existing_forms.len(), fetch_time.as_secs_f64() * 1000.0
            );

            // Merge updates with existing forms and validate in parallel
            let merge_start = Instant::now();
            let merge_results: Vec<_> = req.forms
                .into_par_iter()
                .enumerate()
                .map(|(index, update_item)| {
                    let id = update_item.id.clone();

                    // Check if form exists
                    let mut form = match existing_forms.get(&id) {
                        Some(f) => f.clone(),
                        None => {
                            let detail = format!("Form with id '{}' not found", id);
                            return (index, id, Err(vec![ApiErrorDetail {
                                status: "404".to_string(),
                                title: "Not Found".to_string(),
                                detail,
                                source: None,
                            }]));
                        }
                    };

                    // Apply updates directly (no JSON serialization overhead)
                    form.apply_updates(update_item.updates);

                    // Validate the updated form
                    let validation_result = form.validate();
                    if !validation_result.is_valid() {
                        let errors = ApiError::validation_with_field_map(&validation_result, &field_map)
                            .to_error_details();
                        return (index, id, Err(errors));
                    }

                    (index, id, Ok(form))
                })
                .collect();
            let merge_time = merge_start.elapsed();

            // Separate valid forms from errors
            let collect_start = Instant::now();
            let mut results: Vec<(usize, Result<(String, #form_type), Vec<ApiErrorDetail>>)> = Vec::with_capacity(merge_results.len());
            let mut forms_to_update: Vec<(String, #form_type)> = Vec::with_capacity(merge_results.len());
            let mut updated_count = 0usize;
            let mut failed_count = 0usize;

            for (index, id, result) in merge_results {
                match result {
                    Ok(form) => forms_to_update.push((id.clone(), form)),
                    Err(errors) => {
                        failed_count += 1;
                        results.push((index, Err(errors)));
                    }
                }
            }
            let collect_time = collect_start.elapsed();

            // Update all valid forms via repository
            let db_start = Instant::now();
            if !forms_to_update.is_empty() {
                let update_results = repo.update_many(forms_to_update.clone()).await?;

                for ((id, form), update_result) in forms_to_update.into_iter().zip(update_results) {
                    // Find original index using the id_to_index map
                    let index = *id_to_index.get(&id).unwrap_or(&0);
                    match update_result {
                        Ok(()) => {
                            updated_count += 1;
                            results.push((index, Ok((id, form))));
                        }
                        Err(e) => {
                            failed_count += 1;
                            results.push((index, Err(vec![ApiErrorDetail {
                                status: "500".to_string(),
                                title: "Internal Error".to_string(),
                                detail: e.to_string(),
                                source: None,
                            }])));
                        }
                    }
                }
            }
            let db_time = db_start.elapsed();

            // Sort results by original index and convert to final format
            let resp_start = Instant::now();
            results.sort_by_key(|(index, _)| *index);
            let final_results: Vec<#bulk_update_result> = results
                .into_iter()
                .map(|(index, result)| match result {
                    Ok((id, form)) => #bulk_update_result::Updated {
                        index,
                        response: #response_type { id, form },
                    },
                    Err(errors) => #bulk_update_result::Failed { index, errors },
                })
                .collect();
            let resp_time = resp_start.elapsed();

            // Serialize response
            let response = #bulk_update_response {
                updated: updated_count,
                failed: failed_count,
                results: final_results,
            };
            let serialize_start = Instant::now();
            let response_bytes = serde_json::to_vec(&response)
                .map_err(|e| ApiError::internal(&e.to_string()))?;
            let serialize_time = serialize_start.elapsed();

            let total_time = total_start.elapsed();

            // Log timing breakdown
            tracing::info!(
                "BULK_UPDATE_TIMING forms={} fetch={:.2}ms merge={:.2}ms collect={:.2}ms db={:.2}ms resp={:.2}ms serialize={:.2}ms resp_bytes={} total={:.2}ms",
                form_count,
                fetch_time.as_secs_f64() * 1000.0,
                merge_time.as_secs_f64() * 1000.0,
                collect_time.as_secs_f64() * 1000.0,
                db_time.as_secs_f64() * 1000.0,
                resp_time.as_secs_f64() * 1000.0,
                serialize_time.as_secs_f64() * 1000.0,
                response_bytes.len(),
                total_time.as_secs_f64() * 1000.0,
            );

            Ok((
                [(axum::http::header::CONTENT_TYPE, "application/json")],
                response_bytes
            ).into_response())
        }
    };

    // Generate PDF handlers if template path is specified
    let pdf_handlers = if let Some(_pdf_path) = pdf_template_path {
        let render_pdf_fn = format_ident!("render_{}_pdf", snake_name);
        let render_pdf_from_body_fn = format_ident!("render_{}_pdf_from_body", snake_name);
        let bulk_render_pdf_fn = format_ident!("bulk_render_{}_pdf", snake_name);
        let schema_const = format_ident!("{}_SCHEMA_JSON", snake_name.to_uppercase());

        // Get form ID for template module.
        let template_module_name = form_id.unwrap_or(&snake_name);
        let template_module = format_ident!("{}", template_module_name);
        // PDF constant name, e.g. "signup" -> "SIGNUP_PDF_BYTES".
        let pdf_bytes_const = format_ident!("{}_PDF_BYTES", template_module_name.to_uppercase());

        let render_pdf_doc = format!("Render a {} as PDF.", display_name);
        let render_pdf_from_body_doc =
            format!("Render a {} as PDF from request body.", display_name);
        let bulk_render_pdf_doc = format!(
            "Render multiple {} forms as PDFs and return as ZIP.",
            display_name
        );

        quote! {
            #[doc = #render_pdf_doc]
            pub async fn #render_pdf_fn<R: #trait_name + 'static>(
                State(repo): State<Arc<R>>,
                Path(id): Path<String>,
                Query(params): Query<PdfRenderParams>,
            ) -> Result<Response, ApiError> {
                use axum::http::header;
                use wireform_acroform::FormFiller;
                use std::sync::OnceLock;

                // Static marker filler (initialized once)
                static FILLER: OnceLock<FormFiller> = OnceLock::new();

                // Get form data
                let form = repo.get(&id).await?.ok_or_else(|| ApiError::not_found(&id))?;

                // Parse the schema for field mappings
                let schema: wellformed::Schema = serde_json::from_str(#schema_const)
                    .expect("embedded schema should be valid");

                // Render PDF using FormFiller
                let filler = FILLER.get_or_init(|| {
                    FormFiller::new(templates::#template_module::#pdf_bytes_const)
                        .expect("template should be valid")
                });
                let pdf_output = crate::render::render_form_to_pdf(&form, &schema, filler, None, None)
                    .map_err(|e| ApiError::internal(&e.to_string()))?;

                // Return PDF response
                Ok((
                    [(header::CONTENT_TYPE, "application/pdf")],
                    pdf_output,
                ).into_response())
            }

            #[doc = #render_pdf_from_body_doc]
            pub async fn #render_pdf_from_body_fn(
                Query(params): Query<PdfRenderParams>,
                Json(req): Json<#create_request>,
            ) -> Result<Response, ApiError> {
                use axum::http::header;
                use wireform_acroform::FormFiller;
                use std::sync::OnceLock;

                // Static marker filler (initialized once)
                static FILLER: OnceLock<FormFiller> = OnceLock::new();

                // Convert request to domain type (direct field mapping, no JSON)
                let form: #form_type = req.into();

                // Validate (unless skip_validation is set)
                if !params.skip_validation {
                    let result = form.validate();
                    if !result.is_valid() {
                        return Err(ApiError::validation_with_field_map(&result, &field_name_map()));
                    }
                }

                // Parse the schema for field mappings
                let schema: wellformed::Schema = serde_json::from_str(#schema_const)
                    .expect("embedded schema should be valid");

                // Render PDF using FormFiller
                let filler = FILLER.get_or_init(|| {
                    FormFiller::new(templates::#template_module::#pdf_bytes_const)
                        .expect("template should be valid")
                });
                let pdf_output = crate::render::render_form_to_pdf(&form, &schema, filler, None, None)
                    .map_err(|e| ApiError::internal(&e.to_string()))?;

                // Return PDF response
                Ok((
                    [(header::CONTENT_TYPE, "application/pdf")],
                    pdf_output,
                ).into_response())
            }

            #[doc = #bulk_render_pdf_doc]
            pub async fn #bulk_render_pdf_fn(
                Query(params): Query<PdfRenderParams>,
                mut multipart: axum::extract::Multipart,
            ) -> Result<Response, ApiError> {
                use axum::http::header;
                use wireform_acroform::FormFiller;
                use std::sync::OnceLock;
                use std::io::Write;

                // Static marker filler (initialized once)
                static FILLER: OnceLock<FormFiller> = OnceLock::new();

                // Read the CSV file from multipart
                let mut csv_data: Option<Vec<u8>> = None;
                while let Some(field) = multipart.next_field().await
                    .map_err(|e| ApiError::bad_request(&format!("Multipart error: {}", e)))?
                {
                    if field.name() == Some("file") {
                        csv_data = Some(field.bytes().await
                            .map_err(|e| ApiError::bad_request(&format!("Failed to read file: {}", e)))?
                            .to_vec());
                        break;
                    }
                }

                let csv_data = csv_data.ok_or_else(|| ApiError::bad_request("No file provided"))?;
                if csv_data.is_empty() {
                    return Err(ApiError::bad_request("Empty CSV file"));
                }

                // Parse the schema for field mappings
                let schema: wellformed::Schema = serde_json::from_str(#schema_const)
                    .expect("embedded schema should be valid");

                // Build column mapper from schema
                let column_map = crate::jobs::csv_mapper::build_column_map_from_schema(#schema_const);

                // Parse CSV
                let mut reader = csv::ReaderBuilder::new()
                    .has_headers(true)
                    .flexible(true)
                    .from_reader(csv_data.as_slice());

                let original_headers = reader.headers()
                    .map_err(|e| ApiError::bad_request(&format!("Failed to read CSV headers: {}", e)))?
                    .clone();

                let (headers, _unmapped) = crate::jobs::csv_mapper::transform_headers(&original_headers, &column_map);

                // Get or initialize the FormFiller
                let filler = FILLER.get_or_init(|| {
                    FormFiller::new(templates::#template_module::#pdf_bytes_const)
                        .expect("template should be valid")
                });

                // Collect records
                let records: Vec<csv::StringRecord> = reader.records()
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| ApiError::bad_request(&format!("CSV parse error: {}", e)))?;

                if records.is_empty() {
                    return Err(ApiError::bad_request("CSV file has no data rows"));
                }

                // Create ZIP file in memory
                let mut zip_buffer = std::io::Cursor::new(Vec::with_capacity(records.len() * 700_000));
                {
                    let mut zip = zip::ZipWriter::new(&mut zip_buffer);
                    let options = zip::write::SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Stored);

                    // Find _filename column index if present
                    let filename_col_idx = headers.iter().position(|h| h == "_filename");

                    for (idx, record) in records.iter().enumerate() {
                        // Build JSON from CSV record
                        let mut json_map = serde_json::Map::new();
                        for (i, header) in headers.iter().enumerate() {
                            if let Some(value) = record.get(i) {
                                if !value.is_empty() && header != "_filename" {
                                    // Try to parse as number
                                    if let Ok(n) = value.parse::<i64>() {
                                        json_map.insert(header.to_string(), serde_json::Value::Number(n.into()));
                                    } else if let Ok(n) = value.parse::<f64>() {
                                        json_map.insert(header.to_string(), serde_json::json!(n));
                                    } else {
                                        json_map.insert(header.to_string(), serde_json::Value::String(value.to_string()));
                                    }
                                }
                            }
                        }

                        // Deserialize to create request type
                        let req: #create_request = serde_json::from_value(serde_json::Value::Object(json_map))
                            .map_err(|e| ApiError::bad_request(&format!("Row {}: {}", idx + 1, e)))?;

                        // Convert to domain type
                        let form: #form_type = req.into();

                        // Validate (unless skip_validation is set)
                        if !params.skip_validation {
                            let result = form.validate();
                            if !result.is_valid() {
                                return Err(ApiError::validation_with_field_map(&result, &field_name_map()));
                            }
                        }

                        // Render PDF
                        let pdf_output = crate::render::render_form_to_pdf(&form, &schema, filler, None, None)
                            .map_err(|e| ApiError::internal(&e.to_string()))?;

                        // Determine filename
                        let filename = filename_col_idx
                            .and_then(|i| record.get(i))
                            .filter(|s| !s.is_empty())
                            .map(|f| {
                                if f.ends_with(".pdf") { f.to_string() } else { format!("{}.pdf", f) }
                            })
                            .unwrap_or_else(|| format!("{}.pdf", idx));

                        // Add to ZIP
                        zip.start_file(&filename, options)
                            .map_err(|e| ApiError::internal(&format!("Failed to create ZIP entry: {}", e)))?;
                        zip.write_all(&pdf_output)
                            .map_err(|e| ApiError::internal(&format!("Failed to write PDF to ZIP: {}", e)))?;
                    }

                    zip.finish()
                        .map_err(|e| ApiError::internal(&format!("Failed to finalize ZIP: {}", e)))?;
                }

                let zip_data = zip_buffer.into_inner();

                // Return ZIP response
                Ok((
                    [
                        (header::CONTENT_TYPE, "application/zip"),
                        (header::CONTENT_DISPOSITION, "attachment; filename=\"forms.zip\""),
                    ],
                    zip_data,
                ).into_response())
            }
        }
    } else {
        quote! {}
    };

    let tokens = quote! {
        #base_handlers
        #pdf_handlers
    };

    tokens.to_string()
}

/// Generate router setup function.
fn generate_router(
    root_name: &str,
    route_path: &str,
    api_prefix: &str,
    has_pdf: bool,
    _api_tag: &str,
    display_name: &str,
) -> String {
    let snake_name = to_snake_case(root_name);
    let full_path = format!("{}/{}", api_prefix, route_path);
    let full_path_with_id = format!("{}/:id", full_path);

    // Create identifiers
    let trait_name = format_ident!("{}Repository", root_name);
    let routes_fn = format_ident!("{}_routes", snake_name);
    let create_fn = format_ident!("create_{}", snake_name);
    let get_fn = format_ident!("get_{}", snake_name);
    let update_fn = format_ident!("update_{}", snake_name);
    let delete_fn = format_ident!("delete_{}", snake_name);

    let router_doc = format!("Create router for {} endpoints.", display_name);

    // Build PDF routes if needed
    let pdf_routes = if has_pdf {
        let render_pdf_fn = format_ident!("render_{}_pdf", snake_name);
        let render_pdf_from_body_fn = format_ident!("render_{}_pdf_from_body", snake_name);
        let bulk_render_pdf_fn = format_ident!("bulk_render_{}_pdf", snake_name);
        let pdf_id_path = format!("{}/:id/pdf", full_path);
        let pdf_path = format!("{}/pdf", full_path);
        let pdf_bulk_path = format!("{}/pdf/bulk", full_path);

        quote! {
            .route(#pdf_id_path, get(#render_pdf_fn::<R>))
            .route(#pdf_path, post(#render_pdf_from_body_fn))
            .route(#pdf_bulk_path, post(#bulk_render_pdf_fn))
        }
    } else {
        quote! {}
    };

    let tokens = quote! {
        // ============================================================================
        // Router
        // ============================================================================

        #[doc = #router_doc]
        pub fn #routes_fn<R: #trait_name + 'static>() -> Router<Arc<R>> {
            Router::new()
                .route(#full_path, post(#create_fn::<R>))
                .route(#full_path_with_id, get(#get_fn::<R>))
                .route(#full_path_with_id, patch(#update_fn::<R>))
                .route(#full_path_with_id, delete(#delete_fn::<R>))
                #pdf_routes
        }
    };

    tokens.to_string()
}

/// Convert a TypeSchema to an API request type string.
fn type_to_api_request_type(schema: &TypeSchema, required: bool) -> String {
    let base_type = match schema {
        TypeSchema::String(_) => "String",
        TypeSchema::Number(_) => "f64",
        TypeSchema::Integer(_) => "i64",
        TypeSchema::Int32(_) => "i32",
        TypeSchema::Int64(_) => "i64",
        TypeSchema::Uint32(_) => "u32",
        TypeSchema::Uint64(_) => "u64",
        TypeSchema::Boolean(_) => "bool",
        // Use Decimal type directly for API compatibility
        TypeSchema::Money(_) => "rust_decimal::Decimal",
        TypeSchema::Currency(_) => "rust_decimal::Decimal",
        TypeSchema::Decimal(_) => "rust_decimal::Decimal",
        TypeSchema::Percentage(_) => "f64",
        TypeSchema::Date(_) => "chrono::NaiveDate",
        TypeSchema::Array(arr) => {
            let item_type = type_to_api_request_type(&arr.items, true);
            return if required {
                format!("Vec<{}>", item_type)
            } else {
                format!("Option<Vec<{}>>", item_type)
            };
        }
        TypeSchema::Tuple(_) => "serde_json::Value",
        TypeSchema::Object(_) => "serde_json::Value",
        TypeSchema::Enum(_) => "String",
        TypeSchema::Literal(_) => "serde_json::Value",
        TypeSchema::Never(_) => "serde_json::Value",
        TypeSchema::Union(_) => "serde_json::Value",
        TypeSchema::Intersection(_) => "serde_json::Value",
        TypeSchema::Record(_) => "serde_json::Value",
        TypeSchema::Preprocess(_) => "serde_json::Value",
        TypeSchema::Catch(_) => "serde_json::Value",
        TypeSchema::Ref { name } => {
            return if required {
                format!("Create{}Request", to_pascal_case(name))
            } else {
                format!("Option<Create{}Request>", to_pascal_case(name))
            };
        }
        TypeSchema::Any(_) => "serde_json::Value",
    };

    if required {
        base_type.to_string()
    } else {
        format!("Option<{}>", base_type)
    }
}

fn rust_field_name(field_name: &str) -> String {
    escape_keyword(&to_snake_case(field_name))
}

/// Generate an example value for a field based on its name.
fn generate_field_example(field_name: &str) -> String {
    // TIN fields
    if field_name.contains("tin") && !field_name.contains("notice") {
        return "12-3456789".to_string();
    }
    // Name fields
    if field_name.contains("first_name") {
        return "John".to_string();
    }
    if field_name.contains("last_name") {
        return "Doe".to_string();
    }
    if field_name.contains("middle_name") {
        return "Q".to_string();
    }
    if field_name.contains("suffix") {
        return "Jr".to_string();
    }
    if field_name.contains("business_name") {
        return "Acme Corporation".to_string();
    }
    // Address fields
    if field_name.contains("address_line_1") {
        return "123 Main Street".to_string();
    }
    if field_name.contains("address_line_2") {
        return "Suite 100".to_string();
    }
    if field_name.contains("city") {
        return "San Francisco".to_string();
    }
    if field_name.contains("state")
        && !field_name.contains("income")
        && !field_name.contains("number")
        && !field_name.contains("tax")
        && !field_name.contains("data")
    {
        return "CA".to_string();
    }
    if field_name.contains("zip") {
        return "94102".to_string();
    }
    if field_name.contains("country") {
        return "US".to_string();
    }
    // Phone fields
    if field_name.contains("phone") && !field_name.contains("type") {
        return "415-555-1234".to_string();
    }
    // Tax year
    if field_name == "tax_year" {
        return "2024".to_string();
    }
    // Money/amount fields (boxes, income, tax, etc.)
    if field_name.contains("box_")
        || field_name.contains("income")
        || field_name.contains("tax_withheld")
        || field_name.contains("penalty")
        || field_name.contains("discount")
        || field_name.contains("premium")
        || field_name.contains("expenses")
        || field_name.contains("_paid")
    {
        return "1234.56".to_string();
    }
    // Account number
    if field_name.contains("account_number") {
        return "****1234".to_string();
    }
    // CUSIP
    if field_name.contains("cusip") {
        return "037833100".to_string();
    }
    // RTN (routing number)
    if field_name.contains("rtn") {
        return "121000358".to_string();
    }
    // Type fields (name_type, tin_type, phone_type, form_type)
    if field_name.contains("_type") {
        if field_name.contains("name") {
            return "individual".to_string();
        }
        if field_name.contains("tin") {
            return "SSN".to_string();
        }
        if field_name.contains("phone") {
            return "business".to_string();
        }
        if field_name.contains("form") {
            return "1099-INT".to_string();
        }
    }
    // Boolean-ish fields
    if field_name.contains("filing")
        || field_name.contains("notice")
        || field_name.contains("fatca")
    {
        return "false".to_string();
    }
    // State payer number
    if field_name.contains("payer_state_number") {
        return "123456789".to_string();
    }
    // Office code
    if field_name.contains("office_code") {
        return "".to_string();
    }
    // Special data
    if field_name.contains("special_data") {
        return "".to_string();
    }
    // Default empty
    "".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{ObjectSchema, PropertySchema, StringSchema};
    use indexmap::IndexMap;

    #[test]
    fn test_generate_request_types() {
        let mut props = IndexMap::new();
        props.insert(
            "name".to_string(),
            PropertySchema {
                schema: TypeSchema::String(StringSchema::default()),
                required: true,
                description: Some("The name".to_string()),
                label: None,
                render: None,
                acroform: None,
                section: None,
            },
        );
        props.insert(
            "tin".to_string(),
            PropertySchema {
                schema: TypeSchema::String(StringSchema::default()),
                required: true,
                description: Some("Tax identifier".to_string()),
                label: Some("Tax ID".to_string()),
                render: None,
                acroform: None,
                section: None,
            },
        );

        let schema = Schema::new(
            "1.0.0",
            TypeSchema::Object(ObjectSchema {
                fields: props,
                pages: IndexMap::new(),
                acroform_mappings: Vec::new(),
                additional_properties: false,
                unknown_keys: None,
                catchall: None,
                rules: Vec::new(),
                description: None,
            }),
        )
        .with_id("test_form");

        let code = generate_request_types(&schema, "TestForm");
        println!("Generated code:\n{}", code);
        assert!(code.contains("pub struct CreateTestFormRequest"));
        // quote! generates `pub name : String` with spaces around the colon
        assert!(
            code.contains("pub name") && code.contains("String"),
            "Should contain field 'name' of type String"
        );
        assert!(
            code.contains("pub tin") && code.contains("String"),
            "Should use schema key for labeled field"
        );
        assert!(
            !code.contains("pub tax_id"),
            "Should not derive API field names from labels"
        );
    }

    #[test]
    fn test_generate_field_example() {
        assert_eq!(generate_field_example("payer_tin"), "12-3456789");
        assert_eq!(generate_field_example("payer_first_name"), "John");
        assert_eq!(generate_field_example("payer_city"), "San Francisco");
        assert_eq!(generate_field_example("box_1_interest_income"), "1234.56");
        assert_eq!(generate_field_example("tax_year"), "2024");
    }

    #[test]
    fn test_generate_openapi_spec_has_examples() {
        let mut props = IndexMap::new();
        props.insert(
            "payer_tin".to_string(),
            PropertySchema {
                schema: TypeSchema::String(StringSchema::default()),
                required: true,
                description: None,
                label: Some("Payer TIN".to_string()),
                render: None,
                acroform: None,
                section: None,
            },
        );

        let schema = Schema::new(
            "1.0.0",
            TypeSchema::Object(ObjectSchema {
                fields: props,
                pages: IndexMap::new(),
                acroform_mappings: Vec::new(),
                additional_properties: false,
                unknown_keys: None,
                catchall: None,
                rules: Vec::new(),
                description: None,
            }),
        )
        .with_id("test_form");

        let spec = generate_openapi_spec(&schema, "/api/v1/forms");
        println!("Generated spec: {}", spec);
        assert!(
            spec.contains("\"example\":\"12-3456789\""),
            "Should contain TIN example"
        );
    }

    #[test]
    fn test_openapi_spec_uses_schema_keys_not_labels() {
        let mut props = IndexMap::new();
        props.insert(
            "tin".to_string(),
            PropertySchema {
                schema: TypeSchema::String(StringSchema::default()),
                required: true,
                description: None,
                label: Some("Tax ID".to_string()),
                render: None,
                acroform: None,
                section: None,
            },
        );

        let schema = Schema::new(
            "1.0.0",
            TypeSchema::Object(ObjectSchema {
                fields: props,
                pages: IndexMap::new(),
                acroform_mappings: Vec::new(),
                additional_properties: false,
                unknown_keys: None,
                catchall: None,
                rules: Vec::new(),
                description: None,
            }),
        )
        .with_id("test_form");

        let spec = generate_openapi_spec(&schema, "/api/v1/forms");
        let json: serde_json::Value = serde_json::from_str(&spec).expect("valid OpenAPI JSON");
        let fields = json
            .pointer("/components/schemas/TestFormFields/properties")
            .expect("fields properties");

        assert!(fields.get("tin").is_some());
        assert!(fields.get("tax_id").is_none());
    }

    #[test]
    fn test_generate_repository_trait() {
        let code = generate_repository_trait("TestForm");
        assert!(code.contains("pub trait TestFormRepository"));
        assert!(code.contains("fn create"));
        assert!(code.contains("fn get"));
        assert!(code.contains("fn update"));
        assert!(code.contains("fn delete"));
    }

    #[test]
    fn test_generate_openapi_spec_with_pdf() {
        use crate::ir::PdfTemplate;

        let mut props = IndexMap::new();
        props.insert(
            "payer_tin".to_string(),
            PropertySchema {
                schema: TypeSchema::String(StringSchema::default()),
                required: true,
                description: None,
                label: Some("Payer TIN".to_string()),
                render: None,
                acroform: None,
                section: None,
            },
        );

        let mut schema = Schema::new(
            "1.0.0",
            TypeSchema::Object(ObjectSchema {
                fields: props,
                pages: IndexMap::new(),
                acroform_mappings: Vec::new(),
                additional_properties: false,
                unknown_keys: None,
                catchall: None,
                rules: Vec::new(),
                description: None,
            }),
        )
        .with_id("test_form");

        // Add PDF template
        schema.pdf_template = Some(PdfTemplate {
            hash: None,
            filename: None,
            path: Some("../templates/test.pdf".to_string()),
            source_uri: None,
        });

        let spec = generate_openapi_spec(&schema, "/api/v1/forms");
        println!("Generated spec with PDF: {}", spec);

        // Verify it's valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&spec);
        assert!(
            parsed.is_ok(),
            "OpenAPI spec should be valid JSON: {:?}",
            parsed.err()
        );

        // Verify PDF endpoints exist
        let json = parsed.unwrap();
        let paths = json.get("paths").expect("should have paths");
        assert!(
            paths.get("/api/v1/forms/test/form/{id}/pdf").is_some(),
            "Should have GET PDF endpoint"
        );
        assert!(
            paths.get("/api/v1/forms/test/form/pdf").is_some(),
            "Should have POST PDF endpoint"
        );
    }
}
