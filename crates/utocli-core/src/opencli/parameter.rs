//! Parameter entity for CLI arguments, flags, and options.

use super::{Schema, extensions::Extensions, schema::RefOr};

/// Defines command-line parameters (arguments, flags, options).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Parameter {
    /// The name of the parameter.
    pub name: String,

    /// The location of the parameter.
    #[serde(rename = "in", skip_serializing_if = "Option::is_none")]
    pub in_: Option<ParameterIn>,

    /// The position of the parameter (for positional arguments).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u32>,

    /// Alternative names for the parameter (e.g., short flags).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<Vec<String>>,

    /// A description of the parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether the parameter is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    /// The scope of the parameter (local or inherited).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<ParameterScope>,

    /// The arity (number of values) for the parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arity: Option<Arity>,

    /// The schema for the parameter value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<RefOr<Schema>>,

    /// Extension properties.
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

impl Parameter {
    /// Creates a new parameter with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            in_: None,
            position: None,
            alias: None,
            description: None,
            required: None,
            scope: None,
            arity: None,
            schema: None,
            extensions: None,
        }
    }

    /// Creates a new positional argument parameter.
    pub fn new_argument(name: impl Into<String>, position: u32) -> Self {
        Self {
            name: name.into(),
            in_: Some(ParameterIn::Argument),
            position: Some(position),
            alias: None,
            description: None,
            required: Some(true),
            scope: None,
            arity: None,
            schema: None,
            extensions: None,
        }
    }

    /// Creates a new flag parameter (boolean switch).
    pub fn new_flag(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            in_: Some(ParameterIn::Flag),
            position: None,
            alias: None,
            description: None,
            required: None,
            scope: None,
            arity: None,
            schema: None,
            extensions: None,
        }
    }

    /// Creates a new option parameter (named parameter with value).
    pub fn new_option(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            in_: Some(ParameterIn::Option),
            position: None,
            alias: None,
            description: None,
            required: None,
            scope: None,
            arity: None,
            schema: None,
            extensions: None,
        }
    }

    /// Sets the parameter location.
    pub fn in_(mut self, in_: ParameterIn) -> Self {
        self.in_ = Some(in_);
        self
    }

    /// Sets the position for positional arguments.
    pub fn position(mut self, position: u32) -> Self {
        self.position = Some(position);
        self
    }

    /// Sets aliases for the parameter.
    pub fn alias(mut self, alias: Vec<String>) -> Self {
        self.alias = Some(alias);
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets whether the parameter is required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }

    /// Sets the parameter scope.
    pub fn scope(mut self, scope: ParameterScope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Sets the arity.
    pub fn arity(mut self, arity: Arity) -> Self {
        self.arity = Some(arity);
        self
    }

    /// Sets the schema.
    pub fn schema(mut self, schema: RefOr<Schema>) -> Self {
        self.schema = Some(schema);
        self
    }

    /// Sets the extensions.
    pub fn extensions(mut self, extensions: Extensions) -> Self {
        self.extensions = Some(extensions);
        self
    }
}

/// The location of the parameter in the command line.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterIn {
    /// Positional argument.
    Argument,
    /// Boolean flag (e.g., --verbose).
    Flag,
    /// Named option with value (e.g., --output <file>).
    Option,
}

/// The scope of the parameter.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterScope {
    /// Available only to this command.
    Local,
    /// Available to subcommands.
    Inherited,
}

/// The arity (number of values) for a parameter.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Arity {
    /// Minimum number of values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<u32>,

    /// Maximum number of values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<u32>,
}

impl Arity {
    /// Creates a new arity specification.
    pub fn new() -> Self {
        Self {
            min: None,
            max: None,
        }
    }

    /// Sets the minimum number of values.
    pub fn min(mut self, min: u32) -> Self {
        self.min = Some(min);
        self
    }

    /// Sets the maximum number of values.
    pub fn max(mut self, max: u32) -> Self {
        self.max = Some(max);
        self
    }

    /// Creates an arity with an exact count.
    pub fn exact(count: u32) -> Self {
        Self {
            min: Some(count),
            max: Some(count),
        }
    }

    /// Creates an arity for a range of values.
    pub fn range(min: u32, max: u32) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }
}

impl Default for Arity {
    fn default() -> Self {
        Self::new()
    }
}
