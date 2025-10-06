//! Schema types and validation.

use super::map::Map;

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

    /// A description of the schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

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
    pub properties: Option<Map<String, RefOr<Schema>>>,

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

    /// Multiple of value for numeric types.
    #[serde(rename = "multipleOf", skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,

    /// Exclusive minimum value for numeric types.
    #[serde(rename = "exclusiveMinimum", skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<bool>,

    /// Exclusive maximum value for numeric types.
    #[serde(rename = "exclusiveMaximum", skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<bool>,

    /// Maximum number of properties for object types.
    #[serde(rename = "maxProperties", skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<usize>,

    /// Minimum number of properties for object types.
    #[serde(rename = "minProperties", skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<usize>,

    /// Title of the schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Whether the schema is deprecated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    /// Whether the field is read-only.
    #[serde(rename = "readOnly", skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    /// Whether the field is write-only.
    #[serde(rename = "writeOnly", skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,

    /// Whether the value can be null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,

    /// Whether additional properties are allowed (for object types).
    #[serde(
        rename = "additionalProperties",
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_properties: Option<bool>,
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

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
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
    pub fn example(mut self, value: impl Into<Option<serde_json::Value>>) -> Self {
        self.example = value.into();
        self
    }

    /// Sets the title.
    pub fn title(mut self, title: Option<impl Into<String>>) -> Self {
        self.title = title.map(|t| t.into());
        self
    }

    /// Sets the deprecated flag.
    pub fn deprecated(mut self, deprecated: Option<bool>) -> Self {
        self.deprecated = deprecated;
        self
    }

    /// Sets the read-only flag.
    pub fn read_only(mut self, read_only: Option<bool>) -> Self {
        self.read_only = read_only;
        self
    }

    /// Sets the write-only flag.
    pub fn write_only(mut self, write_only: Option<bool>) -> Self {
        self.write_only = write_only;
        self
    }

    /// Sets the nullable flag.
    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = Some(nullable);
        self
    }

    /// Sets whether additional properties are allowed.
    pub fn additional_properties(mut self, allowed: Option<bool>) -> Self {
        self.additional_properties = allowed;
        self
    }

    /// Sets the properties.
    pub fn properties(mut self, properties: Map<String, RefOr<Schema>>) -> Self {
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

    /// Sets the multiple of value.
    pub fn multiple_of(mut self, multiple_of: f64) -> Self {
        self.multiple_of = Some(multiple_of);
        self
    }

    /// Sets the exclusive minimum flag.
    pub fn exclusive_minimum(mut self, exclusive_minimum: bool) -> Self {
        self.exclusive_minimum = Some(exclusive_minimum);
        self
    }

    /// Sets the exclusive maximum flag.
    pub fn exclusive_maximum(mut self, exclusive_maximum: bool) -> Self {
        self.exclusive_maximum = Some(exclusive_maximum);
        self
    }

    /// Sets the maximum number of properties.
    pub fn max_properties(mut self, max_properties: usize) -> Self {
        self.max_properties = Some(max_properties);
        self
    }

    /// Sets the minimum number of properties.
    pub fn min_properties(mut self, min_properties: usize) -> Self {
        self.min_properties = Some(min_properties);
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

    /// Maximum number of items in the array.
    #[serde(rename = "maxItems", skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,

    /// Minimum number of items in the array.
    #[serde(rename = "minItems", skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
}

impl Array {
    /// Creates a new array schema.
    pub fn new() -> Self {
        Self {
            schema_type: SchemaType::Array,
            items: None,
            max_items: None,
            min_items: None,
        }
    }

    /// Sets the items schema.
    pub fn items(mut self, items: RefOr<Schema>) -> Self {
        self.items = Some(Box::new(items));
        self
    }

    /// Sets the maximum number of items.
    pub fn max_items(mut self, max_items: usize) -> Self {
        self.max_items = Some(max_items);
        self
    }

    /// Sets the minimum number of items.
    pub fn min_items(mut self, min_items: usize) -> Self {
        self.min_items = Some(min_items);
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
    /// Null type (for untagged enums and nullable values).
    Null,
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
