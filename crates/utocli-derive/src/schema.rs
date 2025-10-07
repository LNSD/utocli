//! Schema generation for ToSchema derive macro.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Data, DeriveInput, Fields, Lit, Result};

use crate::{diagnostics::Diagnostics, doc_comment::parse_doc_comments};

mod enums;
mod serde;

use enums::{MixedEnum, PlainEnum, Root};

use crate::{AnyValue, parse_utils};

/// Check whether either serde `container_rule` or `field_rule` has _`default`_ attribute set.
#[inline]
fn is_default(container_rules: &serde::SerdeContainer, field_rule: &serde::SerdeValue) -> bool {
    container_rules.default || field_rule.default
}

/// Check whether field is required based on following rules (matches utoipa exactly):
///
/// * If field has not serde's `skip_serializing_if`
/// * Field has not `serde_with` double option
/// * Field is not default
pub fn is_required(
    field_rule: &serde::SerdeValue,
    container_rules: &serde::SerdeContainer,
) -> bool {
    !field_rule.skip_serializing_if
        && !field_rule.double_option
        && !is_default(container_rules, field_rule)
}

/// Parsed schema attributes from `#[schema(...)]`.
#[derive(Default)]
struct SchemaAttributes {
    description: Option<String>,
    title: Option<String>,
    rename_all: Option<String>,
    no_recursion: bool,
    as_name: Option<String>,
    example: Option<AnyValue>,
    deprecated: bool,
    additional_properties: Option<bool>,
    bound: Option<syn::WherePredicate>,
}

impl SchemaAttributes {
    fn parse(attrs: &[syn::Attribute]) -> Result<Self> {
        let mut result = Self::default();

        for attr in attrs {
            if attr.path().is_ident("schema") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("description") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.description = Some(s.value());
                        }
                    } else if meta.path.is_ident("title") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.title = Some(s.value());
                        }
                    } else if meta.path.is_ident("rename_all") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.rename_all = Some(s.value());
                        }
                    } else if meta.path.is_ident("no_recursion") {
                        result.no_recursion = true;
                    } else if meta.path.is_ident("as") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.as_name = Some(s.value());
                        }
                    } else if meta.path.is_ident("example") {
                        result.example = Some(parse_utils::parse_next(meta.input, || {
                            AnyValue::parse_any(meta.input)
                        })?);
                    } else if meta.path.is_ident("deprecated") {
                        result.deprecated = true;
                    } else if meta.path.is_ident("additional_properties") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Bool(b) = lit {
                            result.additional_properties = Some(b.value);
                        }
                    } else if meta.path.is_ident("bound") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            // Parse the string as a where predicate
                            let predicate: syn::WherePredicate = s.parse()?;
                            result.bound = Some(predicate);
                        }
                    }
                    Ok(())
                })?;
            }
        }

        Ok(result)
    }
}

/// Field-level schema attributes.
#[derive(Default)]
struct FieldAttributes {
    description: Option<String>,
    rename: Option<String>,
    format: Option<String>,
    skip: bool,
    inline: bool,
    no_recursion: bool,
    schema_with: Option<syn::TypePath>,
    // Validation attributes
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
    // Default value
    default: Option<DefaultValue>,
    example: Option<AnyValue>,
    deprecated: bool,
    read_only: bool,
    write_only: bool,
    nullable: Option<bool>,
    value_type: Option<syn::Type>,
    title: Option<String>,
}

/// Represents different ways a default value can be specified
enum DefaultValue {
    /// Use Default::default() - from #[serde(default)]
    DefaultTrait,
    /// Explicit value - from #[schema(default = "value")]
    Explicit(AnyValue),
    /// Custom function - from #[serde(default = "path")]
    Function(syn::Path),
}

