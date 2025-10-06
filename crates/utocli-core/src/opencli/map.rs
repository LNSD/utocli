//! Map type re-exports for conditional ordering.
//!
//! This module provides the `Map` type alias that is used throughout the OpenCLI specification.
//! The concrete type depends on the `preserve_order` feature:
//! - When `preserve_order` is enabled: `IndexMap` (preserves insertion order)
//! - When disabled (default): `BTreeMap` (sorted by keys)

cfg_if::cfg_if! {
    if #[cfg(feature = "preserve_order")] {
        pub use indexmap::IndexMap as Map;
    } else {
        pub use std::collections::BTreeMap as Map;
    }
}
