//! Parsing for clap's `#[arg(...)]` attributes.
//!
//! This module handles extraction of argument metadata from clap's arg attributes
//! and maps them to OpenCLI parameter fields.

use quote::ToTokens;
use syn::{Attribute, Lit};

use crate::diagnostics::Diagnostics;

/// Parsed clap `#[arg(...)]` attribute values.
#[allow(dead_code)] // Will be used in Phase 2
#[derive(Debug, Default)]
pub struct ClapArgAttrs {
    /// Short flag (`-v`)
    pub short: Option<char>,
    /// Long flag (`--verbose`)
    pub long: Option<String>,
    /// Help text
    pub help: Option<String>,
    /// Long help text (preferred over help)
    pub long_help: Option<String>,
    /// Default value
    pub default_value: Option<String>,
    /// Value name hint (e.g., "FILE")
    pub value_name: Option<String>,
    /// Possible values for enum constraint
    pub possible_values: Option<Vec<serde_json::Value>>,
    /// Number of arguments (e.g., "1..=3")
    pub num_args: Option<NumArgs>,
    /// Whether this is a global parameter
    pub global: bool,
    /// Whether this is required
    pub required: bool,
    /// Explicit index for positional arguments
    pub index: Option<usize>,
}

/// Represents the num_args constraint from clap.
#[allow(dead_code)] // Will be used in Phase 4
#[derive(Debug, Clone)]
pub enum NumArgs {
    /// Exact count
    Exact(usize),
    /// Range with optional min and max
    Range {
        min: Option<usize>,
        max: Option<usize>,
    },
}

impl ClapArgAttrs {
    /// Parse `#[arg(...)]` attributes from a list of attributes.
    #[allow(dead_code)] // Will be used in parameter extraction
    pub fn parse(attrs: &[Attribute]) -> Result<Self, Diagnostics> {
        let mut result = Self::default();

        for attr in attrs {
            if !attr.path().is_ident("arg") {
                continue;
            }

            // Parse the arg attribute using parse_nested_meta
            attr.parse_nested_meta(|meta| {
                let path = &meta.path;

                if path.is_ident("short") {
                    // Parse short flag: #[arg(short)] or #[arg(short = 'v')]
                    if meta.input.peek(syn::Token![=]) {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Char(ch) = lit {
                            result.short = Some(ch.value());
                        } else {
                            return Err(syn::Error::new_spanned(lit, "expected character literal"));
                        }
                    } else {
                        // Infer from field name later
                        result.short = Some('\0'); // Placeholder
                    }
                } else if path.is_ident("long") {
                    // Parse long flag: #[arg(long)] or #[arg(long = "verbose")]
                    if meta.input.peek(syn::Token![=]) {
                        result.long = Some(parse_string_value(&meta)?);
                    } else {
                        // Infer from field name later
                        result.long = Some(String::new()); // Placeholder
                    }
                } else if path.is_ident("help") {
                    result.help = Some(parse_string_value(&meta)?);
                } else if path.is_ident("long_help") {
                    result.long_help = Some(parse_string_value(&meta)?);
                } else if path.is_ident("default_value") {
                    result.default_value = Some(parse_string_value(&meta)?);
                } else if path.is_ident("value_name") {
                    result.value_name = Some(parse_string_value(&meta)?);
                } else if path.is_ident("global") {
                    // Boolean flag
                    result.global = parse_bool_flag(&meta);
                } else if path.is_ident("required") {
                    // Boolean flag
                    result.required = parse_bool_flag(&meta);
                } else if path.is_ident("index") {
                    result.index = Some(parse_usize_value(&meta)?);
                } else if path.is_ident("value_parser") {
                    // Parse value_parser = ["val1", "val2", "val3"]
                    result.possible_values = Some(parse_value_parser(&meta)?);
                } else if path.is_ident("num_args") {
                    // Parse num_args = 1 or num_args = 1..=3
                    result.num_args = Some(parse_num_args(&meta)?);
                }

                Ok(())
            })
            .map_err(Diagnostics::from_syn_error)?;
        }

        Ok(result)
    }
}

/// Parse a string value from a meta attribute.
#[allow(dead_code)] // Will be used in parameter extraction
fn parse_string_value(meta: &syn::meta::ParseNestedMeta) -> syn::Result<String> {
    let value = meta.value()?;
    let lit: Lit = value.parse()?;

    match lit {
        Lit::Str(s) => Ok(s.value()),
        _ => Err(syn::Error::new_spanned(lit, "expected string literal")),
    }
}

/// Parse a usize value from a meta attribute.
#[allow(dead_code)] // Will be used in parameter extraction
fn parse_usize_value(meta: &syn::meta::ParseNestedMeta) -> syn::Result<usize> {
    let value = meta.value()?;
    let lit: Lit = value.parse()?;

    match lit {
        Lit::Int(i) => i
            .base10_parse::<usize>()
            .map_err(|e| syn::Error::new_spanned(i, format!("expected usize: {}", e))),
        _ => Err(syn::Error::new_spanned(lit, "expected integer literal")),
    }
}

