//! Environment variable entity.

/// Maps environment variables to CLI configuration.
///
/// Environment variables can be used to configure CLI behavior or provide
/// default values for parameters.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EnvironmentVariable {
    /// The name of the environment variable.
    pub name: String,

    /// A description of what the environment variable controls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl EnvironmentVariable {
    /// Creates a new `EnvironmentVariable` with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
        }
    }

    /// Sets the description for the environment variable.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}
