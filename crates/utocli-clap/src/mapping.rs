//! Mapping logic from clap types to OpenCLI types.
//!
//! This module contains the core logic for converting clap's type system
//! (arguments, flags, options) into OpenCLI parameter and schema definitions.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Field, GenericArgument, PathArguments, Type, TypePath};

use super::arg::{ClapArgAttrs, OpenCliArgAttrs};
use crate::diagnostics::Diagnostics;

/// Infer OpenCLI parameter type from Rust field type and clap attributes.
///
/// Rules:
/// - `bool` with `short`/`long` → Flag
/// - `T` with `short`/`long` → Option
/// - `T` without `short`/`long` → Argument (positional)
pub fn infer_parameter_in(field: &Field, arg_attrs: &ClapArgAttrs) -> Result<String, Diagnostics> {
    // Check if field type is bool
    if is_bool_type(&field.ty) {
        return Ok("flag".to_string());
    }

    // Check if has short/long attributes
    if arg_attrs.short.is_some() || arg_attrs.long.is_some() {
        return Ok("option".to_string());
    }

    // Default to positional argument
    Ok("argument".to_string())
}

/// Check if a type is bool.
pub(crate) fn is_bool_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(ident) = type_path.path.get_ident()
    {
        return ident == "bool";
    }
    false
}

/// Infer OpenCLI schema from Rust type.
///
/// Returns TokenStream for schema generation.
pub fn infer_schema_from_type(ty: &Type) -> Result<TokenStream2, Diagnostics> {
    // Unwrap Option<T> to get inner type
    let inner_ty = unwrap_option_type(ty).unwrap_or(ty);

    // Check for Vec<T>
    let (is_vec, element_ty) = if let Some(elem) = unwrap_vec_type(inner_ty) {
        (true, elem)
    } else {
        (false, inner_ty)
    };

    // Build schema for element type
    let schema = build_base_schema(element_ty)?;

    // Wrap in array if Vec<T>
    if is_vec {
        Ok(quote! {
            ::utocli::RefOr::T(::utocli::Schema::Array(
                ::utocli::Array::new().items(::utocli::RefOr::T(#schema))
            ))
        })
    } else {
        Ok(quote! {
            ::utocli::RefOr::T(#schema)
        })
    }
}

/// Build base schema for a type.
fn build_base_schema(ty: &Type) -> Result<TokenStream2, Diagnostics> {
    let type_str = quote!(#ty).to_string();

    match type_str.as_str() {
        "bool" => Ok(quote! {
            ::utocli::Schema::Object(Box::new(
                ::utocli::Object::new().schema_type(::utocli::SchemaType::Boolean)
            ))
        }),
        "String" | "str" | "& str" => Ok(quote! {
            ::utocli::Schema::Object(Box::new(
                ::utocli::Object::new().schema_type(::utocli::SchemaType::String)
            ))
        }),
        "PathBuf" | "Path" | "& Path" => Ok(quote! {
            ::utocli::Schema::Object(Box::new(
                ::utocli::Object::new()
                    .schema_type(::utocli::SchemaType::String)
                    .format(::utocli::SchemaFormat::Path)
            ))
        }),
        "i8" | "i16" | "i32" | "i64" | "isize" => Ok(quote! {
            ::utocli::Schema::Object(Box::new(
                ::utocli::Object::new()
                    .schema_type(::utocli::SchemaType::Integer)
                    .format(::utocli::SchemaFormat::Int64)
            ))
        }),
        "u8" | "u16" | "u32" | "u64" | "usize" => Ok(quote! {
            ::utocli::Schema::Object(Box::new(
                ::utocli::Object::new()
                    .schema_type(::utocli::SchemaType::Integer)
                    .format(::utocli::SchemaFormat::Int64)
            ))
        }),
        "f32" => Ok(quote! {
            ::utocli::Schema::Object(Box::new(
                ::utocli::Object::new()
                    .schema_type(::utocli::SchemaType::Number)
                    .format(::utocli::SchemaFormat::Float)
            ))
        }),
        "f64" => Ok(quote! {
            ::utocli::Schema::Object(Box::new(
                ::utocli::Object::new()
                    .schema_type(::utocli::SchemaType::Number)
                    .format(::utocli::SchemaFormat::Double)
            ))
        }),
        _ => {
            // Default to string
            Ok(quote! {
                ::utocli::Schema::Object(Box::new(
                    ::utocli::Object::new().schema_type(::utocli::SchemaType::String)
                ))
            })
        }
    }
}

/// Unwrap Option<T> to get T.
fn unwrap_option_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty
        && let Some(segment) = path.segments.last()
        && segment.ident == "Option"
        && let PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return Some(inner_ty);
    }
    None
}

/// Unwrap Vec<T> to get T.
fn unwrap_vec_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty
        && let Some(segment) = path.segments.last()
        && segment.ident == "Vec"
        && let PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return Some(inner_ty);
    }
    None
}

