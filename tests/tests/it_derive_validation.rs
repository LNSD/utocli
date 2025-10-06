//! E2E tests for validation attributes on ToParameter and ToSchema derives

#![allow(dead_code)]

use utocli::{ParameterIn, RefOr, Schema, ToSchema};

#[test]
fn derive_to_parameter_with_numeric_minimum_and_maximum_applies_constraints() {
    #[derive(utocli::ToParameter)]
    struct QueryParams {
        #[param(minimum = 5, maximum = 100)]
        count: i32,

        #[param(minimum = 0.5, maximum = 99.9)]
        ratio: f64,
    }

    //* When
    let params = QueryParams::parameters();

    //* Then
    assert_eq!(params.len(), 2, "should generate 2 parameters");

    // Verify count parameter has numeric constraints
    let count_param = &params[0];
    assert_eq!(count_param.name, "count");
    if let Some(RefOr::T(Schema::Object(obj))) = &count_param.schema {
        assert_eq!(
            obj.minimum,
            Some(5.0),
            "count should have minimum constraint"
        );
        assert_eq!(
            obj.maximum,
            Some(100.0),
            "count should have maximum constraint"
        );
    } else {
        panic!("Expected object schema for count parameter");
    }

    // Verify ratio parameter has numeric constraints
    let ratio_param = &params[1];
    assert_eq!(ratio_param.name, "ratio");
    if let Some(RefOr::T(Schema::Object(obj))) = &ratio_param.schema {
        assert_eq!(
            obj.minimum,
            Some(0.5),
            "ratio should have minimum constraint"
        );
        assert_eq!(
            obj.maximum,
            Some(99.9),
            "ratio should have maximum constraint"
        );
    } else {
        panic!("Expected object schema for ratio parameter");
    }
}

#[test]
fn derive_to_parameter_with_string_length_and_pattern_applies_constraints() {
    #[derive(utocli::ToParameter)]
    struct QueryParams {
        #[param(min_length = 3, max_length = 50)]
        username: String,

        #[param(pattern = "[a-z0-9]+")]
        slug: String,
    }

    //* When
    let params = QueryParams::parameters();

    //* Then
    assert_eq!(params.len(), 2, "should generate 2 parameters");

    // Verify username parameter has length constraints
    let username_param = &params[0];
    assert_eq!(username_param.name, "username");
    if let Some(RefOr::T(Schema::Object(obj))) = &username_param.schema {
        assert_eq!(
            obj.min_length,
            Some(3),
            "username should have min_length constraint"
        );
        assert_eq!(
            obj.max_length,
            Some(50),
            "username should have max_length constraint"
        );
    } else {
        panic!("Expected object schema for username parameter");
    }

    // Verify slug parameter has pattern constraint
    let slug_param = &params[1];
    assert_eq!(slug_param.name, "slug");
    if let Some(RefOr::T(Schema::Object(obj))) = &slug_param.schema {
        assert_eq!(
            obj.pattern,
            Some("[a-z0-9]+".to_string()),
            "slug should have pattern constraint"
        );
    } else {
        panic!("Expected object schema for slug parameter");
    }
}

#[test]
fn derive_to_parameter_with_combined_validations_applies_all_constraints() {
    #[derive(utocli::ToParameter)]
    struct QueryParams {
        #[param(minimum = 1, maximum = 10, in = "option")]
        page: i32,

        #[param(min_length = 1, max_length = 100, pattern = ".*")]
        query: String,
    }

    //* When
    let params = QueryParams::parameters();

    //* Then
    assert_eq!(params.len(), 2, "should generate 2 parameters");

    // Verify page parameter has both numeric validations and parameter type
    let page_param = &params[0];
    assert_eq!(page_param.name, "page");
    assert_eq!(
        page_param.in_,
        Some(ParameterIn::Option),
        "page should be an option parameter"
    );
    if let Some(RefOr::T(Schema::Object(obj))) = &page_param.schema {
        assert_eq!(
            obj.minimum,
            Some(1.0),
            "page should have minimum constraint"
        );
        assert_eq!(
            obj.maximum,
            Some(10.0),
            "page should have maximum constraint"
        );
    } else {
        panic!("Expected object schema for page parameter");
    }

    // Verify query parameter has all string validations
    let query_param = &params[1];
    assert_eq!(query_param.name, "query");
    if let Some(RefOr::T(Schema::Object(obj))) = &query_param.schema {
        assert_eq!(
            obj.min_length,
            Some(1),
            "query should have min_length constraint"
        );
        assert_eq!(
            obj.max_length,
            Some(100),
            "query should have max_length constraint"
        );
        assert_eq!(
            obj.pattern,
            Some(".*".to_string()),
            "query should have pattern constraint"
        );
    } else {
        panic!("Expected object schema for query parameter");
    }
}

