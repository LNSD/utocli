//! Type tree analysis for generic type handling.
//!
//! This module provides infrastructure for analyzing Rust types, particularly
//! for handling generic types like `Vec<T>`, `Option<T>`, `HashMap<K, V>`, etc.

use syn::{GenericArgument, Path, PathArguments, Type, TypePath};

use crate::diagnostics::Diagnostics;

/// Represents the value type category of a type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ValueType {
    /// Primitive type (integers, floats, bool, string)
    Primitive,
    /// Custom object type
    Object,
    /// Tuple type
    Tuple,
    /// Generic value type
    Value,
}

/// Known generic wrapper types that require special handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericType {
    /// Vec<T>
    Vec,
    /// Option<T>
    Option,
    /// Box<T>
    Box,
    /// HashMap<K, V> or BTreeMap<K, V>
    Map,
}

/// A tree structure representing a Rust type with its generic parameters.
///
/// This follows utoipa's TypeTree pattern for analyzing and tracking generic types.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeTree<'t> {
    /// The type path (e.g., `std::vec::Vec`)
    pub path: Option<&'t Path>,
    /// The value type category
    pub value_type: ValueType,
    /// If this is a known generic wrapper, which one
    pub generic_type: Option<GenericType>,
    /// Child type trees for generic arguments
    pub children: Option<Vec<TypeTree<'t>>>,
}

impl<'t> TypeTree<'t> {
    /// Create a TypeTree from a syn::Type.
    ///
    /// This analyzes the type structure and creates a tree representation
    /// that can be used for schema generation and generic handling.
    pub fn from_type(ty: &'t Type) -> Result<Self, Diagnostics> {
        match ty {
            Type::Path(type_path) => Self::from_type_path(type_path),
            Type::Reference(reference) => Self::from_type(reference.elem.as_ref()),
            Type::Tuple(tuple) if tuple.elems.is_empty() => {
                // Unit type ()
                Ok(Self {
                    path: None,
                    value_type: ValueType::Tuple,
                    generic_type: None,
                    children: None,
                })
            }
            Type::Tuple(_) => Ok(Self {
                path: None,
                value_type: ValueType::Tuple,
                generic_type: None,
                children: None,
            }),
            _ => Err(Diagnostics::new("Unsupported type for TypeTree analysis")
                .help("Only path types, references, and tuples are currently supported")
                .note(format!("Found type: {:?}", ty))),
        }
    }

    fn from_type_path(type_path: &'t TypePath) -> Result<Self, Diagnostics> {
        let path = &type_path.path;
        let last_segment = path
            .segments
            .last()
            .ok_or_else(|| Diagnostics::new("Type path must have at least one segment"))?;

        let type_name = last_segment.ident.to_string();

        // Check if this is a known generic wrapper type
        let generic_type = match type_name.as_str() {
            "Vec" => Some(GenericType::Vec),
            "Option" => Some(GenericType::Option),
            "Box" => Some(GenericType::Box),
            "HashMap" | "BTreeMap" => Some(GenericType::Map),
            _ => None,
        };

        // Parse generic arguments if present
        let children = if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
            let mut child_trees = Vec::new();

            for arg in &args.args {
                if let GenericArgument::Type(inner_ty) = arg {
                    child_trees.push(Self::from_type(inner_ty)?);
                }
            }

            if !child_trees.is_empty() {
                Some(child_trees)
            } else {
                None
            }
        } else {
            None
        };

        // Determine value type
        let value_type = if Self::is_primitive(&type_name) {
            ValueType::Primitive
        } else {
            ValueType::Object
        };

