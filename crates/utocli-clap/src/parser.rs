//! Parsing for clap's `#[derive(Parser)]` structs.
//!
//! This module handles extraction of command metadata from clap's Parser derive
//! and generates OpenCLI specifications from those types.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Data, DataStruct, DeriveInput, Fields, Lit};

use crate::{
    arg::{ClapArgAttrs, OpenCliArgAttrs},
    diagnostics::Diagnostics,
    mapping,
};

/// Parsed clap `#[command(...)]` attribute values from Parser structs.
#[allow(dead_code)] // Will be used in Phase 2
#[derive(Debug, Default)]
pub struct ClapParserAttrs {
    /// Command name
    pub name: Option<String>,
    /// Version string
    pub version: Option<String>,
    /// Short description (about)
    pub about: Option<String>,
    /// Long description (long_about)
    pub long_about: Option<String>,
    /// Author
    pub author: Option<String>,
}

impl ClapParserAttrs {
    /// Parse `#[command(...)]` attributes from a list of attributes.
    pub fn parse(attrs: &[Attribute]) -> Result<Self, Diagnostics> {
        let mut result = Self::default();

        for attr in attrs {
            if !attr.path().is_ident("command") {
                continue;
            }

            // Parse the command attribute using parse_nested_meta
            attr.parse_nested_meta(|meta| {
                let path = &meta.path;

                if path.is_ident("name") {
                    result.name = Some(parse_string_value(&meta)?);
                } else if path.is_ident("version") {
                    result.version = Some(parse_string_value(&meta)?);
                } else if path.is_ident("about") {
                    result.about = Some(parse_string_value(&meta)?);
                } else if path.is_ident("long_about") {
                    result.long_about = Some(parse_string_value(&meta)?);
                } else if path.is_ident("author") {
                    result.author = Some(parse_string_value(&meta)?);
                }
                // Ignore other command attributes for now

                Ok(())
            })
            .map_err(Diagnostics::from_syn_error)?;
        }

        Ok(result)
    }
}

/// Parse a string value from a meta attribute (e.g., `name = "value"`).
fn parse_string_value(meta: &syn::meta::ParseNestedMeta) -> syn::Result<String> {
    let value = meta.value()?;
    let lit: Lit = value.parse()?;

    match lit {
        Lit::Str(s) => Ok(s.value()),
        _ => Err(syn::Error::new_spanned(lit, "expected string literal")),
    }
}

/// Parsed `#[opencli(...)]` attribute values for Parser-level augmentation.
#[allow(dead_code)] // Will be used in Phase 2
#[derive(Debug, Default)]
pub struct OpenCliParserAttrs {
    /// Info section metadata
    pub info: Option<InfoAttrs>,
    /// Operation ID
    pub operation_id: Option<String>,
    /// Command aliases
    pub aliases: Vec<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// External documentation
    pub external_docs: Option<ExternalDocsAttrs>,
}

impl OpenCliParserAttrs {
    /// Parse `#[opencli(...)]` attributes from a list of attributes.
    pub fn parse(attrs: &[Attribute]) -> Result<Self, Diagnostics> {
        let mut result = Self::default();

        for attr in attrs {
            if !attr.path().is_ident("opencli") {
                continue;
            }

            // Parse the opencli attribute using parse_nested_meta
            attr.parse_nested_meta(|meta| {
                let path = &meta.path;

                if path.is_ident("info") {
                    result.info = Some(parse_info_attrs(&meta)?);
                } else if path.is_ident("operation_id") {
                    result.operation_id = Some(parse_string_value(&meta)?);
                } else if path.is_ident("aliases") {
                    result.aliases = parse_string_list(&meta)?;
                } else if path.is_ident("tags") {
                    result.tags = parse_string_list(&meta)?;
                } else if path.is_ident("external_docs") {
                    result.external_docs = Some(parse_external_docs_attrs(&meta)?);
                }

                Ok(())
            })
            .map_err(Diagnostics::from_syn_error)?;
        }

        Ok(result)
    }
}

