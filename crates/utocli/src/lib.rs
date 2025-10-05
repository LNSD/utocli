//! # utocli
//!
//! Auto-generate OpenCLI specifications.
//!
//! This crate provides the main API for working with OpenCLI specifications,
//! re-exporting all types from the `utocli-core` crate.

// Re-export the opencli module for access to builders and internal types
pub use utocli_core::opencli;
// Re-export all main types at the crate root for convenience
pub use utocli_core::{
    Architecture, Arity, Array, Command, Commands, Components, Contact, EnvironmentVariable,
    Extensions, ExternalDocs, Info, License, MediaType, Object, OpenCli, Parameter, ParameterIn,
    ParameterScope, Platform, PlatformName, Ref, RefOr, Response, Schema, SchemaFormat, SchemaType,
    Tag,
};
