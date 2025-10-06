//! E2E tests for serde attribute support in ToSchema derive macro.
//!
//! This test suite validates that utocli-derive correctly handles all serde attributes
//! following the same patterns as utoipa-gen, ensuring 100% serde feature parity.

#![allow(dead_code)]

use utocli::{Schema, ToSchema};

#[test]
fn derive_to_schema_with_serde_skip_excludes_field() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    struct Person {
        name: String,
        #[serde(skip)]
        secret: String,
    }

    //* When
    let schema = Person::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(props.contains_key("name"), "name field should be present");
    assert!(
        !props.contains_key("secret"),
        "field with serde(skip) should be excluded from schema"
    );
}

#[test]
fn derive_to_schema_with_serde_skip_serializing_excludes_field() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    struct Config {
        visible: String,
        #[serde(skip_serializing)]
        hidden: String,
    }

    //* When
    let schema = Config::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(
        props.contains_key("visible"),
        "visible field should be present"
    );
    assert!(
        !props.contains_key("hidden"),
        "field with serde(skip_serializing) should be excluded from schema"
    );
}

#[test]
fn derive_to_schema_with_serde_skip_deserializing_excludes_field() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    struct Data {
        input: String,
        #[serde(skip_deserializing)]
        computed: String,
    }

    //* When
    let schema = Data::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(props.contains_key("input"), "input field should be present");
    assert!(
        !props.contains_key("computed"),
        "field with serde(skip_deserializing) should be excluded from schema"
    );
}

#[test]
fn derive_to_schema_with_schema_skip_takes_precedence() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    struct Combined {
        visible: String,
        #[schema(skip)]
        schema_hidden: String,
    }

    //* When
    let schema = Combined::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(
        props.contains_key("visible"),
        "visible field should be present"
    );
    assert!(
        !props.contains_key("schema_hidden"),
        "field with schema(skip) should be excluded"
    );
}

#[test]
fn derive_to_schema_with_serde_skip_combined_with_schema_attributes_excludes_field() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    struct AttributeCombination {
        visible: String,
        #[serde(skip)]
        #[schema(description = "This should not appear")]
        invisible: String,
    }

    //* When
    let schema = AttributeCombination::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(
        props.contains_key("visible"),
        "visible field should be present"
    );
    assert!(
        !props.contains_key("invisible"),
        "field with serde(skip) should be excluded even with other schema attributes"
    );
}

#[test]
fn derive_to_schema_with_serde_rename_applies_field_rename() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    struct ApiConfig {
        #[serde(rename = "configValue")]
        config_value: String,
    }

    //* When
    let schema = ApiConfig::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(
        props.contains_key("configValue"),
        "field should be renamed according to serde(rename)"
    );
    assert!(
        !props.contains_key("config_value"),
        "original field name should not be present"
    );
}

#[test]
fn derive_to_schema_with_serde_rename_takes_precedence_over_schema_rename() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    struct PrecedenceTest {
        #[serde(rename = "fromSerde")]
        #[schema(rename = "fromSchema")]
        field: String,
    }

    //* When
    let schema = PrecedenceTest::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(
        props.contains_key("fromSerde"),
        "serde(rename) should take precedence over schema(rename)"
    );
    assert!(
        !props.contains_key("fromSchema"),
        "schema(rename) should be overridden by serde(rename)"
    );
    assert!(
        !props.contains_key("field"),
        "original field name should not be present"
    );
}

#[test]
fn derive_to_schema_with_serde_rename_all_container_applies_camel_case() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    #[serde(rename_all = "camelCase")]
    struct ApiResponse {
        status_code: u32,
        error_message: String,
        request_id: String,
    }

    //* When
    let schema = ApiResponse::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(
        props.contains_key("statusCode"),
        "status_code should be renamed to camelCase"
    );
    assert!(
        props.contains_key("errorMessage"),
        "error_message should be renamed to camelCase"
    );
    assert!(
        props.contains_key("requestId"),
        "request_id should be renamed to camelCase"
    );
    assert!(
        !props.contains_key("status_code"),
        "original field names should not be present"
    );
}