/// Parse `info(...)` nested attributes.
fn parse_info_attrs(meta: &syn::meta::ParseNestedMeta) -> syn::Result<InfoAttrs> {
    let content;
    syn::parenthesized!(content in meta.input);

    let mut info = InfoAttrs::default();

    while !content.is_empty() {
        let ident: syn::Ident = content.parse()?;

        if ident == "title" {
            content.parse::<syn::Token![=]>()?;
            let lit: Lit = content.parse()?;
            if let Lit::Str(s) = lit {
                info.title = Some(s.value());
            }
        } else if ident == "description" {
            content.parse::<syn::Token![=]>()?;
            let lit: Lit = content.parse()?;
            if let Lit::Str(s) = lit {
                info.description = Some(s.value());
            }
        } else if ident == "contact" {
            info.contact = Some(parse_contact_attrs_inline(&content)?);
        } else if ident == "license" {
            info.license = Some(parse_license_attrs_inline(&content)?);
        }

        // Parse optional comma
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(info)
}

/// Parse `contact(...)` nested attributes inline.
fn parse_contact_attrs_inline(content: &syn::parse::ParseBuffer) -> syn::Result<ContactAttrs> {
    let inner;
    syn::parenthesized!(inner in content);

    let mut contact = ContactAttrs::default();

    while !inner.is_empty() {
        let ident: syn::Ident = inner.parse()?;
        inner.parse::<syn::Token![=]>()?;
        let lit: Lit = inner.parse()?;

        if let Lit::Str(s) = lit {
            if ident == "name" {
                contact.name = Some(s.value());
            } else if ident == "url" {
                contact.url = Some(s.value());
            } else if ident == "email" {
                contact.email = Some(s.value());
            }
        }

        // Parse optional comma
        if !inner.is_empty() {
            inner.parse::<syn::Token![,]>()?;
        }
    }

    Ok(contact)
}

/// Parse `license(...)` nested attributes inline.
fn parse_license_attrs_inline(content: &syn::parse::ParseBuffer) -> syn::Result<LicenseAttrs> {
    let inner;
    syn::parenthesized!(inner in content);

    let mut license = LicenseAttrs::default();

    while !inner.is_empty() {
        let ident: syn::Ident = inner.parse()?;
        inner.parse::<syn::Token![=]>()?;
        let lit: Lit = inner.parse()?;

        if let Lit::Str(s) = lit {
            if ident == "name" {
                license.name = s.value();
            } else if ident == "url" {
                license.url = Some(s.value());
            }
        }

        // Parse optional comma
        if !inner.is_empty() {
            inner.parse::<syn::Token![,]>()?;
        }
    }

    Ok(license)
}

/// Parse `external_docs(...)` nested attributes.
fn parse_external_docs_attrs(meta: &syn::meta::ParseNestedMeta) -> syn::Result<ExternalDocsAttrs> {
    let content;
    syn::parenthesized!(content in meta.input);

    let mut external_docs = ExternalDocsAttrs::default();

    while !content.is_empty() {
        let ident: syn::Ident = content.parse()?;
        content.parse::<syn::Token![=]>()?;
        let lit: Lit = content.parse()?;

        if let Lit::Str(s) = lit {
            if ident == "description" {
                external_docs.description = Some(s.value());
            } else if ident == "url" {
                external_docs.url = s.value();
            }
        }

        // Parse optional comma
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(external_docs)
}

/// Parse a list of strings: `("val1", "val2", "val3")`.
fn parse_string_list(meta: &syn::meta::ParseNestedMeta) -> syn::Result<Vec<String>> {
    let content;
    syn::parenthesized!(content in meta.input);

    let mut strings = Vec::new();

    while !content.is_empty() {
        let lit: Lit = content.parse()?;
        if let Lit::Str(s) = lit {
            strings.push(s.value());
        }

        // Parse optional comma
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(strings)
}

/// Info section attributes from `#[opencli(info(...))]`.
#[allow(dead_code)] // Will be used in Phase 2
#[derive(Debug, Default)]
pub struct InfoAttrs {
    /// Application title (required)
    pub title: Option<String>,
    /// Application description
    pub description: Option<String>,
    /// Contact information
    pub contact: Option<ContactAttrs>,
    /// License information
    pub license: Option<LicenseAttrs>,
}

/// Contact attributes from `#[opencli(info(contact(...)))]`.
#[allow(dead_code)] // Will be used in Phase 2
#[derive(Debug, Default)]
pub struct ContactAttrs {
    /// Contact name
    pub name: Option<String>,
    /// Contact URL
    pub url: Option<String>,
    /// Contact email
    pub email: Option<String>,
}

/// License attributes from `#[opencli(info(license(...)))]`.
#[allow(dead_code)] // Will be used in Phase 2
#[derive(Debug, Default)]
pub struct LicenseAttrs {
    /// License name
    pub name: String,
    /// License URL
    pub url: Option<String>,
}

/// External docs attributes from `#[opencli(external_docs(...))]`.
#[allow(dead_code)] // Will be used in Phase 2
#[derive(Debug, Default)]
pub struct ExternalDocsAttrs {
    /// Description of external docs
    pub description: Option<String>,
    /// URL to external docs (required)
    pub url: String,
}

/// Generate OpenCli implementation from a clap Parser struct.
pub fn generate_opencli(input: &DeriveInput) -> Result<TokenStream2, Diagnostics> {
    // Parse clap and opencli attributes
    let clap_attrs = ClapParserAttrs::parse(&input.attrs)?;
    let opencli_attrs = OpenCliParserAttrs::parse(&input.attrs)?;

    // Generate Info
    let info = generate_info(&clap_attrs, &opencli_attrs)?;

    // Generate root command (returns a BTreeMap)
    let root_command = generate_root_command(input, &clap_attrs, &opencli_attrs)?;

    // Build OpenCli struct
    let name = &input.ident;
    let impl_block = quote! {
        impl #name {
            /// Generate an OpenCLI specification for this Parser struct.
            pub fn opencli() -> ::utocli::OpenCli {
                #info

                let commands = #root_command;

                ::utocli::OpenCli::new(info).commands(commands)
            }
        }
    };

    Ok(impl_block)
}

/// Generate the Info struct from clap and opencli attributes.
fn generate_info(
    clap_attrs: &ClapParserAttrs,
    opencli_attrs: &OpenCliParserAttrs,
) -> Result<TokenStream2, Diagnostics> {
    // Determine title: use opencli.info.title, fallback to clap.name or empty
    let title = opencli_attrs
        .info
        .as_ref()
        .and_then(|info| info.title.as_ref())
        .or(clap_attrs.name.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");

    // Determine version: use clap.version or "0.0.0"
    let version = clap_attrs.version.as_deref().unwrap_or("0.0.0");

    // Generate Info builder
    let mut info_tokens = quote! {
        let info = ::utocli::Info::new(#title, #version)
    };

    // Add description if available
    if let Some(ref info_attrs) = opencli_attrs.info {
        if let Some(ref desc) = info_attrs.description {
            info_tokens.extend(quote! {
                .description(#desc)
            });
        }

        // Add contact if available
        if let Some(ref contact) = info_attrs.contact {
            let mut contact_tokens = quote! {
                ::utocli::Contact::new()
            };

            if let Some(ref name) = contact.name {
                contact_tokens.extend(quote! {
                    .name(#name)
                });
            }
            if let Some(ref url) = contact.url {
                contact_tokens.extend(quote! {
                    .url(#url)
                });
            }
            if let Some(ref email) = contact.email {
                contact_tokens.extend(quote! {
                    .email(#email)
                });
            }

            info_tokens.extend(quote! {
                .contact(#contact_tokens)
            });
        }

        // Add license if available
        if let Some(ref license) = info_attrs.license {
            let name = &license.name;
            let mut license_tokens = quote! {
                ::utocli::License::new(#name)
            };

            if let Some(ref url) = license.url {
                license_tokens.extend(quote! {
                    .url(#url)
                });
            }

            info_tokens.extend(quote! {
                .license(#license_tokens)
            });
        }
    } else {
        // Use clap.about as description fallback
        if let Some(ref about) = clap_attrs.about {
            info_tokens.extend(quote! {
                .description(#about)
            });
        } else if let Some(ref long_about) = clap_attrs.long_about {
            info_tokens.extend(quote! {
                .description(#long_about)
            });
        }
    }

    info_tokens.extend(quote! { ; });

    Ok(info_tokens)
}

/// Generate the root command from the Parser struct.
fn generate_root_command(
    input: &DeriveInput,
    clap_attrs: &ClapParserAttrs,
    _opencli_attrs: &OpenCliParserAttrs,
) -> Result<TokenStream2, Diagnostics> {
    // Get command name
    let command_name = clap_attrs.name.as_ref().ok_or_else(|| {
        Diagnostics::new("Parser must have a name (use #[command(name = \"...\")])")
    })?;

    // Build Command struct
    let mut command_tokens = quote! {
        let mut command = ::utocli::Command::new()
    };

    // Add summary from about
    if let Some(ref about) = clap_attrs.about {
        command_tokens.extend(quote! {
            .summary(#about)
        });
    }

    // Add description from long_about
    if let Some(ref long_about) = clap_attrs.long_about {
        command_tokens.extend(quote! {
            .description(#long_about)
        });
    }

    command_tokens.extend(quote! { ; });

    // Extract parameters and subcommand type from fields
    let (parameters, subcommand_type) = extract_parameters_and_subcommand(input)?;

    // Add parameters if any
    if !parameters.is_empty() {
        command_tokens.extend(quote! {
            let mut parameters = vec![];
            #(#parameters)*
            command = command.parameters(parameters);
        });
    }

    // Build the final return expression - always return a BTreeMap
    if let Some(subcommand_ty) = subcommand_type {
        Ok(quote! {
            {
                #command_tokens

                // Merge subcommands from CommandCollection impl
                let subcommands = <#subcommand_ty as ::utocli::CommandCollection>::commands();
                let mut commands_map = ::utocli::Map::new();
                commands_map.insert(#command_name.to_string(), command);
                commands_map.extend(subcommands);

                commands_map
            }
        })
    } else {
        Ok(quote! {
            {
                #command_tokens

                let mut commands_map = ::utocli::Map::new();
                commands_map.insert(#command_name.to_string(), command);
                commands_map
            }
        })
    }
}

/// Extract parameters and subcommand type from struct fields.
///
/// Returns (parameters, subcommand_type).
fn extract_parameters_and_subcommand(
    input: &DeriveInput,
) -> Result<(Vec<TokenStream2>, Option<syn::Type>), Diagnostics> {
    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields,
        _ => {
            return Err(Diagnostics::new(
                "Parser must be a struct with named fields",
            ));
        }
    };

    let mut parameters = Vec::new();
    let mut subcommand_type = None;

    for field in &fields.named {
        // Check if this is a subcommand field
        if is_subcommand_field(field)? {
            // Extract the subcommand type (unwrap Option if present)
            let ty = &field.ty;
            subcommand_type = Some(unwrap_option_or_clone(ty));
            continue;
        }

        // Parse clap and opencli attributes
        let arg_attrs = ClapArgAttrs::parse(&field.attrs)?;
        let opencli_attrs = OpenCliArgAttrs::parse(&field.attrs)?;

        // Map to parameter
        let param_tokens = mapping::map_arg_to_parameter(field, &arg_attrs, &opencli_attrs)?;

        parameters.push(quote! {
            parameters.push(#param_tokens);
        });
    }

    Ok((parameters, subcommand_type))
}

/// Unwrap Option<T> to get T, or clone the type if not Option.
fn unwrap_option_or_clone(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return inner_ty.clone();
    }
    ty.clone()
}

/// Check if a field is a subcommand field.
fn is_subcommand_field(field: &syn::Field) -> Result<bool, Diagnostics> {
    for attr in &field.attrs {
        if attr.path().is_ident("command") {
            // Check if it's #[command(subcommand)]
            let mut found_subcommand = false;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("subcommand") {
                    found_subcommand = true;
                }
                Ok(())
            })
            .map_err(Diagnostics::from_syn_error)?;

            if found_subcommand {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
