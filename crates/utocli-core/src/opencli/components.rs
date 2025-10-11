//! Components container for reusable definitions.

use super::{Parameter, Response, Schema, map::Map, schema::RefOr};
use crate::ToResponse;

/// Reusable component definitions.
///
/// Components allow defining reusable schemas, parameters, and responses that can
/// be referenced from multiple locations in the specification.
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Components {
    /// Reusable schema definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<Map<String, RefOr<Schema>>>,

    /// Reusable parameter definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Map<String, RefOr<Parameter>>>,

    /// Reusable response definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responses: Option<Map<String, RefOr<Response>>>,
}

impl Components {
    /// Creates a new empty components container.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the schemas.
    pub fn schemas(mut self, schemas: Map<String, RefOr<Schema>>) -> Self {
        self.schemas = Some(schemas);
        self
    }

    /// Sets the parameters.
    pub fn parameters(mut self, parameters: Map<String, RefOr<Parameter>>) -> Self {
        self.parameters = Some(parameters);
        self
    }

    /// Sets the responses.
    pub fn responses(mut self, responses: Map<String, RefOr<Response>>) -> Self {
        self.responses = Some(responses);
        self
    }

    /// Add a response from a type implementing [`ToResponse`] trait.
    ///
    /// This method allows adding a response definition from a type that implements
    /// the `ToResponse` trait, similar to utoipa's component builder pattern.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use utocli::Components;
    ///
    /// #[derive(ToResponse)]
    /// struct SuccessResponse {
    ///     message: String,
    /// }
    ///
    /// let components = Components::new()
    ///     .response_from::<SuccessResponse>();
    /// ```
    pub fn response_from<'r, T: ToResponse<'r>>(mut self) -> Self {
        let (name, response) = T::response();
        let mut responses = self.responses.unwrap_or_default();
        responses.insert(name.to_string(), response);
        self.responses = Some(responses);
        self
    }
}
