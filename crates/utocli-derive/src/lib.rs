//! Procedural macros for utocli - derive macros for OpenCLI types.
//!
//! This crate provides derive macros for automatically generating OpenCLI schema,
//! parameter, and response definitions from Rust types.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```ignore
//! /// A user in the system
//! #[derive(utocli_derive::ToSchema)]
//! struct User {
//!     /// The user's unique identifier
//!     id: u64,
//!     /// The user's name
//!     name: String,
//!     /// Optional email address
//!     email: Option<String>,
//! }
//!
//! // Generate the schema
//! let schema = User::schema();
//! let name = User::schema_name();
//! assert_eq!(name, "User");
//! ```
//!
//! ## Custom Attributes
//!
//! ```ignore
//! #[derive(utocli_derive::ToSchema)]
//! #[schema(description = "A custom description")]
//! struct Config {
//!     #[schema(description = "Port number", rename = "portNumber")]
//!     port: u16,
//!
//!     #[schema(skip)]
//!     internal_field: String,
//! }
//! ```
//!
//! ## Enums
//!
//! ```ignore
//! #[derive(utocli_derive::ToSchema)]
//! enum Status {
//!     Active,
//!     Inactive,
//!     Pending,
//! }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{DeriveInput, Member, parse_macro_input};

mod command;
mod diagnostics;
mod doc_comment;
mod opencli;
mod parameter;
mod response;
mod schema;
mod type_tree;

use diagnostics::{Diagnostics, ToTokensDiagnostics};
use opencli::OpenCli;
use parameter::Parameter;
use response::{IntoResponses, ToResponse};
use schema::Schema;

/// Represents any value used in example and default fields.
/// Matches utoipa-gen/src/lib.rs lines 25643-25653
#[derive(Clone)]
enum AnyValue {
    #[allow(dead_code)]
    String(TokenStream2),
    Json(TokenStream2),
    DefaultTrait {
        struct_ident: syn::Ident,
        field_ident: Member,
    },
}

