//! Enum schema generation.
//!
//! This module handles generating schemas for Rust enums, supporting both:
//! - PlainEnum: All unit variants (e.g., `enum Color { Red, Green, Blue }`)
//! - MixedEnum: Variants with fields (e.g., `enum Result<T> { Success(T), Error { code: u32 } }`)
//!
//! Follows utoipa-gen's architecture closely to maintain alignment.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Fields, Variant, punctuated::Punctuated, token::Comma};

use super::serde::{self, RenameRule, SerdeContainer, SerdeEnumRepr, SerdeValue};
use crate::doc_comment::parse_doc_comments;

/// Root context for enum schema generation
#[derive(Debug)]
#[allow(dead_code)]
pub struct Root<'a> {
    pub ident: &'a syn::Ident,
    pub attributes: &'a [syn::Attribute],
}

impl<'a> Root<'a> {
    pub fn new(ident: &'a syn::Ident, attributes: &'a [syn::Attribute]) -> Self {
        Self { ident, attributes }
    }
}

/// Plain enum with only unit variants
#[derive(Debug)]
#[allow(dead_code)]
pub struct PlainEnum<'e> {
    pub root: &'e Root<'e>,
    variants: Vec<String>,
    serde_enum_repr: SerdeEnumRepr,
    pub description: Option<String>,
}

impl<'e> PlainEnum<'e> {
    pub fn new(
        root: &'e Root,
        variants: &Punctuated<Variant, Comma>,
        rename_all: Option<RenameRule>,
    ) -> syn::Result<Self> {
        let container_rules = serde::parse_container(root.attributes)?;

        // Collect variant names, applying rename rules
        let variant_names: Vec<String> = variants
            .iter()
            .filter_map(|variant| {
                // Check for #[serde(skip)]
                let variant_serde = serde::parse_value(&variant.attrs).ok()?;
                if variant_serde.skip {
                    return None;
                }

                // Use serde rename if present, otherwise use rename_all, otherwise use original name
                let name = if let Some(rename) = variant_serde.rename {
                    rename
                } else if let Some(rule) = variant_serde.rename_all.or(rename_all) {
                    rule.apply(&variant.ident.to_string())
                } else if let Some(rule) = container_rules.rename_all {
                    rule.apply(&variant.ident.to_string())
                } else {
                    variant.ident.to_string()
                };

                Some(name)
            })
            .collect();

        let description = parse_doc_comments(root.attributes);

        Ok(Self {
            root,
            variants: variant_names,
            serde_enum_repr: container_rules.enum_repr,
            description,
        })
    }

    /// Generate the schema tokens
    pub fn to_token_stream(&self) -> TokenStream {
        match &self.serde_enum_repr {
            SerdeEnumRepr::ExternallyTagged => {
                // Simple string enum
                self.generate_string_enum()
            }
            SerdeEnumRepr::InternallyTagged { tag } => {
                // OneOf with each variant as object with tag property
                self.generate_internally_tagged_enum(tag)
            }
            SerdeEnumRepr::AdjacentlyTagged { tag, content } => {
                // OneOf with each variant as object with tag and content properties
                self.generate_adjacently_tagged_enum(tag, content)
            }
            SerdeEnumRepr::Untagged => {
                // For CLI, untagged unit enums generate a null schema (following utoipa's pattern)
                quote! {
                    ::utocli::Schema::Object(Box::new(
                        ::utocli::Object::new()
                            .schema_type(::utocli::SchemaType::Null)
                    ))
                }
            }
            SerdeEnumRepr::UnfinishedAdjacentlyTagged { .. } => {
                unreachable!(
                    "Invalid serde enum repr, serde should have panicked before reaching here"
                )
            }
        }
    }

