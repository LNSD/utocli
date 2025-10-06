//! Extension support for OpenCLI specification.
//!
//! Extensions allow vendor-specific properties (x-something) to be added to any object.

use super::map::Map;

/// A map of extension properties.
///
/// Extensions are key-value pairs where the key must start with "x-" and the value
/// can be any valid JSON value.
pub type Extensions = Map<String, serde_json::Value>;
