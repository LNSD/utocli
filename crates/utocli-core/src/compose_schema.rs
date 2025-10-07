//! ComposeSchema trait for runtime generic type composition.
//!
//! This module provides the infrastructure for composing generic schemas at runtime,
//! allowing types like `Response<T>` to generate proper schemas when instantiated with
//! concrete types like `Response<User>`.

use crate::{Array, RefOr, Schema, SchemaType};

/// Trait for composing schemas with generic type parameters at runtime.
///
/// This trait enables types with generic parameters to generate proper schemas
/// by substituting the generic placeholders with concrete type schemas.
///
/// # Example
///
/// ```ignore
/// // For a generic type Response<T>:
/// struct Response<T> { data: T }
///
/// // When instantiated as Response<User>, ComposeSchema allows:
/// // 1. Get the schema for User
/// // 2. Substitute T with User's schema
/// // 3. Generate the complete Response<User> schema
/// ```
pub trait ComposeSchema {
    /// Compose a schema by substituting generic placeholders with provided schemas.
    ///
    /// The `generics` parameter contains schemas for the generic type parameters,
    /// indexed in the order they appear in the type definition.
    fn compose(generics: Vec<RefOr<Schema>>) -> RefOr<Schema>;
}

/// Helper function to get a schema from the generics vector or compose a default.
///
/// If a schema exists at the given index, return it. Otherwise, compose a default
/// schema using the type's ComposeSchema implementation.
pub fn schema_or_compose<T: ComposeSchema>(
    generics: Vec<RefOr<Schema>>,
    index: usize,
) -> RefOr<Schema> {
    if let Some(schema) = generics.get(index).cloned() {
        schema
    } else {
        T::compose(generics)
    }
}

// Implement ComposeSchema for primitive types - they have no generic parameters

macro_rules! impl_compose_schema_primitive {
    ($($ty:ty => $schema_fn:expr);* $(;)?) => {
        $(
            impl ComposeSchema for $ty {
                fn compose(_generics: Vec<RefOr<Schema>>) -> RefOr<Schema> {
                    RefOr::T($schema_fn)
                }
            }
        )*
    };
}

impl_compose_schema_primitive! {
    i8 => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Integer)));
    i16 => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Integer)));
    i32 => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Integer).format(crate::SchemaFormat::Int32)));
    i64 => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Integer).format(crate::SchemaFormat::Int64)));
    i128 => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Integer)));
    isize => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Integer)));
    u8 => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Integer)));
    u16 => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Integer)));
    u32 => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Integer).format(crate::SchemaFormat::Int32)));
    u64 => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Integer).format(crate::SchemaFormat::Int64)));
    u128 => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Integer)));
    usize => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Integer)));
    f32 => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Number).format(crate::SchemaFormat::Float)));
    f64 => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Number).format(crate::SchemaFormat::Double)));
    bool => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::Boolean)));
    String => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::String)));
    str => Schema::Object(Box::new(crate::Object::new().schema_type(SchemaType::String)));
}

impl ComposeSchema for &str {
    fn compose(_generics: Vec<RefOr<Schema>>) -> RefOr<Schema> {
        str::compose(_generics)
    }
}

// Implement ComposeSchema for generic container types

impl<T: ComposeSchema> ComposeSchema for Option<T> {
    fn compose(generics: Vec<RefOr<Schema>>) -> RefOr<Schema> {
        // Option is represented as the inner type (it's nullable at the field level)
        // We just pass through to the inner type's schema
        schema_or_compose::<T>(generics, 0)
    }
}

impl<T: ComposeSchema> ComposeSchema for Vec<T> {
    fn compose(generics: Vec<RefOr<Schema>>) -> RefOr<Schema> {
        RefOr::T(Schema::Array(
            Array::new().items(schema_or_compose::<T>(generics, 0)),
        ))
    }
}

impl<T: ComposeSchema> ComposeSchema for Box<T> {
    fn compose(generics: Vec<RefOr<Schema>>) -> RefOr<Schema> {
        // Box is transparent - just use the inner type's schema
        schema_or_compose::<T>(generics, 0)
    }
}

impl<K: ComposeSchema, V: ComposeSchema> ComposeSchema for std::collections::HashMap<K, V> {
    fn compose(_generics: Vec<RefOr<Schema>>) -> RefOr<Schema> {
        // Map is represented as an object with additionalProperties = true
        // OpenCLI doesn't support schemas for additionalProperties like OpenAPI does
        RefOr::T(Schema::Object(Box::new(
            crate::Object::new()
                .schema_type(SchemaType::Object)
                .additional_properties(Some(true)),
        )))
    }
}

impl<K: ComposeSchema, V: ComposeSchema> ComposeSchema for std::collections::BTreeMap<K, V> {
    fn compose(generics: Vec<RefOr<Schema>>) -> RefOr<Schema> {
        // Same as HashMap
        std::collections::HashMap::<K, V>::compose(generics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_primitive_type_with_empty_generics_returns_object_schema() {
        //* When
        let schema = i32::compose(vec![]);

        //* Then
        assert!(
            matches!(schema, RefOr::T(Schema::Object(_))),
            "i32::compose should return Object schema"
        );
    }

    #[test]
    fn compose_vec_type_with_empty_generics_returns_array_schema() {
        //* When
        let schema = Vec::<String>::compose(vec![]);

        //* Then
        assert!(
            matches!(schema, RefOr::T(Schema::Array(_))),
            "Vec<String>::compose should return Array schema"
        );
    }

    #[test]
    fn compose_option_type_with_empty_generics_returns_wrapped_schema() {
        //* When
        let schema = Option::<i32>::compose(vec![]);

        //* Then
        assert!(
            matches!(schema, RefOr::T(Schema::Object(_))),
            "Option<i32>::compose should return inner type's schema"
        );
    }

    #[test]
    fn schema_or_compose_with_provided_schema_returns_provided() {
        //* Given
        let provided = RefOr::T(Schema::Object(Box::new(
            crate::Object::new().schema_type(SchemaType::String),
        )));
        let generics = vec![provided.clone()];

        //* When
        let result = schema_or_compose::<String>(generics, 0);

        //* Then
        assert!(
            matches!(result, RefOr::T(Schema::Object(_))),
            "should return provided schema from generics vector"
        );
    }

    #[test]
    fn schema_or_compose_without_provided_schema_composes_default() {
        //* Given
        let generics = vec![];

        //* When
        let result = schema_or_compose::<String>(generics, 0);

        //* Then
        assert!(
            matches!(result, RefOr::T(Schema::Object(_))),
            "should compose default schema when none provided"
        );
    }
}