impl AnyValue {
    /// Parse any value: literal, macro invocation, or function call.
    ///
    /// Accepts:
    /// - Literals: `42`, `"string"`, `true`, `-1`
    /// - Macro invocations: `json!(...)`, `serde_json::json!(...)`, or any custom macro
    /// - Function calls: `my_module::example_fn` (will be called as `example_fn()`)
    fn parse_any(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse as expression to handle all cases uniformly
        let expr = input.parse::<syn::Expr>().map_err(|error| {
            Diagnostics::with_span(
                error.span(),
                "expected literal value, macro invocation, or method reference",
            )
            .help("Use a literal: \"value\" or 42")
            .help("Use a macro: json!({\"key\": \"value\"}) or serde_json::json!(...)")
            .note("Method references should be valid function paths like `my_module::example_fn`")
        })?;

        // Check if it's a path expression (function reference) that needs to be called
        match &expr {
            syn::Expr::Path(_) => {
                // Function reference - add () to call it
                Ok(AnyValue::Json(quote! { #expr() }))
            }
            _ => {
                // Literal, macro, or other expression - use as-is
                Ok(AnyValue::Json(expr.to_token_stream()))
            }
        }
    }

    /// Parse literal string or any macro/expression.
    ///
    /// Prefers literal strings but allows any macro invocation or expression for complex values.
    /// Used in contexts where strings are preferred (e.g., response examples).
    ///
    /// Accepts: `"string"`, `json!(...)`, `serde_json::json!(...)`, or any expression.
    fn parse_lit_str_or_expr(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::LitStr) {
            Ok(AnyValue::String(
                input.parse::<syn::LitStr>().unwrap().to_token_stream(),
            ))
        } else {
            Ok(AnyValue::Json(parse_utils::parse_macro_or_expr(input)?))
        }
    }

    #[allow(dead_code)]
    fn new_default_trait(struct_ident: syn::Ident, field_ident: Member) -> Self {
        Self::DefaultTrait {
            struct_ident,
            field_ident,
        }
    }
}

impl ToTokens for AnyValue {
    /// Matches utoipa-gen/src/lib.rs lines 25711-25726
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Json(json) => tokens.extend(quote! {
                serde_json::json!(#json)
            }),
            Self::String(string) => string.to_tokens(tokens),
            Self::DefaultTrait {
                struct_ident,
                field_ident,
            } => tokens.extend(quote! {
                serde_json::to_value(#struct_ident::default().#field_ident).unwrap()
            }),
        }
    }
}

/// Parsing utilities
/// Matches utoipa-gen/src/lib.rs lines 26012-26177
mod parse_utils {
    use proc_macro2::TokenStream;
    use quote::ToTokens;
    use syn::{Token, parse::ParseStream};

    pub fn parse_next<T: FnOnce() -> Result<R, syn::Error>, R: Sized>(
        input: ParseStream,
        next: T,
    ) -> Result<R, syn::Error> {
        input.parse::<Token![=]>()?;
        next()
    }

    /// Parse any macro invocation (e.g., `json!(...)`, `serde_json::json!(...)`) or expression.
    ///
    /// This function accepts any macro invocation pattern without validating the macro name,
    /// allowing flexibility to use standard or custom macros that produce compatible values.
    ///
    /// Unlike the previous implementation that special-cased `json!()`, this treats all
    /// macro invocations uniformly, following Rust's expression-based approach.
    pub fn parse_macro_or_expr(input: ParseStream) -> syn::Result<TokenStream> {
        // Simply parse as an expression and return its token stream
        // This handles all cases: literals, macro invocations (json!(...), path::macro!(...)),
        // and function references uniformly
        Ok(input.parse::<syn::Expr>()?.to_token_stream())
    }
}

/// Derive macro for generating OpenCLI schema definitions.
///
/// This macro generates an implementation that produces OpenCLI schema definitions
/// for structs and enums, which can be used in OpenCLI components.
///
/// # Examples
///
/// ```ignore
/// /// A user in the system
/// #[derive(utocli_derive::ToSchema)]
/// struct User {
///     /// The user's unique identifier
///     id: u64,
///     /// The user's name
///     name: String,
/// }
/// ```
///
/// # Attributes
///
/// ## Container attributes (`#[schema(...)]`)
///
/// * `description = "..."` - Override the description from doc comments
/// * `example = ...` - Provide an example value (accepts literals, `json!(...)`, `serde_json::json!(...)`, or any expression)
/// * `title = "..."` - Set a custom title for the schema
/// * `rename_all = "..."` - Rename all fields (e.g., "camelCase", "snake_case")
/// * `no_recursion` - Break recursion in case of looping schema tree (e.g., `Pet` -> `Owner` -> `Pet`).
///   When set on a container, it applies to all fields.
///
/// ## Field attributes (`#[schema(...)]`)
///
/// * `description = "..."` - Override field description
/// * `example = ...` - Provide an example value (accepts literals, `json!(...)`, `serde_json::json!(...)`, or any expression)
/// * `format = "..."` - Specify the schema format
/// * `rename = "..."` - Rename this specific field
/// * `inline` - Inline the schema instead of using a reference
/// * `skip` - Skip this field from the schema
/// * `no_recursion` - Break recursion for this specific field. Use this to prevent infinite
///   loops in recursive data structures.
///
/// # Recursion handling
///
/// For recursive data structures, use the `no_recursion` attribute to break the cycle.
/// Without it, the macro will recurse infinitely and cause a runtime panic.
///
/// ```ignore
/// #[derive(ToSchema)]
/// pub struct Pet {
///     name: String,
///     owner: Option<Owner>,
/// }
///
/// #[derive(ToSchema)]
/// pub struct Owner {
///     name: String,
///     #[schema(no_recursion)]  // Breaks the Pet -> Owner -> Pet cycle
///     pets: Vec<Pet>,
/// }
/// ```
///
/// # Serde compatibility
///
/// This macro respects serde attributes like `#[serde(rename)]` and `#[serde(skip)]`.
#[proc_macro_derive(ToSchema, attributes(schema))]
pub fn derive_to_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match Schema::new(input) {
        Ok(schema) => schema.into_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive macro for generating OpenCLI parameter definitions.
///
/// This macro generates parameter definitions from struct fields, useful for
/// defining CLI command parameters, flags, and options.
///
/// # Examples
///
/// ```ignore
/// #[derive(utocli_derive::ToParameter)]
/// struct ListOptions {
///     /// Show all items including hidden
///     #[param(alias = "a", scope = "local")]
///     all: bool,
///
///     /// Filter by name pattern
///     #[param(description = "Name filter pattern")]
///     filter: Option<String>,
/// }
/// ```
///
/// # Attributes
///
/// ## Field attributes (`#[param(...)]`)
///
/// * `alias = "..."` - Alternative short name for the parameter (e.g., "v" for verbose)
/// * `description = "..."` - Parameter description (overrides doc comments)
/// * `example = ...` - Example value (accepts literals, `json!(...)`, `serde_json::json!(...)`, or any expression)
/// * `default = ...` - Default value (accepts literals, `json!(...)`, `serde_json::json!(...)`, or any expression)
/// * `scope = "local"|"inherited"` - Parameter scope (local to command or inherited by subcommands)
/// * `position = N` - Position for positional arguments
/// * `in = "argument"|"flag"|"option"` - Explicitly set parameter type
/// * `skip` - Skip this field
///
/// # Serde compatibility
///
/// This macro respects `#[serde(skip)]` attribute.
#[proc_macro_derive(ToParameter, attributes(param))]
pub fn derive_to_parameter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match Parameter::new(input) {
        Ok(parameter) => parameter.to_token_stream().into(),
        Err(diagnostics) => diagnostics.to_token_stream().into(),
    }
}

/// Derive macro for generating OpenCLI response definitions.
///
/// This macro generates response definitions from enums or structs, mapping
/// different variants to different exit codes or response types.
///
/// # Examples
///
/// ```ignore
/// /// A successful validation response
/// #[derive(utocli_derive::ToResponse)]
/// #[response(description = "Validation completed successfully")]
/// struct ValidationSuccess {
///     #[content(media_type = "application/json", schema = "ValidationResult")]
///     json_output: (),
///
///     #[content(media_type = "text/plain", example = "✓ Validation successful")]
///     text_output: (),
/// }
/// ```
///
/// # Attributes
///
/// ## Container attributes (`#[response(...)]`)
///
/// * `description = "..."` - Response description (overrides doc comments)
/// * `status = "..."` - Exit status code (e.g., "0", "1")
///
/// ## Field attributes (`#[content(...)]`)
///
/// * `media_type = "..."` - Media type for this content (e.g., "application/json", "text/plain")
/// * `schema = "..."` - Schema reference name (e.g., "ValidationResult")
/// * `example = "..."` - Example value for this media type
#[proc_macro_derive(ToResponse, attributes(response, content))]
pub fn derive_to_response(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    } = parse_macro_input!(input);

    match ToResponse::new(attrs, &data, generics, ident) {
        Ok(response) => response.to_token_stream().into(),
        Err(diagnostics) => diagnostics.to_token_stream().into(),
    }
}

/// Derive macro for generating multiple OpenCLI responses.
///
/// This macro generates response maps with status codes, typically used for enum types
/// where each variant represents a different response with its own exit code.
///
/// This is parallel to utoipa's `IntoResponses` derive, adapted for CLI exit codes.
///
/// # Examples
///
/// ## Enum with multiple response variants
///
/// ```ignore
/// #[derive(utocli_derive::IntoResponses)]
/// enum CommandResponse {
///     /// Success response
///     #[response(status = "0")]
///     Success { message: String },
///
///     /// Not found error
///     #[response(status = "1")]
///     NotFound,
///
///     /// Validation error
///     #[response(status = "2")]
///     ValidationError(ValidationDetails),
/// }
/// ```
///
/// ## Single struct response
///
/// ```ignore
/// /// Success response
/// #[derive(utocli_derive::IntoResponses)]
/// #[response(status = "0")]
/// struct SuccessResponse {
///     value: String,
/// }
/// ```
///
/// # Attributes
///
/// ## Container/Variant attributes (`#[response(...)]`)
///
/// * `status = "..."` - Exit status code (required, e.g., "0", "1", "2")
/// * `description = "..."` - Response description (overrides doc comments)
/// * `content_type = "..."` - Media type (e.g., "application/json", "text/plain")
/// * `example = ...` - Example value (accepts literals, `json!(...)`, `serde_json::json!(...)`, or any expression)
///
/// ## Field attributes (unnamed fields only)
///
/// * `#[to_schema]` - Inline the schema instead of using a reference
/// * `#[ref_response]` - Use a schema reference
/// * `#[to_response]` - Inline the response
///
/// # Generated trait
///
/// This macro implements the `IntoResponses` trait which provides:
/// ```ignore
/// fn responses() -> BTreeMap<String, RefOr<Response>>
/// ```
#[proc_macro_derive(
    IntoResponses,
    attributes(response, to_schema, ref_response, to_response, content)
)]
pub fn derive_into_responses(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    } = parse_macro_input!(input);

    let into_responses = IntoResponses {
        attributes: attrs,
        ident,
        generics,
        data,
    };

    into_responses.to_token_stream().into()
}

