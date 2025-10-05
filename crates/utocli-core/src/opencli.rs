//! OpenCLI specification types and structures.
//!
//! This module provides the core types for representing an OpenCLI v1.0.0 specification,
//! which describes a CLI application in a machine-readable format.

pub mod command;
pub mod components;
pub mod environment;
pub mod extensions;
pub mod external_docs;
pub mod info;
pub mod parameter;
pub mod platform;
pub mod response;
pub mod schema;
pub mod tag;

pub use self::{
    command::{Command, Commands},
    components::Components,
    environment::EnvironmentVariable,
    extensions::Extensions,
    external_docs::ExternalDocs,
    info::{Contact, Info, License},
    parameter::{Arity, Parameter, ParameterIn, ParameterScope},
    platform::{Architecture, Platform, PlatformName},
    response::{MediaType, Response},
    schema::{Array, Object, Ref, RefOr, Schema, SchemaFormat, SchemaType},
    tag::Tag,
};

/// The root object representing a complete OpenCLI specification document.
///
/// This is the main entry point for an OpenCLI specification, containing all the
/// metadata, commands, and component definitions for a CLI application.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OpenCli {
    /// The OpenCLI version (always "1.0.0" for this implementation).
    pub opencli: String,

    /// Core metadata about the CLI application.
    pub info: Info,

    /// The commands exposed by the CLI application.
    pub commands: Commands,

    /// Reusable component definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,

    /// Tags for organizing commands into groups.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,

    /// Platform and architecture support information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platforms: Option<Vec<Platform>>,

    /// Environment variable mappings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Vec<EnvironmentVariable>>,

    /// External documentation reference.
    #[serde(rename = "externalDocs", skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,
}

impl OpenCli {
    /// Creates a new OpenCLI specification with the given info.
    pub fn new(info: Info) -> Self {
        Self {
            opencli: "1.0.0".to_string(),
            info,
            commands: Commands::new(),
            components: None,
            tags: None,
            platforms: None,
            environment: None,
            external_docs: None,
        }
    }

    /// Sets the commands for the CLI.
    pub fn commands(mut self, commands: Commands) -> Self {
        self.commands = commands;
        self
    }

    /// Sets the components.
    pub fn components(mut self, components: Components) -> Self {
        self.components = Some(components);
        self
    }

    /// Sets the tags.
    pub fn tags(mut self, tags: Vec<Tag>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Sets the platforms.
    pub fn platforms(mut self, platforms: Vec<Platform>) -> Self {
        self.platforms = Some(platforms);
        self
    }

    /// Sets the environment variables.
    pub fn environment(mut self, environment: Vec<EnvironmentVariable>) -> Self {
        self.environment = Some(environment);
        self
    }

    /// Sets the external documentation.
    pub fn external_docs(mut self, external_docs: ExternalDocs) -> Self {
        self.external_docs = Some(external_docs);
        self
    }
}
