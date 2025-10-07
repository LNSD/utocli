//! Parameter generation for ToParameter derive macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Lit};

use crate::{
    AnyValue,
    diagnostics::{Diagnostics, ToTokensDiagnostics},
    doc_comment::parse_doc_comments,
    parse_utils,
};

/// Parsed parameter attributes from `#[param(...)]`.
/// Matches utoipa's pattern for using AnyValue for example/default
#[derive(Default)]
struct ParameterAttributes {
    alias: Option<Vec<String>>,
    description: Option<String>,
    scope: Option<String>,
    position: Option<u32>,
    in_: Option<String>,
    format: Option<String>,
    enum_values: Option<Vec<String>>,
    /// Default value using AnyValue for flexible parsing
    /// Matches utoipa-gen/src/component/features/attributes.rs line 31520
    default: Option<AnyValue>,
    /// Example value using AnyValue for flexible parsing
    /// Matches utoipa-gen/src/component/features/attributes.rs line 31557
    example: Option<AnyValue>,
    skip: bool,
    schema_with: Option<syn::TypePath>,
    minimum: Option<f64>,
    maximum: Option<f64>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    pattern: Option<String>,
    multiple_of: Option<f64>,
    exclusive_minimum: Option<bool>,
    exclusive_maximum: Option<bool>,
    max_properties: Option<usize>,
    min_properties: Option<usize>,
    min_items: Option<usize>,
    max_items: Option<usize>,
}

impl ParameterAttributes {
    fn parse(attrs: &[syn::Attribute]) -> Result<Self, Diagnostics> {
        let mut result = Self::default();

        for attr in attrs {
            if attr.path().is_ident("param") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("alias") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            let alias_str = s.value();
                            result.alias = Some(vec![alias_str]);
                        }
                    } else if meta.path.is_ident("description") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.description = Some(s.value());
                        }
                    } else if meta.path.is_ident("scope") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.scope = Some(s.value());
                        }
                    } else if meta.path.is_ident("position") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Int(i) = lit {
                            result.position = Some(i.base10_parse()?);
                        }
                    } else if meta.path.is_ident("in") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.in_ = Some(s.value());
                        }
                    } else if meta.path.is_ident("format") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.format = Some(s.value());
                        }
                    } else if meta.path.is_ident("enum_values") {
                        // Parse enum_values as a list
                        let content;
                        syn::parenthesized!(content in meta.input);
                        let mut values = Vec::new();
                        while !content.is_empty() {
                            let lit: Lit = content.parse()?;
                            if let Lit::Str(s) = lit {
                                values.push(s.value());
                            }
                            if !content.is_empty() {
                                content.parse::<syn::Token![,]>()?;
                            }
                        }
                        result.enum_values = Some(values);
                    } else if meta.path.is_ident("default") {
                        // Matches utoipa-gen/src/component/features/attributes.rs line 31532
                        result.default = Some(parse_utils::parse_next(meta.input, || {
                            AnyValue::parse_any(meta.input)
                        })?);
                    } else if meta.path.is_ident("example") {
                        // Matches utoipa-gen/src/component/features/attributes.rs line 31562
                        result.example = Some(parse_utils::parse_next(meta.input, || {
                            AnyValue::parse_any(meta.input)
                        })?);
                    } else if meta.path.is_ident("skip") {
                        result.skip = true;
                    } else if meta.path.is_ident("schema_with") {
                        let value = meta.value()?;
                        result.schema_with = Some(value.parse()?);
                    } else if meta.path.is_ident("minimum") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Float(f) = lit {
                            result.minimum = Some(f.base10_parse()?);
                        } else if let Lit::Int(i) = lit {
                            result.minimum = Some(i.base10_parse::<i64>()? as f64);
                        }
                    } else if meta.path.is_ident("maximum") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Float(f) = lit {
                            result.maximum = Some(f.base10_parse()?);
                        } else if let Lit::Int(i) = lit {
                            result.maximum = Some(i.base10_parse::<i64>()? as f64);
                        }
                    } else if meta.path.is_ident("min_length") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Int(i) = lit {
                            result.min_length = Some(i.base10_parse()?);
                        }
                    } else if meta.path.is_ident("max_length") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Int(i) = lit {
                            result.max_length = Some(i.base10_parse()?);
                        }
                    } else if meta.path.is_ident("pattern") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.pattern = Some(s.value());
                        }
                    } else if meta.path.is_ident("multiple_of") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Float(f) = lit {
                            result.multiple_of = Some(f.base10_parse()?);
                        } else if let Lit::Int(i) = lit {
                            result.multiple_of = Some(i.base10_parse::<i64>()? as f64);
                        }
                    } else if meta.path.is_ident("exclusive_minimum") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Bool(b) = lit {
                            result.exclusive_minimum = Some(b.value());
                        }
                    } else if meta.path.is_ident("exclusive_maximum") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Bool(b) = lit {
                            result.exclusive_maximum = Some(b.value());
                        }
                    } else if meta.path.is_ident("max_properties") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Int(i) = lit {
                            result.max_properties = Some(i.base10_parse()?);
                        }
                    } else if meta.path.is_ident("min_properties") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Int(i) = lit {
                            result.min_properties = Some(i.base10_parse()?);
                        }
                    } else if meta.path.is_ident("min_items") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Int(i) = lit {
                            result.min_items = Some(i.base10_parse()?);
                        }
                    } else if meta.path.is_ident("max_items") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Int(i) = lit {
                            result.max_items = Some(i.base10_parse()?);
                        }
                    }
                    Ok(())
                })?;
            } else if attr.path().is_ident("serde") {
                // Parse serde attributes for compatibility
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("skip") {
                        result.skip = true;
                    }
                    Ok(())
                })?;
            }
        }

        Ok(result)
    }
}