/// Derive macro for generating complete OpenCLI specifications.
///
/// This macro generates a complete OpenCLI specification from a single struct,
/// collecting commands, components (schemas, parameters, responses), and metadata.
///
/// # Examples
///
/// ```ignore
/// #[derive(utocli_derive::OpenCli)]
/// #[opencli(
///     info(
///         title = "My CLI Tool",
///         version = "1.0.0",
///         description = "A sample CLI application"
///     ),
///     commands(build_root_command, build_validate_command),
///     components(
///         schemas(Error, ValidationError),
///         parameters(ConfigParam),
///         responses(SuccessResponse, ErrorResponse)
///     ),
///     tags(
///         (name = "core", description = "Core commands"),
///         (name = "validation", description = "Validation commands")
///     )
/// )]
/// struct CliDoc;
///
/// let spec = CliDoc::opencli();
/// ```
///
/// # Attributes
///
/// ## `info(...)` - Application metadata
///
/// * `title = "..."` - CLI application title (required)
/// * `version = "..."` - Application version (required)
/// * `description = "..."` - Application description (optional, can use doc comments)
///
/// ## `commands(...)` - Command definitions
///
/// List of function references that return `Commands`:
/// ```ignore
/// commands(build_root_command, build_validate_command)
/// ```
///
/// ## `components(...)` - Reusable components
///
/// * `schemas(...)` - List of types implementing `ToSchema`
/// * `parameters(...)` - List of types implementing `ToParameter`
/// * `responses(...)` - List of types implementing `ToResponse`
///
/// ## `tags(...)` - Tag definitions
///
/// List of tag definitions:
/// ```ignore
/// tags(
///     (name = "core", description = "Core commands"),
///     (name = "validation")
/// )
/// ```
#[proc_macro_derive(OpenCli, attributes(opencli))]
pub fn derive_opencli(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match OpenCli::new(input) {
        Ok(opencli) => opencli.to_token_stream().into(),
        Err(diagnostics) => diagnostics.to_token_stream().into(),
    }
}

