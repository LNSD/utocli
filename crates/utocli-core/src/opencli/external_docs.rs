//! External documentation entity.

/// Links to external documentation.
///
/// This struct represents a reference to external documentation resources
/// that provide additional information about the CLI tool.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExternalDocs {
    /// A description of the target documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The URL for the target documentation.
    pub url: String,
}

impl ExternalDocs {
    /// Creates a new `ExternalDocs` with the given URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            description: None,
            url: url.into(),
        }
    }

    /// Sets the description for the external documentation.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}