    fn generate_string_enum(&self) -> TokenStream {
        let variants = &self.variants;
        let enum_values: Vec<_> = variants
            .iter()
            .map(|v| {
                quote! { serde_json::Value::String(#v.to_string()) }
            })
            .collect();

        quote! {
            ::utocli::Schema::Object(Box::new(
                ::utocli::Object::new()
                    .schema_type(::utocli::SchemaType::String)
                    .enum_values(vec![#(#enum_values),*])
            ))
        }
    }

    fn generate_internally_tagged_enum(&self, tag: &str) -> TokenStream {
        // For CLI, we generate a schema that describes each variant as an object
        // In a real OneOf implementation, each variant would be in the oneOf array
        // For now, we'll use properties to list variants
        let variants = &self.variants;
        let variant_schemas: Vec<_> = variants
            .iter()
            .map(|v| {
                quote! {
                    (#v.to_string(), ::utocli::RefOr::T(
                        ::utocli::Schema::Object(Box::new(
                            ::utocli::Object::new()
                                .schema_type(::utocli::SchemaType::Object)
                                .properties({
                                    use ::utocli::Map;
                                    Map::from_iter(vec![
                                        (#tag.to_string(), ::utocli::RefOr::T(
                                            ::utocli::Schema::Object(Box::new(
                                                ::utocli::Object::new()
                                                    .schema_type(::utocli::SchemaType::String)
                                                    .enum_values(vec![serde_json::Value::String(#v.to_string())])
                                            ))
                                        ))
                                    ])
                                })
                                .required(vec![#tag.to_string()])
                        ))
                    ))
                }
            })
            .collect();

        quote! {
            ::utocli::Schema::Object(Box::new(
                ::utocli::Object::new()
                    .schema_type(::utocli::SchemaType::Object)
                    .properties({
                        use ::utocli::Map;
                        Map::from_iter(vec![#(#variant_schemas),*])
                    })
            ))
        }
    }

    fn generate_adjacently_tagged_enum(&self, tag: &str, _content: &str) -> TokenStream {
        // Similar to internally tagged for plain enums (no content since they're unit variants)
        let variants = &self.variants;
        let enum_values: Vec<_> = variants
            .iter()
            .map(|v| {
                quote! { serde_json::Value::String(#v.to_string()) }
            })
            .collect();

        quote! {
            ::utocli::Schema::Object(Box::new(
                ::utocli::Object::new()
                    .schema_type(::utocli::SchemaType::Object)
                    .properties({
                        use ::utocli::Map;
                        Map::from_iter(vec![
                            (#tag.to_string(), ::utocli::RefOr::T(
                                ::utocli::Schema::Object(Box::new(
                                    ::utocli::Object::new()
                                        .schema_type(::utocli::SchemaType::String)
                                        .enum_values(vec![#(#enum_values),*])
                                ))
                            ))
                        ])
                    })
                    .required(vec![#tag.to_string()])
            ))
        }
    }
}

impl ToTokens for PlainEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.to_token_stream().to_tokens(tokens);
    }
}

/// Mixed enum with variants containing fields
#[derive(Debug)]
#[allow(dead_code)]
pub struct MixedEnum<'p> {
    pub root: &'p Root<'p>,
    pub tokens: TokenStream,
    pub description: Option<String>,
}

impl<'p> MixedEnum<'p> {
    pub fn new(root: &'p Root, variants: &Punctuated<Variant, Comma>) -> syn::Result<Self> {
        let container_rules = serde::parse_container(root.attributes)?;
        let rename_all = container_rules.rename_all;

        let mut variant_schemas = Vec::new();

        for variant in variants {
            let variant_serde = serde::parse_value(&variant.attrs)?;

            // Skip variants marked with #[serde(skip)]
            if variant_serde.skip {
                continue;
            }

            let name = if let Some(rename) = &variant_serde.rename {
                rename.clone()
            } else if let Some(rule) = variant_serde.rename_all.or(rename_all) {
                rule.apply(&variant.ident.to_string())
            } else {
                variant.ident.to_string()
            };

            let variant_schema = Self::generate_variant_schema(
                &variant.fields,
                &name,
                &container_rules,
                &variant_serde,
            )?;

            variant_schemas.push((name, variant_schema));
        }

        // Generate final schema combining all variants
        let schema_tokens = Self::combine_variant_schemas(&variant_schemas, &container_rules);

        let description = parse_doc_comments(root.attributes);

        Ok(Self {
            root,
            tokens: schema_tokens,
            description,
        })
    }

    fn generate_variant_schema(
        fields: &Fields,
        variant_name: &str,
        container: &SerdeContainer,
        _variant_serde: &SerdeValue,
    ) -> syn::Result<TokenStream> {
        match fields {
            Fields::Named(named) => {
                // Generate object schema with properties for each field
                let mut properties = Vec::new();
                let mut required = Vec::new();

                for field in &named.named {
                    let field_serde = serde::parse_value(&field.attrs)?;
                    if field_serde.skip {
                        continue;
                    }

                    let field_name = field.ident.as_ref().unwrap();
                    let field_name_str = if let Some(rename) = field_serde.rename {
                        rename
                    } else {
                        field_name.to_string()
                    };

                    let ty = &field.ty;
                    let is_optional = is_option_type(ty);

                    if !is_optional {
                        required.push(field_name_str.clone());
                    }

                    // Use simple type inference
                    let schema_ref_or = super::infer_schema_ref_or(ty, false, false);

                    properties.push(quote! {
                        (#field_name_str.to_string(), #schema_ref_or)
                    });
                }

                let mut object_builder = quote! {
                    ::utocli::Object::new()
                        .schema_type(::utocli::SchemaType::Object)
                        .properties({
                            use ::utocli::Map;
                            Map::from_iter(vec![#(#properties),*])
                        })
                };

                if !required.is_empty() {
                    object_builder.extend(quote! {
                        .required(vec![#(#required.to_string()),*])
                    });
                }

                let schema = quote! {
                    ::utocli::Schema::Object(Box::new(#object_builder))
                };

                Ok(Self::wrap_variant_schema(
                    schema,
                    variant_name,
                    &container.enum_repr,
                ))
            }
            Fields::Unnamed(unnamed) => {
                // For unnamed fields (tuple variants), we'll generate a schema based on the fields
                if unnamed.unnamed.len() == 1 {
                    // Single field - use its type directly
                    let field = unnamed.unnamed.first().unwrap();
                    let ty = &field.ty;
                    let schema_ref_or = super::infer_schema_ref_or(ty, false, false);

                    let schema = quote! {
                        match #schema_ref_or {
                            ::utocli::RefOr::T(s) => s,
                            ::utocli::RefOr::Ref(_) => {
                                // For references in enums, we need to inline them
                                // For now, generate a generic object schema
                                ::utocli::Schema::Object(Box::new(
                                    ::utocli::Object::new()
                                        .schema_type(::utocli::SchemaType::Object)
                                ))
                            }
                        }
                    };

                    Ok(Self::wrap_variant_schema(
                        schema,
                        variant_name,
                        &container.enum_repr,
                    ))
                } else {
                    // Multiple fields - treat as array or object
                    let schema = quote! {
                        ::utocli::Schema::Object(Box::new(
                            ::utocli::Object::new()
                                .schema_type(::utocli::SchemaType::Array)
                        ))
                    };

                    Ok(Self::wrap_variant_schema(
                        schema,
                        variant_name,
                        &container.enum_repr,
                    ))
                }
            }
            Fields::Unit => {
                // Unit variant - string enum value
                let schema = quote! {
                    ::utocli::Schema::Object(Box::new(
                        ::utocli::Object::new()
                            .schema_type(::utocli::SchemaType::String)
                            .enum_values(vec![serde_json::Value::String(#variant_name.to_string())])
                    ))
                };

                Ok(Self::wrap_variant_schema(
                    schema,
                    variant_name,
                    &container.enum_repr,
                ))
            }
        }
    }

    fn wrap_variant_schema(
        schema: TokenStream,
        variant_name: &str,
        repr: &SerdeEnumRepr,
    ) -> TokenStream {
        match repr {
            SerdeEnumRepr::ExternallyTagged => {
                // Wrap in object with variant name as property
                quote! {
                    ::utocli::Schema::Object(Box::new(
                        ::utocli::Object::new()
                            .schema_type(::utocli::SchemaType::Object)
                            .properties({
                                use ::utocli::Map;
                                Map::from_iter(vec![
                                    (#variant_name.to_string(), ::utocli::RefOr::T(#schema))
                                ])
                            })
                            .required(vec![#variant_name.to_string()])
                    ))
                }
            }
            SerdeEnumRepr::InternallyTagged { tag } => {
                // Add tag property to the schema
                quote! {
                    match #schema {
                        ::utocli::Schema::Object(mut obj) => {
                            let mut props = obj.properties.unwrap_or_default();
                            props.insert(
                                #tag.to_string(),
                                ::utocli::RefOr::T(::utocli::Schema::Object(Box::new(
                                    ::utocli::Object::new()
                                        .schema_type(::utocli::SchemaType::String)
                                        .enum_values(vec![serde_json::Value::String(#variant_name.to_string())])
                                )))
                            );
                            obj.properties = Some(props);

                            let mut req = obj.required.unwrap_or_default();
                            req.push(#tag.to_string());
                            obj.required = Some(req);

                            ::utocli::Schema::Object(obj)
                        }
                        other => other,
                    }
                }
            }
            SerdeEnumRepr::AdjacentlyTagged { tag, content } => {
                // Wrap in object with tag and content properties
                quote! {
                    ::utocli::Schema::Object(Box::new(
                        ::utocli::Object::new()
                            .schema_type(::utocli::SchemaType::Object)
                            .properties({
                                use ::utocli::Map;
                                Map::from_iter(vec![
                                    (#tag.to_string(), ::utocli::RefOr::T(
                                        ::utocli::Schema::Object(Box::new(
                                            ::utocli::Object::new()
                                                .schema_type(::utocli::SchemaType::String)
                                                .enum_values(vec![serde_json::Value::String(#variant_name.to_string())])
                                        ))
                                    )),
                                    (#content.to_string(), ::utocli::RefOr::T(#schema))
                                ])
                            })
                            .required(vec![#tag.to_string(), #content.to_string()])
                    ))
                }
            }
            SerdeEnumRepr::Untagged => {
                // No wrapping - use schema as-is
                schema
            }
            SerdeEnumRepr::UnfinishedAdjacentlyTagged { .. } => {
                unreachable!("Invalid serde enum repr")
            }
        }
    }

    fn combine_variant_schemas(
        variants: &[(String, TokenStream)],
        _container: &SerdeContainer,
    ) -> TokenStream {
        // For CLI, we use a properties-based approach to represent the enum variants
        // In a true OpenAPI implementation, this would use oneOf
        let variant_props: Vec<_> = variants
            .iter()
            .map(|(name, schema)| {
                quote! {
                    (#name.to_string(), ::utocli::RefOr::T(#schema))
                }
            })
            .collect();

        quote! {
            ::utocli::Schema::Object(Box::new(
                ::utocli::Object::new()
                    .schema_type(::utocli::SchemaType::Object)
                    .properties({
                        use ::utocli::Map;
                        Map::from_iter(vec![#(#variant_props),*])
                    })
            ))
        }
    }
}

impl ToTokens for MixedEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.tokens.to_tokens(tokens);
    }
}

/// Check if a type is Option<T>
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}