/// Parameter generator for the ToParameter derive macro.
pub struct Parameter {
    input: DeriveInput,
}

impl Parameter {
    pub fn new(input: DeriveInput) -> Result<Self, Diagnostics> {
        Ok(Self { input })
    }
}

impl ToTokensDiagnostics for Parameter {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        let name = &self.input.ident;
        let (impl_generics, ty_generics, where_clause) = self.input.generics.split_for_impl();

        // Generate parameters based on data structure
        let params_impl = match &self.input.data {
            Data::Struct(data_struct) => self.generate_struct_parameters(&data_struct.fields)?,
            Data::Enum(_) => {
                return Err(Diagnostics::new("ToParameter cannot be derived for enums")
                    .help("ToParameter can only be derived for structs with named fields")
                    .note("Each struct field becomes a CLI parameter"));
            }
            Data::Union(_) => {
                return Err(Diagnostics::new("ToParameter cannot be derived for unions")
                    .help("ToParameter can only be derived for structs with named fields"));
            }
        };

        // Generate component name by stripping "Param" suffix if present
        let name_str = name.to_string();
        let component_name = if name_str.ends_with("Param") {
            &name_str[..name_str.len() - 5]
        } else {
            &name_str
        };

        tokens.extend(quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                /// Generate the OpenCLI parameters for this type.
                pub fn parameters() -> Vec<::utocli::Parameter> {
                    use ::utocli::{Parameter, ParameterScope, ParameterIn, Schema, SchemaType, Object, RefOr};

                    #params_impl
                }

                /// Get the parameter component name for this type.
                pub fn parameter_name() -> &'static str {
                    #component_name
                }
            }
        });

        Ok(())
    }
}

