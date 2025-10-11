//! Base tests for ToSchema derive macro.
//!
//! This test suite covers fundamental ToSchema functionality:
//! - Basic struct and enum schema generation
//! - Custom functions (schema_with)
//! - Inline attribute
//! - Schema naming (as attribute)
//! - Default values (schema default attribute)
//!
//! For specialized tests, see:
//! - it_derive_schema_struct.rs: Struct-specific tests
//! - it_derive_schema_enum.rs: Enum-specific tests
//! - it_derive_schema_attributes.rs: Schema attributes
//! - it_derive_serde.rs: Serde attribute integration
//! - it_derive_validation.rs: Validation attributes

#![allow(dead_code)]

use utocli::{Object, RefOr, Schema, SchemaFormat, SchemaType, ToSchema};

#[test]
fn derive_to_schema_with_struct_and_doc_comments_generates_object_schema() {
    //* Given
    /// A simple user struct
    #[derive(utocli::ToSchema)]
    struct User {
        /// The user's unique identifier
        id: u64,
        /// The user's name
        name: String,
        /// The user's email (optional)
        email: Option<String>,
    }

    //* When
    let schema = User::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    assert_eq!(
        User::schema_name(),
        "User",
        "schema name should match struct name"
    );
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Object),
        "schema type should be Object for structs"
    );
    assert!(
        obj.description.is_some(),
        "description should be extracted from doc comments"
    );
    assert!(
        obj.properties.is_some(),
        "properties should be generated for struct fields"
    );
}

#[test]
fn derive_to_schema_with_enum_generates_string_schema_with_enum_values() {
    //* Given
    /// A simple enum
    #[derive(utocli::ToSchema)]
    enum Status {
        Active,
        Inactive,
        Pending,
    }

    //* When
    let schema = Status::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for enum");
    };

    assert_eq!(
        Status::schema_name(),
        "Status",
        "schema name should match enum name"
    );
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::String),
        "enum schema type should be String"
    );
    assert!(
        obj.enum_values.is_some(),
        "enum should have enum_values defined"
    );

    let enum_values = obj
        .enum_values
        .expect("enum_values should be present for enums");
    assert_eq!(enum_values.len(), 3, "should have 3 enum variants");
}

#[test]
fn derive_to_schema_with_inline_vec_generates_inline_schema() {
    //* Given
    /// A command information struct
    #[derive(utocli::ToSchema)]
    struct CommandInfo {
        /// The command name
        name: String,
        /// The command description
        description: String,
    }

    /// CLI specification with inline commands array
    #[derive(utocli::ToSchema)]
    struct CliSpec {
        /// CLI version
        version: String,
        /// List of available commands (inlined)
        #[schema(inline)]
        commands: Vec<CommandInfo>,
    }

    //* When
    let schema = CliSpec::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for CliSpec");
    };

    let props = obj
        .properties
        .expect("CliSpec should have properties defined");

    // Check that commands field exists
    assert!(
        props.contains_key("commands"),
        "commands field should be present in schema"
    );

    // Check that commands is an array with inline object items
    let RefOr::T(Schema::Array(array)) =
        props.get("commands").expect("commands field should exist")
    else {
        panic!("Expected commands to be an inline Array schema");
    };

    let items = array
        .items
        .as_ref()
        .expect("commands array should have items defined");

    let RefOr::T(Schema::Object(obj)) = &**items else {
        panic!("Expected inline object schema for array items, not a reference");
    };

    // Verify it's an object type
    assert_eq!(
        obj.schema_type,
        Some(SchemaType::Object),
        "inlined array items should be object type"
    );

    // Verify the object has properties (name and description)
    let item_props = obj
        .properties
        .as_ref()
        .expect("inlined object should have properties");

    assert!(
        item_props.contains_key("name"),
        "inlined object should have 'name' property"
    );
    assert!(
        item_props.contains_key("description"),
        "inlined object should have 'description' property"
    );
}

#[test]
fn derive_to_schema_with_custom_function_overrides_inference() {
    //* Given
    fn custom_email_schema() -> Schema {
        Schema::Object(Box::new(
            Object::new()
                .schema_type(SchemaType::String)
                .format(SchemaFormat::Email),
        ))
    }

    #[derive(utocli::ToSchema)]
    struct UserWithCustomEmail {
        name: String,
        #[schema(schema_with = custom_email_schema)]
        email: String,
    }

    //* When
    let schema = UserWithCustomEmail::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema");
    };

    let props = obj
        .properties
        .expect("struct should have properties defined");

    // Verify email field uses custom schema
    let RefOr::T(Schema::Object(email_schema)) =
        props.get("email").expect("email field should exist")
    else {
        panic!("Expected email to use custom schema as RefOr::T, not a reference");
    };

    // Verify custom schema has email format
    assert_eq!(
        email_schema.format,
        Some(SchemaFormat::Email),
        "email field should use custom format from schema_with function"
    );

    // Verify name field still uses default inference
    assert!(
        props.contains_key("name"),
        "name field should use standard type inference"
    );
}

