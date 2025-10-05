//! Tag entity for organizing commands.

/// Organizes commands into logical groups.
///
/// Tags allow grouping of commands for better organization and documentation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Tag {
    /// The name of the tag.
    pub name: String,

    /// A description for the tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Tag {
    /// Creates a new `Tag` with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
        }
    }

    /// Sets the description for the tag.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}
