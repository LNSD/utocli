//! Schema types and validation.

use std::collections::BTreeMap;

/// A schema definition or a reference to a schema component.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum RefOr<T> {
    /// A reference to a component.
    Ref(Ref),
    /// An inline definition.
    T(T),
}

impl<T> RefOr<T> {
    /// Creates a new reference to a component.
    pub fn new_ref(ref_path: impl Into<String>) -> Self {
        RefOr::Ref(Ref {
            ref_path: ref_path.into(),
        })
    }

    /// Creates a new inline definition.
    pub fn new_inline(value: T) -> Self {
        RefOr::T(value)
    }
}

/// A reference to a component.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Ref {
    /// The reference path to the component (e.g., "#/components/schemas/Pet").
    #[serde(rename = "$ref")]
    pub ref_path: String,
}

/// A schema definition.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Schema {
    /// An object schema.
    Object(Box<Object>),
    /// An array schema.
    Array(Array),
}

/// An object schema definition.
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Object {
    /// The schema type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<SchemaType>,

    /// The schema format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<SchemaFormat>,

    /// Possible values for an enumeration.
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<serde_json::Value>>,

    /// Default value for this schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    /// Example value for this schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,

    /// Properties for object types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<BTreeMap<String, RefOr<Schema>>>,

    /// Required properties for object types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    /// Minimum value for numeric types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,

    /// Maximum value for numeric types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,

    /// Minimum length for string types.
    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    /// Maximum length for string types.
    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    /// Pattern for string types (regular expression).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

impl Object {
    /// Creates a new empty object schema.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the schema type.
    pub fn schema_type(mut self, schema_type: SchemaType) -> Self {
        self.schema_type = Some(schema_type);
        self
    }

    /// Sets the schema format.
    pub fn format(mut self, format: SchemaFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Sets the enum values.
    pub fn enum_values(mut self, values: Vec<serde_json::Value>) -> Self {
        self.enum_values = Some(values);
        self
    }

    /// Sets the default value.
    pub fn default_value(mut self, value: serde_json::Value) -> Self {
        self.default = Some(value);
        self
    }

    /// Sets the example value.
    pub fn example(mut self, value: serde_json::Value) -> Self {
        self.example = Some(value);
        self
    }

    /// Sets the properties.
    pub fn properties(mut self, properties: BTreeMap<String, RefOr<Schema>>) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Sets the required properties.
    pub fn required(mut self, required: Vec<String>) -> Self {
        self.required = Some(required);
        self
    }

    /// Sets the minimum value.
    pub fn minimum(mut self, minimum: f64) -> Self {
        self.minimum = Some(minimum);
        self
    }

    /// Sets the maximum value.
    pub fn maximum(mut self, maximum: f64) -> Self {
        self.maximum = Some(maximum);
        self
    }

    /// Sets the minimum length.
    pub fn min_length(mut self, min_length: usize) -> Self {
        self.min_length = Some(min_length);
        self
    }

    /// Sets the maximum length.
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Sets the pattern.
    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }
}

/// An array schema definition.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Array {
    /// The schema type (always "array").
    #[serde(rename = "type")]
    pub schema_type: SchemaType,

    /// The schema for array items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<RefOr<Schema>>>,
}

impl Array {
    /// Creates a new array schema.
    pub fn new() -> Self {
        Self {
            schema_type: SchemaType::Array,
            items: None,
        }
    }

    /// Sets the items schema.
    pub fn items(mut self, items: RefOr<Schema>) -> Self {
        self.items = Some(Box::new(items));
        self
    }
}

impl Default for Array {
    fn default() -> Self {
        Self::new()
    }
}

/// Schema type enumeration.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    /// String type.
    String,
    /// Integer type.
    Integer,
    /// Number type (floating point).
    Number,
    /// Boolean type.
    Boolean,
    /// Array type.
    Array,
    /// Object type.
    Object,
}

/// Schema format enumeration for additional type information.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaFormat {
    // String formats
    /// File system path.
    Path,
    /// Email address.
    Email,
    /// URI (Uniform Resource Identifier).
    Uri,
    /// URL (Uniform Resource Locator).
    Url,
    /// Date (YYYY-MM-DD).
    Date,
    /// Date and time (RFC 3339).
    #[serde(rename = "date-time")]
    DateTime,
    /// Time (HH:MM:SS).
    Time,
    /// UUID (Universally Unique Identifier).
    Uuid,
    /// IPv4 address.
    Ipv4,
    /// IPv6 address.
    Ipv6,
    /// Hostname.
    Hostname,

    // Integer formats
    /// 32-bit signed integer.
    Int32,
    /// 64-bit signed integer.
    Int64,

    // Number formats
    /// Single-precision floating point.
    Float,
    /// Double-precision floating point.
    Double,
}
