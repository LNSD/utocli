//! Response and media type entities.

use std::collections::BTreeMap;

use super::{Schema, schema::RefOr};

/// Describes command exit codes and output formats.
///
/// Responses are keyed by exit code strings (e.g., "0", "1", "2") and describe
/// the expected output format for each exit code.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Response {
    /// A description of the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A map of media types to their schemas.
    ///
    /// Common media types for CLI output:
    /// - `text/plain` - Plain text output
    /// - `application/json` - JSON formatted output
    /// - `application/yaml` - YAML formatted output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<BTreeMap<String, MediaType>>,
}

impl Response {
    /// Creates a new empty response.
    pub fn new() -> Self {
        Self {
            description: None,
            content: None,
        }
    }

    /// Sets the description for the response.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the content (media types) for the response.
    pub fn content(mut self, content: BTreeMap<String, MediaType>) -> Self {
        self.content = Some(content);
        self
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

/// A media type and its schema.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MediaType {
    /// The schema for this media type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<RefOr<Schema>>,

    /// Example value for this media type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
}

impl MediaType {
    /// Creates a new empty media type.
    pub fn new() -> Self {
        Self {
            schema: None,
            example: None,
        }
    }

    /// Sets the schema for the media type.
    pub fn schema(mut self, schema: RefOr<Schema>) -> Self {
        self.schema = Some(schema);
        self
    }

    /// Sets the example value.
    pub fn example(mut self, example: serde_json::Value) -> Self {
        self.example = Some(example);
        self
    }
}

impl Default for MediaType {
    fn default() -> Self {
        Self::new()
    }
}
