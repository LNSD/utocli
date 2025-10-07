//! Tests for enum schema generation (ToSchema derive macro).
//!
//! These tests verify full enum support following utoipa's architecture:
//! - PlainEnum (unit variants only)
//! - MixedEnum (variants with fields)
//! - Serde enum representations (externally tagged, internally tagged, adjacently tagged, untagged)
//! - Rename rules and variant renaming

#![allow(dead_code)]

use serde_json::json;
use utocli::{Schema, SchemaType, ToSchema};

#[test]
fn derive_to_schema_with_plain_enum_generates_string_schema() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    enum Status {
        Active,
        Inactive,
        Pending,
    }

    //* When
    let schema = Status::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for plain enum");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::String),
        "plain enum should generate string schema"
    );
    assert_eq!(
        obj.enum_values,
        Some(vec![json!("Active"), json!("Inactive"), json!("Pending")]),
        "enum values should match variant names"
    );
}

#[test]
fn derive_to_schema_with_serde_rename_all_transforms_variant_names() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    #[allow(clippy::enum_variant_names)]
    enum HttpMethod {
        GetRequest,
        PostRequest,
        DeleteRequest,
    }

    //* When
    let schema = HttpMethod::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for enum with rename_all");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::String),
        "enum should generate string schema"
    );
    assert_eq!(
        obj.enum_values,
        Some(vec![
            json!("getRequest"),
            json!("postRequest"),
            json!("deleteRequest")
        ]),
        "variant names should be transformed by rename_all rule"
    );
}

#[test]
fn derive_to_schema_with_serde_rename_variant_overrides_variant_name() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    enum Color {
        Red,
        #[serde(rename = "GREEN")]
        Green,
        Blue,
    }

    //* When
    let schema = Color::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for enum with renamed variant");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::String),
        "enum should generate string schema"
    );
    assert_eq!(
        obj.enum_values,
        Some(vec![json!("Red"), json!("GREEN"), json!("Blue")]),
        "renamed variant should use custom name"
    );
}

#[test]
fn derive_to_schema_with_serde_skip_excludes_variant() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    enum Priority {
        High,
        #[serde(skip)]
        Internal,
        Low,
    }

    //* When
    let schema = Priority::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for enum with skipped variant");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::String),
        "enum should generate string schema"
    );
    assert_eq!(
        obj.enum_values,
        Some(vec![json!("High"), json!("Low")]),
        "skipped variant should not appear in enum values"
    );
}

#[test]
fn derive_to_schema_with_internally_tagged_generates_object_schema() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    #[serde(tag = "type")]
    enum Event {
        Start,
        Stop,
        Pause,
    }

    //* When
    let schema = Event::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for internally tagged enum");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Object),
        "internally tagged enum should generate object schema"
    );
    assert!(
        obj.properties.is_some(),
        "internally tagged enum should have variant properties"
    );
}

#[test]
fn derive_to_schema_with_adjacently_tagged_generates_object_with_tag_property() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    #[serde(tag = "type", content = "value")]
    enum Command {
        Start,
        Stop,
        Restart,
    }

    //* When
    let schema = Command::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for adjacently tagged enum");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Object),
        "adjacently tagged enum should generate object schema"
    );
    let props = obj
        .properties
        .expect("adjacently tagged enum should have properties");
    assert!(
        props.contains_key("type"),
        "adjacently tagged enum should have tag property"
    );
}

#[test]
fn derive_to_schema_with_untagged_generates_null_schema() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    #[serde(untagged)]
    enum UntaggedEnum {
        A,
        B,
        C,
    }

    //* When
    let schema = UntaggedEnum::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for untagged enum");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Null),
        "untagged unit enum should generate null schema"
    );
}

#[test]
fn derive_to_schema_with_mixed_enum_named_fields_generates_object_with_variant_properties() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    enum ApiResponse {
        Success { code: u32, message: String },
        Error { code: u32, details: String },
    }

    //* When
    let schema = ApiResponse::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for mixed enum with named fields");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Object),
        "mixed enum should generate object schema"
    );
    let props = obj
        .properties
        .expect("mixed enum should have variant properties");
    assert!(
        props.contains_key("Success") || props.contains_key("Error"),
        "properties should contain variant names"
    );
}

#[test]
fn derive_to_schema_with_mixed_enum_unnamed_fields_generates_object_schema() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    enum Result<T> {
        Ok(T),
        Err(String),
    }

    //* When
    let schema = Result::<u32>::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for mixed enum with unnamed fields");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Object),
        "mixed enum should generate object schema"
    );
    assert!(
        obj.properties.is_some(),
        "mixed enum should have properties"
    );
}

#[test]
fn derive_to_schema_with_mixed_variants_includes_all_variant_types() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    enum Message {
        Text(String),
        Image { url: String, alt: String },
        Empty,
    }

    //* When
    let schema = Message::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for enum with mixed variants");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Object),
        "mixed enum should generate object schema"
    );
    let props = obj.properties.expect("mixed enum should have properties");
    assert_eq!(
        props.len(),
        3,
        "all three variants should be present in properties"
    );
}

