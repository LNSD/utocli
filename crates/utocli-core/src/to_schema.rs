//! ToSchema trait for types that can be converted to OpenCLI schemas.

use crate::{Schema, SchemaFormat, SchemaType};

/// Trait for implementing OpenCLI schema generation.
///
/// This trait is typically implemented via the `#[derive(ToSchema)]` macro and there is
/// usually no need to implement this trait manually.
///
/// # Examples
///
/// Use `#[derive(ToSchema)]` to implement ToSchema trait:
/// ```ignore
/// #[derive(ToSchema)]
/// struct User {
///     id: u64,
///     name: String,
/// }
/// ```
pub trait ToSchema {
    /// Get the schema for this type.
    fn schema() -> Schema;

    /// Get the schema name for this type.
    ///
    /// The name is used for referencing this schema in the OpenCLI document.
    fn schema_name() -> &'static str;
}

// Implement ToSchema for primitive types

// Integer types - only Int32 and Int64 have formats in OpenCLI
impl ToSchema for i8 {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new().schema_type(SchemaType::Integer),
        ))
    }
    fn schema_name() -> &'static str {
        "i8"
    }
}

impl ToSchema for i16 {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new().schema_type(SchemaType::Integer),
        ))
    }
    fn schema_name() -> &'static str {
        "i16"
    }
}

impl ToSchema for i32 {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new()
                .schema_type(SchemaType::Integer)
                .format(SchemaFormat::Int32),
        ))
    }
    fn schema_name() -> &'static str {
        "i32"
    }
}

impl ToSchema for i64 {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new()
                .schema_type(SchemaType::Integer)
                .format(SchemaFormat::Int64),
        ))
    }
    fn schema_name() -> &'static str {
        "i64"
    }
}

impl ToSchema for i128 {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new().schema_type(SchemaType::Integer),
        ))
    }
    fn schema_name() -> &'static str {
        "i128"
    }
}

impl ToSchema for isize {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new().schema_type(SchemaType::Integer),
        ))
    }
    fn schema_name() -> &'static str {
        "isize"
    }
}

impl ToSchema for u8 {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new().schema_type(SchemaType::Integer),
        ))
    }
    fn schema_name() -> &'static str {
        "u8"
    }
}

impl ToSchema for u16 {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new().schema_type(SchemaType::Integer),
        ))
    }
    fn schema_name() -> &'static str {
        "u16"
    }
}

impl ToSchema for u32 {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new()
                .schema_type(SchemaType::Integer)
                .format(SchemaFormat::Int32), // Use Int32 for u32 as well
        ))
    }
    fn schema_name() -> &'static str {
        "u32"
    }
}

impl ToSchema for u64 {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new()
                .schema_type(SchemaType::Integer)
                .format(SchemaFormat::Int64), // Use Int64 for u64 as well
        ))
    }
    fn schema_name() -> &'static str {
        "u64"
    }
}

impl ToSchema for u128 {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new().schema_type(SchemaType::Integer),
        ))
    }
    fn schema_name() -> &'static str {
        "u128"
    }
}

impl ToSchema for usize {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new().schema_type(SchemaType::Integer),
        ))
    }
    fn schema_name() -> &'static str {
        "usize"
    }
}

// Float types
impl ToSchema for f32 {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new()
                .schema_type(SchemaType::Number)
                .format(SchemaFormat::Float),
        ))
    }
    fn schema_name() -> &'static str {
        "f32"
    }
}

impl ToSchema for f64 {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new()
                .schema_type(SchemaType::Number)
                .format(SchemaFormat::Double),
        ))
    }
    fn schema_name() -> &'static str {
        "f64"
    }
}

impl ToSchema for bool {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new().schema_type(SchemaType::Boolean),
        ))
    }

    fn schema_name() -> &'static str {
        "bool"
    }
}

impl ToSchema for String {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new().schema_type(SchemaType::String),
        ))
    }

    fn schema_name() -> &'static str {
        "String"
    }
}

impl ToSchema for str {
    fn schema() -> Schema {
        Schema::Object(Box::new(
            crate::Object::new().schema_type(SchemaType::String),
        ))
    }

    fn schema_name() -> &'static str {
        "str"
    }
}

impl ToSchema for &str {
    fn schema() -> Schema {
        str::schema()
    }

    fn schema_name() -> &'static str {
        "str"
    }
}
