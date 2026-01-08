// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use jsonschema::Validator;
use schemars::schema_for;
use serde_json::{json, Value};

use dsc_lib::{dscerror::DscError, types::FullyQualifiedTypeName};

#[test]
fn test_schema_without_segments() {
    let schema = Validator::new(schema_for!(FullyQualifiedTypeName).as_value()).unwrap();
    let name = "invalid_type_name";

    assert!(schema
        .validate(&json!(name))
        .unwrap_err()
        .to_string()
        .starts_with(format!(r#""{name}" does not match"#).as_str()))
}

#[test]
fn test_schema_with_invalid_character() {
    let schema = Validator::new(schema_for!(FullyQualifiedTypeName).as_value()).unwrap();
    let name = "With&Invalid/Character";

    assert!(schema
        .validate(&json!(name))
        .unwrap_err()
        .to_string()
        .starts_with(format!(r#""{name}" does not match"#).as_str()))
}

#[test]
fn test_schema_without_namespaces() {
    let schema = Validator::new(schema_for!(FullyQualifiedTypeName).as_value()).unwrap();
    let name = "Owner/Name";

    assert!(schema.validate(&json!(name)).is_ok())
}

#[test]
fn test_schema_with_one_namespace() {
    let schema = Validator::new(schema_for!(FullyQualifiedTypeName).as_value()).unwrap();
    let name = "Owner.Namespace/Name";

    assert!(schema.validate(&json!(name)).is_ok())
}

#[test]
fn test_schema_with_many_namespaces() {
    let schema = Validator::new(schema_for!(FullyQualifiedTypeName).as_value()).unwrap();
    let name = "Owner.A.B.C.D.E.F/Name";

    assert!(schema.validate(&json!(name)).is_ok())
}

#[test]
fn test_deserialize_valid() {
    let name = "Owner/Name";
    let deserialized: FullyQualifiedTypeName = serde_json::from_value(json!(name)).unwrap();
    assert_eq!(deserialized.to_string(), name.to_string())
}

#[test]
fn test_deserialize_invalid() {
    let name = "invalid_name";
    let deserializing_error = serde_json::from_value::<FullyQualifiedTypeName>(json!(name))
        .unwrap_err()
        .to_string();
    let expected_error = DscError::InvalidTypeName(
        name.to_string(),
        FullyQualifiedTypeName::VALIDATING_PATTERN.to_string(),
    )
    .to_string();

    assert_eq!(deserializing_error, expected_error)
}

#[test]
fn test_serialize_valid() {
    let name = "Owner/Name";
    let instance = FullyQualifiedTypeName::new(name).unwrap();
    let serialized: Value = serde_json::to_value(instance).unwrap();
    assert_eq!(serialized, json!(name))
}

#[test]
fn test_display() {
    let name = "Owner/Name";
    let instance = FullyQualifiedTypeName::new(name).unwrap();
    assert_eq!(format!("{instance}"), format!("{name}"))
}

#[test]
fn test_as_ref() {
    let name = "Owner/Name";
    let instance = FullyQualifiedTypeName::new(name).unwrap();
    assert_eq!(name, instance.as_ref())
}

#[test]
fn test_deref() {
    let name = "Owner/Name";
    let instance = FullyQualifiedTypeName::new(name).unwrap();
    assert_eq!(*name, *instance)
}

#[test]
fn test_default_is_empty() {
    let instance = FullyQualifiedTypeName::default();
    assert!(instance.is_empty())
}
