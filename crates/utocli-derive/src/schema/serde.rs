//! Serde attribute parsing for schema generation.
//!
//! This module handles parsing of `#[serde(...)]` attributes to determine
//! enum representations and field/container renaming rules.
//!
//! Follows utoipa-gen's implementation closely to maintain architectural alignment.

use syn::{Attribute, spanned::Spanned};

use crate::diagnostics::Diagnostics;

/// The [Serde Enum representation](https://serde.rs/enum-representations.html) being used.
/// The default case (when no serde attributes are present) is `ExternallyTagged`.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum SerdeEnumRepr {
    /// Default representation: `{"VariantName": value}` or `"VariantName"` for unit variants
    #[default]
    ExternallyTagged,
    /// Internally tagged: `{"type": "VariantName", ...fields}`
    InternallyTagged { tag: String },
    /// Adjacently tagged: `{"tag": "VariantName", "content": value}`
    AdjacentlyTagged { tag: String, content: String },
    /// Untagged: No discriminator field
    Untagged,
    /// This is a variant that can never happen because `serde` will not accept it.
    /// With the current implementation it is necessary to have it as an intermediate state when parsing the
    /// attributes
    UnfinishedAdjacentlyTagged { content: String },
}

/// Attributes defined within a `#[serde(...)]` container attribute.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct SerdeContainer {
    /// Rename rule for all fields/variants
    pub rename_all: Option<RenameRule>,
    /// Enum representation strategy
    pub enum_repr: SerdeEnumRepr,
    /// Whether #[serde(default)] is set
    pub default: bool,
    /// Whether #[serde(deny_unknown_fields)] is set
    pub deny_unknown_fields: bool,
}

impl SerdeContainer {
    /// Parse a single serde attribute, currently supported attributes are:
    ///     * `rename_all = ...`
    ///     * `tag = ...`
    ///     * `content = ...`
    ///     * `untagged`
    ///     * `default`
    ///     * `deny_unknown_fields`
    fn parse_attribute(&mut self, attr: &Attribute) -> syn::Result<()> {
        if !attr.path().is_ident("serde") {
            return Ok(());
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                let value = meta.value()?;
                let s: syn::LitStr = value.parse()?;
                self.rename_all = Some(RenameRule::from_str(&s.value())?);
            } else if meta.path.is_ident("tag") {
                let value = meta.value()?;
                let tag: syn::LitStr = value.parse()?;
                let tag_value = tag.value();

                self.enum_repr = match &self.enum_repr {
                    SerdeEnumRepr::ExternallyTagged => {
                        SerdeEnumRepr::InternallyTagged { tag: tag_value }
                    }
                    SerdeEnumRepr::UnfinishedAdjacentlyTagged { content } => {
                        SerdeEnumRepr::AdjacentlyTagged {
                            tag: tag_value,
                            content: content.clone(),
                        }
                    }
                    SerdeEnumRepr::InternallyTagged { .. }
                    | SerdeEnumRepr::AdjacentlyTagged { .. } => {
                        return Err(Diagnostics::with_span(
                            tag.span(),
                            "Duplicate serde tag argument",
                        )
                        .help("Each enum can have only one #[serde(tag = \"...\")] attribute")
                        .into());
                    }
                    SerdeEnumRepr::Untagged => {
                        return Err(Diagnostics::with_span(
                            tag.span(),
                            "Untagged enum cannot have tag",
                        )
                        .help("Remove either #[serde(untagged)] or #[serde(tag = \"...\")] from the enum")
                        .note("See https://serde.rs/enum-representations.html for valid enum representations")
                        .into());
                    }
                };
            } else if meta.path.is_ident("content") {
                let value = meta.value()?;
                let content: syn::LitStr = value.parse()?;
                let content_value = content.value();

                self.enum_repr = match &self.enum_repr {
                    SerdeEnumRepr::InternallyTagged { tag } => SerdeEnumRepr::AdjacentlyTagged {
                        tag: tag.clone(),
                        content: content_value,
                    },
                    SerdeEnumRepr::ExternallyTagged => SerdeEnumRepr::UnfinishedAdjacentlyTagged {
                        content: content_value,
                    },
                    SerdeEnumRepr::AdjacentlyTagged { .. }
                    | SerdeEnumRepr::UnfinishedAdjacentlyTagged { .. } => {
                        return Err(Diagnostics::with_span(
                            content.span(),
                            "Duplicate serde content argument",
                        )
                        .help("Each enum can have only one #[serde(content = \"...\")] attribute")
                        .into());
                    }
                    SerdeEnumRepr::Untagged => {
                        return Err(Diagnostics::with_span(
                            content.span(),
                            "Untagged enum cannot have content",
                        )
                        .help("Remove either #[serde(untagged)] or #[serde(content = \"...\")] from the enum")
                        .note("See https://serde.rs/enum-representations.html for valid enum representations")
                        .into());
                    }
                };
            } else if meta.path.is_ident("untagged") {
                if !matches!(self.enum_repr, SerdeEnumRepr::ExternallyTagged) {
                    return Err(Diagnostics::with_span(
                        meta.path.span(),
                        "Cannot combine untagged with tag or content",
                    )
                    .help("Remove either #[serde(untagged)] or the #[serde(tag/content)] attributes")
                    .note("Untagged enums cannot have discriminator fields")
                    .into());
                }
                self.enum_repr = SerdeEnumRepr::Untagged;
            } else if meta.path.is_ident("default") {
                self.default = true;
            } else if meta.path.is_ident("deny_unknown_fields") {
                self.deny_unknown_fields = true;
            }

            Ok(())
        })?;

        Ok(())
    }
}

