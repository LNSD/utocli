//! ToResponse trait for types that can be converted to OpenCLI responses.

use crate::opencli::{RefOr, Response};

/// Trait for implementing OpenCLI response generation.
///
/// This trait is typically implemented via the `#[derive(ToResponse)]` macro and there is
/// usually no need to implement this trait manually.
///
/// The trait follows the same design pattern as utoipa's `ToResponse` trait, providing
/// a way to generate reusable response definitions that can be referenced in OpenCLI
/// specifications.
///
/// # Examples
///
/// Use `#[derive(ToResponse)]` to implement ToResponse trait:
/// ```ignore
/// #[derive(ToResponse)]
/// #[response(description = "Successful validation response")]
/// struct ValidationSuccess {
///     #[content(media_type = "application/json", schema = "ValidationResult")]
///     json_output: (),
///
///     #[content(media_type = "text/plain", example = "✓ Validation successful")]
///     text_output: (),
/// }
/// ```
///
/// Using the response in a component:
/// ```ignore
/// let (name, response) = ValidationSuccess::response();
/// // name == "ValidationSuccess"
/// // response contains the Response definition
/// ```
pub trait ToResponse<'r> {
    /// Returns a tuple of response component name (to be referenced) and the response.
    ///
    /// The name is used for referencing this response in the OpenCLI document components.
    /// The response contains the actual response definition with description and content types.
    fn response() -> (&'r str, RefOr<Response>);
}

// Implement ToResponse for common primitive types that might be used as simple responses

impl<'r> ToResponse<'r> for String {
    fn response() -> (&'r str, RefOr<Response>) {
        (
            "String",
            RefOr::T(Response::new().description("A string response")),
        )
    }
}

impl<'r> ToResponse<'r> for &str {
    fn response() -> (&'r str, RefOr<Response>) {
        (
            "str",
            RefOr::T(Response::new().description("A string response")),
        )
    }
}

impl<'r> ToResponse<'r> for bool {
    fn response() -> (&'r str, RefOr<Response>) {
        (
            "bool",
            RefOr::T(Response::new().description("A boolean response")),
        )
    }
}

impl<'r> ToResponse<'r> for () {
    fn response() -> (&'r str, RefOr<Response>) {
        (
            "unit",
            RefOr::T(Response::new().description("No content response")),
        )
    }
}

// Integer types
impl<'r> ToResponse<'r> for i32 {
    fn response() -> (&'r str, RefOr<Response>) {
        (
            "i32",
            RefOr::T(Response::new().description("32-bit signed integer response")),
        )
    }
}

impl<'r> ToResponse<'r> for i64 {
    fn response() -> (&'r str, RefOr<Response>) {
        (
            "i64",
            RefOr::T(Response::new().description("64-bit signed integer response")),
        )
    }
}

impl<'r> ToResponse<'r> for u32 {
    fn response() -> (&'r str, RefOr<Response>) {
        (
            "u32",
            RefOr::T(Response::new().description("32-bit unsigned integer response")),
        )
    }
}

impl<'r> ToResponse<'r> for u64 {
    fn response() -> (&'r str, RefOr<Response>) {
        (
            "u64",
            RefOr::T(Response::new().description("64-bit unsigned integer response")),
        )
    }
}

// Float types
impl<'r> ToResponse<'r> for f32 {
    fn response() -> (&'r str, RefOr<Response>) {
        (
            "f32",
            RefOr::T(Response::new().description("32-bit floating point response")),
        )
    }
}

impl<'r> ToResponse<'r> for f64 {
    fn response() -> (&'r str, RefOr<Response>) {
        (
            "f64",
            RefOr::T(Response::new().description("64-bit floating point response")),
        )
    }
}