impl FieldAttributes {
    fn parse(attrs: &[syn::Attribute]) -> Result<Self> {
        let mut result = Self::default();

        for attr in attrs {
            if attr.path().is_ident("schema") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("description") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.description = Some(s.value());
                        }
                    } else if meta.path.is_ident("rename") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.rename = Some(s.value());
                        }
                    } else if meta.path.is_ident("format") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.format = Some(s.value());
                        }
                    } else if meta.path.is_ident("skip") {
                        result.skip = true;
                    } else if meta.path.is_ident("inline") {
                        result.inline = true;
                    } else if meta.path.is_ident("no_recursion") {
                        result.no_recursion = true;
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
                            result.exclusive_minimum = Some(b.value);
                        }
                    } else if meta.path.is_ident("exclusive_maximum") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Bool(b) = lit {
                            result.exclusive_maximum = Some(b.value);
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
                    } else if meta.path.is_ident("default") {
                        result.default = Some(DefaultValue::Explicit(parse_utils::parse_next(
                            meta.input,
                            || AnyValue::parse_any(meta.input),
                        )?));
                    } else if meta.path.is_ident("example") {
                        result.example = Some(parse_utils::parse_next(meta.input, || {
                            AnyValue::parse_any(meta.input)
                        })?);
                    } else if meta.path.is_ident("deprecated") {
                        result.deprecated = true;
                    } else if meta.path.is_ident("read_only") {
                        result.read_only = true;
                    } else if meta.path.is_ident("write_only") {
                        result.write_only = true;
                    } else if meta.path.is_ident("nullable") {
                        // Can be used with or without explicit value
                        if meta.input.peek(syn::Token![=]) {
                            let value = meta.value()?;
                            let lit: Lit = value.parse()?;
                            if let Lit::Bool(b) = lit {
                                result.nullable = Some(b.value);
                            }
                        } else {
                            result.nullable = Some(true);
                        }
                    } else if meta.path.is_ident("value_type") {
                        let value = meta.value()?;
                        result.value_type = Some(value.parse()?);
                    } else if meta.path.is_ident("title") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.title = Some(s.value());
                        }
                    }
                    Ok(())
                })?;
            } else if attr.path().is_ident("serde") {
                // Parse serde attributes for compatibility
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("skip") {
                        result.skip = true;
                    } else if meta.path.is_ident("rename") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.rename = Some(s.value());
                        }
                    } else if meta.path.is_ident("default") {
                        // Check if it has a value (custom function) or not (Default::default())
                        if meta.input.peek(syn::Token![=]) {
                            let value = meta.value()?;
                            let lit: Lit = value.parse()?;
                            if let Lit::Str(s) = lit {
                                // Parse the string as a path
                                let path: syn::Path = s.parse()?;
                                result.default = Some(DefaultValue::Function(path));
                            }
                        } else {
                            result.default = Some(DefaultValue::DefaultTrait);
                        }
                    }
                    Ok(())
                })?;
            }
        }

        Ok(result)
    }
}

/// Schema generator for the ToSchema derive macro.
pub struct Schema {
    input: DeriveInput,
    attributes: SchemaAttributes,
}

impl Schema {
    pub fn new(input: DeriveInput) -> Result<Self> {
        let attributes = SchemaAttributes::parse(&input.attrs)?;
        Ok(Self { input, attributes })
    }

