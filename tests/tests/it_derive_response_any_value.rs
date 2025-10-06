//! E2E tests for AnyValue support in ToResponse and IntoResponses derive macros

#![allow(dead_code)]

use utocli::IntoResponses;

/// Test ToResponse with literal string example in content
#[test]
fn to_response_with_literal_string_example_generates_correct_response() {
    //* Given
    #[derive(utocli::ToResponse)]
    #[response(description = "A successful response")]
    struct MyResponse {
        #[content(media_type = "text/plain", example = "success")]
        text: (),
    }

    //* When
    let response = MyResponse::response();

    //* Then
    assert_eq!(
        response.description,
        Some("A successful response".to_string())
    );
    let content = response.content.expect("should have content");
    let media = content.get("text/plain").expect("should have text/plain");
    assert_eq!(media.example, Some(serde_json::json!("success")));
}

/// Test ToResponse with json!() object example in content
#[test]
fn to_response_with_json_object_example_generates_correct_response() {
    //* Given
    #[derive(utocli::ToResponse)]
    #[response(description = "JSON response example")]
    struct MyResponse {
        #[content(
            media_type = "application/json",
            example = r#"{"status":"ok","code":200}"#
        )]
        json: (),
    }

    //* When
    let response = MyResponse::response();

    //* Then
    assert_eq!(
        response.description,
        Some("JSON response example".to_string())
    );
    let content = response.content.expect("should have content");
    let media = content.get("application/json").expect("should have json");
    let example = media.example.as_ref().expect("should have example");
    assert_eq!(example.get("status").and_then(|v| v.as_str()), Some("ok"));
    assert_eq!(example.get("code").and_then(|v| v.as_i64()), Some(200));
}

/// Test ToResponse with serde_json::json!() macro in content example
#[test]
fn to_response_with_json_macro_in_content_example_generates_correct_response() {
    //* Given
    #[derive(utocli::ToResponse)]
    struct MyResponse {
        #[content(
            media_type = "application/json",
            example = r#"{"items":["item1","item2","item3"],"count":3}"#
        )]
        json: (),
    }

    //* When
    let response = MyResponse::response();

    //* Then
    let content = response.content.expect("should have content");
    let media = content.get("application/json").expect("should have json");
    let example = media.example.as_ref().expect("should have example");
    let items = example.get("items").expect("should have items");
    let items_arr = items.as_array().expect("items should be array");
    assert_eq!(items_arr.len(), 3);
    assert_eq!(items_arr[0], "item1");
    assert_eq!(items_arr[1], "item2");
    assert_eq!(items_arr[2], "item3");
    assert_eq!(example.get("count").and_then(|v| v.as_i64()), Some(3));
}

/// Test IntoResponses with literal string descriptions
#[test]
fn into_responses_with_literal_descriptions_generates_correct_responses() {
    //* Given
    #[derive(utocli::IntoResponses)]
    enum MyResponses {
        #[response(status = 200, description = "Success")]
        Success(String),
        #[response(status = 404, description = "Not found")]
        NotFound,
    }

    //* When
    let responses = MyResponses::responses();

    //* Then
    assert!(responses.contains_key("200"));
    assert!(responses.contains_key("404"));
    let success_response = responses.get("200").expect("should have 200");
    let utocli::RefOr::T(success) = success_response else {
        panic!("expected T variant")
    };
    assert_eq!(success.description, Some("Success".to_string()));
    let not_found_response = responses.get("404").expect("should have 404");
    let utocli::RefOr::T(not_found) = not_found_response else {
        panic!("expected T variant")
    };
    assert_eq!(not_found.description, Some("Not found".to_string()));
}

/// Test IntoResponses with multiple variants and descriptions
#[test]
fn into_responses_with_multiple_variants_generates_correct_responses() {
    //* Given
    #[derive(utocli::IntoResponses)]
    enum MyResponses {
        #[response(status = 200, description = "Success response")]
        Success { result: String, count: i32 },
        #[response(status = 400, description = "Bad request")]
        BadRequest { error: String, field: String },
    }

    //* When
    let responses = MyResponses::responses();

    //* Then
    assert!(responses.contains_key("200"));
    assert!(responses.contains_key("400"));
    assert_eq!(responses.len(), 2);
    let success_response = responses.get("200").expect("should have 200");
    let utocli::RefOr::T(success) = success_response else {
        panic!("expected T variant")
    };
    assert_eq!(success.description, Some("Success response".to_string()));
    let error_response = responses.get("400").expect("should have 400");
    let utocli::RefOr::T(error) = error_response else {
        panic!("expected T variant")
    };
    assert_eq!(error.description, Some("Bad request".to_string()));
}
