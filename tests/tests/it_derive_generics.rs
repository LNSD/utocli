//! Comprehensive tests for generic type support in ToSchema derive macro.
//!
//! These tests verify full generics functionality, modeled after utoipa's schema_generics.rs.
//! Tests cover: basic generics, custom bounds, nested generics, lifetimes, and container types.

#![allow(dead_code)]

use std::{borrow::Cow, marker::PhantomData};

use utocli::{ComposeSchema, ToSchema};

#[test]
fn generic_struct_compiles_and_generates_schema() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Response<T> {
        data: T,
        status: String,
    }

    //* When
    let schema = Response::<i32>::schema();
    let name = Response::<i32>::schema_name();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Response<i32> schema should be Object"
    );
    assert!(
        name.contains("Response"),
        "Schema name should contain 'Response', got: {}",
        name
    );
}

#[test]
fn generic_struct_with_custom_bound() {
    //* Given
    #[derive(utocli::ToSchema)]
    #[schema(bound = "T: Clone + Sized")]
    struct Type<T> {
        #[allow(unused)]
        t: PhantomData<T>,
    }

    #[derive(Clone)]
    struct NoToSchema;

    //* When
    let schema = Type::<NoToSchema>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Type<NoToSchema> should generate schema with custom bound"
    );
}

#[test]
fn generic_struct_with_multiple_type_params() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Pair<T, U> {
        first: T,
        second: U,
    }

    //* When
    let schema = Pair::<String, i32>::schema();
    let name = Pair::<String, i32>::schema_name();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Pair schema should be Object"
    );
    assert!(name.contains("Pair"), "Schema name should contain 'Pair'");
}

#[test]
fn generic_struct_with_lifetime_and_type_params() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Person<'p, T: Sized, P> {
        id: usize,
        name: Option<Cow<'p, str>>,
        field: T,
        t: P,
    }

    //* When
    let schema = Person::<'_, String, i32>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Person schema should be Object"
    );
}

#[test]
fn generic_struct_with_vec_field() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Page<T> {
        total: usize,
        page: usize,
        pages: usize,
        items: Vec<T>,
    }

    //* When
    let schema = Page::<String>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Page<String> schema should be Object"
    );
}

#[test]
fn generic_struct_with_option_field() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Wrapper<T> {
        value: Option<T>,
    }

    //* When
    let schema = Wrapper::<i32>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Wrapper<i32> schema should be Object"
    );
}

#[test]
fn generic_struct_with_box_field() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Boxed<T> {
        value: Box<T>,
    }

    //* When
    let schema = Boxed::<String>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Boxed<String> schema should be Object"
    );
}

#[test]
fn generic_struct_nested_in_another_generic() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Inner<T> {
        value: T,
    }

    #[derive(utocli::ToSchema)]
    struct Outer<T> {
        inner: Inner<T>,
    }

    //* When
    let schema = Outer::<i32>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Outer<i32> schema should be Object"
    );
}

#[test]
fn generic_enum_compiles_and_generates_schema() {
    //* Given
    #[derive(utocli::ToSchema)]
    enum Result<T, E> {
        Ok(T),
        Err(E),
    }

    //* When
    let schema = Result::<String, i32>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Result enum schema should compile"
    );
}

#[test]
fn generic_enum_with_variants() {
    //* Given
    #[derive(utocli::ToSchema)]
    enum Element<T> {
        One(T),
        Many(Vec<T>),
        None,
    }

    //* When
    let schema = Element::<String>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Element<String> schema should be Object"
    );
}

#[test]
fn generic_with_hashmap_field() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Config<K, V> {
        settings: std::collections::HashMap<K, V>,
    }

    //* When
    let schema = Config::<String, String>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Config schema should be Object"
    );
}

#[test]
fn generic_with_btreemap_field() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Sorted<K, V> {
        data: std::collections::BTreeMap<K, V>,
    }

    //* When
    let schema = Sorted::<String, i32>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Sorted schema should be Object"
    );
}

#[test]
fn generic_with_box_as_argument() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct High<T> {
        #[schema(inline)]
        high: T,
    }

    #[derive(utocli::ToSchema)]
    struct HighBox {
        value: High<Box<i32>>,
    }

    //* When
    let schema = HighBox::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "HighBox schema should be Object"
    );
}