/// Attribute macro for generating OpenCLI command definitions.
///
/// This macro is parallel to utoipa's `#[utoipa::path(...)]` macro. It decorates functions
/// to generate OpenCLI command specifications, mapping the function to a CLI command with
/// parameters, responses, and metadata.
///
/// # Examples
///
/// ```ignore
/// #[utocli::command(
///     name = "validate",
///     summary = "Validate CLI specification",
///     description = "Validate a CLI specification file against the OpenCLI standard",
///     operation_id = "validateCommand",
///     aliases("val", "check"),
///     tags("core"),
///     extend(x_cli_category = "validation", x_performance = "fast"),
///     parameters(
///         (name = "file", in = "argument", position = 1, description = "Path to specification file", required = true),
///         (name = "strict", in = "flag", alias = "s", description = "Enable strict mode")
///     ),
///     responses(
///         (status = "0", description = "Validation successful"),
///         (status = "1", description = "Validation failed")
///     )
/// )]
/// fn validate_command() {
///     // Command implementation
/// }
/// ```
///
/// # Attributes
///
/// * `name = "..."` - Command name (defaults to function name without "_command" suffix)
/// * `summary = "..."` - Short command summary
/// * `description = "..."` - Detailed description (overrides doc comments)
/// * `operation_id = "..."` - Unique operation identifier
/// * `aliases(...)` - Command aliases as a list: `aliases("val", "check")`
/// * `tags(...)` - Associated tags as a list: `tags("core", "validation")`
/// * `parameters(...)` - Parameter definitions (see below)
/// * `responses(...)` - Response definitions (see below)
/// * `extend(...)` - OpenAPI extensions: `extend(x_cli_category = "validation")`
///
/// ## Parameter Definitions
///
/// ```ignore
/// parameters(
///     (
///         name = "file",
///         in = "argument",  // or "flag", "option"
///         position = 1,
///         description = "Path to file",
///         required = true,
///         alias = "f",
///         extend(x_completion = "file", x_validation = "file-exists")
///     )
/// )
/// ```
///
/// ## Response Definitions
///
/// ```ignore
/// responses(
///     (
///         status = "0",
///         description = "Success",
///         content(
///             (media_type = "application/json", schema = "ValidationResult"),
///             (media_type = "text/plain", example = "✓ Success")
///         )
///     )
/// )
/// ```
#[proc_macro_attribute]
pub fn command(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: proc_macro2::TokenStream = args.into();
    let input = parse_macro_input!(input as syn::ItemFn);

    match command::command(args, input) {
        Ok(tokens) => tokens.into(),
        Err(diagnostics) => diagnostics.to_token_stream().into(),
    }
}