/// Parse serde container attributes from a list of attributes
pub fn parse_container(attributes: &[Attribute]) -> syn::Result<SerdeContainer> {
    let mut container = SerdeContainer::default();

    for attr in attributes {
        container.parse_attribute(attr)?;
    }

    Ok(container)
}

/// Attributes that can be defined on individual fields or enum variants
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SerdeValue {
    /// Field/variant rename
    pub rename: Option<String>,
    /// Rename rule for nested fields (variants only)
    pub rename_all: Option<RenameRule>,
    /// Skip this field during serialization
    pub skip: bool,
    /// Skip serialization based on a function
    pub skip_serializing_if: bool,
    /// Skip deserialization
    pub skip_deserializing: bool,
    /// Default value for this field
    pub default: bool,
    /// Flatten this field
    pub flatten: bool,
    /// Field uses serde_with double_option pattern
    pub double_option: bool,
}

impl SerdeValue {
    const SERDE_WITH_DOUBLE_OPTION: &'static str = "::serde_with::rust::double_option";
}

impl SerdeValue {
    fn parse_attribute(&mut self, attr: &Attribute) -> syn::Result<()> {
        if !attr.path().is_ident("serde") {
            return Ok(());
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let value = meta.value()?;
                let s: syn::LitStr = value.parse()?;
                self.rename = Some(s.value());
            } else if meta.path.is_ident("rename_all") {
                let value = meta.value()?;
                let s: syn::LitStr = value.parse()?;
                self.rename_all = Some(RenameRule::from_str(&s.value())?);
            } else if meta.path.is_ident("skip") {
                self.skip = true;
            } else if meta.path.is_ident("skip_serializing") {
                // Following utoipa: skip_serializing is treated as skip for schema generation
                self.skip = true;
            } else if meta.path.is_ident("skip_deserializing") {
                // Following utoipa: skip_deserializing is treated as skip for schema generation
                self.skip = true;
            } else if meta.path.is_ident("skip_serializing_if") {
                // Parse and ignore the value (e.g., "Option::is_none")
                if meta.input.peek(syn::Token![=]) {
                    let _ = meta.value()?;
                    let _: syn::LitStr = meta.input.parse()?;
                }
                self.skip_serializing_if = true;
            } else if meta.path.is_ident("default") {
                self.default = true;
            } else if meta.path.is_ident("flatten") {
                self.flatten = true;
            } else if meta.path.is_ident("with") {
                // Parse `with = "path"` to detect serde_with double_option
                let value = meta.value()?;
                let s: syn::LitStr = value.parse()?;
                if s.value() == Self::SERDE_WITH_DOUBLE_OPTION {
                    self.double_option = true;
                }
            }

            Ok(())
        })?;

        Ok(())
    }
}

/// Parse serde value attributes from a list of attributes
pub fn parse_value(attributes: &[Attribute]) -> syn::Result<SerdeValue> {
    let mut value = SerdeValue::default();

    for attr in attributes {
        value.parse_attribute(attr)?;
    }

    Ok(value)
}