impl Parameter {
    fn generate_struct_parameters(&self, fields: &Fields) -> Result<TokenStream, Diagnostics> {
        match fields {
            Fields::Named(named_fields) => {
                let mut parameters = Vec::new();

                for field in &named_fields.named {
                    let field_attrs = ParameterAttributes::parse(&field.attrs)?;

                    if field_attrs.skip {
                        continue;
                    }

                    let field_name = field.ident.as_ref().unwrap();
                    let field_name_str = field_name.to_string();

                    let ty = &field.ty;
                    let is_optional = is_option_type(ty);
                    let is_bool = is_bool_type(ty);

                    // Determine parameter type based on type and attributes
                    let param_in = if let Some(in_val) = &field_attrs.in_ {
                        match in_val.as_str() {
                            "argument" => quote! { Some(ParameterIn::Argument) },
                            "flag" => quote! { Some(ParameterIn::Flag) },
                            "option" => quote! { Some(ParameterIn::Option) },
                            _ => quote! { None },
                        }
                    } else if is_bool {
                        quote! { Some(ParameterIn::Flag) }
                    } else if field_attrs.position.is_some() {
                        quote! { Some(ParameterIn::Argument) }
                    } else {
                        // Don't set a default - let it be None so it's not serialized
                        quote! { None }
                    };

                    // Get description from attributes or doc comments
                    let description = field_attrs
                        .description
                        .clone()
                        .or_else(|| parse_doc_comments(&field.attrs))
                        .map(|d| quote! { Some(#d.to_string()) })
                        .unwrap_or_else(|| quote! { None });

                    let alias = if let Some(alias_vec) = &field_attrs.alias {
                        quote! { Some(vec![#(#alias_vec.to_string()),*]) }
                    } else {
                        quote! { None }
                    };

                    let scope = if let Some(scope_str) = &field_attrs.scope {
                        match scope_str.as_str() {
                            "inherited" => quote! { Some(ParameterScope::Inherited) },
                            "local" => quote! { Some(ParameterScope::Local) },
                            _ => quote! { None },
                        }
                    } else {
                        quote! { None }
                    };

                    let position = if let Some(pos) = field_attrs.position {
                        quote! { Some(#pos) }
                    } else {
                        quote! { None }
                    };

                    let required = if !is_optional && field_attrs.position.is_some() {
                        quote! { Some(true) }
                    } else {
                        quote! { None }
                    };

                    // Use schema_with if provided, otherwise generate schema from type
                    let schema = if let Some(schema_with) = field_attrs.schema_with {
                        // Call the custom schema function
                        quote! { Some(#schema_with()) }
                    } else {
                        // Generate basic schema based on type
                        let schema_type = if is_bool || (is_optional && is_inner_bool_type(ty)) {
                            quote! { SchemaType::Boolean }
                        } else {
                            quote! { SchemaType::String }
                        };

                        // Build schema with optional format, enum, default, and example
                        let mut object_builder = quote! {
                            let mut obj = Object::new().schema_type(#schema_type);
                        };

                        if let Some(format_str) = &field_attrs.format {
                            let format_ident = syn::Ident::new(
                                &(format_str
                                    .chars()
                                    .next()
                                    .unwrap()
                                    .to_uppercase()
                                    .collect::<String>()
                                    + &format_str[1..]),
                                proc_macro2::Span::call_site(),
                            );
                            object_builder.extend(quote! {
                                obj = obj.format(::utocli::SchemaFormat::#format_ident);
                            });
                        }

                        if let Some(enum_vals) = &field_attrs.enum_values {
                            object_builder.extend(quote! {
                                obj = obj.enum_values(vec![#(serde_json::Value::String(#enum_vals.to_string())),*]);
                            });
                        }

                        if let Some(default_any) = &field_attrs.default {
                            // AnyValue::to_tokens already wraps in serde_json::json!()
                            // Matches utoipa pattern - much simpler than manual parsing
                            object_builder.extend(quote! {
                                obj = obj.default_value(#default_any);
                            });
                        }

                        if let Some(example_any) = &field_attrs.example {
                            // AnyValue::to_tokens already wraps in serde_json::json!()
                            // Matches utoipa pattern - much simpler than manual parsing
                            object_builder.extend(quote! {
                                obj = obj.example(#example_any);
                            });
                        }

                        // Apply validation attributes
                        if let Some(min) = field_attrs.minimum {
                            object_builder.extend(quote! {
                                obj = obj.minimum(#min);
                            });
                        }

                        if let Some(max) = field_attrs.maximum {
                            object_builder.extend(quote! {
                                obj = obj.maximum(#max);
                            });
                        }

                        if let Some(min_len) = field_attrs.min_length {
                            object_builder.extend(quote! {
                                obj = obj.min_length(#min_len);
                            });
                        }

                        if let Some(max_len) = field_attrs.max_length {
                            object_builder.extend(quote! {
                                obj = obj.max_length(#max_len);
                            });
                        }

                        if let Some(ref pattern) = field_attrs.pattern {
                            object_builder.extend(quote! {
                                obj = obj.pattern(#pattern);
                            });
                        }

                        if let Some(mult) = field_attrs.multiple_of {
                            object_builder.extend(quote! {
                                obj = obj.multiple_of(#mult);
                            });
                        }

                        if let Some(excl_min) = field_attrs.exclusive_minimum {
                            object_builder.extend(quote! {
                                obj = obj.exclusive_minimum(#excl_min);
                            });
                        }

                        if let Some(excl_max) = field_attrs.exclusive_maximum {
                            object_builder.extend(quote! {
                                obj = obj.exclusive_maximum(#excl_max);
                            });
                        }

                        if let Some(max_props) = field_attrs.max_properties {
                            object_builder.extend(quote! {
                                obj = obj.max_properties(#max_props);
                            });
                        }

                        if let Some(min_props) = field_attrs.min_properties {
                            object_builder.extend(quote! {
                                obj = obj.min_properties(#min_props);
                            });
                        }

                        // Note: min_items and max_items are not applied here for Object schema
                        // They would only be used if we generate Array schemas for Vec<T> types

                        quote! {
                            Some(RefOr::T(Schema::Object(Box::new({
                                #object_builder
                                obj
                            }))))
                        }
                    };

                    parameters.push(quote! {
                        Parameter {
                            name: #field_name_str.to_string(),
                            in_: #param_in,
                            position: #position,
                            alias: #alias,
                            description: #description,
                            required: #required,
                            scope: #scope,
                            arity: None,
                            schema: #schema,
                            extensions: None,
                        }
                    });
                }

                Ok(quote! {
                    vec![#(#parameters),*]
                })
            }
            Fields::Unnamed(_) => Ok(quote! {
                vec![]
            }),
            Fields::Unit => Ok(quote! {
                vec![]
            }),
        }
    }
}

/// Check if a type is `Option<T>`.
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}

/// Check if a type is `bool`.
fn is_bool_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "bool";
    }
    false
}

/// Check if the inner type of Option<T> is bool.
fn is_inner_bool_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return is_bool_type(inner_ty);
    }
    false
}
