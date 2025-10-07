//! Tests for recursion detection with `no_recursion` attribute.
//!
//! These tests verify that the `no_recursion` attribute works correctly
//! to break infinite loops in recursive schema definitions.

#![allow(dead_code)]

use utocli::{Schema, ToSchema as _};

/// Test case from utoipa: Pet -> Owner -> Pet recursion
///
/// This should compile without infinite recursion because Owner has
/// `no_recursion` on the pets field.
#[test]
fn schema_with_field_level_no_recursion_breaks_pet_owner_cycle() {
    //* Given
    #[derive(utocli::ToSchema)]
    pub struct Pet {
        name: String,
        owner: Option<Owner>,
    }

    #[derive(utocli::ToSchema)]
    pub struct Owner {
        name: String,
        #[schema(no_recursion)]
        pets: Vec<Pet>,
    }

    //* When
    let pet_schema = Pet::schema();
    let owner_schema = Owner::schema();

    //* Then
    assert!(
        matches!(pet_schema, Schema::Object(_)),
        "Pet schema should be an Object schema"
    );
    assert!(
        matches!(owner_schema, Schema::Object(_)),
        "Owner schema should be an Object schema"
    );
}

/// Test case: Container-level no_recursion
///
/// When applied at container level, all fields should have no_recursion set.
#[test]
fn schema_with_container_level_no_recursion_breaks_tree_cycle() {
    //* Given
    #[derive(utocli::ToSchema)]
    #[schema(no_recursion)]
    pub struct Tree {
        value: i32,
        left: Box<Tree>,
        right: Box<Tree>,
    }

    //* When
    let schema = Tree::schema();

    //* Then
    assert!(
        matches!(schema, Schema::Object(_)),
        "Tree schema should be an Object schema"
    );
}

/// Test that no_recursion works with container types like Vec.
#[test]
fn schema_with_vec_field_no_recursion_breaks_node_children_cycle() {
    //* Given
    #[derive(utocli::ToSchema)]
    pub struct Node {
        id: u64,
        #[schema(no_recursion)]
        children: Vec<Node>,
    }

    //* When
    let schema = Node::schema();

    //* Then
    assert!(
        matches!(schema, Schema::Object(_)),
        "Node schema should be an Object schema"
    );
}

/// Test that no_recursion works with Option types.
#[test]
fn schema_with_option_field_no_recursion_breaks_linked_list_cycle() {
    //* Given
    #[derive(utocli::ToSchema)]
    pub struct LinkedNode {
        value: String,
        #[schema(no_recursion)]
        next: Option<Box<LinkedNode>>,
    }

    //* When
    let schema = LinkedNode::schema();

    //* Then
    assert!(
        matches!(schema, Schema::Object(_)),
        "LinkedNode schema should be an Object schema"
    );
}

/// Only one field needs no_recursion to break the cycle.
#[test]
fn schema_with_selective_no_recursion_breaks_graph_cycle() {
    //* Given
    #[derive(utocli::ToSchema)]
    pub struct GraphNode {
        id: u64,
        #[schema(no_recursion)]
        neighbors: Vec<GraphNode>,
        metadata: String,
    }

    //* When
    let schema = GraphNode::schema();

    //* Then
    assert!(
        matches!(schema, Schema::Object(_)),
        "GraphNode schema should be an Object schema"
    );
}

/// Test A -> B -> A pattern with `no_recursion`.
#[test]
fn schema_with_mutual_recursion_no_recursion_breaks_type_a_type_b_cycle() {
    //* Given
    #[derive(utocli::ToSchema)]
    pub struct TypeA {
        name: String,
        b_ref: Option<Box<TypeB>>,
    }

    #[derive(utocli::ToSchema)]
    pub struct TypeB {
        id: u64,
        #[schema(no_recursion)]
        a_ref: Option<Box<TypeA>>,
    }

    //* When
    let schema_a = TypeA::schema();
    let schema_b = TypeB::schema();

    //* Then
    assert!(
        matches!(schema_a, Schema::Object(_)),
        "TypeA schema should be an Object schema"
    );
    assert!(
        matches!(schema_b, Schema::Object(_)),
        "TypeB schema should be an Object schema"
    );
}
