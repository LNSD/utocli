//! Parsing for clap's `#[derive(Subcommand)]` enums.
//!
//! This module handles extraction of subcommand definitions from clap's Subcommand derive
//! and generates the CommandCollection implementation.

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
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

    // Check for #[command(about = "...", long_about = "...")] or #[opencli(...)] attributes
    let (clap_about, clap_long_about, opencli_attrs) = parse_variant_attributes(&variant.attrs)?;

    // Use clap about as summary if no doc comment
    if let Some(ref about) = clap_about
        && summary.is_none()
    {
        command_tokens.extend(quote! {
            .summary(#about)
        });
    }

    // Add description from opencli attribute, clap long_about, or fall back to doc comments
    let description = opencli_attrs
        .description
        .as_ref()
        .or(clap_long_about.as_ref());

    if let Some(ref desc) = description {
        command_tokens.extend(quote! {
            .description(#desc)
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
                            let schema_tokens = build_inline_schema_tokens(schema_def);
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

    // Add extensions if present
    if !opencli_attrs.extensions.is_empty() {
        let ext_inserts: Vec<_> = opencli_attrs
            .extensions
            .iter()
            .map(|(key, value)| {
                quote! {
                    extensions.insert(#key.to_string(), serde_json::Value::String(#value.to_string()));
                }
            })
            .collect();

        command_tokens.extend(quote! {
            let mut extensions = ::utocli::Extensions::new();
            #(#ext_inserts)*
            command = command.extensions(extensions);
        });
    }

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
    description: Option<String>,
    aliases: Vec<String>,
    tags: Vec<String>,
    responses: Vec<ResponseDefAttrs>,
    extensions: Vec<(String, String)>, // (key, value) pairs for x-* extensions
}

/// Response definition attributes
#[derive(Debug, Clone)]
pub(crate) struct ResponseDefAttrs {
    pub(crate) status: String,
    pub(crate) description: String,
    pub(crate) content: Vec<ResponseContentAttrs>,
}

/// Response content for different media types
#[derive(Debug, Clone)]
pub(crate) struct ResponseContentAttrs {
    pub(crate) media_type: String,
    pub(crate) schema: Option<SchemaDefAttrs>, // Inline schema definition
    pub(crate) schema_ref: Option<String>,     // Reference like "#/components/schemas/Error"
    pub(crate) example: Option<serde_json::Value>, // Example value (string or JSON)
}

/// Schema definition attributes for inline schemas
#[derive(Debug, Clone, Default)]
pub(crate) struct SchemaDefAttrs {
    pub(crate) schema_type: Option<String>, // "object", "string", "integer", etc.
    pub(crate) format: Option<String>,      // "int32", "int64", etc.
    pub(crate) properties: Vec<PropertyDefAttrs>, // Object properties
}

/// Property definition for object schemas
#[derive(Debug, Clone)]
pub(crate) struct PropertyDefAttrs {
    pub(crate) name: String,
    pub(crate) schema_type: String,
    pub(crate) format: Option<String>,
    pub(crate) items_ref: Option<String>, // For array types: items reference
}

/// Parse variant-level attributes.
fn parse_variant_attributes(
    attrs: &[syn::Attribute],
) -> Result<(Option<String>, Option<String>, VariantOpenCliAttrs), Diagnostics> {
    let mut about = None;
    let mut long_about = None;
    let mut opencli_attrs = VariantOpenCliAttrs::default();

    for attr in attrs {
        if attr.path().is_ident("command") {
            // Parse #[command(about = "...", long_about = "...")]
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("about") {
                    about = Some(parse_string_value(&meta)?);
                } else if meta.path.is_ident("long_about") {
                    long_about = Some(parse_string_value(&meta)?);
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
                } else if path.is_ident("description") {
                    opencli_attrs.description = Some(parse_string_value(&meta)?);
                } else if path.is_ident("aliases") {
                    opencli_attrs.aliases = parse_string_list(&meta)?;
                } else if path.is_ident("tags") {
                    opencli_attrs.tags = parse_string_list(&meta)?;
                } else if path.is_ident("responses") {
                    opencli_attrs.responses = parse_responses(&meta)?;
                } else if path.to_token_stream().to_string().starts_with("x_") {
                    // Handle custom extensions (x-*)
                    let key = path.to_token_stream().to_string().replace('_', "-");
                    let value = parse_string_value(&meta)?;
                    opencli_attrs.extensions.push((key, value));
                }

                Ok(())
            })
            .map_err(Diagnostics::from_syn_error)?;
        }
    }

    Ok((about, long_about, opencli_attrs))
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

/// Parse responses((status = "0", description = "...", content(...)), ...)
pub(crate) fn parse_responses(
    meta: &syn::meta::ParseNestedMeta,
) -> syn::Result<Vec<ResponseDefAttrs>> {
    let content;
    syn::parenthesized!(content in meta.input);

    let mut responses = Vec::new();

    while !content.is_empty() {
        // Parse each response definition group
        let resp_content;
        syn::parenthesized!(resp_content in content);

        let mut status = String::new();
        let mut description = String::new();
        let mut response_content = Vec::new();

        while !resp_content.is_empty() {
            let ident: syn::Ident = resp_content.parse()?;

            if ident == "status" {
                resp_content.parse::<syn::Token![=]>()?;
                let lit: syn::Lit = resp_content.parse()?;
                if let syn::Lit::Str(s) = lit {
                    status = s.value();
                }
            } else if ident == "description" {
                resp_content.parse::<syn::Token![=]>()?;
                let lit: syn::Lit = resp_content.parse()?;
                if let syn::Lit::Str(s) = lit {
                    description = s.value();
                }
            } else if ident == "content" {
                // Parse content(...)
                response_content = parse_response_content(&resp_content)?;
            }

            // Parse optional comma within response definition
            if !resp_content.is_empty() {
                resp_content.parse::<syn::Token![,]>()?;
            }
        }

        responses.push(ResponseDefAttrs {
            status,
            description,
            content: response_content,
        });

        // Parse optional comma between response definitions
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(responses)
}

/// Parse content((media_type = "...", schema(...), example = "..."), ...)
fn parse_response_content(
    input: &syn::parse::ParseBuffer,
) -> syn::Result<Vec<ResponseContentAttrs>> {
    let content;
    syn::parenthesized!(content in input);

    let mut content_list = Vec::new();

    while !content.is_empty() {
        // Parse each content definition group
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
                let lit: syn::Lit = content_inner.parse()?;
                if let syn::Lit::Str(s) = lit {
                    media_type = s.value();
                }
            } else if ident == "schema" {
                // Parse inline schema or schema reference
                schema = Some(parse_inline_schema(&content_inner)?);
            } else if ident == "schema_ref" {
                content_inner.parse::<syn::Token![=]>()?;
                let lit: syn::Lit = content_inner.parse()?;
                if let syn::Lit::Str(s) = lit {
                    schema_ref = Some(s.value());
                }
            } else if ident == "example" {
                content_inner.parse::<syn::Token![=]>()?;
                let lit: syn::Lit = content_inner.parse()?;
                if let syn::Lit::Str(s) = lit {
                    example = Some(serde_json::Value::String(s.value()));
                }
            }

            // Parse optional comma
            if !content_inner.is_empty() {
                content_inner.parse::<syn::Token![,]>()?;
            }
        }

        content_list.push(ResponseContentAttrs {
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

/// Parse inline schema definition: schema(type = "object", properties(...))
fn parse_inline_schema(input: &syn::parse::ParseBuffer) -> syn::Result<SchemaDefAttrs> {
    let content;
    syn::parenthesized!(content in input);

    let mut schema_type = None;
    let mut format = None;
    let mut properties = Vec::new();

    while !content.is_empty() {
        let ident: syn::Ident = content.parse()?;

        if ident == "type" || ident == "r#type" {
            content.parse::<syn::Token![=]>()?;
            let lit: syn::Lit = content.parse()?;
            if let syn::Lit::Str(s) = lit {
                schema_type = Some(s.value());
            }
        } else if ident == "format" {
            content.parse::<syn::Token![=]>()?;
            let lit: syn::Lit = content.parse()?;
            if let syn::Lit::Str(s) = lit {
                format = Some(s.value());
            }
        } else if ident == "properties" {
            properties = parse_schema_properties(&content)?;
        }

        // Parse optional comma
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }

    Ok(SchemaDefAttrs {
        schema_type,
        format,
        properties,
    })
}

/// Parse schema properties: properties((name = "field", type = "string"), ...)
fn parse_schema_properties(input: &syn::parse::ParseBuffer) -> syn::Result<Vec<PropertyDefAttrs>> {
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
            let lit: syn::Lit = prop_content.parse()?;

            if let syn::Lit::Str(s) = lit {
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

        properties.push(PropertyDefAttrs {
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

/// Build tokens for inline schema definition
pub(crate) fn build_inline_schema_tokens(schema_def: &SchemaDefAttrs) -> TokenStream2 {
    let mut schema_tokens = quote! {
        ::utocli::Schema::Object(Box::new(::utocli::Object::new()))
    };

    // Set schema type if present
    if let Some(ref schema_type) = schema_def.schema_type {
        let type_variant = match schema_type.as_str() {
            "object" => quote! { ::utocli::SchemaType::Object },
            "string" => quote! { ::utocli::SchemaType::String },
            "integer" => quote! { ::utocli::SchemaType::Integer },
            "number" => quote! { ::utocli::SchemaType::Number },
            "boolean" => quote! { ::utocli::SchemaType::Boolean },
            "array" => quote! { ::utocli::SchemaType::Array },
            _ => quote! { ::utocli::SchemaType::String },
        };

        schema_tokens = quote! {
            {
                let mut schema = #schema_tokens;
                if let ::utocli::Schema::Object(ref mut obj) = schema {
                    obj.schema_type = Some(#type_variant);
                }
                schema
            }
        };
    }

    // Set format if present
    if let Some(ref format) = schema_def.format {
        let format_variant = match format.as_str() {
            "int32" => quote! { ::utocli::SchemaFormat::Int32 },
            "int64" => quote! { ::utocli::SchemaFormat::Int64 },
            "float" => quote! { ::utocli::SchemaFormat::Float },
            "double" => quote! { ::utocli::SchemaFormat::Double },
            _ => quote! { ::utocli::SchemaFormat::Int32 },
        };

        schema_tokens = quote! {
            {
                let mut schema = #schema_tokens;
                if let ::utocli::Schema::Object(ref mut obj) = schema {
                    obj.format = Some(#format_variant);
                }
                schema
            }
        };
    }

    // Add properties if present
    if !schema_def.properties.is_empty() {
        let prop_inserts: Vec<_> = schema_def
            .properties
            .iter()
            .map(|prop| {
                let name = &prop.name;
                let type_variant = match prop.schema_type.as_str() {
                    "string" => quote! { ::utocli::SchemaType::String },
                    "integer" => quote! { ::utocli::SchemaType::Integer },
                    "number" => quote! { ::utocli::SchemaType::Number },
                    "boolean" => quote! { ::utocli::SchemaType::Boolean },
                    "array" => quote! { ::utocli::SchemaType::Array },
                    "object" => quote! { ::utocli::SchemaType::Object },
                    _ => quote! { ::utocli::SchemaType::String },
                };

                let mut prop_schema = quote! {
                    ::utocli::Schema::Object(Box::new(
                        ::utocli::Object::new().schema_type(#type_variant)
                    ))
                };

                // Add format if present
                if let Some(ref format) = prop.format {
                    let format_variant = match format.as_str() {
                        "int32" => quote! { ::utocli::SchemaFormat::Int32 },
                        "int64" => quote! { ::utocli::SchemaFormat::Int64 },
                        "float" => quote! { ::utocli::SchemaFormat::Float },
                        "double" => quote! { ::utocli::SchemaFormat::Double },
                        _ => quote! { ::utocli::SchemaFormat::Int32 },
                    };

                    prop_schema = quote! {
                        {
                            let mut schema = #prop_schema;
                            if let ::utocli::Schema::Object(ref mut obj) = schema {
                                obj.format = Some(#format_variant);
                            }
                            schema
                        }
                    };
                }

                // Add items reference for array types
                if let Some(ref items_ref) = prop.items_ref {
                    prop_schema = quote! {
                        ::utocli::Schema::Array(
                            ::utocli::Array::new()
                                .items(::utocli::RefOr::new_ref(#items_ref))
                        )
                    };
                }

                quote! {
                    properties.insert(#name.to_string(), ::utocli::RefOr::T(#prop_schema));
                }
            })
            .collect();

        schema_tokens = quote! {
            {
                let mut schema = #schema_tokens;
                let mut properties = ::utocli::Map::new();
                #(#prop_inserts)*
                if let ::utocli::Schema::Object(ref mut obj) = schema {
                    obj.properties = Some(properties);
                }
                schema
            }
        };
    }

    schema_tokens
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