/// Parse a boolean flag (attribute present means true).
#[allow(dead_code)] // Will be used in parameter extraction
fn parse_bool_flag(meta: &syn::meta::ParseNestedMeta) -> bool {
    // If we can read a value, parse it as bool
    if let Ok(value) = meta.value()
        && let Ok(Lit::Bool(b)) = value.parse::<Lit>()
    {
        return b.value();
    }
    // Otherwise, presence of attribute means true
    true
}

/// Parse value_parser = ["val1", "val2", "val3"] into enum values.
fn parse_value_parser(meta: &syn::meta::ParseNestedMeta) -> syn::Result<Vec<serde_json::Value>> {
    let value = meta.value()?;

    // Parse array literal
    let expr: syn::Expr = value.parse()?;

    match expr {
        syn::Expr::Array(arr) => {
            let mut values = Vec::new();
            for elem in &arr.elems {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(s), ..
                }) = elem
                {
                    values.push(serde_json::Value::String(s.value()));
                }
            }
            Ok(values)
        }
        _ => Err(syn::Error::new_spanned(
            expr,
            "expected array of string literals",
        )),
    }
}

/// Parse num_args = 1 or num_args = 1..=3.
fn parse_num_args(meta: &syn::meta::ParseNestedMeta) -> syn::Result<NumArgs> {
    let value = meta.value()?;
    let expr: syn::Expr = value.parse()?;

    match expr {
        // Exact count: num_args = 1
        syn::Expr::Lit(syn::ExprLit {
            lit: Lit::Int(i), ..
        }) => {
            let count = i.base10_parse::<usize>()?;
            Ok(NumArgs::Exact(count))
        }
        // Range: num_args = 1..=3 or 1.. or ..=3
        syn::Expr::Range(range) => {
            let min = if let Some(start) = &range.start {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: Lit::Int(i), ..
                }) = &**start
                {
                    Some(i.base10_parse::<usize>()?)
                } else {
                    None
                }
            } else {
                None
            };

            let max = if let Some(end) = &range.end {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: Lit::Int(i), ..
                }) = &**end
                {
                    Some(i.base10_parse::<usize>()?)
                } else {
                    None
                }
            } else {
                None
            };

            Ok(NumArgs::Range { min, max })
        }
        _ => Err(syn::Error::new_spanned(
            expr,
            "expected integer or range (e.g., 1 or 1..=3)",
        )),
    }
}

/// Parsed `#[opencli(...)]` attribute values for field-level augmentation.
#[allow(dead_code)] // Will be used in Phase 2
#[derive(Debug, Default)]
pub struct OpenCliArgAttrs {
    /// Override parameter scope
    pub scope: Option<String>,
    /// Schema format hint (e.g., "path", "email")
    pub format: Option<String>,
    /// Example value
    pub example: Option<String>,
    /// Override description
    pub description: Option<String>,
    /// Arity constraint (explicit OpenCLI)
    pub arity: Option<(Option<usize>, Option<usize>)>,
    /// Mark as deprecated
    pub deprecated: bool,
    /// Custom extensions (x-*)
    pub extensions: Vec<(String, String)>,
}

impl OpenCliArgAttrs {
    /// Parse `#[opencli(...)]` attributes from a list of attributes.
    #[allow(dead_code)] // Will be used in parameter extraction
    pub fn parse(attrs: &[Attribute]) -> Result<Self, Diagnostics> {
        let mut result = Self::default();

        for attr in attrs {
            if !attr.path().is_ident("opencli") {
                continue;
            }

            // Parse the opencli attribute using parse_nested_meta
            attr.parse_nested_meta(|meta| {
                let path = &meta.path;

                if path.is_ident("scope") {
                    result.scope = Some(parse_string_value(&meta)?);
                } else if path.is_ident("format") {
                    result.format = Some(parse_string_value(&meta)?);
                } else if path.is_ident("example") {
                    result.example = Some(parse_string_value(&meta)?);
                } else if path.is_ident("description") {
                    result.description = Some(parse_string_value(&meta)?);
                } else if path.is_ident("deprecated") {
                    result.deprecated = true;
                } else if path.to_token_stream().to_string().starts_with("x_") {
                    // Handle custom extensions (x-*)
                    let key = path.to_token_stream().to_string().replace('_', "-");
                    let value = parse_string_value(&meta)?;
                    result.extensions.push((key, value));
                } else if path.is_ident("arity") {
                    // Parse arity(min = 1, max = 3) or arity(min = 1)
                    let content;
                    syn::parenthesized!(content in meta.input);

                    let mut min = None;
                    let mut max = None;

                    while !content.is_empty() {
                        let ident: syn::Ident = content.parse()?;
                        content.parse::<syn::Token![=]>()?;
                        let value: syn::LitInt = content.parse()?;

                        if ident == "min" {
                            min = Some(value.base10_parse()?);
                        } else if ident == "max" {
                            max = Some(value.base10_parse()?);
                        }

                        // Parse optional comma
                        if !content.is_empty() {
                            content.parse::<syn::Token![,]>()?;
                        }
                    }

                    result.arity = Some((min, max));
                }

                Ok(())
            })
            .map_err(Diagnostics::from_syn_error)?;
        }

        Ok(result)
    }
}