#[test]
fn derive_to_schema_with_rename_all_and_individual_rename_respects_precedence() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    #[serde(rename_all = "camelCase")]
    struct MixedRenames {
        normal_field: String,
        #[serde(rename = "customName")]
        special_field: String,
    }

    //* When
    let schema = MixedRenames::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(
        props.contains_key("normalField"),
        "normal_field should be renamed according to rename_all"
    );
    assert!(
        props.contains_key("customName"),
        "field with explicit rename should use individual rename"
    );
    assert!(
        !props.contains_key("special_field"),
        "original field name should not be present"
    );
}

#[test]
fn derive_to_schema_with_serde_default_field_makes_field_optional() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    struct Settings {
        required_field: String,
        #[serde(default)]
        optional_field: String,
    }

    //* When
    let schema = Settings::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let required = obj.required.as_ref().expect("required should be present");
    assert!(
        required.contains(&"required_field".to_string()),
        "field without default should be required"
    );
    assert!(
        !required.contains(&"optional_field".to_string()),
        "field with serde(default) should not be required"
    );
}

#[test]
fn derive_to_schema_with_serde_default_container_makes_all_fields_optional() {
    //* Given
    #[derive(Default, serde::Deserialize, utocli::ToSchema)]
    #[serde(default)]
    struct AllOptional {
        field_one: String,
        field_two: i32,
        field_three: bool,
    }

    //* When
    let schema = AllOptional::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    assert!(
        obj.required.as_ref().is_none_or(|r| r.is_empty()),
        "container-level default should make all fields optional"
    );
}

#[test]
fn derive_to_schema_with_mixed_default_and_option_types_handles_requirements() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    struct MixedRequirements {
        required: String,
        #[serde(default)]
        optional_via_default: String,
        optional_via_option: Option<String>,
    }

    //* When
    let schema = MixedRequirements::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let required = obj.required.as_ref().expect("required should be present");
    assert_eq!(required.len(), 1, "only one field should be required");
    assert!(
        required.contains(&"required".to_string()),
        "field without default or Option should be required"
    );
    assert!(
        !required.contains(&"optional_via_default".to_string()),
        "field with default should not be required"
    );
    assert!(
        !required.contains(&"optional_via_option".to_string()),
        "Option field should not be required"
    );
}

#[test]
fn derive_to_schema_with_container_default_overrides_field_requirements() {
    //* Given
    #[derive(Default, serde::Deserialize, utocli::ToSchema)]
    #[serde(default)]
    struct ContainerDefault {
        normally_required: String,
        already_optional: Option<i32>,
    }

    //* When
    let schema = ContainerDefault::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    assert!(
        obj.required.as_ref().is_none_or(|r| r.is_empty()),
        "container default should make even normally required fields optional"
    );
}

#[test]
fn derive_to_schema_with_skip_serializing_if_makes_field_optional() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    struct Response {
        always_present: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sometimes_present: Option<String>,
    }

    //* When
    let schema = Response::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let required = obj.required.as_ref().expect("required should be present");
    assert!(
        required.contains(&"always_present".to_string()),
        "field without skip_serializing_if should be required"
    );
    assert!(
        !required.contains(&"sometimes_present".to_string()),
        "field with skip_serializing_if should not be required"
    );
}

#[test]
fn derive_to_schema_with_skip_serializing_if_on_non_option_field_makes_optional() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    struct ConditionalField {
        always: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        conditional: String,
    }

    //* When
    let schema = ConditionalField::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let required = obj.required.as_ref().expect("required should be present");
    assert_eq!(required.len(), 1, "only one field should be required");
    assert!(
        required.contains(&"always".to_string()),
        "field without skip_serializing_if should be required"
    );
    assert!(
        !required.contains(&"conditional".to_string()),
        "field with skip_serializing_if should not be required"
    );
}

#[test]
fn derive_to_schema_with_skip_serializing_if_combined_with_option_makes_optional() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    struct DoubleOptional {
        required: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        optional: Option<String>,
    }

    //* When
    let schema = DoubleOptional::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let required = obj.required.as_ref().expect("required should be present");
    assert_eq!(
        required.len(),
        1,
        "only required field should be in required list"
    );
    assert!(
        required.contains(&"required".to_string()),
        "non-optional field should be required"
    );
    assert!(
        !required.contains(&"optional".to_string()),
        "Option field with skip_serializing_if should not be required"
    );
}

