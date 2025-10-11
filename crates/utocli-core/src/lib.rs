//! # utocli-core
//!
//! Core types and traits for utocli - OpenCLI specification support.
//!
//! This crate provides the fundamental types for working with OpenCLI v1.0.0 specifications,
//! which describe CLI applications in a machine-readable format similar to OpenAPI for REST APIs.

mod builder_macros;
mod compose_schema;
pub mod opencli;
mod to_response;
mod to_schema;

use std::collections::BTreeMap;

// Re-export main types at the crate root for convenience
pub use self::{
    compose_schema::{ComposeSchema, schema_or_compose},
    opencli::{
        Architecture, Arity, Array, Command, Commands, Components, Contact, EnvironmentVariable,
        Extensions, ExternalDocs, Info, License, Map, MediaType, Object, Parameter, ParameterIn,
        ParameterScope, Platform, PlatformName, Ref, RefOr, Response, Schema, SchemaFormat,
        SchemaType, Tag,
    },
    to_response::ToResponse,
    to_schema::ToSchema,
};

/// Trait for types that can generate OpenCLI specifications.
///
/// This trait is similar to utoipa's [`OpenApi`](https://docs.rs/utoipa/latest/utoipa/trait.OpenApi.html)
/// trait, but adapted for CLI applications instead of REST APIs.
///
/// This trait is implemented via [`#[derive(OpenCli)]`][macro@crate::OpenCli] and there
/// is no need to implement this trait manually.
///
/// # Examples
///
/// ```rust,ignore
/// use utocli::OpenCli;
///
/// #[derive(OpenCli)]
/// #[opencli(
///     info(
///         title = "My CLI Tool",
///         version = "1.0.0",
///         description = "A sample CLI application"
///     ),
///     commands(build_root_command),
///     tags(
///         (name = "core", description = "Core commands")
///     )
/// )]
/// struct CliDoc;
///
/// let spec = CliDoc::opencli();
/// ```
pub trait OpenCli {
    /// Returns the [`opencli::OpenCli`] instance which can be parsed with serde
    /// or served via CLI documentation tools.
    fn opencli() -> opencli::OpenCli;
}

/// Trait for implementing OpenCLI Command object.
///
/// This trait is implemented via [`#[utocli::command(...)]`][macro@crate::command] attribute macro and there
/// is no need to implement this trait manually.
///
/// # Examples
///
/// Use `#[utocli::command(..)]` to implement Command trait:
/// ```rust,ignore
/// # use utocli::{Command as CommandType, Map, MediaType, Parameter, ParameterIn, ParameterScope, RefOr, Response, Schema, Object, SchemaType};
/// #
/// /// Validate CLI specification
/// ///
/// /// Validate a CLI specification file against the OpenCLI standard
/// #[utocli::command(
///     name = "validate",
///     summary = "Validate CLI specification",
///     description = "Validate a CLI specification file against the OpenCLI standard",
///     operation_id = "validateCommand",
///     aliases("val", "check"),
///     tags("core"),
///     parameters(
///         (
///             name = "file",
///             in = "argument",
///             position = 1,
///             description = "Path to the CLI specification file",
///             required = true,
///             scope = "local",
///             schema_type = "string"
///         )
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
/// Example of what would manual implementation roughly look like of above `#[utocli::command(...)]` macro:
/// ```rust,ignore
/// # use utocli::{Command as CommandType, Map, MediaType, Parameter, ParameterIn, ParameterScope, RefOr, Response, Schema, Object, SchemaType};
/// # fn validate_command() {}
/// # struct __command_validate_command;
/// #
/// impl utocli::CommandPath for __command_validate_command {
///     fn command() -> utocli::opencli::Command {
///         utocli::opencli::Command::new()
///             .summary("Validate CLI specification")
///             .description("Validate a CLI specification file against the OpenCLI standard")
///             .operation_id("validateCommand")
///             .aliases(vec!["val".to_string(), "check".to_string()])
///             .tags(vec!["core".to_string()])
///             .parameters(vec![
///                 Parameter::new("file")
///                     .in_(ParameterIn::Argument)
///                     .position(1)
///                     .description("Path to the CLI specification file")
///                     .required(true)
///                     .scope(ParameterScope::Local)
///                     .schema(RefOr::T(Schema::Object(Box::new(
///                         Object::new().schema_type(SchemaType::String)
///                     ))))
///             ])
///             .responses({
///                 let mut responses = utocli::Map::new();
///                 responses.insert("0".to_string(), Response::new().description("Validation successful"));
///                 responses.insert("1".to_string(), Response::new().description("Validation failed"));
///                 responses
///             })
///     }
/// }
/// ```
pub trait CommandPath {
    /// Returns the command path/name used as the key in the Commands map.
    ///
    /// For root commands, this is typically just the command name (e.g., "ocs").
    /// For subcommands, this uses the path format (e.g., "/validate" for a subcommand).
    fn path() -> &'static str;

    /// Returns [`opencli::Command`] describing the CLI command specification including
    /// parameters, responses, and metadata.
    fn command() -> Command;
}

/// Trait for types that can be converted into multiple OpenCLI responses.
///
/// This trait is similar to utoipa's [`IntoResponses`](https://docs.rs/utoipa/latest/utoipa/trait.IntoResponses.html)
/// but adapted for CLI exit codes instead of HTTP status codes.
///
/// This trait is implemented via [`#[derive(IntoResponses)]`][macro@crate::IntoResponses] and there
/// is no need to implement this trait manually.
///
/// # Examples
///
/// ```rust,ignore
/// use utocli::IntoResponses;
///
/// #[derive(IntoResponses)]
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
pub trait IntoResponses {
    /// Returns an ordered map of exit codes to responses.
    ///
    /// Exit codes are strings like "0", "1", "2" etc., following shell exit code conventions.
    fn responses() -> BTreeMap<String, RefOr<Response>>;
}
