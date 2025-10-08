//! Parsing for clap's `#[derive(Subcommand)]` enums.
//!
//! This module handles extraction of subcommand definitions from clap's Subcommand derive
//! and generates the CommandCollection implementation.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Variant};

use crate::{
    arg::{ClapArgAttrs, OpenCliArgAttrs},
    diagnostics::Diagnostics,
    mapping,
};

/// Generate CommandCollection implementation from a clap Subcommand enum.
pub fn generate_command_collection(input: &DeriveInput) -> Result<TokenStream2, Diagnostics> {
    let enum_name = &input.ident;

    // Extract enum variants
    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return Err(Diagnostics::new(
                "CommandCollection can only be derived for enums",
            ));
        }
    };

    // Generate commands for each variant
    let command_inserts: Vec<_> = variants
        .iter()
        .map(generate_command_from_variant)
        .collect::<Result<_, _>>()?;

    // Generate the CommandCollection impl
    Ok(quote! {
        impl ::utocli::CommandCollection for #enum_name {
            fn commands() -> ::utocli::Map<String, ::utocli::Command> {
                let mut commands = ::utocli::Map::new();
                #(#command_inserts)*
                commands
            }
        }
    })
}

/// Generate a command from a single enum variant.
fn generate_command_from_variant(variant: &Variant) -> Result<TokenStream2, Diagnostics> {
    // Get command name from variant name (convert to kebab-case)
    let variant_name = &variant.ident;
    let command_name = to_kebab_case(&variant_name.to_string());
    let command_path = format!("/{}", command_name);

    // Extract doc comments for summary
    let summary = extract_doc_comment(&variant.attrs);

    // Build Command
    let mut command_tokens = quote! {
        let mut command = ::utocli::Command::new()
    };

    // Add summary from doc comments
    if let Some(ref summary_text) = summary {
        command_tokens.extend(quote! {
            .summary(#summary_text)
        });
    }

    // Check for #[command(about = "...")] or #[opencli(...)] attributes
    let (clap_about, opencli_attrs) = parse_variant_attributes(&variant.attrs)?;

    if let Some(ref about) = clap_about
        && summary.is_none()
    {
        command_tokens.extend(quote! {
            .summary(#about)
        });
    }

    // Add operation_id, aliases, tags from opencli attributes
    if let Some(ref operation_id) = opencli_attrs.operation_id {
        command_tokens.extend(quote! {
            .operation_id(#operation_id)
        });
    }

    if !opencli_attrs.aliases.is_empty() {
        let aliases = &opencli_attrs.aliases;
        command_tokens.extend(quote! {
            .aliases(vec![#(#aliases.to_string()),*])
        });
    }

    if !opencli_attrs.tags.is_empty() {
        let tags = &opencli_attrs.tags;
        command_tokens.extend(quote! {
            .tags(vec![#(#tags.to_string()),*])
        });
    }

    command_tokens.extend(quote! { ; });

    // Extract parameters from variant fields
    let parameters = extract_variant_parameters(variant)?;

    // Add parameters if any
    if !parameters.is_empty() {
        command_tokens.extend(quote! {
            let mut parameters = vec![];
            #(#parameters)*
            command = command.parameters(parameters);
        });
    }

    Ok(quote! {
        {
            #command_tokens
            commands.insert(#command_path.to_string(), command);
        }
    })
}

/// Convert PascalCase to kebab-case.
fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Extract doc comment from attributes (/// ...).
fn extract_doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    let mut docs = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc")
            && let Ok(meta) = attr.meta.require_name_value()
            && let syn::Expr::Lit(expr_lit) = &meta.value
            && let syn::Lit::Str(lit_str) = &expr_lit.lit
        {
            docs.push(lit_str.value().trim().to_string());
        }
    }

    if docs.is_empty() {
        None
    } else {
        Some(docs.join(" "))
    }
}

/// Parsed opencli attributes for a variant.
#[derive(Debug, Default)]
struct VariantOpenCliAttrs {
    operation_id: Option<String>,
    aliases: Vec<String>,
    tags: Vec<String>,
}

/// Parse variant-level attributes.
fn parse_variant_attributes(
    attrs: &[syn::Attribute],
) -> Result<(Option<String>, VariantOpenCliAttrs), Diagnostics> {
    let mut about = None;
    let mut opencli_attrs = VariantOpenCliAttrs::default();

    for attr in attrs {
        if attr.path().is_ident("command") {
            // Parse #[command(about = "...")]
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("about") {
                    about = Some(parse_string_value(&meta)?);
                }
                Ok(())
            })
            .map_err(Diagnostics::from_syn_error)?;
        } else if attr.path().is_ident("opencli") {
            // Parse #[opencli(...)]
            attr.parse_nested_meta(|meta| {
                let path = &meta.path;

                if path.is_ident("operation_id") {
                    opencli_attrs.operation_id = Some(parse_string_value(&meta)?);
                } else if path.is_ident("aliases") {
                    opencli_attrs.aliases = parse_string_list(&meta)?;
                } else if path.is_ident("tags") {
                    opencli_attrs.tags = parse_string_list(&meta)?;
                }

                Ok(())
            })
            .map_err(Diagnostics::from_syn_error)?;
        }
    }

    Ok((about, opencli_attrs))
}

/// Parse a string value from a meta attribute.
fn parse_string_value(meta: &syn::meta::ParseNestedMeta) -> syn::Result<String> {
    let value = meta.value()?;
    let lit: syn::Lit = value.parse()?;

    match lit {
        syn::Lit::Str(s) => Ok(s.value()),
        _ => Err(syn::Error::new_spanned(lit, "expected string literal")),
    }
}

/// Parse a list of strings: `("val1", "val2", "val3")`.
fn parse_string_list(meta: &syn::meta::ParseNestedMeta) -> syn::Result<Vec<String>> {
    let content;
    syn::parenthesized!(content in meta.input);

    let mut strings = Vec::new();

    while !content.is_empty() {
        let lit: syn::Lit = content.parse()?;
        if let syn::Lit::Str(s) = lit {
            strings.push(s.value());
        }

        // Parse optional comma
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(strings)
}

/// Extract parameters from variant fields.
fn extract_variant_parameters(variant: &Variant) -> Result<Vec<TokenStream2>, Diagnostics> {
    let mut parameters = Vec::new();

    match &variant.fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                // Parse clap and opencli attributes
                let arg_attrs = ClapArgAttrs::parse(&field.attrs)?;
                let opencli_attrs = OpenCliArgAttrs::parse(&field.attrs)?;

                // Map to parameter
                let param_tokens =
                    mapping::map_arg_to_parameter(field, &arg_attrs, &opencli_attrs)?;

                parameters.push(quote! {
                    parameters.push(#param_tokens);
                });
            }
        }
        Fields::Unnamed(_) => {
            // TODO: Handle tuple variant fields if needed
            // For now, skip unnamed fields
        }
        Fields::Unit => {
            // Unit variants have no parameters
        }
    }

    Ok(parameters)
}