// NOTE: These tests are commented out because serde_with is not a dependency.
// The double_option detection logic is implemented but requires serde_with crate.
//
// #[test]
// fn derive_to_schema_with_double_option_makes_field_optional() {
//     //* Given
//     #[derive(serde::Deserialize, utocli::ToSchema)]
//     struct DoubleOptionField {
//         required: String,
//         #[serde(with = "::serde_with::rust::double_option")]
//         double_optional: String,
//     }
//
//     //* When
//     let schema = DoubleOptionField::schema();
//
//     //* Then
//     let Schema::Object(obj) = schema else {
//         panic!("Expected Object schema for struct");
//     };
//
//     let required = obj.required.as_ref().expect("required should be present");
//     assert_eq!(
//         required.len(),
//         1,
//         "only one field should be required"
//     );
//     assert!(
//         required.contains(&"required".to_string()),
//         "field without double_option should be required"
//     );
//     assert!(
//         !required.contains(&"double_optional".to_string()),
//         "field with double_option should not be required"
//     );
// }
//
// #[test]
// fn derive_to_schema_with_double_option_and_option_type_handles_correctly() {
//     //* Given
//     #[derive(serde::Deserialize, utocli::ToSchema)]
//     struct MixedOptionals {
//         required: String,
//         #[serde(with = "::serde_with::rust::double_option")]
//         double_opt: Option<Option<String>>,
//         single_opt: Option<String>,
//     }
//
//     //* When
//     let schema = MixedOptionals::schema();
//
//     //* Then
//     let Schema::Object(obj) = schema else {
//         panic!("Expected Object schema for struct");
//     };
//
//     let required = obj.required.as_ref().expect("required should be present");
//     assert_eq!(
//         required.len(),
//         1,
//         "only required field should be in required list"
//     );
//     assert!(
//         required.contains(&"required".to_string()),
//         "non-optional field should be required"
//     );
//     assert!(
//         !required.contains(&"double_opt".to_string()),
//         "double_option field should not be required"
//     );
//     assert!(
//         !required.contains(&"single_opt".to_string()),
//         "Option field should not be required"
//     );
// }

#[test]
fn derive_to_schema_with_complex_required_fields_applies_correct_logic() {
    //* Given
    #[derive(Default, serde::Deserialize, utocli::ToSchema)]
    struct ComplexRequirements {
        // Required: not Option, no serde attributes
        definitely_required: String,

        // Not required: has default
        #[serde(default)]
        has_default: String,

        // Not required: has skip_serializing_if
        #[serde(skip_serializing_if = "String::is_empty")]
        conditionally_serialized: String,

        // Not required: is Option
        is_option: Option<String>,
    }

    //* When
    let schema = ComplexRequirements::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let required = obj.required.as_ref().expect("required should be present");
    assert_eq!(required.len(), 1, "only one field should be required");
    assert!(
        required.contains(&"definitely_required".to_string()),
        "field without any optional indicators should be required"
    );
    assert!(
        !required.contains(&"has_default".to_string()),
        "field with default should not be required"
    );
    assert!(
        !required.contains(&"conditionally_serialized".to_string()),
        "field with skip_serializing_if should not be required"
    );
    assert!(
        !required.contains(&"is_option".to_string()),
        "Option field should not be required"
    );
}

#[test]
fn derive_to_schema_with_all_optional_indicators_makes_field_optional() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    struct AllOptional {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        triple_optional: Option<String>,
    }

    //* When
    let schema = AllOptional::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    assert!(
        obj.required.as_ref().is_none_or(|r| r.is_empty()),
        "field with multiple optional indicators should not be required"
    );
}

#[test]
fn derive_to_schema_with_no_optional_indicators_makes_field_required() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    struct AllRequired {
        field_one: String,
        field_two: i32,
        field_three: bool,
    }

    //* When
    let schema = AllRequired::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let required = obj.required.as_ref().expect("required should be present");
    assert_eq!(
        required.len(),
        3,
        "all fields without optional indicators should be required"
    );
    assert!(required.contains(&"field_one".to_string()));
    assert!(required.contains(&"field_two".to_string()));
    assert!(required.contains(&"field_three".to_string()));
}

