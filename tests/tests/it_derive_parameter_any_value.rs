//! E2E tests for AnyValue support in ToParameter derive macro

#![allow(dead_code)]

use utocli::{RefOr, Schema};

#[test]
fn to_parameter_with_literal_string_example_and_default_generates_correct_values() {
    //* Given
    #[derive(utocli::ToParameter)]
    struct MyParam {
        #[param(example = "5", default = "1")]
        count: i32,
    }

    //* When
    let params = MyParam::parameters();

    //* Then
    assert_eq!(params.len(), 1, "should generate 1 parameter");
    let param = &params[0];
    assert_eq!(param.name, "count");
    let Some(RefOr::T(Schema::Object(obj))) = &param.schema else {
        panic!("Expected object schema for count parameter");
    };
    assert_eq!(
        obj.example,
        Some(serde_json::json!("5")),
        "example should be the literal string '5'"
    );
    assert_eq!(
        obj.default,
        Some(serde_json::json!("1")),
        "default should be the literal string '1'"
    );
}

#[test]
fn to_parameter_with_json_object_example_generates_correct_value() {
    //* Given
    #[derive(utocli::ToParameter)]
    struct MyParam {
        #[param(example = serde_json::json!({"min": 0, "max": 100}))]
        range: String,
    }

    //* When
    let params = MyParam::parameters();

    //* Then
    assert_eq!(params.len(), 1, "should generate 1 parameter");
    let param = &params[0];
    assert_eq!(param.name, "range");
    let Some(RefOr::T(Schema::Object(obj))) = &param.schema else {
        panic!("Expected object schema for range parameter");
    };
    let example = obj.example.as_ref().expect("should have example");
    assert_eq!(
        example.get("min").and_then(|v| v.as_i64()),
        Some(0),
        "example should contain 'min': 0"
    );
    assert_eq!(
        example.get("max").and_then(|v| v.as_i64()),
        Some(100),
        "example should contain 'max': 100"
    );
}

#[test]
fn to_parameter_with_mixed_literal_and_json_generates_correct_values() {
    //* Given
    #[derive(utocli::ToParameter)]
    struct MyParam {
        #[param(example = "test", default = serde_json::json!({"fallback": "default"}))]
        value: String,
    }

    //* When
    let params = MyParam::parameters();

    //* Then
    assert_eq!(params.len(), 1, "should generate 1 parameter");
    let param = &params[0];
    assert_eq!(param.name, "value");
    let Some(RefOr::T(Schema::Object(obj))) = &param.schema else {
        panic!("Expected object schema for value parameter");
    };
    assert_eq!(
        obj.example,
        Some(serde_json::json!("test")),
        "example should be the literal string 'test'"
    );
    let default = obj.default.as_ref().expect("should have default");
    assert_eq!(
        default.get("fallback").and_then(|v| v.as_str()),
        Some("default"),
        "default should contain 'fallback': 'default' from json!()"
    );
}
