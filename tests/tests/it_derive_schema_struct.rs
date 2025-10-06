//! Tests for struct schema generation (ToSchema derive macro).
//!
//! These tests verify struct support following utoipa's architecture:
//! - NamedStructSchema (structs with named fields)
//! - UnnamedStructSchema (tuple structs - single field, multiple same type, multiple different types)
//! - Unit structs

#![allow(dead_code)]

use utocli::{Schema, SchemaType, ToSchema};

#[test]
fn derive_to_schema_with_single_field_unnamed_struct_inlines_wrapped_type() {
    //* Given
    /// A newtype wrapper around String
    #[derive(utocli::ToSchema)]
    struct UserId(String);

    //* When
    let schema = UserId::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for single-field unnamed struct");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::String),
        "single-field unnamed struct should inline the wrapped type's schema"
    );
}

#[test]
fn derive_to_schema_with_single_field_unnamed_struct_wrapping_custom_type() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Inner {
        value: String,
    }

    #[derive(utocli::ToSchema)]
    struct Wrapper(Inner);

    //* When
    let schema = Wrapper::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for unnamed struct wrapping custom type");
    };
    // Should inline the Inner struct's schema
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Object),
        "should inline wrapped custom type schema"
    );
}

#[test]
fn derive_to_schema_with_multiple_same_type_fields_inlines_type() {
    //* Given
    /// RGB color as tuple struct with three u8 values
    #[derive(utocli::ToSchema)]
    struct Color(u8, u8, u8);

    //* When
    let schema = Color::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for unnamed struct with same-type fields");
    };
    // All fields are same type (u8), so inline the type
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Integer),
        "multiple same-type fields should inline the common type"
    );
}

#[test]
fn derive_to_schema_with_multiple_different_type_fields_generates_array_schema() {
    //* Given
    /// Mixed tuple struct with different types
    #[derive(utocli::ToSchema)]
    struct Mixed(String, u32, bool);

    //* When
    let schema = Mixed::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for unnamed struct with mixed types");
    };
    // Multiple different types serialize as array (serde default)
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Array),
        "multiple different-type fields should generate array schema"
    );
}

#[test]
fn derive_to_schema_with_unnamed_struct_respects_description() {
    //* Given
    /// A user ID wrapper
    #[derive(utocli::ToSchema)]
    #[schema(description = "Custom user identifier")]
    struct UserId(u64);

    //* When
    let schema = UserId::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema");
    };
    assert_eq!(
        obj.description,
        Some("Custom user identifier".to_string()),
        "description should be applied to unnamed struct schema"
    );
}

#[test]
fn derive_to_schema_with_empty_unnamed_struct_generates_string_schema() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Empty();

    //* When
    let schema = Empty::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for empty unnamed struct");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::String),
        "empty unnamed struct should generate string schema (like unit struct)"
    );
}

#[test]
fn derive_to_schema_with_unit_struct_generates_string_schema() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Marker;

    //* When
    let schema = Marker::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for unit struct");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::String),
        "unit struct should generate string schema"
    );
}

#[test]
fn derive_to_schema_with_unit_struct_respects_description() {
    //* Given
    /// A marker struct
    #[derive(utocli::ToSchema)]
    #[schema(description = "Marker type")]
    struct Marker;

    //* When
    let schema = Marker::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema");
    };
    assert_eq!(
        obj.description,
        Some("Marker type".to_string()),
        "description should be applied to unit struct schema"
    );
}

// Named Struct Tests (existing functionality verification)

#[test]
fn derive_to_schema_with_named_struct_generates_object_with_properties() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct User {
        id: u64,
        name: String,
        email: Option<String>,
    }

    //* When
    let schema = User::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for named struct");
    };
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Object),
        "named struct should generate object schema"
    );
    assert!(
        obj.properties.is_some(),
        "named struct should have properties"
    );
    let props = obj.properties.expect("properties should be present");
    assert_eq!(props.len(), 3, "should have 3 properties");
    assert!(props.contains_key("id"), "should have id property");
    assert!(props.contains_key("name"), "should have name property");
    assert!(props.contains_key("email"), "should have email property");
}

#[test]
fn schema_name_returns_struct_identifier() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct TestStruct {
        field: String,
    }

    //* When
    let name = TestStruct::schema_name();

    //* Then
    assert_eq!(name, "TestStruct", "schema name should match struct name");
}

#[test]
fn schema_name_with_schema_as_returns_custom_name() {
    //* Given
    #[derive(utocli::ToSchema)]
    #[schema(as = "CustomName")]
    struct RenamedStruct {
        field: String,
    }

    //* When
    let name = RenamedStruct::schema_name();

    //* Then
    assert_eq!(
        name, "CustomName",
        "schema name should use custom name from schema(as) attribute"
    );
}