#[test]
fn derive_to_schema_with_snake_case_rename_all_applies_correctly() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    #[serde(rename_all = "snake_case")]
    #[allow(non_snake_case)]
    struct SnakeCaseTest {
        FirstField: String,
        SecondField: String,
    }

    //* When
    let schema = SnakeCaseTest::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(props.contains_key("first_field"), "should apply snake_case");
    assert!(
        props.contains_key("second_field"),
        "should apply snake_case"
    );
}

#[test]
fn derive_to_schema_with_pascal_case_rename_all_applies_correctly() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    #[serde(rename_all = "PascalCase")]
    struct PascalCaseTest {
        first_field: String,
        second_field: String,
    }

    //* When
    let schema = PascalCaseTest::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(props.contains_key("FirstField"), "should apply PascalCase");
    assert!(props.contains_key("SecondField"), "should apply PascalCase");
}

#[test]
fn derive_to_schema_with_screaming_snake_case_rename_all_applies_correctly() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    struct ScreamingSnakeCaseTest {
        first_field: String,
        second_field: String,
    }

    //* When
    let schema = ScreamingSnakeCaseTest::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(
        props.contains_key("FIRST_FIELD"),
        "should apply SCREAMING_SNAKE_CASE"
    );
    assert!(
        props.contains_key("SECOND_FIELD"),
        "should apply SCREAMING_SNAKE_CASE"
    );
}

#[test]
fn derive_to_schema_with_kebab_case_rename_all_applies_correctly() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    #[serde(rename_all = "kebab-case")]
    struct KebabCaseTest {
        first_field: String,
        second_field: String,
    }

    //* When
    let schema = KebabCaseTest::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(props.contains_key("first-field"), "should apply kebab-case");
    assert!(
        props.contains_key("second-field"),
        "should apply kebab-case"
    );
}

#[test]
fn derive_to_schema_with_screaming_kebab_case_rename_all_applies_correctly() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    #[serde(rename_all = "SCREAMING-KEBAB-CASE")]
    struct ScreamingKebabCaseTest {
        first_field: String,
        second_field: String,
    }

    //* When
    let schema = ScreamingKebabCaseTest::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(
        props.contains_key("FIRST-FIELD"),
        "should apply SCREAMING-KEBAB-CASE"
    );
    assert!(
        props.contains_key("SECOND-FIELD"),
        "should apply SCREAMING-KEBAB-CASE"
    );
}

#[test]
fn derive_to_schema_with_serde_and_schema_attributes_combined_works() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[schema(description = "A comprehensive test struct")]
    struct CombinedAttributes {
        #[serde(rename = "customId")]
        #[schema(description = "The unique identifier")]
        id: u64,

        #[serde(default)]
        #[schema(example = "test@example.com")]
        email: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        optional_field: Option<String>,

        #[serde(skip)]
        internal: String,
    }

    //* When
    let schema = CombinedAttributes::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    // Verify properties
    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(
        props.contains_key("customId"),
        "serde rename should be applied"
    );
    assert!(props.contains_key("email"), "email should be present");
    assert!(
        props.contains_key("optionalField"),
        "optional field should be present with camelCase"
    );
    assert!(
        !props.contains_key("internal"),
        "skipped field should be excluded"
    );

    // Verify required fields
    let required = obj.required.as_ref().expect("required should be present");
    assert!(
        required.contains(&"customId".to_string()),
        "id should be required"
    );
    assert!(
        !required.contains(&"email".to_string()),
        "field with default should not be required"
    );
    assert!(
        !required.contains(&"optionalField".to_string()),
        "optional field should not be required"
    );
}