#[test]
fn derive_to_schema_with_internally_tagged_mixed_enum_generates_object_schema() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    #[serde(tag = "kind")]
    enum Shape {
        Circle { radius: f64 },
        Rectangle { width: f64, height: f64 },
    }

    //* When
    let schema = Shape::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for internally tagged mixed enum");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Object),
        "internally tagged mixed enum should generate object schema"
    );
    assert!(
        obj.properties.is_some(),
        "internally tagged mixed enum should have properties"
    );
}

#[test]
fn derive_to_schema_with_adjacently_tagged_mixed_enum_generates_object_schema() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    #[serde(tag = "type", content = "data")]
    enum Animal {
        Dog { name: String },
        Cat { name: String, lives: u8 },
    }

    //* When
    let schema = Animal::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for adjacently tagged mixed enum");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Object),
        "adjacently tagged mixed enum should generate object schema"
    );
    assert!(
        obj.properties.is_some(),
        "adjacently tagged mixed enum should have properties"
    );
}

#[test]
fn derive_to_schema_with_untagged_mixed_enum_generates_object_schema() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    #[serde(untagged)]
    enum Value {
        Number(i32),
        Text(String),
    }

    //* When
    let schema = Value::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for untagged mixed enum");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Object),
        "untagged mixed enum should generate object schema"
    );
    assert!(
        obj.properties.is_some(),
        "untagged mixed enum should have properties"
    );
}

#[test]
fn derive_to_schema_with_doc_comments_extracts_description() {
    //* Given
    /// An enumeration of user roles
    #[derive(utocli::ToSchema, serde::Serialize)]
    enum UserRole {
        Admin,
        User,
        Guest,
    }

    //* When
    let schema = UserRole::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for enum with doc comments");
    };
    let description = obj
        .description
        .expect("description should be extracted from doc comments");
    assert!(
        description.contains("An enumeration of user roles"),
        "description should contain doc comment text"
    );
}

#[test]
fn derive_to_schema_with_schema_description_uses_attribute() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    #[schema(description = "Custom status enumeration")]
    enum Status {
        Active,
        Inactive,
    }

    //* When
    let schema = Status::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for enum with schema description");
    };
    assert_eq!(
        obj.description,
        Some("Custom status enumeration".to_string()),
        "description should use schema attribute value"
    );
}

#[test]
fn schema_name_returns_enum_identifier() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    enum TestEnum {
        Variant1,
        Variant2,
    }

    //* When
    let name = TestEnum::schema_name();

    //* Then
    assert_eq!(name, "TestEnum", "schema name should match enum name");
}

#[test]
fn schema_name_with_schema_as_returns_custom_name() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    #[schema(as = "CustomName")]
    enum RenamedEnum {
        A,
        B,
    }

    //* When
    let name = RenamedEnum::schema_name();

    //* Then
    assert_eq!(
        name, "CustomName",
        "schema name should use custom name from schema(as) attribute"
    );
}

#[test]
fn derive_to_schema_with_snake_case_rename_all_transforms_to_snake_case() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    #[serde(rename_all = "snake_case")]
    #[allow(clippy::enum_variant_names)]
    enum HttpMethod {
        GetRequest,
        PostRequest,
        DeleteRequest,
    }

    //* When
    let schema = HttpMethod::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for enum with snake_case rename_all");
    };
    assert_eq!(
        obj.enum_values,
        Some(vec![
            json!("get_request"),
            json!("post_request"),
            json!("delete_request")
        ]),
        "variant names should be transformed to snake_case"
    );
}

#[test]
fn derive_to_schema_with_screaming_snake_case_rename_all_transforms_to_screaming_snake_case() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    enum Severity {
        LowPriority,
        HighPriority,
    }

    //* When
    let schema = Severity::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for enum with SCREAMING_SNAKE_CASE rename_all");
    };
    assert_eq!(
        obj.enum_values,
        Some(vec![json!("LOW_PRIORITY"), json!("HIGH_PRIORITY")]),
        "variant names should be transformed to SCREAMING_SNAKE_CASE"
    );
}

#[test]
fn derive_to_schema_with_kebab_case_rename_all_transforms_to_kebab_case() {
    //* Given
    #[derive(utocli::ToSchema, serde::Serialize)]
    #[serde(rename_all = "kebab-case")]
    #[allow(clippy::enum_variant_names)]
    enum RequestType {
        GetUser,
        CreateUser,
        DeleteUser,
    }

    //* When
    let schema = RequestType::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for enum with kebab-case rename_all");
    };
    assert_eq!(
        obj.enum_values,
        Some(vec![
            json!("get-user"),
            json!("create-user"),
            json!("delete-user")
        ]),
        "variant names should be transformed to kebab-case"
    );
}