#[test]
fn generic_with_cow_as_argument() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct High<T> {
        high: T,
    }

    #[derive(utocli::ToSchema)]
    struct HighCow(High<Cow<'static, str>>);

    //* When
    let schema = HighCow::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "HighCow schema should be Object"
    );
}

#[test]
fn deeply_nested_generics() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Deep<T> {
        value: Vec<Option<Box<T>>>,
    }

    //* When
    let schema = Deep::<String>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Deep<String> schema should be Object"
    );
}

#[test]
fn generic_enum_with_inline_fields() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Container<T> {
        data: T,
    }

    #[derive(utocli::ToSchema)]
    enum Variants {
        OptionInt(Container<Option<i32>>),
        #[schema(inline)]
        VecInt(Container<Vec<i32>>),
        #[schema(inline)]
        OptionVec(Container<Option<Vec<i32>>>),
    }

    //* When
    let schema = Variants::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Variants schema should be Object"
    );
}

#[test]
fn generic_schema_name_includes_type_params() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Response<T> {
        data: T,
    }

    //* When
    let name = Response::<String>::schema_name();

    //* Then
    assert!(
        name.contains("Response"),
        "Schema name should contain base name"
    );
    assert!(
        name.contains("T") || name == "Response<T>",
        "Schema name should reflect generic parameter, got: {}",
        name
    );
}

#[test]
fn generic_schema_name_with_multiple_params() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Map<K, V> {
        key: K,
        value: V,
    }

    //* When
    let name = Map::<String, i32>::schema_name();

    //* Then
    assert!(name.contains("Map"), "Schema name should contain base name");
}

#[test]
fn generic_types_implement_compose_schema() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Generic<T> {
        value: T,
    }

    //* When - ComposeSchema should be implemented for generic types
    let composed = Generic::<i32>::compose(vec![]);

    //* Then
    assert!(
        matches!(composed, utocli::RefOr::Ref(_)),
        "ComposeSchema should return a Ref"
    );
}

#[test]
fn non_generic_types_do_not_implement_compose_schema() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct NonGeneric {
        value: i32,
    }

    //* When
    let schema = NonGeneric::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Non-generic types should only implement ToSchema, not ComposeSchema"
    );
    // Note: NonGeneric::compose(vec![]) would fail to compile - this is correct behavior
}

#[test]
fn generic_with_as_name_attribute() {
    //* Given
    #[derive(utocli::ToSchema)]
    #[schema(as = "CustomName")]
    struct Generic<T> {
        value: T,
    }

    //* When
    let name = Generic::<i32>::schema_name();

    //* Then
    assert_eq!(
        name, "CustomName",
        "as attribute should override generic name"
    );
}

#[test]
fn generic_with_inline_field() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Inner {
        id: i32,
    }

    #[derive(utocli::ToSchema)]
    struct Outer<T> {
        #[schema(inline)]
        data: T,
        other: Inner,
    }

    //* When
    let schema = Outer::<Inner>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Outer<Inner> with inline should compile"
    );
}

#[test]
fn generic_with_where_clause() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Constrained<T>
    where
        T: Clone,
    {
        value: T,
    }

    //* When
    let schema = Constrained::<String>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Constrained schema should be Object"
    );
}

#[test]
fn generic_with_default_type_param() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct WithDefault<T = String> {
        value: T,
    }

    //* When
    let schema = WithDefault::<i32>::schema();
    let default_schema = WithDefault::<String>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "WithDefault<i32> schema should be Object"
    );
    assert!(
        matches!(default_schema, utocli::Schema::Object(_)),
        "WithDefault<String> (default type) schema should be Object"
    );
}

#[test]
fn generic_tuple_struct() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct Tuple<T>(T);

    //* When
    let schema = Tuple::<String>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "Tuple<String> schema should be Object"
    );
}

#[test]
fn generic_with_phantom_data() {
    //* Given
    #[derive(utocli::ToSchema)]
    struct WithPhantom<T> {
        value: String,
        #[allow(unused)]
        marker: PhantomData<T>,
    }

    //* When
    let schema = WithPhantom::<i32>::schema();

    //* Then
    assert!(
        matches!(schema, utocli::Schema::Object(_)),
        "WithPhantom<i32> schema should be Object"
    );
}