#[test]
fn derive_to_schema_with_nested_structures_preserves_serde_rules() {
    //* Given
    #[derive(Default, serde::Deserialize, utocli::ToSchema)]
    #[serde(rename_all = "camelCase")]
    struct Parent {
        parent_field: String,
        #[serde(default)]
        optional_child: Child,
    }

    #[derive(Default, serde::Deserialize, utocli::ToSchema)]
    #[serde(rename_all = "camelCase")]
    struct Child {
        child_field: String,
        #[serde(skip)]
        internal_field: String,
    }

    //* When
    let parent_schema = Parent::schema();
    let child_schema = Child::schema();

    //* Then
    // Verify parent schema
    let Schema::Object(parent_obj) = parent_schema else {
        panic!("Expected Object schema for parent struct");
    };
    let parent_props = parent_obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(
        parent_props.contains_key("parentField"),
        "parent field should use camelCase"
    );
    assert!(
        parent_props.contains_key("optionalChild"),
        "child field should use camelCase"
    );

    let parent_required = parent_obj
        .required
        .as_ref()
        .expect("required should be present");
    assert!(
        !parent_required.contains(&"optionalChild".to_string()),
        "child with default should not be required"
    );

    // Verify child schema
    let Schema::Object(child_obj) = child_schema else {
        panic!("Expected Object schema for child struct");
    };
    let child_props = child_obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert!(
        child_props.contains_key("childField"),
        "child field should use camelCase"
    );
    assert!(
        !child_props.contains_key("internalField"),
        "skipped field should be excluded"
    );
}

#[test]
fn derive_to_schema_with_enum_variants_respects_serde_attributes() {
    //* Given
    #[derive(serde::Deserialize, utocli::ToSchema)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    enum Status {
        Active,
        Inactive,
        #[serde(rename = "CUSTOM_PENDING")]
        Pending,
    }

    //* When
    let schema = Status::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for enum");
    };

    let enum_values = obj
        .enum_values
        .as_ref()
        .expect("enum_values should be present");
    assert_eq!(enum_values.len(), 3, "should have 3 enum variants");

    // Verify enum values match serde attributes
    let values_str: Vec<String> = enum_values
        .iter()
        .map(|v| v.to_string().trim_matches('"').to_string())
        .collect();

    assert!(
        values_str.contains(&"ACTIVE".to_string()),
        "should apply SCREAMING_SNAKE_CASE to Active"
    );
    assert!(
        values_str.contains(&"INACTIVE".to_string()),
        "should apply SCREAMING_SNAKE_CASE to Inactive"
    );
    assert!(
        values_str.contains(&"CUSTOM_PENDING".to_string()),
        "should use custom rename for Pending"
    );
}

#[test]
fn derive_to_schema_with_full_serde_integration_generates_correct_schema() {
    //* Given
    #[derive(Default, serde::Deserialize, utocli::ToSchema)]
    #[serde(rename_all = "camelCase", default)]
    #[schema(
        title = "Complete Example",
        description = "A complete serde integration test"
    )]
    struct CompleteExample {
        // Always required (not Option, no default override)
        id: u64,

        // Optional via Option type
        optional_name: Option<String>,

        // Optional via skip_serializing_if
        #[serde(skip_serializing_if = "String::is_empty")]
        conditional_field: String,

        // Renamed field
        #[serde(rename = "customData")]
        custom_data: String,

        // Skipped field
        #[serde(skip)]
        internal_state: bool,

        // With schema attributes
        #[schema(description = "Email address", example = "user@example.com")]
        email: String,
    }

    //* When
    let schema = CompleteExample::schema();

    //* Then
    let Schema::Object(obj) = schema else {
        panic!("Expected Object schema for struct");
    };

    // Verify all properties are present (except skipped)
    let props = obj
        .properties
        .as_ref()
        .expect("properties should be present");
    assert_eq!(
        props.len(),
        5,
        "should have 5 properties (6 fields - 1 skipped)"
    );
    assert!(props.contains_key("id"));
    assert!(props.contains_key("optionalName"));
    assert!(props.contains_key("conditionalField"));
    assert!(props.contains_key("customData"), "should use serde rename");
    assert!(props.contains_key("email"));
    assert!(
        !props.contains_key("internalState"),
        "skipped field should be excluded"
    );

    // Verify schema metadata
    assert_eq!(obj.title, Some("Complete Example".to_string()));
    assert!(obj.description.is_some());

    // With container default, all fields become optional
    assert!(
        obj.required.as_ref().is_none_or(|r| r.is_empty()),
        "container-level default should make all fields optional"
    );
}