#[test]
fn derive_to_schema_with_numeric_constraints_applies_to_fields() {
    #[derive(utocli::ToSchema)]
    struct Item {
        #[schema(minimum = 0, maximum = 1000)]
        quantity: u32,

        #[schema(minimum = -100.5, maximum = 100.5)]
        temperature: f64,
    }

    //* When
    let schema = Item::schema();

    //* Then
    if let Schema::Object(obj) = schema {
        let properties = obj.properties.expect("schema should have properties");

        // Verify quantity field has numeric constraints
        if let Some(RefOr::T(Schema::Object(qty_obj))) = properties.get("quantity") {
            assert_eq!(
                qty_obj.minimum,
                Some(0.0),
                "quantity should have minimum constraint"
            );
            assert_eq!(
                qty_obj.maximum,
                Some(1000.0),
                "quantity should have maximum constraint"
            );
        } else {
            panic!("Expected quantity property in schema");
        }

        // Verify temperature field has numeric constraints
        if let Some(RefOr::T(Schema::Object(temp_obj))) = properties.get("temperature") {
            assert_eq!(
                temp_obj.minimum,
                Some(-100.5),
                "temperature should have minimum constraint"
            );
            assert_eq!(
                temp_obj.maximum,
                Some(100.5),
                "temperature should have maximum constraint"
            );
        } else {
            panic!("Expected temperature property in schema");
        }
    } else {
        panic!("Expected object schema for Item");
    }
}

#[test]
fn derive_to_schema_with_string_constraints_applies_to_fields() {
    #[derive(utocli::ToSchema)]
    struct Item {
        #[schema(min_length = 5, max_length = 20)]
        name: String,

        #[schema(pattern = "^[A-Z]{2}[0-9]{4}$")]
        code: String,

        #[schema(min_length = 1, max_length = 500, pattern = ".*")]
        description: String,
    }

    //* When
    let schema = Item::schema();

    //* Then
    if let Schema::Object(obj) = schema {
        let properties = obj.properties.expect("schema should have properties");

        // Verify name field has length constraints
        if let Some(RefOr::T(Schema::Object(name_obj))) = properties.get("name") {
            assert_eq!(
                name_obj.min_length,
                Some(5),
                "name should have min_length constraint"
            );
            assert_eq!(
                name_obj.max_length,
                Some(20),
                "name should have max_length constraint"
            );
        } else {
            panic!("Expected name property in schema");
        }

        // Verify code field has pattern constraint
        if let Some(RefOr::T(Schema::Object(code_obj))) = properties.get("code") {
            assert_eq!(
                code_obj.pattern,
                Some("^[A-Z]{2}[0-9]{4}$".to_string()),
                "code should have pattern constraint"
            );
        } else {
            panic!("Expected code property in schema");
        }

        // Verify description field has all string validations
        if let Some(RefOr::T(Schema::Object(desc_obj))) = properties.get("description") {
            assert_eq!(
                desc_obj.min_length,
                Some(1),
                "description should have min_length constraint"
            );
            assert_eq!(
                desc_obj.max_length,
                Some(500),
                "description should have max_length constraint"
            );
            assert_eq!(
                desc_obj.pattern,
                Some(".*".to_string()),
                "description should have pattern constraint"
            );
        } else {
            panic!("Expected description property in schema");
        }
    } else {
        panic!("Expected object schema for Item");
    }
}