/// Map clap arg attributes to OpenCLI parameter.
///
/// This generates TokenStream for creating a Parameter.
/// The `is_root_command` flag indicates whether this parameter belongs to the root command.
pub fn map_arg_to_parameter(
    field: &Field,
    arg_attrs: &ClapArgAttrs,
    opencli_attrs: &OpenCliArgAttrs,
) -> Result<TokenStream2, Diagnostics> {
    map_arg_to_parameter_internal(field, arg_attrs, opencli_attrs, false)
}

/// Map clap arg attributes to OpenCLI parameter for root command.
///
/// This generates TokenStream for creating a Parameter.
/// Root command parameters have different behavior for the "in" field.
pub fn map_arg_to_parameter_for_root(
    field: &Field,
    arg_attrs: &ClapArgAttrs,
    opencli_attrs: &OpenCliArgAttrs,
) -> Result<TokenStream2, Diagnostics> {
    map_arg_to_parameter_internal(field, arg_attrs, opencli_attrs, true)
}

/// Internal implementation of parameter mapping with root command flag.
fn map_arg_to_parameter_internal(
    field: &Field,
    arg_attrs: &ClapArgAttrs,
    opencli_attrs: &OpenCliArgAttrs,
    is_root_command: bool,
) -> Result<TokenStream2, Diagnostics> {
    // Get parameter name from field or long attribute
    let field_name = field
        .ident
        .as_ref()
        .ok_or_else(|| Diagnostics::new("Field must have a name"))?;

    let param_name = if let Some(ref long_name) = arg_attrs.long {
        if long_name.is_empty() {
            field_name.to_string()
        } else {
            long_name.clone()
        }
    } else {
        field_name.to_string()
    };

    // Infer parameter type (argument, flag, option)
    let param_in = infer_parameter_in(field, arg_attrs)?;

    // Build parameter
    // Options (non-flag, non-argument parameters) should NOT have "in" field
    // Only arguments and flags have explicit "in" field
    let mut param_tokens = match param_in.as_str() {
        "flag" => quote! {
            ::utocli::Parameter::new_flag(#param_name)
        },
        "argument" => quote! {
            ::utocli::Parameter::new_argument(#param_name, 1u32)
        },
        "option" | _ => quote! {
            ::utocli::Parameter::new(#param_name)
        },
    };

    // Add aliases (short flag)
    if let Some(short) = arg_attrs.short {
        let short_char = if short == '\0' {
            // Infer from field name (first character)
            field_name.to_string().chars().next().unwrap_or('_')
        } else {
            short
        };
        let short_str = short_char.to_string();
        param_tokens.extend(quote! {
            .alias(vec![#short_str.to_string()])
        });
    }

    // Add description
    let description = opencli_attrs
        .description
        .as_ref()
        .or(arg_attrs.long_help.as_ref())
        .or(arg_attrs.help.as_ref());

    if let Some(desc) = description {
        param_tokens.extend(quote! {
            .description(#desc)
        });
    }

    // Add required flag based on Option type and presence of default value
    // Only add "required" field for arguments and options (not for flags or root options)
    // Don't add "required" if the parameter has a default value
    if param_in == "argument" || (param_in == "option" && !is_root_command) {
        let has_default =
            arg_attrs.default_value.is_some() || (is_bool_type(&field.ty) && arg_attrs.global);

        if !has_default {
            if is_option_type(&field.ty) {
                param_tokens.extend(quote! {
                    .required(false)
                });
            } else if arg_attrs.required {
                param_tokens.extend(quote! {
                    .required(true)
                });
            } else {
                param_tokens.extend(quote! {
                    .required(true)
                });
            }
        }
    }

    // Add scope (inherited for global parameters, local otherwise)
    // Check for explicit opencli scope override first
    if let Some(ref scope_str) = opencli_attrs.scope {
        let scope_variant = match scope_str.as_str() {
            "inherited" => quote! { ::utocli::ParameterScope::Inherited },
            "local" => quote! { ::utocli::ParameterScope::Local },
            _ => quote! { ::utocli::ParameterScope::Local },
        };
        param_tokens.extend(quote! {
            .scope(#scope_variant)
        });
    } else if arg_attrs.global {
        param_tokens.extend(quote! {
            .scope(::utocli::ParameterScope::Inherited)
        });
    } else {
        // Default to local scope for non-global parameters
        param_tokens.extend(quote! {
            .scope(::utocli::ParameterScope::Local)
        });
    }

    // Generate schema with constraints
    let schema = build_schema_with_constraints(&field.ty, arg_attrs, opencli_attrs)?;
    param_tokens.extend(quote! {
        .schema(#schema)
    });

    // Add arity for Vec types or explicit num_args
    if let Some(arity) = compute_arity(&field.ty, arg_attrs)? {
        param_tokens.extend(quote! {
            .arity(#arity)
        });
    }

    // Add extensions if present (x-*)
    if !opencli_attrs.extensions.is_empty() {
        // Build extension inserts
        let ext_inserts: Vec<_> = opencli_attrs
            .extensions
            .iter()
            .map(|(key, value)| {
                quote! {
                    ext_map.insert(#key.to_string(), serde_json::Value::String(#value.to_string()));
                }
            })
            .collect();

        param_tokens = quote! {
            {
                let mut ext_map = ::utocli::Map::new();
                #(#ext_inserts)*
                #param_tokens.extensions(::utocli::Extensions::from(ext_map))
            }
        };
    }

    Ok(param_tokens)
}

/// Build schema with constraints from clap and opencli attributes.
fn build_schema_with_constraints(
    ty: &Type,
    arg_attrs: &ClapArgAttrs,
    opencli_attrs: &OpenCliArgAttrs,
) -> Result<TokenStream2, Diagnostics> {
    // Start with base schema
    let mut schema_tokens = infer_schema_from_type(ty)?;

    // Apply opencli format override
    if let Some(ref format) = opencli_attrs.format {
        // Map string format to SchemaFormat enum
        let format_variant = match format.as_str() {
            "path" => quote! { ::utocli::SchemaFormat::Path },
            "email" => quote! { ::utocli::SchemaFormat::Email },
            "uri" => quote! { ::utocli::SchemaFormat::Uri },
            "url" => quote! { ::utocli::SchemaFormat::Url },
            "date" => quote! { ::utocli::SchemaFormat::Date },
            "date-time" => quote! { ::utocli::SchemaFormat::DateTime },
            "time" => quote! { ::utocli::SchemaFormat::Time },
            "uuid" => quote! { ::utocli::SchemaFormat::Uuid },
            "ipv4" => quote! { ::utocli::SchemaFormat::Ipv4 },
            "ipv6" => quote! { ::utocli::SchemaFormat::Ipv6 },
            "int32" => quote! { ::utocli::SchemaFormat::Int32 },
            "int64" => quote! { ::utocli::SchemaFormat::Int64 },
            "float" => quote! { ::utocli::SchemaFormat::Float },
            "double" => quote! { ::utocli::SchemaFormat::Double },
            _ => {
                // Unknown format, skip it
                return Err(Diagnostics::new(format!("Unknown format: {}", format))
                    .help("Supported formats: path, email, uri, url, date, date-time, time, uuid, ipv4, ipv6, int32, int64, float, double"));
            }
        };

        schema_tokens = quote! {
            {
                let mut schema = #schema_tokens;
                if let ::utocli::RefOr::T(::utocli::Schema::Object(ref mut obj)) = schema {
                    obj.format = Some(#format_variant);
                }
                schema
            }
        };
    }

    // Apply enum constraint from value_parser
    if let Some(ref possible_values) = arg_attrs.possible_values {
        // Convert serde_json::Value to string literals for code generation
        let value_strs: Vec<_> = possible_values
            .iter()
            .filter_map(|v| {
                if let serde_json::Value::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect();

        schema_tokens = quote! {
            {
                let mut schema = #schema_tokens;
                if let ::utocli::RefOr::T(::utocli::Schema::Object(ref mut obj)) = schema {
                    obj.enum_values = Some(vec![
                        #(serde_json::Value::String(#value_strs.to_string())),*
                    ]);
                }
                schema
            }
        };
    }

    // Apply default value
    if let Some(ref default_value) = arg_attrs.default_value {
        schema_tokens = quote! {
            {
                let mut schema = #schema_tokens;
                if let ::utocli::RefOr::T(::utocli::Schema::Object(ref mut obj)) = schema {
                    obj.default = Some(serde_json::Value::String(#default_value.to_string()));
                }
                schema
            }
        };
    } else if is_bool_type(ty) && arg_attrs.global {
        // Only global boolean flags default to false
        schema_tokens = quote! {
            {
                let mut schema = #schema_tokens;
                if let ::utocli::RefOr::T(::utocli::Schema::Object(ref mut obj)) = schema {
                    obj.default = Some(serde_json::Value::Bool(false));
                }
                schema
            }
        };
    }

    // Apply example from opencli attrs or value_name
    let example_value = opencli_attrs
        .example
        .as_ref()
        .or(arg_attrs.value_name.as_ref());
    if let Some(example) = example_value {
        schema_tokens = quote! {
            {
                let mut schema = #schema_tokens;
                if let ::utocli::RefOr::T(::utocli::Schema::Object(ref mut obj)) = schema {
                    obj.example = Some(serde_json::Value::String(#example.to_string()));
                }
                schema
            }
        };
    }

    Ok(schema_tokens)
}

/// Compute arity from Vec type or explicit num_args.
fn compute_arity(ty: &Type, arg_attrs: &ClapArgAttrs) -> Result<Option<TokenStream2>, Diagnostics> {
    use crate::arg::NumArgs;

    // Check for explicit num_args first
    if let Some(ref num_args) = arg_attrs.num_args {
        return match num_args {
            NumArgs::Exact(n) => {
                let n_u32 = *n as u32;
                Ok(Some(quote! {
                    ::utocli::Arity { min: Some(#n_u32), max: Some(#n_u32) }
                }))
            }
            NumArgs::Range { min, max } => {
                let min_tokens = if let Some(m) = min {
                    let m_u32 = *m as u32;
                    quote! { Some(#m_u32) }
                } else {
                    quote! { None }
                };
                let max_tokens = if let Some(m) = max {
                    let m_u32 = *m as u32;
                    quote! { Some(#m_u32) }
                } else {
                    quote! { None }
                };
                Ok(Some(quote! {
                    ::utocli::Arity { min: #min_tokens, max: #max_tokens }
                }))
            }
        };
    }

    // Infer from Vec<T> type
    let inner_ty = unwrap_option_type(ty).unwrap_or(ty);
    if unwrap_vec_type(inner_ty).is_some() {
        // Vec<T> defaults to min: 1, max: None
        return Ok(Some(quote! {
            ::utocli::Arity { min: Some(1), max: None }
        }));
    }

    Ok(None)
}

/// Check if a type is Option<T>.
fn is_option_type(ty: &Type) -> bool {
    unwrap_option_type(ty).is_some()
}
