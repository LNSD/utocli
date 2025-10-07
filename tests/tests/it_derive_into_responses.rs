//! E2E tests for IntoResponses derive macro.
//!
//! These tests verify that the derive macro generates correct code for various
//! response type patterns.

#![allow(dead_code)]

use utocli::{IntoResponses as _, RefOr};

#[test]
fn into_responses_with_enum_with_multiple_variants_generates_response_map() {
    //* Given
    #[derive(utocli::IntoResponses)]
    enum CommandResponse {
        /// Success response
        #[response(status = "0")]
        Success { message: String },

        /// Not found error
        #[response(status = "1")]
        NotFound,

        /// Validation error with details
        #[response(status = "2", description = "Validation failed")]
        ValidationError { errors: Vec<String> },
    }

    //* When
    let responses = CommandResponse::responses();

    //* Then
    assert_eq!(
        responses.len(),
        3,
        "enum with 3 variants should generate 3 responses"
    );

    // Verify each status code is present
    assert!(
        responses.contains_key("0"),
        "should have response for status code 0"
    );
    assert!(
        responses.contains_key("1"),
        "should have response for status code 1"
    );
    assert!(
        responses.contains_key("2"),
        "should have response for status code 2"
    );

    // Verify the validation error has custom description
    let RefOr::T(validation_response) = responses.get("2").expect("status 2 should exist") else {
        panic!("Expected concrete Response, not a reference");
    };

    assert_eq!(
        validation_response.description,
        Some("Validation failed".to_string()),
        "should use custom description from response attribute"
    );

    // Verify NotFound response is present
    let RefOr::T(_not_found_response) = responses.get("1").expect("status 1 should exist") else {
        panic!("Expected concrete Response for unit variant");
    };
}

#[test]
fn into_responses_with_named_struct_generates_single_response() {
    //* Given
    /// Success response
    #[derive(utocli::IntoResponses)]
    #[response(status = "0")]
    struct SuccessResponse {
        value: String,
    }

    //* When
    let responses = SuccessResponse::responses();

    //* Then
    assert_eq!(
        responses.len(),
        1,
        "single struct should generate exactly one response"
    );

    assert!(
        responses.contains_key("0"),
        "should have response for status code 0"
    );

    let RefOr::T(response) = responses.get("0").expect("status 0 should exist") else {
        panic!("Expected concrete Response, not a reference");
    };

    // Verify description comes from doc comment
    assert!(
        response.description.is_some(),
        "should extract description from doc comment"
    );
    assert_eq!(
        response.description,
        Some("Success response".to_string()),
        "description should match doc comment"
    );
}

#[test]
fn into_responses_with_unit_struct_generates_single_response() {
    //* Given
    /// Unit struct response
    #[derive(utocli::IntoResponses)]
    #[response(status = "0")]
    struct NotFound;

    //* When
    let responses = NotFound::responses();

    //* Then
    assert_eq!(
        responses.len(),
        1,
        "unit struct should generate exactly one response"
    );

    assert!(
        responses.contains_key("0"),
        "should have response for status code 0"
    );

    let RefOr::T(response) = responses.get("0").expect("status 0 should exist") else {
        panic!("Expected concrete Response for unit struct");
    };

    assert!(
        response.description.is_some(),
        "should have description from doc comment"
    );
}

#[test]
fn into_responses_with_description_override_uses_attribute_over_doc_comment() {
    //* Given
    #[derive(utocli::IntoResponses)]
    enum ApiResponse {
        /// Doc comment description
        #[response(status = "0")]
        Default,

        /// This will be overridden
        #[response(status = "1", description = "Custom description")]
        Custom,
    }

    //* When
    let responses = ApiResponse::responses();

    //* Then
    assert_eq!(responses.len(), 2, "enum should generate 2 responses");

    // Verify default uses doc comment
    let RefOr::T(default_response) = responses.get("0").expect("status 0 should exist") else {
        panic!("Expected concrete Response");
    };
    assert_eq!(
        default_response.description,
        Some("Doc comment description".to_string()),
        "should use doc comment when no description attribute present"
    );

    // Verify custom uses attribute description over doc comment
    let RefOr::T(custom_response) = responses.get("1").expect("status 1 should exist") else {
        panic!("Expected concrete Response");
    };
    assert_eq!(
        custom_response.description,
        Some("Custom description".to_string()),
        "should override doc comment with description attribute"
    );
}

#[test]
fn into_responses_with_different_exit_codes_generates_correct_map() {
    //* Given
    #[derive(utocli::IntoResponses)]
    enum ExitCodes {
        #[response(status = "0")]
        Success,

        #[response(status = "1")]
        GeneralError,

        #[response(status = "2")]
        MisusedShellBuiltin,

        #[response(status = "126")]
        CannotExecute,

        #[response(status = "127")]
        CommandNotFound,
    }

    //* When
    let responses = ExitCodes::responses();

    //* Then
    assert_eq!(
        responses.len(),
        5,
        "should generate response for each exit code"
    );

    // Verify standard exit codes
    assert!(responses.contains_key("0"), "should have success code");
    assert!(
        responses.contains_key("1"),
        "should have general error code"
    );
    assert!(
        responses.contains_key("2"),
        "should have misused builtin code"
    );

    // Verify special exit codes (126, 127)
    assert!(
        responses.contains_key("126"),
        "should support exit code 126 (cannot execute)"
    );
    assert!(
        responses.contains_key("127"),
        "should support exit code 127 (command not found)"
    );
}