#[test]
fn derive_to_parameter_with_validation_and_other_attributes_applies_all() {
    #[derive(utocli::ToParameter)]
    struct QueryParams {
        /// Page number for pagination
        #[param(minimum = 1, default = "1", example = "5", in = "option")]
        page: i32,

        /// Search query string
        #[param(min_length = 1, max_length = 200, default = "")]
        q: String,
    }

    //* When
    let params = QueryParams::parameters();

    //* Then
    assert_eq!(params.len(), 2, "should generate 2 parameters");

    // Verify validations work with other attributes on page parameter
    let page_param = &params[0];
    assert_eq!(
        page_param.description.as_deref(),
        Some("Page number for pagination"),
        "page should have description from doc comment"
    );
    assert_eq!(
        page_param.in_,
        Some(ParameterIn::Option),
        "page should be an option parameter"
    );
    if let Some(RefOr::T(Schema::Object(obj))) = &page_param.schema {
        assert_eq!(
            obj.minimum,
            Some(1.0),
            "page should have minimum constraint"
        );
        assert!(obj.default.is_some(), "page should have default value");
        assert!(obj.example.is_some(), "page should have example value");
    } else {
        panic!("Expected object schema for page parameter");
    }

    // Verify validations work with other attributes on q parameter
    let q_param = &params[1];
    assert_eq!(
        q_param.description.as_deref(),
        Some("Search query string"),
        "q should have description from doc comment"
    );
    if let Some(RefOr::T(Schema::Object(obj))) = &q_param.schema {
        assert_eq!(
            obj.min_length,
            Some(1),
            "q should have min_length constraint"
        );
        assert_eq!(
            obj.max_length,
            Some(200),
            "q should have max_length constraint"
        );
        assert!(obj.default.is_some(), "q should have default value");
    } else {
        panic!("Expected object schema for q parameter");
    }
}

#[test]
fn derive_to_parameter_with_optional_fields_applies_validations() {
    #[derive(utocli::ToParameter)]
    struct QueryParams {
        #[param(minimum = 10, maximum = 100)]
        limit: Option<i32>,

        #[param(min_length = 3)]
        filter: Option<String>,
    }

    //* When
    let params = QueryParams::parameters();

    //* Then
    assert_eq!(params.len(), 2, "should generate 2 parameters");

    // Verify optional fields still have validations applied
    let limit_param = &params[0];
    if let Some(RefOr::T(Schema::Object(obj))) = &limit_param.schema {
        assert_eq!(
            obj.minimum,
            Some(10.0),
            "optional limit should have minimum constraint"
        );
        assert_eq!(
            obj.maximum,
            Some(100.0),
            "optional limit should have maximum constraint"
        );
    } else {
        panic!("Expected object schema for limit parameter");
    }

    let filter_param = &params[1];
    if let Some(RefOr::T(Schema::Object(obj))) = &filter_param.schema {
        assert_eq!(
            obj.min_length,
            Some(3),
            "optional filter should have min_length constraint"
        );
    } else {
        panic!("Expected object schema for filter parameter");
    }
}

#[test]
fn derive_to_parameter_with_multiple_of_applies_constraint() {
    #[derive(utocli::ToParameter)]
    struct QueryParams {
        #[param(multiple_of = 5)]
        quantity: i32,

        #[param(multiple_of = 0.25)]
        price: f64,
    }

    //* When
    let params = QueryParams::parameters();

    //* Then
    assert_eq!(params.len(), 2, "should generate 2 parameters");

    let quantity_param = &params[0];
    if let Some(RefOr::T(Schema::Object(obj))) = &quantity_param.schema {
        assert_eq!(
            obj.multiple_of,
            Some(5.0),
            "quantity should have multiple_of constraint"
        );
    } else {
        panic!("Expected object schema for quantity parameter");
    }

    let price_param = &params[1];
    if let Some(RefOr::T(Schema::Object(obj))) = &price_param.schema {
        assert_eq!(
            obj.multiple_of,
            Some(0.25),
            "price should have multiple_of constraint"
        );
    } else {
        panic!("Expected object schema for price parameter");
    }
}

#[test]
fn derive_to_parameter_with_exclusive_minimum_and_maximum_applies_constraints() {
    #[derive(utocli::ToParameter)]
    struct QueryParams {
        #[param(
            minimum = 0,
            exclusive_minimum = true,
            maximum = 100,
            exclusive_maximum = true
        )]
        percentage: f64,
    }

    //* When
    let params = QueryParams::parameters();

    //* Then
    assert_eq!(params.len(), 1, "should generate 1 parameter");

    let percentage_param = &params[0];
    if let Some(RefOr::T(Schema::Object(obj))) = &percentage_param.schema {
        assert_eq!(
            obj.minimum,
            Some(0.0),
            "percentage should have minimum constraint"
        );
        assert_eq!(
            obj.exclusive_minimum,
            Some(true),
            "percentage should have exclusive_minimum constraint"
        );
        assert_eq!(
            obj.maximum,
            Some(100.0),
            "percentage should have maximum constraint"
        );
        assert_eq!(
            obj.exclusive_maximum,
            Some(true),
            "percentage should have exclusive_maximum constraint"
        );
    } else {
        panic!("Expected object schema for percentage parameter");
    }
}

