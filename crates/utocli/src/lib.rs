//! # utocli
//!
//! Auto-generate OpenCLI specifications.
//!
//! This crate provides the main API for working with OpenCLI specifications,
//! re-exporting all types from the `utocli-core` crate and derive macros from
//! `utocli-derive` (when the `macros` feature is enabled).

// Re-export utocli_core for derive macros (they generate code that references ::utocli::utocli_core)
#[doc(hidden)]
pub use utocli_core;
// Re-export the opencli module for access to builders and internal types
pub use utocli_core::opencli;
// Re-export all main types at the crate root for convenience
pub use utocli_core::{
    Architecture, Arity, Array, Command, CommandPath, Commands, Components, ComposeSchema, Contact,
    EnvironmentVariable, Extensions, ExternalDocs, Info, IntoResponses, License, Map, MediaType,
    Object, OpenCli, Parameter, ParameterIn, ParameterScope, Platform, PlatformName, Ref, RefOr,
    Response, Schema, SchemaFormat, SchemaType, Tag, ToResponse, ToSchema,
};
// Re-export derive macros when the macros feature is enabled
#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
pub use utocli_derive::{IntoResponses, OpenCli, ToParameter, ToResponse, ToSchema, command};
