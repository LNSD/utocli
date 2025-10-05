//! # utocli-core
//!
//! Core types and traits for utocli - OpenCLI specification support.
//!
//! This crate provides the fundamental types for working with OpenCLI v1.0.0 specifications,
//! which describe CLI applications in a machine-readable format similar to OpenAPI for REST APIs.

pub mod opencli;

// Re-export main types at the crate root for convenience
pub use opencli::{
    Architecture, Arity, Array, Command, Commands, Components, Contact, EnvironmentVariable,
    Extensions, ExternalDocs, Info, License, MediaType, Object, OpenCli, Parameter, ParameterIn,
    ParameterScope, Platform, PlatformName, Ref, RefOr, Response, Schema, SchemaFormat, SchemaType,
    Tag,
};
