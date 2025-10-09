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
    subcommand::ResponseDefAttrs,
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
    /// Tags for categorization (tag names only, not full definitions)
    pub tags: Vec<String>,
    /// Tag definitions with descriptions
    pub tag_definitions: Vec<TagDefAttrs>,
    /// External documentation
    pub external_docs: Option<ExternalDocsAttrs>,
    /// Platform definitions
    pub platforms: Vec<PlatformAttrs>,
    /// Environment variable definitions
    pub environment: Vec<EnvironmentAttrs>,
    /// Response definitions for root command
    pub responses: Vec<ResponseDefAttrs>,
    /// Components (schemas, parameters, responses)
    pub components: Option<ComponentsAttrs>,
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
                } else if path.is_ident("tag_definitions") {
                    result.tag_definitions = parse_tag_definitions(&meta)?;
                } else if path.is_ident("external_docs") {
                    result.external_docs = Some(parse_external_docs_attrs(&meta)?);
                } else if path.is_ident("platforms") {
                    result.platforms = parse_platforms(&meta)?;
                } else if path.is_ident("environment") {
                    result.environment = parse_environment(&meta)?;
                } else if path.is_ident("responses") {
                    result.responses = crate::subcommand::parse_responses(&meta)?;
                } else if path.is_ident("components") {
                    result.components = Some(parse_components(&meta)?);
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

/// Parse tag_definitions((name = "core", description = "..."), ...)
fn parse_tag_definitions(meta: &syn::meta::ParseNestedMeta) -> syn::Result<Vec<TagDefAttrs>> {
    let content;
    syn::parenthesized!(content in meta.input);

    let mut tags = Vec::new();

    while !content.is_empty() {
        // Parse each tag definition group
        let tag_content;
        syn::parenthesized!(tag_content in content);

        let mut tag = TagDefAttrs::default();

        while !tag_content.is_empty() {
            let ident: syn::Ident = tag_content.parse()?;
            tag_content.parse::<syn::Token![=]>()?;
            let lit: Lit = tag_content.parse()?;

            if let Lit::Str(s) = lit {
                if ident == "name" {
                    tag.name = s.value();
                } else if ident == "description" {
                    tag.description = Some(s.value());
                }
            }

            // Parse optional comma within tag definition
            if !tag_content.is_empty() {
                tag_content.parse::<syn::Token![,]>()?;
            }
        }

        tags.push(tag);

        // Parse optional comma between tag definitions
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(tags)
}

/// Parse platforms((name = "linux", architectures = ["amd64", "arm64"]), ...)
fn parse_platforms(meta: &syn::meta::ParseNestedMeta) -> syn::Result<Vec<PlatformAttrs>> {
    let content;
    syn::parenthesized!(content in meta.input);

    let mut platforms = Vec::new();

    while !content.is_empty() {
        // Parse each platform definition group
        let platform_content;
        syn::parenthesized!(platform_content in content);

        let mut platform = PlatformAttrs::default();

        while !platform_content.is_empty() {
            let ident: syn::Ident = platform_content.parse()?;
            platform_content.parse::<syn::Token![=]>()?;

            if ident == "name" {
                let lit: Lit = platform_content.parse()?;
                if let Lit::Str(s) = lit {
                    platform.name = s.value();
                }
            } else if ident == "architectures" {
                // Parse array of strings
                let arch_content;
                syn::bracketed!(arch_content in platform_content);

                while !arch_content.is_empty() {
                    let lit: Lit = arch_content.parse()?;
                    if let Lit::Str(s) = lit {
                        platform.architectures.push(s.value());
                    }

                    // Parse optional comma
                    if !arch_content.is_empty() {
                        arch_content.parse::<syn::Token![,]>()?;
                    }
                }
            }

            // Parse optional comma within platform definition
            if !platform_content.is_empty() {
                platform_content.parse::<syn::Token![,]>()?;
            }
        }

        platforms.push(platform);

        // Parse optional comma between platform definitions
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(platforms)
}

/// Parse environment((name = "VAR_NAME", description = "..."), ...)
fn parse_environment(meta: &syn::meta::ParseNestedMeta) -> syn::Result<Vec<EnvironmentAttrs>> {
    let content;
    syn::parenthesized!(content in meta.input);

    let mut env_vars = Vec::new();

    while !content.is_empty() {
        // Parse each environment variable definition group
        let env_content;
        syn::parenthesized!(env_content in content);

        let mut env_var = EnvironmentAttrs::default();

        while !env_content.is_empty() {
            let ident: syn::Ident = env_content.parse()?;
            env_content.parse::<syn::Token![=]>()?;
            let lit: Lit = env_content.parse()?;

            if let Lit::Str(s) = lit {
                if ident == "name" {
                    env_var.name = s.value();
                } else if ident == "description" {
                    env_var.description = Some(s.value());
                }
            }

            // Parse optional comma within env definition
            if !env_content.is_empty() {
                env_content.parse::<syn::Token![,]>()?;
            }
        }

        env_vars.push(env_var);

        // Parse optional comma between env definitions
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(env_vars)
}

/// Parse components(schemas(...), parameters(...), responses(...))
fn parse_components(meta: &syn::meta::ParseNestedMeta) -> syn::Result<ComponentsAttrs> {
    let content;
    syn::parenthesized!(content in meta.input);

    let mut components = ComponentsAttrs::default();

    while !content.is_empty() {
        let ident: syn::Ident = content.parse()?;

        if ident == "schemas" {
            components.schemas = parse_component_schemas(&content)?;
        } else if ident == "parameters" {
            components.parameters = parse_component_parameters(&content)?;
        } else if ident == "responses" {
            components.responses = parse_component_responses(&content)?;
        }

        // Parse optional comma
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(components)
}

/// Parse schemas((name = "Error", schema(...), required(...)), ...)
fn parse_component_schemas(
    input: &syn::parse::ParseBuffer,
) -> syn::Result<Vec<ComponentSchemaAttrs>> {
    let content;
    syn::parenthesized!(content in input);

    let mut schemas = Vec::new();

    while !content.is_empty() {
        let schema_content;
        syn::parenthesized!(schema_content in content);

        let mut name = String::new();
        let mut schema = None;
        let mut required = Vec::new();

        while !schema_content.is_empty() {
            let ident: syn::Ident = schema_content.parse()?;

            if ident == "name" {
                schema_content.parse::<syn::Token![=]>()?;
                let lit: Lit = schema_content.parse()?;
                if let Lit::Str(s) = lit {
                    name = s.value();
                }
            } else if ident == "schema" {
                schema = Some(parse_inline_schema_direct(&schema_content)?);
            } else if ident == "required" {
                required = parse_string_list_direct(&schema_content)?;
            }

            // Parse optional comma
            if !schema_content.is_empty() {
                schema_content.parse::<syn::Token![,]>()?;
            }
        }

        schemas.push(ComponentSchemaAttrs {
            name,
            schema: schema.unwrap_or_default(),
            required,
        });

        // Parse optional comma between schemas
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(schemas)
}

/// Parse parameters((name = "ConfigFile", param_name = "config", ...), ...)
fn parse_component_parameters(
    input: &syn::parse::ParseBuffer,
) -> syn::Result<Vec<ComponentParameterAttrs>> {
    let content;
    syn::parenthesized!(content in input);

    let mut parameters = Vec::new();

    while !content.is_empty() {
        let param_content;
        syn::parenthesized!(param_content in content);

        let mut name = String::new();
        let mut param_name = String::new();
        let mut alias = Vec::new();
        let mut description = None;
        let mut scope = None;
        let mut schema = None;

        while !param_content.is_empty() {
            let ident: syn::Ident = param_content.parse()?;

            if ident == "name" {
                param_content.parse::<syn::Token![=]>()?;
                let lit: Lit = param_content.parse()?;
                if let Lit::Str(s) = lit {
                    name = s.value();
                }
            } else if ident == "param_name" {
                param_content.parse::<syn::Token![=]>()?;
                let lit: Lit = param_content.parse()?;
                if let Lit::Str(s) = lit {
                    param_name = s.value();
                }
            } else if ident == "alias" {
                alias = parse_string_list_direct(&param_content)?;
            } else if ident == "description" {
                param_content.parse::<syn::Token![=]>()?;
                let lit: Lit = param_content.parse()?;
                if let Lit::Str(s) = lit {
                    description = Some(s.value());
                }
            } else if ident == "scope" {
                param_content.parse::<syn::Token![=]>()?;
                let lit: Lit = param_content.parse()?;
                if let Lit::Str(s) = lit {
                    scope = Some(s.value());
                }
            } else if ident == "schema" {
                schema = Some(parse_inline_schema_direct(&param_content)?);
            }

            // Parse optional comma
            if !param_content.is_empty() {
                param_content.parse::<syn::Token![,]>()?;
            }
        }

        parameters.push(ComponentParameterAttrs {
            name,
            param_name,
            alias,
            description,
            scope,
            schema: schema.unwrap_or_default(),
        });

        // Parse optional comma between parameters
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(parameters)
}

/// Parse responses((name = "ValidationSuccess", description = "...", content(...)), ...)
fn parse_component_responses(
    input: &syn::parse::ParseBuffer,
) -> syn::Result<Vec<ComponentResponseAttrs>> {
    let content;
    syn::parenthesized!(content in input);

    let mut responses = Vec::new();

    while !content.is_empty() {
        let resp_content;
        syn::parenthesized!(resp_content in content);

        let mut name = String::new();
        let mut description = String::new();
        let mut response_content = Vec::new();

        while !resp_content.is_empty() {
            let ident: syn::Ident = resp_content.parse()?;

            if ident == "name" {
                resp_content.parse::<syn::Token![=]>()?;
                let lit: Lit = resp_content.parse()?;
                if let Lit::Str(s) = lit {
                    name = s.value();
                }
            } else if ident == "description" {
                resp_content.parse::<syn::Token![=]>()?;
                let lit: Lit = resp_content.parse()?;
                if let Lit::Str(s) = lit {
                    description = s.value();
                }
            } else if ident == "content" {
                response_content = parse_response_content_direct(&resp_content)?;
            }

            // Parse optional comma
            if !resp_content.is_empty() {
                resp_content.parse::<syn::Token![,]>()?;
            }
        }

        responses.push(ComponentResponseAttrs {
            name,
            description,
            content: response_content,
        });

        // Parse optional comma between responses
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(responses)
}

/// Helper to parse inline schema (direct version for use within this module)
fn parse_inline_schema_direct(
    input: &syn::parse::ParseBuffer,
) -> syn::Result<crate::subcommand::SchemaDefAttrs> {
    let content;
    syn::parenthesized!(content in input);

    let mut schema_type = None;
    let mut format = None;
    let mut properties = Vec::new();

    while !content.is_empty() {
        let ident: syn::Ident = content.parse()?;

        if ident == "type" || ident == "r#type" {
            content.parse::<syn::Token![=]>()?;
            let lit: Lit = content.parse()?;
            if let Lit::Str(s) = lit {
                schema_type = Some(s.value());
            }
        } else if ident == "format" {
            content.parse::<syn::Token![=]>()?;
            let lit: Lit = content.parse()?;
            if let Lit::Str(s) = lit {
                format = Some(s.value());
            }
        } else if ident == "properties" {
            properties = parse_schema_properties_direct(&content)?;
        } else if ident == "enum" || ident == "r#enum" {
            // Skip enum for now - just parse and discard the content
            let _enum_content;
            syn::parenthesized!(_enum_content in content);
        }

        // Parse optional comma
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(crate::subcommand::SchemaDefAttrs {
        schema_type,
        format,
        properties,
    })
}

/// Helper to parse schema properties (direct version)
fn parse_schema_properties_direct(
    input: &syn::parse::ParseBuffer,
) -> syn::Result<Vec<crate::subcommand::PropertyDefAttrs>> {
    let content;
    syn::parenthesized!(content in input);

    let mut properties = Vec::new();

    while !content.is_empty() {
        let prop_content;
        syn::parenthesized!(prop_content in content);

        let mut name = String::new();
        let mut schema_type = String::new();
        let mut format = None;
        let mut items_ref = None;

        while !prop_content.is_empty() {
            let ident: syn::Ident = prop_content.parse()?;
            prop_content.parse::<syn::Token![=]>()?;
            let lit: Lit = prop_content.parse()?;

            if let Lit::Str(s) = lit {
                if ident == "name" {
                    name = s.value();
                } else if ident == "type" || ident == "r#type" {
                    schema_type = s.value();
                } else if ident == "format" {
                    format = Some(s.value());
                } else if ident == "items_ref" {
                    items_ref = Some(s.value());
                }
            }

            // Parse optional comma
            if !prop_content.is_empty() {
                prop_content.parse::<syn::Token![,]>()?;
            }
        }

        properties.push(crate::subcommand::PropertyDefAttrs {
            name,
            schema_type,
            format,
            items_ref,
        });

        // Parse optional comma between properties
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(properties)
}

/// Helper to parse response content (direct version)
fn parse_response_content_direct(
    input: &syn::parse::ParseBuffer,
) -> syn::Result<Vec<crate::subcommand::ResponseContentAttrs>> {
    let content;
    syn::parenthesized!(content in input);

    let mut content_list = Vec::new();

    while !content.is_empty() {
        let content_inner;
        syn::parenthesized!(content_inner in content);

        let mut media_type = String::new();
        let mut schema = None;
        let mut schema_ref = None;
        let mut example = None;

        while !content_inner.is_empty() {
            let ident: syn::Ident = content_inner.parse()?;

            if ident == "media_type" {
                content_inner.parse::<syn::Token![=]>()?;
                let lit: Lit = content_inner.parse()?;
                if let Lit::Str(s) = lit {
                    media_type = s.value();
                }
            } else if ident == "schema" {
                schema = Some(parse_inline_schema_direct(&content_inner)?);
            } else if ident == "schema_ref" {
                content_inner.parse::<syn::Token![=]>()?;
                let lit: Lit = content_inner.parse()?;
                if let Lit::Str(s) = lit {
                    schema_ref = Some(s.value());
                }
            } else if ident == "example" {
                content_inner.parse::<syn::Token![=]>()?;
                let lit: Lit = content_inner.parse()?;
                if let Lit::Str(s) = lit {
                    example = Some(serde_json::Value::String(s.value()));
                }
            }

            // Parse optional comma
            if !content_inner.is_empty() {
                content_inner.parse::<syn::Token![,]>()?;
            }
        }

        content_list.push(crate::subcommand::ResponseContentAttrs {
            media_type,
            schema,
            schema_ref,
            example,
        });

        // Parse optional comma between content definitions
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(content_list)
}

/// Helper to parse string list (direct version)
fn parse_string_list_direct(input: &syn::parse::ParseBuffer) -> syn::Result<Vec<String>> {
    let content;
    syn::parenthesized!(content in input);

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

/// Tag definition attributes from `#[opencli(tag_definitions(...))]`.
#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct TagDefAttrs {
    /// Tag name
    pub name: String,
    /// Tag description
    pub description: Option<String>,
}

/// Platform attributes from `#[opencli(platforms(...))]`.
#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct PlatformAttrs {
    /// Platform name (e.g., "linux", "darwin", "windows")
    pub name: String,
    /// Supported architectures (e.g., ["amd64", "arm64"])
    pub architectures: Vec<String>,
}

/// Environment variable attributes from `#[opencli(environment(...))]`.
#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct EnvironmentAttrs {
    /// Environment variable name
    pub name: String,
    /// Environment variable description
    pub description: Option<String>,
}

/// Components attributes from `#[opencli(components(...))]`.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct ComponentsAttrs {
    /// Schema definitions
    pub schemas: Vec<ComponentSchemaAttrs>,
    /// Parameter definitions
    pub parameters: Vec<ComponentParameterAttrs>,
    /// Response definitions
    pub responses: Vec<ComponentResponseAttrs>,
}

/// Component schema attributes
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ComponentSchemaAttrs {
    pub name: String,
    pub schema: crate::subcommand::SchemaDefAttrs,
    pub required: Vec<String>,
}

/// Component parameter attributes
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ComponentParameterAttrs {
    pub name: String,
    pub param_name: String,
    pub alias: Vec<String>,
    pub description: Option<String>,
    pub scope: Option<String>,
    pub schema: crate::subcommand::SchemaDefAttrs,
}

/// Component response attributes
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ComponentResponseAttrs {
    pub name: String,
    pub description: String,
    pub content: Vec<crate::subcommand::ResponseContentAttrs>,
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

    // Build OpenCli struct with optional fields
    let mut opencli_builder = quote! {
        ::utocli::OpenCli::new(info).commands(commands)
    };

    // Add external_docs if present
    if let Some(ref external_docs) = opencli_attrs.external_docs {
        let url = &external_docs.url;
        if let Some(ref description) = external_docs.description {
            opencli_builder = quote! {
                #opencli_builder.external_docs(
                    ::utocli::ExternalDocs::new(#url)
                        .description(#description)
                )
            };
        } else {
            opencli_builder = quote! {
                #opencli_builder.external_docs(
                    ::utocli::ExternalDocs::new(#url)
                )
            };
        }
    }

    // Add tag_definitions if present (prefer tag_definitions over tags)
    if !opencli_attrs.tag_definitions.is_empty() {
        let tag_defs = &opencli_attrs.tag_definitions;
        let tag_tokens: Vec<_> = tag_defs
            .iter()
            .map(|tag_def| {
                let name = &tag_def.name;
                if let Some(ref desc) = tag_def.description {
                    quote! {
                        ::utocli::Tag::new(#name).description(#desc)
                    }
                } else {
                    quote! {
                        ::utocli::Tag::new(#name)
                    }
                }
            })
            .collect();

        opencli_builder = quote! {
            #opencli_builder.tags(vec![
                #(#tag_tokens),*
            ])
        };
    } else if !opencli_attrs.tags.is_empty() {
        // Fall back to simple tags if no tag_definitions
        let tags = &opencli_attrs.tags;
        opencli_builder = quote! {
            #opencli_builder.tags(vec![
                #(::utocli::Tag::new(#tags)),*
            ])
        };
    }

    // Add platforms if present
    if !opencli_attrs.platforms.is_empty() {
        let platforms = &opencli_attrs.platforms;
        let platform_tokens: Vec<_> = platforms
            .iter()
            .map(|platform| {
                let name_str = &platform.name;
                let platform_name = match name_str.to_lowercase().as_str() {
                    "linux" => quote! { ::utocli::PlatformName::Linux },
                    "darwin" => quote! { ::utocli::PlatformName::Darwin },
                    "macos" => quote! { ::utocli::PlatformName::Macos },
                    "windows" => quote! { ::utocli::PlatformName::Windows },
                    "ios" => quote! { ::utocli::PlatformName::Ios },
                    "android" => quote! { ::utocli::PlatformName::Android },
                    "freebsd" => quote! { ::utocli::PlatformName::Freebsd },
                    _ => quote! { ::utocli::PlatformName::Linux }, // Default to Linux
                };

                let arch_tokens: Vec<_> = platform
                    .architectures
                    .iter()
                    .map(|arch_str| {
                        match arch_str.to_lowercase().as_str() {
                            "amd64" | "x86_64" => quote! { ::utocli::Architecture::Amd64 },
                            "arm64" | "aarch64" => quote! { ::utocli::Architecture::Arm64 },
                            "386" => quote! { ::utocli::Architecture::I386 },
                            "x86" => quote! { ::utocli::Architecture::X86 },
                            "arm" => quote! { ::utocli::Architecture::Arm },
                            _ => quote! { ::utocli::Architecture::Amd64 }, // Default to Amd64
                        }
                    })
                    .collect();

                quote! {
                    ::utocli::Platform::new(#platform_name).architectures(vec![#(#arch_tokens),*])
                }
            })
            .collect();

        opencli_builder = quote! {
            #opencli_builder.platforms(vec![
                #(#platform_tokens),*
            ])
        };
    }

    // Add environment if present
    if !opencli_attrs.environment.is_empty() {
        let env_vars = &opencli_attrs.environment;
        let env_tokens: Vec<_> = env_vars
            .iter()
            .map(|env_var| {
                let name = &env_var.name;
                if let Some(ref desc) = env_var.description {
                    quote! {
                        ::utocli::EnvironmentVariable::new(#name).description(#desc)
                    }
                } else {
                    quote! {
                        ::utocli::EnvironmentVariable::new(#name)
                    }
                }
            })
            .collect();

        opencli_builder = quote! {
            #opencli_builder.environment(vec![
                #(#env_tokens),*
            ])
        };
    }

    // Add components if present
    if let Some(ref components) = opencli_attrs.components {
        let mut has_components = false;
        let mut components_builder = quote! {
            let mut components = ::utocli::Components::new();
        };

        // Add schemas
        if !components.schemas.is_empty() {
            has_components = true;
            let schema_inserts: Vec<_> = components
                .schemas
                .iter()
                .map(|schema| {
                    let name = &schema.name;
                    let schema_tokens =
                        crate::subcommand::build_inline_schema_tokens(&schema.schema);

                    // Add required fields if present
                    if schema.required.is_empty() {
                        quote! {
                            schemas.insert(#name.to_string(), ::utocli::RefOr::T(#schema_tokens));
                        }
                    } else {
                        let required_fields = &schema.required;
                        quote! {
                            {
                                let mut schema = #schema_tokens;
                                if let ::utocli::Schema::Object(ref mut obj) = schema {
                                    obj.required = Some(vec![#(#required_fields.to_string()),*]);
                                }
                                schemas.insert(#name.to_string(), ::utocli::RefOr::T(schema));
                            }
                        }
                    }
                })
                .collect();

            components_builder.extend(quote! {
                let mut schemas = ::utocli::Map::new();
                #(#schema_inserts)*
                components.schemas = Some(schemas);
            });
        }

        // Add parameters
        if !components.parameters.is_empty() {
            has_components = true;
            let param_inserts: Vec<_> = components
                .parameters
                .iter()
                .map(|param| {
                    let name = &param.name;
                    let param_name = &param.param_name;
                    let schema_tokens =
                        crate::subcommand::build_inline_schema_tokens(&param.schema);

                    let mut param_tokens = quote! {
                        ::utocli::Parameter::new(#param_name)
                            .schema(#schema_tokens)
                    };

                    // Add aliases
                    if !param.alias.is_empty() {
                        let aliases = &param.alias;
                        param_tokens.extend(quote! {
                            .alias(vec![#(#aliases.to_string()),*])
                        });
                    }

                    // Add description
                    if let Some(ref desc) = param.description {
                        param_tokens.extend(quote! {
                            .description(#desc)
                        });
                    }

                    // Add scope
                    if let Some(ref scope_str) = param.scope {
                        let scope_variant = match scope_str.as_str() {
                            "local" => quote! { ::utocli::ParameterScope::Local },
                            "inherited" => quote! { ::utocli::ParameterScope::Inherited },
                            _ => quote! { ::utocli::ParameterScope::Local },
                        };
                        param_tokens.extend(quote! {
                            .scope(#scope_variant)
                        });
                    }

                    quote! {
                        parameters.insert(#name.to_string(), ::utocli::RefOr::T(#param_tokens));
                    }
                })
                .collect();

            components_builder.extend(quote! {
                let mut parameters = ::utocli::Map::new();
                #(#param_inserts)*
                components.parameters = Some(parameters);
            });
        }

        // Add responses
        if !components.responses.is_empty() {
            has_components = true;
            let response_inserts: Vec<_> = components.responses.iter().map(|resp| {
                let name = &resp.name;
                let desc = &resp.description;

                if resp.content.is_empty() {
                    quote! {
                        responses.insert(#name.to_string(), ::utocli::RefOr::T(::utocli::Response::new().description(#desc)));
                    }
                } else {
                    let content_inserts: Vec<_> = resp.content.iter().map(|content_item| {
                        let media_type = &content_item.media_type;

                        let mut mt_tokens = quote! {
                            ::utocli::MediaType::new()
                        };

                        // Add schema (either inline or ref)
                        if let Some(ref schema_ref) = content_item.schema_ref {
                            mt_tokens.extend(quote! {
                                .schema(::utocli::RefOr::new_ref(#schema_ref))
                            });
                        } else if let Some(ref schema_def) = content_item.schema {
                            let schema_tokens = crate::subcommand::build_inline_schema_tokens(schema_def);
                            mt_tokens.extend(quote! {
                                .schema(::utocli::RefOr::T(#schema_tokens))
                            });
                        }

                        // Add example if present
                        if let Some(ref example) = content_item.example {
                            let example_json = serde_json::to_string(example).unwrap();
                            mt_tokens.extend(quote! {
                                .example(serde_json::from_str(#example_json).unwrap())
                            });
                        }

                        quote! {
                            content.insert(#media_type.to_string(), #mt_tokens);
                        }
                    }).collect();

                    quote! {
                        {
                            let mut content = ::utocli::Map::new();
                            #(#content_inserts)*
                            responses.insert(#name.to_string(),
                                ::utocli::RefOr::T(::utocli::Response::new()
                                    .description(#desc)
                                    .content(content))
                            );
                        }
                    }
                }
            }).collect();

            components_builder.extend(quote! {
                let mut responses = ::utocli::Map::new();
                #(#response_inserts)*
                components.responses = Some(responses);
            });
        }

        if has_components {
            opencli_builder = quote! {
                {
                    #components_builder
                    #opencli_builder.components(components)
                }
            };
        }
    }

    let name = &input.ident;
    let impl_block = quote! {
        impl #name {
            /// Generate an OpenCLI specification for this Parser struct.
            pub fn opencli() -> ::utocli::OpenCli {
                #info

                let commands = #root_command;

                #opencli_builder
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
    opencli_attrs: &OpenCliParserAttrs,
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

    // Add operation_id from opencli attributes
    if let Some(ref operation_id) = opencli_attrs.operation_id {
        command_tokens.extend(quote! {
            .operation_id(#operation_id)
        });
    }

    // Add aliases from opencli attributes
    if !opencli_attrs.aliases.is_empty() {
        let aliases = &opencli_attrs.aliases;
        command_tokens.extend(quote! {
            .aliases(vec![#(#aliases.to_string()),*])
        });
    }

    // Add tags from opencli attributes
    if !opencli_attrs.tags.is_empty() {
        let tags = &opencli_attrs.tags;
        command_tokens.extend(quote! {
            .tags(vec![#(#tags.to_string()),*])
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

    // Add responses if present
    if !opencli_attrs.responses.is_empty() {
        let response_inserts: Vec<_> = opencli_attrs
            .responses
            .iter()
            .map(|resp| {
                let status = &resp.status;
                let desc = &resp.description;

                // Build response with content if present
                if resp.content.is_empty() {
                    quote! {
                        responses.insert(#status.to_string(), ::utocli::Response::new().description(#desc));
                    }
                } else {
                    // Generate content map
                    let content_inserts: Vec<_> = resp.content.iter().map(|content_item| {
                        let media_type = &content_item.media_type;

                        // Build media type content
                        let mut mt_tokens = quote! {
                            ::utocli::MediaType::new()
                        };

                        // Add schema (either inline or ref)
                        if let Some(ref schema_ref) = content_item.schema_ref {
                            mt_tokens.extend(quote! {
                                .schema(::utocli::RefOr::new_ref(#schema_ref))
                            });
                        } else if let Some(ref schema_def) = content_item.schema {
                            // Build inline schema
                            let schema_tokens = crate::subcommand::build_inline_schema_tokens(schema_def);
                            mt_tokens.extend(quote! {
                                .schema(::utocli::RefOr::T(#schema_tokens))
                            });
                        }

                        // Add example if present
                        if let Some(ref example) = content_item.example {
                            let example_json = serde_json::to_string(example).unwrap();
                            mt_tokens.extend(quote! {
                                .example(serde_json::from_str(#example_json).unwrap())
                            });
                        }

                        quote! {
                            content.insert(#media_type.to_string(), #mt_tokens);
                        }
                    }).collect();

                    quote! {
                        {
                            let mut content = ::utocli::Map::new();
                            #(#content_inserts)*
                            responses.insert(#status.to_string(),
                                ::utocli::Response::new()
                                    .description(#desc)
                                    .content(content)
                            );
                        }
                    }
                }
            })
            .collect();

        command_tokens.extend(quote! {
            let mut responses = ::utocli::Map::new();
            #(#response_inserts)*
            command = command.responses(responses);
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

        // Map to parameter (use root command variant)
        let param_tokens =
            mapping::map_arg_to_parameter_for_root(field, &arg_attrs, &opencli_attrs)?;

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