    pub fn into_token_stream(self) -> TokenStream {
        let name = &self.input.ident;
        let (impl_generics, ty_generics, where_clause) = self.input.generics.split_for_impl();

        // Build where clause with trait bounds for generic parameters
        let mut where_clause = where_clause.map_or_else(|| syn::parse_quote!(where), |w| w.clone());

        // If bound attribute is provided, use it; otherwise add default ToSchema bounds
        if let Some(custom_bound) = &self.attributes.bound {
            // User provided custom bound, use it as-is
            where_clause.predicates.push(custom_bound.clone());
        } else {
            // Add default ToSchema bound for each generic type parameter
            for param in self.input.generics.type_params() {
                let param_ident = &param.ident;
                where_clause
                    .predicates
                    .push(syn::parse_quote!(#param_ident: ::utocli::ToSchema));
            }
        }

        // Generate schema type based on data structure
        let schema_impl = match &self.input.data {
            Data::Struct(data_struct) => self.generate_struct_schema(&data_struct.fields),
            Data::Enum(data_enum) => {
                // For now, generate simple enum schema
                self.generate_enum_schema(&data_enum.variants)
            }
            Data::Union(_) => {
                return Diagnostics::new("ToSchema cannot be derived for unions")
                    .help("ToSchema can only be derived for structs and enums")
                    .note("Union types are not supported by OpenCLI schema generation")
                    .into_token_stream();
            }
        };

        // Format schema name with generic parameters
        let schema_name_value = if let Some(as_name) = &self.attributes.as_name {
            // Use as_name if provided
            as_name.clone()
        } else {
            // Format name with generics: "Foo<T, U>" not just "Foo"
            let type_params: Vec<_> = self
                .input
                .generics
                .type_params()
                .map(|p| p.ident.to_string())
                .collect();

            if type_params.is_empty() {
                name.to_string()
            } else {
                // Build generic name string: "Response<T, U>"
                format!("{}<{}>", name, type_params.join(", "))
            }
        };

        // Check if this type has generic parameters
        let has_generics = !self.input.generics.params.is_empty();

        if has_generics {
            // For generic types, generate both ToSchema and ComposeSchema implementations
            quote! {
                impl #impl_generics ::utocli::ToSchema for #name #ty_generics #where_clause {
                    fn schema() -> ::utocli::Schema {
                        use ::utocli::{Schema, SchemaType, Object};

                        #schema_impl
                    }

                    fn schema_name() -> &'static str {
                        #schema_name_value
                    }
                }

                impl #impl_generics ::utocli::ComposeSchema for #name #ty_generics #where_clause {
                    fn compose(generics: ::std::vec::Vec<::utocli::RefOr<::utocli::Schema>>) -> ::utocli::RefOr<::utocli::Schema> {
                        use ::utocli::{Schema, SchemaType, Object, RefOr};

                        // Generate schema using composed generic schemas
                        // For now, return a reference to the schema name
                        // TODO: Properly compose the schema with generic parameter schemas
                        RefOr::new_ref(format!("#/components/schemas/{}", #schema_name_value))
                    }
                }
            }
        } else {
            // For non-generic types, only generate ToSchema
            quote! {
                impl #impl_generics ::utocli::ToSchema for #name #ty_generics #where_clause {
                    fn schema() -> ::utocli::Schema {
                        use ::utocli::{Schema, SchemaType, Object};

                        #schema_impl
                    }

                    fn schema_name() -> &'static str {
                        #schema_name_value
                    }
                }
            }
        }
    }

    fn generate_struct_schema(&self, fields: &Fields) -> TokenStream {
        match fields {
            Fields::Named(named_fields) => {
                let mut properties = Vec::new();
                let mut required = Vec::new();

                // Parse serde container attributes (following utoipa's exact pattern)
                let container_rules = serde::parse_container(&self.input.attrs).unwrap_or_default();

                // Container-level no_recursion flag
                let container_no_recursion = self.attributes.no_recursion;

                // Container-level rename_all from schema attributes
                let schema_rename_all = self
                    .attributes
                    .rename_all
                    .as_ref()
                    .and_then(|s| serde::RenameRule::from_str(s).ok());

                for field in &named_fields.named {
                    let mut field_attrs = FieldAttributes::parse(&field.attrs).unwrap_or_default();

                    // Parse serde field attributes (following utoipa's exact pattern)
                    let field_rules = serde::parse_value(&field.attrs).unwrap_or_default();

                    // Propagate container-level no_recursion to fields (like utoipa does)
                    if container_no_recursion {
                        field_attrs.no_recursion = true;
                    }

                    // Check both schema skip and serde skip (following utoipa's pattern)
                    if field_attrs.skip || field_rules.skip {
                        continue;
                    }

                    let field_name = field.ident.as_ref().unwrap();

                    // Apply rename precedence: serde rename > schema rename > original
                    // (following utoipa's exact pattern)
                    let field_name_str = if let Some(ref serde_rename) = field_rules.rename {
                        serde_rename.clone()
                    } else if let Some(ref schema_rename) = field_attrs.rename {
                        schema_rename.clone()
                    } else {
                        // Apply container-level rename_all if present
                        let name = field_name.to_string();
                        if let Some(rename_rule) = container_rules
                            .rename_all
                            .as_ref()
                            .or(schema_rename_all.as_ref())
                        {
                            rename_rule.apply(&name)
                        } else {
                            name
                        }
                    };

                    let ty = &field.ty;
                    let is_optional = is_option_type(ty);

                    // Use utoipa's is_required logic: considers skip_serializing_if, double_option, default
                    let component_required =
                        !is_optional && is_required(&field_rules, &container_rules);
                    if component_required {
                        required.push(field_name_str.clone());
                    }

                    // Use schema_with if provided, otherwise infer schema from field type
                    let mut schema_ref_or = if let Some(schema_with) = &field_attrs.schema_with {
                        // Call the custom schema function
                        quote! {
                            ::utocli::RefOr::T(#schema_with())
                        }
                    } else {
                        // Use value_type override if provided
                        let ty_to_use = field_attrs.value_type.as_ref().unwrap_or(ty);

                        // Infer schema from field type (handles Vec, primitives, etc.)
                        // Pass inline flag, no_recursion flag, and validations from field attributes
                        infer_schema_ref_or_with_validations(
                            ty_to_use,
                            field_attrs.inline,
                            field_attrs.no_recursion,
                            &field_attrs,
                        )
                    };

                    // Apply additional field-level attributes (following utoipa's pattern)
                    // These are applied as builder methods on the Object schema
                    let mut property_modifiers = Vec::new();

                    if let Some(ref example) = field_attrs.example {
                        property_modifiers.push(quote! {
                            .example(Some(#example))
                        });
                    }

                    if let Some(ref title) = field_attrs.title {
                        property_modifiers.push(quote! {
                            .title(Some(#title))
                        });
                    }

                    if field_attrs.deprecated {
                        property_modifiers.push(quote! {
                            .deprecated(Some(true))
                        });
                    }

                    if field_attrs.read_only {
                        property_modifiers.push(quote! {
                            .read_only(Some(true))
                        });
                    }

                    if field_attrs.write_only {
                        property_modifiers.push(quote! {
                            .write_only(Some(true))
                        });
                    }

                    if let Some(nullable) = field_attrs.nullable {
                        property_modifiers.push(quote! {
                            .nullable(#nullable)
                        });
                    }

                    // Apply modifiers if any exist
                    if !property_modifiers.is_empty() {
                        schema_ref_or = quote! {
                            {
                                match #schema_ref_or {
                                    ::utocli::RefOr::T(::utocli::Schema::Object(mut obj)) => {
                                        *obj = (*obj) #(#property_modifiers)*;
                                        ::utocli::RefOr::T(::utocli::Schema::Object(obj))
                                    },
                                    other => other,
                                }
                            }
                        };
                    }

                    properties.push(quote! {
                        (#field_name_str.to_string(), #schema_ref_or)
                    });
                }

                // Build Object with builder pattern
                let mut object_builder = quote! {
                    ::utocli::Object::new()
                        .schema_type(::utocli::SchemaType::Object)
                        .properties({
                            use ::utocli::Map;
                            Map::from_iter(vec![
                                #(#properties),*
                            ])
                        })
                };

                // Add required fields if any
                if !required.is_empty() {
                    object_builder.extend(quote! {
                        .required(vec![#(#required.to_string()),*])
                    });
                }

                // Add description if present
                if let Some(desc) = &self
                    .attributes
                    .description
                    .clone()
                    .or_else(|| parse_doc_comments(&self.input.attrs))
                {
                    object_builder.extend(quote! {
                        .description(#desc)
                    });
                }

                // Add title if present (container-level)
                if let Some(ref title) = self.attributes.title {
                    object_builder.extend(quote! {
                        .title(Some(#title))
                    });
                }

                // Add example if present (container-level)
                if let Some(ref example) = self.attributes.example {
                    object_builder.extend(quote! {
                        .example(Some(#example))
                    });
                }

                // Add deprecated if present (container-level)
                if self.attributes.deprecated {
                    object_builder.extend(quote! {
                        .deprecated(Some(true))
                    });
                }

                // Add additional_properties if specified (container-level)
                if let Some(additional_properties) = self.attributes.additional_properties
                    && !additional_properties
                {
                    object_builder.extend(quote! {
                        .additional_properties(Some(false))
                    });
                }

                quote! {
                    ::utocli::Schema::Object(Box::new(#object_builder))
                }
            }
            Fields::Unnamed(unnamed_fields) => {
                self.generate_unnamed_struct_schema(&unnamed_fields.unnamed)
            }
            Fields::Unit => self.generate_unit_struct_schema(),
        }
    }

    fn generate_unnamed_struct_schema(
        &self,
        fields: &syn::punctuated::Punctuated<syn::Field, syn::Token![,]>,
    ) -> TokenStream {
        let fields_len = fields.len();

        if fields_len == 0 {
            return self.generate_unit_struct_schema();
        }

        let first_field = fields.first().unwrap();
        let first_ty = &first_field.ty;

        // Check if all fields are the same type (following utoipa's exact logic from line 30684)
        let all_fields_are_same = fields_len == 1
            || fields.iter().skip(1).all(|field| {
                let field_ty = &field.ty;
                // Compare types by their token representation (simplified from utoipa's TypeTree comparison)
                quote!(#first_ty).to_string() == quote!(#field_ty).to_string()
            });

        let mut tokens = TokenStream::new();

        if all_fields_are_same {
            // Single field or all same type: inline the type's schema
            // Following utoipa's pattern from lines 30701-30768

            // For single field, check for inline attribute on the field
            // For now, simplified: always inline for unnamed structs
            let schema_ref_or = infer_schema_ref_or(first_ty, false, self.attributes.no_recursion);

            // Unwrap RefOr to get Schema (utoipa does this via ComponentSchema)
            tokens.extend(quote! {
                match #schema_ref_or {
                    ::utocli::RefOr::T(s) => s,
                    ::utocli::RefOr::Ref(_) => {
                        // For references, generate generic object schema
                        ::utocli::Schema::Object(Box::new(
                            ::utocli::Object::new()
                                .schema_type(::utocli::SchemaType::Object)
                        ))
                    }
                }
            });
        } else {
            // Multiple fields with different types: serialize as array (serde default)
            // Following utoipa's pattern from lines 30769-30779
            tokens.extend(quote! {
                ::utocli::Schema::Object(Box::new(
                    ::utocli::Object::new()
                        .schema_type(::utocli::SchemaType::Array)
                ))
            });
        }

        // Add description if present
        if let Some(desc) = &self
            .attributes
            .description
            .clone()
            .or_else(|| parse_doc_comments(&self.input.attrs))
        {
            tokens = quote! {
                {
                    match #tokens {
                        ::utocli::Schema::Object(mut obj) => {
                            obj.description = Some(#desc.to_string());
                            ::utocli::Schema::Object(obj)
                        }
                        other => other,
                    }
                }
            };
        }

        tokens
    }

    fn generate_unit_struct_schema(&self) -> TokenStream {
        let mut object_builder = quote! {
            ::utocli::Object::new()
                .schema_type(::utocli::SchemaType::String)
        };

        if let Some(desc) = &self
            .attributes
            .description
            .clone()
            .or_else(|| parse_doc_comments(&self.input.attrs))
        {
            object_builder.extend(quote! {
                .description(#desc)
            });
        }

        quote! {
            ::utocli::Schema::Object(Box::new(#object_builder))
        }
    }

    fn generate_enum_schema(
        &self,
        variants: &syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
    ) -> TokenStream {
        // Determine if this is a plain enum (all unit variants) or mixed enum
        let is_plain = variants.iter().all(|v| matches!(v.fields, Fields::Unit));

        let root = Root::new(&self.input.ident, &self.input.attrs);

        let rename_all = self.attributes.rename_all.as_ref().and_then(|r| {
            // Parse rename_all string into RenameRule
            match r.as_str() {
                "lowercase" => Some(serde::RenameRule::Lowercase),
                "UPPERCASE" => Some(serde::RenameRule::Uppercase),
                "PascalCase" => Some(serde::RenameRule::PascalCase),
                "camelCase" => Some(serde::RenameRule::CamelCase),
                "snake_case" => Some(serde::RenameRule::SnakeCase),
                "SCREAMING_SNAKE_CASE" => Some(serde::RenameRule::ScreamingSnakeCase),
                "kebab-case" => Some(serde::RenameRule::KebabCase),
                "SCREAMING-KEBAB-CASE" => Some(serde::RenameRule::ScreamingKebabCase),
                _ => None,
            }
        });

        if is_plain {
            // Use PlainEnum for unit variants only
            match PlainEnum::new(&root, variants, rename_all) {
                Ok(plain_enum) => {
                    let mut schema = plain_enum.to_token_stream();

                    // Add description if present
                    if let Some(desc) = &self
                        .attributes
                        .description
                        .clone()
                        .or_else(|| parse_doc_comments(&self.input.attrs))
                    {
                        // Wrap schema to add description
                        schema = quote! {
                            {
                                match #schema {
                                    ::utocli::Schema::Object(mut obj) => {
                                        obj.description = Some(#desc.to_string());
                                        ::utocli::Schema::Object(obj)
                                    }
                                    other => other,
                                }
                            }
                        };
                    }

                    schema
                }
                Err(err) => err.to_compile_error(),
            }
        } else {
            // Use MixedEnum for enums with field variants
            match MixedEnum::new(&root, variants) {
                Ok(mixed_enum) => {
                    let mut schema = mixed_enum.to_token_stream();

                    // Add description if present
                    if let Some(desc) = &self
                        .attributes
                        .description
                        .clone()
                        .or_else(|| parse_doc_comments(&self.input.attrs))
                    {
                        // Wrap schema to add description
                        schema = quote! {
                            {
                                match #schema {
                                    ::utocli::Schema::Object(mut obj) => {
                                        obj.description = Some(#desc.to_string());
                                        ::utocli::Schema::Object(obj)
                                    }
                                    other => other,
                                }
                            }
                        };
                    }

                    schema
                }
                Err(err) => err.to_compile_error(),
            }
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

/// Extract inner type from `Option<T>`.
fn get_option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return Some(inner_ty);
    }
    None
}

/// Check if a type is `Vec<T>`.
fn is_vec_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Vec";
    }
    false
}

/// Extract inner type from `Vec<T>`.
fn get_vec_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Vec"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return Some(inner_ty);
    }
    None
}

/// Infer schema RefOr from Rust type with validation attributes. Returns RefOr<Schema> tokens.
/// This is a wrapper around `infer_schema_ref_or` that applies field-level validations and default.
fn infer_schema_ref_or_with_validations(
    ty: &syn::Type,
    inline: bool,
    no_recursion: bool,
    field_attrs: &FieldAttributes,
) -> TokenStream {
    // First get the base schema
    let base_schema = infer_schema_ref_or(ty, inline, no_recursion);

    // Check if we have any validations or default value to apply
    let has_validations = field_attrs.minimum.is_some()
        || field_attrs.maximum.is_some()
        || field_attrs.min_length.is_some()
        || field_attrs.max_length.is_some()
        || field_attrs.pattern.is_some()
        || field_attrs.multiple_of.is_some()
        || field_attrs.exclusive_minimum.is_some()
        || field_attrs.exclusive_maximum.is_some()
        || field_attrs.max_properties.is_some()
        || field_attrs.min_properties.is_some()
        || field_attrs.min_items.is_some()
        || field_attrs.max_items.is_some();

    let has_default = field_attrs.default.is_some();

    if !has_validations && !has_default {
        return base_schema;
    }

    // Build validation and default method calls
    let mut method_calls = Vec::new();

    if let Some(min) = field_attrs.minimum {
        method_calls.push(quote! { .minimum(#min) });
    }
    if let Some(max) = field_attrs.maximum {
        method_calls.push(quote! { .maximum(#max) });
    }
    if let Some(min_len) = field_attrs.min_length {
        method_calls.push(quote! { .min_length(#min_len) });
    }
    if let Some(max_len) = field_attrs.max_length {
        method_calls.push(quote! { .max_length(#max_len) });
    }
    if let Some(ref pattern) = field_attrs.pattern {
        method_calls.push(quote! { .pattern(#pattern) });
    }
    if let Some(mult) = field_attrs.multiple_of {
        method_calls.push(quote! { .multiple_of(#mult) });
    }
    if let Some(excl_min) = field_attrs.exclusive_minimum {
        method_calls.push(quote! { .exclusive_minimum(#excl_min) });
    }
    if let Some(excl_max) = field_attrs.exclusive_maximum {
        method_calls.push(quote! { .exclusive_maximum(#excl_max) });
    }
    if let Some(max_props) = field_attrs.max_properties {
        method_calls.push(quote! { .max_properties(#max_props) });
    }
    if let Some(min_props) = field_attrs.min_properties {
        method_calls.push(quote! { .min_properties(#min_props) });
    }
    // Note: min_items and max_items would only be applied to Array schemas
    // For Object schemas they are ignored (matching utoipa architecture)

    // Add default value if specified (but not for DefaultTrait from #[serde(default)])
    // Note: #[serde(default)] only affects required status, not the actual default value in schema
    if let Some(ref default) = field_attrs.default {
        match default {
            DefaultValue::DefaultTrait => {
                // Skip - #[serde(default)] is only used for required field determination
                // We don't generate Default::default() as it would require the type to implement Default
            }
            DefaultValue::Explicit(any_value) => {
                // AnyValue::to_tokens already wraps in serde_json::json!()
                method_calls.push(quote! { .default_value(#any_value) });
            }
            DefaultValue::Function(path) => {
                let default_expr = quote! { serde_json::json!(#path()) };
                method_calls.push(quote! { .default_value(#default_expr) });
            }
        }
    }

    // For inline schemas, we can apply validations and defaults directly
    // For references, we can't modify them, so they are ignored
    // This matches utoipa's behavior
    quote! {
        {
            match #base_schema {
                ::utocli::RefOr::T(::utocli::Schema::Object(obj)) => {
                    ::utocli::RefOr::T(::utocli::Schema::Object(
                        Box::new((*obj) #(#method_calls)*)
                    ))
                },
                other => other,
            }
        }
    }
}

/// Infer schema RefOr from Rust type. Returns RefOr<Schema> tokens.
/// For primitive types, returns `RefOr::T(Schema::...)`.
/// For custom types (structs/enums), returns `RefOr::Ref(Ref { ... })` unless `inline` is true.
/// When `no_recursion` is true, custom types won't generate inline schemas to prevent infinite loops.
fn infer_schema_ref_or(ty: &syn::Type, inline: bool, no_recursion: bool) -> TokenStream {
    use crate::type_tree::TypeTree;

    // Use TypeTree for proper generic analysis
    let type_tree = match TypeTree::from_type(ty) {
        Ok(tree) => tree,
        Err(_) => {
            // Fallback for unsupported types
            return quote! {
                ::utocli::RefOr::T(::utocli::Schema::Object(Box::new(
                    ::utocli::Object::new()
                        .schema_type(::utocli::SchemaType::String)
                )))
            };
        }
    };

    // Unwrap Option<T> to get inner type - use old helpers for now to get actual syn::Type
    let actual_ty = if type_tree.is_option() && is_option_type(ty) {
        get_option_inner_type(ty).unwrap_or(ty)
    } else {
        ty
    };

    // Check for Vec<T> using TypeTree - propagate no_recursion to inner type
    if type_tree.is_vec()
        || (type_tree.is_option() && type_tree.get_wrapped_type().is_some_and(|t| t.is_vec()))
    {
        // Get the actual Vec type (might be wrapped in Option)
        let vec_ty = if type_tree.is_option() {
            get_option_inner_type(ty).unwrap_or(ty)
        } else {
            actual_ty
        };

        if is_vec_type(vec_ty)
            && let Some(inner_ty) = get_vec_inner_type(vec_ty)
        {
            let inner_ref_or = infer_schema_ref_or(inner_ty, inline, no_recursion);
            return quote! {
                ::utocli::RefOr::T(::utocli::Schema::Array(
                    ::utocli::opencli::Array::new()
                        .items(#inner_ref_or)
                ))
            };
        }
    }

    // Extract type identifier for primitive and custom types
    if let syn::Type::Path(type_path) = actual_ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let type_name = segment.ident.to_string();

        return match type_name.as_str() {
            // Primitive types - wrap in RefOr::T (no_recursion doesn't apply)
            "i8" | "i16" | "i32" | "isize" | "i64" | "u8" | "u16" | "u32" | "usize" | "u64"
            | "f32" | "f64" | "bool" | "String" | "str" => {
                let schema = infer_schema_inline(actual_ty);
                quote! { ::utocli::RefOr::T(#schema) }
            }
            // For custom types (structs/enums), handle no_recursion
            _ => {
                if no_recursion {
                    // When no_recursion is set, don't generate inline schema
                    // Just use a reference - this breaks the recursion cycle
                    let ref_path = format!("#/components/schemas/{}", type_name);
                    quote! {
                        ::utocli::RefOr::Ref(::utocli::Ref {
                            ref_path: #ref_path.to_string()
                        })
                    }
                } else if inline {
                    // Generate inline schema by calling the type's schema() method
                    let type_ident = &segment.ident;
                    quote! {
                        ::utocli::RefOr::T(#type_ident::schema())
                    }
                } else {
                    // Generate reference
                    let ref_path = format!("#/components/schemas/{}", type_name);
                    quote! {
                        ::utocli::RefOr::Ref(::utocli::Ref {
                            ref_path: #ref_path.to_string()
                        })
                    }
                }
            }
        };
    }

    // Default fallback - string schema wrapped in RefOr::T
    let schema = infer_schema_inline(actual_ty);
    quote! { ::utocli::RefOr::T(#schema) }
}

/// Infer inline schema from Rust type. Returns Schema tokens (not RefOr).
/// Only handles primitive types - custom types should use `infer_schema_ref_or` instead.
fn infer_schema_inline(ty: &syn::Type) -> TokenStream {
    // Extract type identifier for primitive types
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let type_name = segment.ident.to_string();

        return match type_name.as_str() {
            // Integer types
            "i8" | "i16" | "i32" | "isize" => quote! {
                ::utocli::Schema::Object(Box::new(
                    ::utocli::Object::new()
                        .schema_type(::utocli::SchemaType::Integer)
                        .format(::utocli::SchemaFormat::Int32)
                ))
            },
            "i64" => quote! {
                ::utocli::Schema::Object(Box::new(
                    ::utocli::Object::new()
                        .schema_type(::utocli::SchemaType::Integer)
                        .format(::utocli::SchemaFormat::Int64)
                ))
            },
            "u8" | "u16" | "u32" | "usize" => quote! {
                ::utocli::Schema::Object(Box::new(
                    ::utocli::Object::new()
                        .schema_type(::utocli::SchemaType::Integer)
                        .format(::utocli::SchemaFormat::Int32)
                ))
            },
            "u64" => quote! {
                ::utocli::Schema::Object(Box::new(
                    ::utocli::Object::new()
                        .schema_type(::utocli::SchemaType::Integer)
                        .format(::utocli::SchemaFormat::Int64)
                ))
            },
            // Float types
            "f32" => quote! {
                ::utocli::Schema::Object(Box::new(
                    ::utocli::Object::new()
                        .schema_type(::utocli::SchemaType::Number)
                        .format(::utocli::SchemaFormat::Float)
                ))
            },
            "f64" => quote! {
                ::utocli::Schema::Object(Box::new(
                    ::utocli::Object::new()
                        .schema_type(::utocli::SchemaType::Number)
                        .format(::utocli::SchemaFormat::Double)
                ))
            },
            // Boolean type
            "bool" => quote! {
                ::utocli::Schema::Object(Box::new(
                    ::utocli::Object::new()
                        .schema_type(::utocli::SchemaType::Boolean)
                ))
            },
            // String types
            "String" | "str" => quote! {
                ::utocli::Schema::Object(Box::new(
                    ::utocli::Object::new()
                        .schema_type(::utocli::SchemaType::String)
                ))
            },
            // Unknown types default to string
            _ => quote! {
                ::utocli::Schema::Object(Box::new(
                    ::utocli::Object::new()
                        .schema_type(::utocli::SchemaType::String)
                ))
            },
        };
    }

    // Default fallback
    quote! {
        ::utocli::Schema::Object(Box::new(
            ::utocli::Object::new()
                .schema_type(::utocli::SchemaType::String)
        ))
    }
}

impl ToTokens for Schema {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.clone().into_token_stream());
    }
}

// Needed for ToTokens
impl Clone for Schema {
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            attributes: SchemaAttributes {
                description: self.attributes.description.clone(),
                title: self.attributes.title.clone(),
                rename_all: self.attributes.rename_all.clone(),
                no_recursion: self.attributes.no_recursion,
                as_name: self.attributes.as_name.clone(),
                example: self.attributes.example.clone(),
                deprecated: self.attributes.deprecated,
                additional_properties: self.attributes.additional_properties,
                bound: self.attributes.bound.clone(),
            },
        }
    }
}