        Ok(Self {
            path: Some(path),
            value_type,
            generic_type,
            children,
        })
    }

    /// Check if a type name represents a primitive type.
    pub fn is_primitive(type_name: &str) -> bool {
        matches!(
            type_name,
            "i8" | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "f32"
                | "f64"
                | "bool"
                | "String"
                | "str"
                | "char"
        )
    }

    /// Check if this is an Option<T> type.
    pub fn is_option(&self) -> bool {
        self.generic_type == Some(GenericType::Option)
    }

    /// Check if this is a Vec<T> type.
    pub fn is_vec(&self) -> bool {
        self.generic_type == Some(GenericType::Vec)
    }

    /// Check if this is a Map type (HashMap or BTreeMap).
    #[allow(dead_code)]
    pub fn is_map(&self) -> bool {
        self.generic_type == Some(GenericType::Map)
    }

    /// Get the inner type for a wrapper type (Option<T>, Vec<T>, Box<T>).
    ///
    /// Returns the first child if this is a wrapper with exactly one generic argument.
    pub fn get_wrapped_type(&self) -> Option<&TypeTree<'t>> {
        self.children.as_ref()?.first()
    }

    /// Get the type name as a string.
    #[allow(dead_code)]
    pub fn type_name(&self) -> Option<String> {
        self.path
            .and_then(|p| p.segments.last().map(|s| s.ident.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn from_type_with_primitive_creates_primitive_tree() {
        //* Given
        let ty: Type = parse_quote!(i32);

        //* When
        let tree = TypeTree::from_type(&ty).expect("should parse primitive type");

        //* Then
        assert_eq!(
            tree.value_type,
            ValueType::Primitive,
            "should be primitive value type"
        );
        assert_eq!(tree.generic_type, None, "primitives have no generic type");
        assert_eq!(tree.children, None, "primitives have no children");
    }

    #[test]
    fn from_type_with_option_creates_option_tree() {
        //* Given
        let ty: Type = parse_quote!(Option<String>);

        //* When
        let tree = TypeTree::from_type(&ty).expect("should parse Option type");

        //* Then
        assert_eq!(
            tree.generic_type,
            Some(GenericType::Option),
            "should be Option generic"
        );
        assert!(tree.is_option(), "is_option() should return true");
        assert!(tree.children.is_some(), "Option should have children");

        let inner = tree.get_wrapped_type().expect("should have wrapped type");
        assert_eq!(
            inner.value_type,
            ValueType::Primitive,
            "inner type should be primitive"
        );
    }

    #[test]
    fn from_type_with_vec_creates_vec_tree() {
        //* Given
        let ty: Type = parse_quote!(Vec<i32>);

        //* When
        let tree = TypeTree::from_type(&ty).expect("should parse Vec type");

        //* Then
        assert_eq!(
            tree.generic_type,
            Some(GenericType::Vec),
            "should be Vec generic"
        );
        assert!(tree.is_vec(), "is_vec() should return true");
        assert!(tree.children.is_some(), "Vec should have children");

        let inner = tree.get_wrapped_type().expect("should have wrapped type");
        assert_eq!(
            inner.value_type,
            ValueType::Primitive,
            "inner type should be primitive"
        );
    }

    #[test]
    fn from_type_with_nested_generics_creates_nested_tree() {
        //* Given
        let ty: Type = parse_quote!(Vec<Option<String>>);

        //* When
        let tree = TypeTree::from_type(&ty).expect("should parse nested generic type");

        //* Then
        assert!(tree.is_vec(), "outer type should be Vec");

        let option_tree = tree.get_wrapped_type().expect("should have Option wrapper");
        assert!(option_tree.is_option(), "middle type should be Option");

        let string_tree = option_tree
            .get_wrapped_type()
            .expect("should have String inner type");
        assert_eq!(
            string_tree.value_type,
            ValueType::Primitive,
            "inner type should be String primitive"
        );
    }

    #[test]
    fn from_type_with_hashmap_creates_map_tree() {
        //* Given
        let ty: Type = parse_quote!(HashMap<String, i32>);

        //* When
        let tree = TypeTree::from_type(&ty).expect("should parse HashMap type");

        //* Then
        assert!(tree.is_map(), "should be Map generic");
        assert_eq!(
            tree.children
                .as_ref()
                .expect("Map should have children")
                .len(),
            2,
            "HashMap should have 2 children (key and value)"
        );
    }
}
