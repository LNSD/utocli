//! Extension support for OpenCLI specification.
//!
//! Extensions allow vendor-specific properties (x-something) to be added to any object.

use std::collections::BTreeMap;

/// A map of extension properties.
///
/// Extensions are key-value pairs where the key must start with "x-" and the value
/// can be any valid JSON value.
pub type Extensions = BTreeMap<String, serde_json::Value>;