/// Rename rules from serde
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameRule {
    /// Rename to lowercase
    Lowercase,
    /// Rename to UPPERCASE
    Uppercase,
    /// Rename to PascalCase
    PascalCase,
    /// Rename to camelCase
    CamelCase,
    /// Rename to snake_case
    SnakeCase,
    /// Rename to SCREAMING_SNAKE_CASE
    ScreamingSnakeCase,
    /// Rename to kebab-case
    KebabCase,
    /// Rename to SCREAMING-KEBAB-CASE
    ScreamingKebabCase,
}

impl RenameRule {
    /// Parse a rename rule from a string
    pub fn from_str(s: &str) -> syn::Result<Self> {
        match s {
            "lowercase" => Ok(RenameRule::Lowercase),
            "UPPERCASE" => Ok(RenameRule::Uppercase),
            "PascalCase" => Ok(RenameRule::PascalCase),
            "camelCase" => Ok(RenameRule::CamelCase),
            "snake_case" => Ok(RenameRule::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(RenameRule::ScreamingSnakeCase),
            "kebab-case" => Ok(RenameRule::KebabCase),
            "SCREAMING-KEBAB-CASE" => Ok(RenameRule::ScreamingKebabCase),
            _ => Err(Diagnostics::new(format!("Unknown serde rename rule: {}", s))
                .help("Valid rename rules are: lowercase, UPPERCASE, PascalCase, camelCase, snake_case, SCREAMING_SNAKE_CASE, kebab-case, SCREAMING-KEBAB-CASE")
                .note("See https://serde.rs/container-attrs.html#rename_all for documentation")
                .into()),
        }
    }

    /// Apply this rename rule to a string
    pub fn apply(&self, s: &str) -> String {
        match self {
            RenameRule::Lowercase => s.to_lowercase(),
            RenameRule::Uppercase => s.to_uppercase(),
            RenameRule::PascalCase => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for ch in s.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            }
            RenameRule::CamelCase => {
                let pascal = RenameRule::PascalCase.apply(s);
                let mut chars = pascal.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_lowercase().chain(chars).collect(),
                }
            }
            RenameRule::SnakeCase => {
                let mut snake = String::new();
                for (i, ch) in s.chars().enumerate() {
                    if ch.is_uppercase() && i > 0 {
                        snake.push('_');
                    }
                    snake.push(ch.to_lowercase().next().unwrap());
                }
                snake
            }
            RenameRule::ScreamingSnakeCase => RenameRule::SnakeCase.apply(s).to_uppercase(),
            RenameRule::KebabCase => RenameRule::SnakeCase.apply(s).replace('_', "-"),
            RenameRule::ScreamingKebabCase => {
                RenameRule::ScreamingSnakeCase.apply(s).replace('_', "-")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_rule_apply_transforms_string_correctly() {
        //* When/Then - Testing pure transformation logic
        assert_eq!(
            RenameRule::Lowercase.apply("FooBar"),
            "foobar",
            "Lowercase should transform FooBar to foobar"
        );
        assert_eq!(
            RenameRule::Uppercase.apply("FooBar"),
            "FOOBAR",
            "Uppercase should transform FooBar to FOOBAR"
        );
        assert_eq!(
            RenameRule::PascalCase.apply("foo_bar"),
            "FooBar",
            "PascalCase should transform foo_bar to FooBar"
        );
        assert_eq!(
            RenameRule::CamelCase.apply("foo_bar"),
            "fooBar",
            "CamelCase should transform foo_bar to fooBar"
        );
        assert_eq!(
            RenameRule::SnakeCase.apply("FooBar"),
            "foo_bar",
            "SnakeCase should transform FooBar to foo_bar"
        );
        assert_eq!(
            RenameRule::ScreamingSnakeCase.apply("FooBar"),
            "FOO_BAR",
            "ScreamingSnakeCase should transform FooBar to FOO_BAR"
        );
        assert_eq!(
            RenameRule::KebabCase.apply("FooBar"),
            "foo-bar",
            "KebabCase should transform FooBar to foo-bar"
        );
        assert_eq!(
            RenameRule::ScreamingKebabCase.apply("FooBar"),
            "FOO-BAR",
            "ScreamingKebabCase should transform FooBar to FOO-BAR"
        );
    }
}
