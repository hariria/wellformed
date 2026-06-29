//! Integration tests for wel_schema! macro.

use wellformed_macros::{form_schema, wel_schema, wellformed};

// Generate struct from the simple test schema
wel_schema!("test_schemas/simple.json");
form_schema!(pub mod signup = "test_schemas/simple.json");
form_schema!(pub mod contact = "test_schemas/simple.json");

const SIMPLE_SCHEMA: wellformed::EmbeddedSchema = wellformed!("test_schemas/simple.json");

#[test]
fn test_generated_struct_exists() {
    // Verify the struct was generated with expected fields
    let form = SimpleForm {
        name: "Alice".to_string(),
        age: Some(30),
        email: "alice@example.com".to_string(),
    };

    assert_eq!(form.name, "Alice");
    assert_eq!(form.age, Some(30));
    assert_eq!(form.email, "alice@example.com");
}

#[test]
fn test_generated_struct_default() {
    // Verify Default is derived
    let form = SimpleForm::default();
    assert_eq!(form.name, "");
    assert_eq!(form.age, None);
    assert_eq!(form.email, "");
}

#[test]
fn test_validate_method() {
    let form = SimpleForm {
        name: "Bob".to_string(),
        age: Some(25),
        email: "bob@example.com".to_string(),
    };

    // The validate method should exist and return a ValidationResult
    let result = form.validate();
    assert!(result.is_valid());
}

#[test]
fn test_serde_roundtrip() {
    let form = SimpleForm {
        name: "Charlie".to_string(),
        age: Some(40),
        email: "charlie@example.com".to_string(),
    };

    // Verify serde derives work
    let json = serde_json::to_string(&form).unwrap();
    let parsed: SimpleForm = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.name, form.name);
    assert_eq!(parsed.age, form.age);
    assert_eq!(parsed.email, form.email);
}

#[test]
fn test_form_schema_generates_module_facade() {
    let form = signup::Values {
        name: "Ada".to_string(),
        age: Some(36),
        email: "ada@example.com".to_string(),
    };

    let result = signup::validate(&form);
    assert!(result.is_valid());
    assert_eq!(signup::ID, "simple_form");
    assert_eq!(signup::TITLE, Some("Simple Test Form"));
    assert_eq!(signup::FIELDS.len(), 3);
    assert_eq!(signup::FIELD_NAME.name, "name");
    assert_eq!(signup::FIELD_NAME.rust_name, "name");
    assert_eq!(signup::FIELD_NAME.kind, wellformed::FieldKind::String);
    assert_eq!(signup::CLIENT.client_module, None);
    assert!(signup::CLIENT.schema_json.contains("\"simple_form\""));
}

#[test]
fn test_form_schema_can_generate_another_module_facade() {
    let values = contact::validate_json(r#"{"name":"Ada","age":36,"email":"ada@example.com"}"#)
        .expect("valid form");

    assert_eq!(values.name, "Ada");
    assert_eq!(contact::FIELDS.len(), 3);
}

#[test]
fn test_wellformed_expression_embeds_schema() {
    let schema = wellformed!("test_schemas/simple.json");
    let (result, value) = schema
        .validate_json(r#"{"name":"Ada","age":36,"email":"ada@example.com"}"#)
        .expect("runtime validation should succeed");

    assert!(result.is_valid());
    assert_eq!(value["name"], serde_json::json!("Ada"));
}

#[test]
fn test_wellformed_expression_supports_const_binding() {
    let (result, value) = SIMPLE_SCHEMA
        .validate_json(r#"{"name":"Ada","age":36,"email":"ada@example.com"}"#)
        .expect("runtime validation should succeed");

    assert!(result.is_valid());
    assert_eq!(value["email"], serde_json::json!("ada@example.com"));
}

#[test]
fn test_form_schema_validates_json_values() {
    let values = signup::validate_json(r#"{"name":"Ada","age":36,"email":"ada@example.com"}"#)
        .expect("valid form");

    assert_eq!(values.name, "Ada");
    assert_eq!(values.age, Some(36));
    assert_eq!(values.email, "ada@example.com");
}

#[test]
fn test_form_schema_returns_errors_with_submitted_values() {
    let errors = signup::validate_json(r#"{"name":42,"age":36,"email":"ada@example.com"}"#)
        .expect_err("invalid field type");

    assert!(!errors.is_valid());
    assert_eq!(errors.values["name"], 42);
    assert_eq!(errors.errors[0].path, "/name");
}

#[test]
fn test_form_schema_builds_form_state() {
    let values = signup::Values {
        name: "Ada".to_string(),
        age: None,
        email: "ada@example.com".to_string(),
    };

    let state = signup::state(values);
    let name = signup::field_name(&state);

    assert!(!name.invalid());
    assert_eq!(name.error_id(), "simple_form-name-error");
    assert_eq!(name.value_str(), "Ada");
}

#[test]
fn test_form_schema_state_with_errors_preserves_submitted_values() {
    let errors = signup::validate_json(r#"{"name":42,"age":36,"email":"ada@example.com"}"#)
        .expect_err("invalid field type");
    let state = signup::state_with_errors(errors);
    let name = signup::field_name(&state);

    assert!(name.invalid());
    assert_eq!(name.value_json(), state.submitted_values.pointer("/name"));
}