#[test]
fn derive_to_schema_with_custom_function_returns_complex_schema() {
    //* Given
    fn geo_coordinate_schema() -> Schema {
        use serde_json::json;

        Schema::Object(Box::new(
            Object::new()
                .schema_type(SchemaType::Object)
                .properties(
                    [
                        (
                            "lat".to_string(),
                            RefOr::T(Schema::Object(Box::new(
                                Object::new().schema_type(SchemaType::Number),
                            ))),
                        ),
                        (
                            "lon".to_string(),
                            RefOr::T(Schema::Object(Box::new(
                                Object::new().schema_type(SchemaType::Number),
                            ))),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                )
                .example(json!({"lat": 37.7749, "lon": -122.4194})),
        ))
    }

    #[derive(utocli::ToSchema)]
    struct Location {
        name: String,
        #[schema(schema_with = geo_coordinate_schema)]
        coordinates: String, // Type doesn't matter with schema_with
    }

    //* When
    let schema = Location::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema");
    };

    let props = obj.properties.expect("should have properties");

    // Verify coordinates uses complex custom schema
    let RefOr::T(Schema::Object(coord_schema)) = props
        .get("coordinates")
        .expect("coordinates field should exist")
    else {
        panic!("Expected coordinates to use custom schema");
    };

    // Verify the custom schema is an object with lat/lon properties
    assert_eq!(
        coord_schema.schema_type,
        Some(SchemaType::Object),
        "coordinates should be object type from custom schema"
    );

    let coord_props = coord_schema
        .properties
        .as_ref()
        .expect("coordinates should have properties");

    assert!(
        coord_props.contains_key("lat"),
        "custom schema should define lat property"
    );
    assert!(
        coord_props.contains_key("lon"),
        "custom schema should define lon property"
    );

    // Verify example is present
    assert!(
        coord_schema.example.is_some(),
        "custom schema should include example"
    );
}

#[test]
fn derive_to_schema_with_default_values_includes_defaults() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct UserWithDefaults {
        /// User ID (no default)
        id: u64,

        /// Username with explicit string default
        #[schema(default = "guest")]
        username: String,

        /// Score with numeric default
        #[schema(default = 0)]
        score: i32,

        /// Rate with float default
        #[schema(default = 1.5)]
        rate: f64,
    }

    //* When
    let schema = UserWithDefaults::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema");
    };

    let props = obj.properties.expect("should have properties");

    // Check id field has no default
    let RefOr::T(Schema::Object(id_schema)) = props.get("id").expect("id field should exist")
    else {
        panic!("Expected inline schema for id field");
    };
    assert_eq!(id_schema.default, None, "id should not have default value");

    // Check username field has explicit default
    let RefOr::T(Schema::Object(username_schema)) =
        props.get("username").expect("username field should exist")
    else {
        panic!("Expected inline schema for username field");
    };
    assert!(
        username_schema.default.is_some(),
        "username should have default value"
    );
    assert_eq!(
        username_schema.default.as_ref().unwrap().as_str(),
        Some("guest"),
        "username default should be 'guest'"
    );

    // Check score field has numeric default
    let RefOr::T(Schema::Object(score_schema)) =
        props.get("score").expect("score field should exist")
    else {
        panic!("Expected inline schema for score field");
    };
    assert!(
        score_schema.default.is_some(),
        "score should have default value"
    );
    assert_eq!(
        score_schema.default.as_ref().unwrap().as_i64(),
        Some(0),
        "score default should be 0"
    );

    // Check rate field has float default
    let RefOr::T(Schema::Object(rate_schema)) = props.get("rate").expect("rate field should exist")
    else {
        panic!("Expected inline schema for rate field");
    };
    assert!(
        rate_schema.default.is_some(),
        "rate should have default value"
    );
    assert_eq!(
        rate_schema.default.as_ref().unwrap().as_f64(),
        Some(1.5),
        "rate default should be 1.5"
    );
}

#[test]
fn derive_to_schema_with_as_attribute_uses_custom_schema_name() {
    //* Given
    #[derive(utocli::ToSchema)]
    #[schema(as = "CustomSchemaName")]
    struct InternalStruct {
        field: String,
    }

    //* When
    let schema_name = InternalStruct::schema_name();

    //* Then
    assert_eq!(
        schema_name, "CustomSchemaName",
        "schema_name should use custom name from 'as' attribute"
    );
}

#[test]
fn derive_to_schema_without_as_attribute_uses_struct_name() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct NormalStruct {
        field: String,
    }

    //* When
    let schema_name = NormalStruct::schema_name();

    //* Then
    assert_eq!(
        schema_name, "NormalStruct",
        "schema_name should default to struct name when no 'as' attribute"
    );
}
