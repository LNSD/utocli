//! Command entity for CLI commands.

use std::collections::BTreeMap;

use super::{Parameter, Response, extensions::Extensions};

/// Represents a CLI command with its parameters and responses.
///
/// Commands can have subcommands by nesting Command entries in the Commands map.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Command {
    /// A summary of what the command does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// A detailed description of the command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A unique identifier for the command operation.
    #[serde(rename = "operationId", skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,

    /// Alternative names for the command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,

    /// Tags for organizing commands into groups.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Parameters (arguments, flags, options) for the command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,

    /// Responses keyed by exit code (e.g., "0", "1", "2").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responses: Option<BTreeMap<String, Response>>,

    /// Extension properties.
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

impl Command {
    /// Creates a new empty command.
    pub fn new() -> Self {
        Self {
            summary: None,
            description: None,
            operation_id: None,
            aliases: None,
            tags: None,
            parameters: None,
            responses: None,
            extensions: None,
        }
    }

    /// Sets the summary for the command.
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    /// Sets the description for the command.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the operation ID for the command.
    pub fn operation_id(mut self, operation_id: impl Into<String>) -> Self {
        self.operation_id = Some(operation_id.into());
        self
    }

    /// Sets the aliases for the command.
    pub fn aliases(mut self, aliases: Vec<String>) -> Self {
        self.aliases = Some(aliases);
        self
    }

    /// Sets the tags for the command.
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Sets the parameters for the command.
    pub fn parameters(mut self, parameters: Vec<Parameter>) -> Self {
        self.parameters = Some(parameters);
        self
    }

    /// Sets the responses for the command.
    pub fn responses(mut self, responses: BTreeMap<String, Response>) -> Self {
        self.responses = Some(responses);
        self
    }

    /// Sets the extensions for the command.
    pub fn extensions(mut self, extensions: Extensions) -> Self {
        self.extensions = Some(extensions);
        self
    }
}

impl Default for Command {
    fn default() -> Self {
        Self::new()
    }
}

/// A map of command names to their definitions.
///
/// Commands can be nested to represent subcommands. For example:
/// - "build" -> Command
/// - "build.watch" -> Subcommand of build
pub type Commands = BTreeMap<String, Command>;
