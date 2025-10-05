//! Components container for reusable definitions.

use std::collections::BTreeMap;

use super::{Parameter, Response, Schema, schema::RefOr};

/// Reusable component definitions.
///
/// Components allow defining reusable schemas, parameters, and responses that can
/// be referenced from multiple locations in the specification.
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Components {
    /// Reusable schema definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<BTreeMap<String, RefOr<Schema>>>,

    /// Reusable parameter definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<BTreeMap<String, RefOr<Parameter>>>,

    /// Reusable response definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responses: Option<BTreeMap<String, RefOr<Response>>>,
}

impl Components {
    /// Creates a new empty components container.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the schemas.
    pub fn schemas(mut self, schemas: BTreeMap<String, RefOr<Schema>>) -> Self {
        self.schemas = Some(schemas);
        self
    }

    /// Sets the parameters.
    pub fn parameters(mut self, parameters: BTreeMap<String, RefOr<Parameter>>) -> Self {
        self.parameters = Some(parameters);
        self
    }

    /// Sets the responses.
    pub fn responses(mut self, responses: BTreeMap<String, RefOr<Response>>) -> Self {
        self.responses = Some(responses);
        self
    }
}