#[test]
fn derive_to_schema_with_multiple_of_applies_to_fields() {
    #[derive(utocli::ToSchema)]
    struct Item {
        #[schema(multiple_of = 10)]
        quantity: u32,

        #[schema(multiple_of = 0.5)]
        weight: f64,
    }

    //* When
    let schema = Item::schema();

    //* Then
    if let Schema::Object(obj) = schema {
        let properties = obj.properties.expect("schema should have properties");

        if let Some(RefOr::T(Schema::Object(qty_obj))) = properties.get("quantity") {
            assert_eq!(
                qty_obj.multiple_of,
                Some(10.0),
                "quantity should have multiple_of constraint"
            );
        } else {
            panic!("Expected quantity property in schema");
        }

        if let Some(RefOr::T(Schema::Object(weight_obj))) = properties.get("weight") {
            assert_eq!(
                weight_obj.multiple_of,
                Some(0.5),
                "weight should have multiple_of constraint"
            );
        } else {
            panic!("Expected weight property in schema");
        }
    } else {
        panic!("Expected object schema for Item");
    }
}

#[test]
fn derive_to_schema_with_exclusive_minimum_and_maximum_applies_to_fields() {
    #[derive(utocli::ToSchema)]
    struct Item {
        #[schema(minimum = 0, exclusive_minimum = true)]
        positive_value: f64,

        #[schema(maximum = 100, exclusive_maximum = true)]
        below_hundred: f64,
    }

    //* When
    let schema = Item::schema();

    //* Then
    if let Schema::Object(obj) = schema {
        let properties = obj.properties.expect("schema should have properties");

        if let Some(RefOr::T(Schema::Object(pos_obj))) = properties.get("positive_value") {
            assert_eq!(
                pos_obj.minimum,
                Some(0.0),
                "positive_value should have minimum constraint"
            );
            assert_eq!(
                pos_obj.exclusive_minimum,
                Some(true),
                "positive_value should have exclusive_minimum constraint"
            );
        } else {
            panic!("Expected positive_value property in schema");
        }

        if let Some(RefOr::T(Schema::Object(below_obj))) = properties.get("below_hundred") {
            assert_eq!(
                below_obj.maximum,
                Some(100.0),
                "below_hundred should have maximum constraint"
            );
            assert_eq!(
                below_obj.exclusive_maximum,
                Some(true),
                "below_hundred should have exclusive_maximum constraint"
            );
        } else {
            panic!("Expected below_hundred property in schema");
        }
    } else {
        panic!("Expected object schema for Item");
    }
}

#[test]
fn derive_to_schema_with_min_and_max_properties_applies_to_fields() {
    #[derive(utocli::ToSchema)]
    struct Item {
        #[schema(min_properties = 1, max_properties = 10)]
        metadata: String,
    }

    //* When
    let schema = Item::schema();

    //* Then
    if let Schema::Object(obj) = schema {
        let properties = obj.properties.expect("schema should have properties");

        if let Some(RefOr::T(Schema::Object(meta_obj))) = properties.get("metadata") {
            assert_eq!(
                meta_obj.min_properties,
                Some(1),
                "metadata should have min_properties constraint"
            );
            assert_eq!(
                meta_obj.max_properties,
                Some(10),
                "metadata should have max_properties constraint"
            );
        } else {
            panic!("Expected metadata property in schema");
        }
    } else {
        panic!("Expected object schema for Item");
    }
}

#[test]
fn derive_to_parameter_with_all_advanced_validations_applies_all_constraints() {
    #[derive(utocli::ToParameter)]
    struct QueryParams {
        #[param(
            minimum = 1,
            maximum = 100,
            multiple_of = 5,
            exclusive_minimum = false,
            exclusive_maximum = true
        )]
        score: i32,
    }

    //* When
    let params = QueryParams::parameters();

    //* Then
    assert_eq!(params.len(), 1, "should generate 1 parameter");

    let score_param = &params[0];
    if let Some(RefOr::T(Schema::Object(obj))) = &score_param.schema {
        assert_eq!(
            obj.minimum,
            Some(1.0),
            "score should have minimum constraint"
        );
        assert_eq!(
            obj.maximum,
            Some(100.0),
            "score should have maximum constraint"
        );
        assert_eq!(
            obj.multiple_of,
            Some(5.0),
            "score should have multiple_of constraint"
        );
        assert_eq!(
            obj.exclusive_minimum,
            Some(false),
            "score should have exclusive_minimum set to false"
        );
        assert_eq!(
            obj.exclusive_maximum,
            Some(true),
            "score should have exclusive_maximum set to true"
        );
    } else {
        panic!("Expected object schema for score parameter");
    }
}
