//! Response generation for ToResponse and IntoResponses derive macros.
//!
//! This module follows the exact architecture of utoipa-gen/src/path/response/derive.rs,
//! with shared infrastructure for both ToResponse and IntoResponses derive macros.

use std::{borrow::Cow, mem};

use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Attribute, Data, Field, Fields, Generics, Lit, LitInt, LitStr, Token, Type, parse::ParseStream,
    punctuated::Punctuated, spanned::Spanned, token::Comma,
};

use crate::{
    AnyValue,
    diagnostics::{Diagnostics, ToTokensDiagnostics},
    doc_comment::parse_doc_comments,
};

/// Parse helpers for response attributes.
/// Matches utoipa-gen/src/path/response/derive.rs lines 27869-27892
mod parse {
    use syn::parse::ParseStream;

    use crate::{AnyValue, parse_utils};

    /// Parse example value from `#[response(example = ...)]`.
    /// Accepts literal strings or any macro/expression.
    #[inline]
    pub(super) fn example(input: ParseStream) -> syn::Result<AnyValue> {
        parse_utils::parse_next(input, || AnyValue::parse_lit_str_or_expr(input))
    }
}

/// Content attributes from `#[content(...)]`.
///
/// This is an utocli-specific extension to support multiple content types (e.g., JSON and text)
/// for CLI responses. Each field with a `#[content(...)]` attribute represents a different
/// media type for the response.
#[derive(Default)]
struct ContentAttributes {
    media_type: Option<String>,
    schema: Option<String>,
    example: Option<String>,
}

impl ContentAttributes {
    fn parse(attrs: &[syn::Attribute]) -> Result<Self, Diagnostics> {
        let mut result = Self::default();

        for attr in attrs {
            if attr.path().is_ident("content") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("media_type") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.media_type = Some(s.value());
                        }
                    } else if meta.path.is_ident("schema") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.schema = Some(s.value());
                        }
                    } else if meta.path.is_ident("example") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.example = Some(s.value());
                        }
                    }
                    Ok(())
                })
                .map_err(Diagnostics::from)?;
            }
        }

        Ok(result)
    }
}

/// Trait for parsing response attribute values from `#[response(...)]`.
///
/// This trait is implemented by both `DeriveToResponseValue` and `DeriveIntoResponsesValue`
/// to provide a unified interface for parsing and merging response attributes.
trait DeriveResponseValue: syn::parse::Parse {
    fn merge_from(self, other: Self) -> Self;

    fn from_attributes(attributes: &[Attribute]) -> Result<Option<Self>, Diagnostics> {
        Ok(attributes
            .iter()
            .filter(|attribute| {
                attribute
                    .path()
                    .get_ident()
                    .map(|ident| ident == "response")
                    .unwrap_or(false)
            })
            .map(|attribute| attribute.parse_args::<Self>().map_err(Diagnostics::from))
            .collect::<Result<Vec<_>, Diagnostics>>()?
            .into_iter()
            .reduce(|acc, item| acc.merge_from(item)))
    }
}

/// Response status code (exit code for CLI).
///
/// Utoipa uses HTTP status codes (200, 404, etc.), we use CLI exit codes ("0", "1", "2", etc.).
#[derive(Default)]
pub(crate) struct ResponseStatus(TokenStream);

impl syn::parse::Parse for ResponseStatus {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse as literal integer or string
        let lookahead = input.lookahead1();
        if lookahead.peek(LitInt) {
            let lit = input.parse::<LitInt>()?;
            Ok(Self(lit.to_token_stream()))
        } else if lookahead.peek(LitStr) {
            let lit = input.parse::<LitStr>()?;
            Ok(Self(lit.to_token_stream()))
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for ResponseStatus {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

/// Parsed representation of response tuple with status code and inner content.
///
/// This mirrors utoipa's `ResponseTuple` structure exactly.
#[derive(Default)]
pub struct ResponseTuple<'r> {
    pub status_code: ResponseStatus,
    pub inner: Option<ResponseTupleInner<'r>>,
}

impl<'r> From<(ResponseStatus, ResponseValue)> for ResponseTuple<'r> {
    fn from((status_code, response_value): (ResponseStatus, ResponseValue)) -> Self {
        ResponseTuple {
            inner: Some(ResponseTupleInner::Value(response_value)),
            status_code,
        }
    }
}

impl<'r> From<ResponseValue> for ResponseTuple<'r> {
    fn from(value: ResponseValue) -> Self {
        ResponseTuple {
            inner: Some(ResponseTupleInner::Value(value)),
            ..Default::default()
        }
    }
}

/// Inner content of a response tuple.
///
/// This mirrors utoipa's `ResponseTupleInner` enum exactly.
pub enum ResponseTupleInner<'r> {
    Value(ResponseValue),
    Ref(ParsedType<'r>),
}

/// Parsed type reference with inline flag.
///
/// This mirrors utoipa's `ParsedType` structure.
pub struct ParsedType<'r> {
    pub ty: Cow<'r, Type>,
    pub is_inline: bool,
}

/// Response value with description and content.
///
/// This mirrors utoipa's `ResponseValue` structure, adapted for CLI (no headers, links).
#[derive(Default)]
#[allow(dead_code)]
pub struct ResponseValue {
    pub description: Option<String>,
    pub content_type: Option<String>,
    /// Example value for the response (no Ident needed here, that's only in parsing).
    /// After extraction from DeriveToResponseValue/DeriveIntoResponsesValue, only AnyValue is stored.
    pub example: Option<AnyValue>,
    /// Content map: media_type -> (schema, example)
    pub content: Vec<(String, Option<String>, Option<String>)>, // (media_type, schema, example)
}

impl ResponseValue {
    fn from_derive_to_response_value(
        derive_value: DeriveToResponseValue,
        description: Option<String>,
    ) -> Self {
        ResponseValue {
            description: if derive_value.description.is_some() {
                derive_value.description
            } else {
                description
            },
            content_type: derive_value.content_type,
            // Extract AnyValue from tuple, discarding Ident
            // Matches utoipa-gen/src/path/response/derive.rs line 37853
            example: derive_value.example.map(|(example, _)| example),
            content: Vec::new(),
        }
    }

    fn from_derive_to_response_value_with_content(
        derive_value: DeriveToResponseValue,
        description: Option<String>,
        content: Vec<(String, Option<String>, Option<String>)>,
    ) -> Self {
        ResponseValue {
            description: if derive_value.description.is_some() {
                derive_value.description
            } else {
                description
            },
            content_type: derive_value.content_type,
            // Extract AnyValue from tuple, discarding Ident
            example: derive_value.example.map(|(example, _)| example),
            content,
        }
    }

    fn from_derive_into_responses_value(
        response_value: DeriveIntoResponsesValue,
        description: Option<String>,
    ) -> Self {
        ResponseValue {
            description: if response_value.description.is_some() {
                response_value.description
            } else {
                description
            },
            content_type: response_value.content_type,
            // Extract AnyValue from tuple, discarding Ident
            // Matches utoipa-gen/src/path/response/derive.rs line 37883
            example: response_value.example.map(|(example, _)| example),
            content: Vec::new(),
        }
    }
}

impl ToTokensDiagnostics for ResponseTuple<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        match self.inner.as_ref() {
            Some(ResponseTupleInner::Ref(res)) => {
                let path = &res.ty;
                if res.is_inline {
                    tokens.extend(quote! {
                        <#path as ::utocli::ToResponse>::response().1
                    });
                } else {
                    tokens.extend(quote! {
                        {
                            let (name, _) = <#path as ::utocli::ToResponse>::response();
                            ::utocli::opencli::RefOr::Ref(::utocli::Ref {
                                ref_path: format!("#/components/responses/{}", name)
                            })
                        }
                    });
                }
            }
            Some(ResponseTupleInner::Value(value)) => {
                let description = value
                    .description
                    .as_ref()
                    .map(|d| quote! { Some(#d.to_string()) })
                    .unwrap_or_else(|| quote! { None });

                let content = if value.content.is_empty() {
                    quote! { None }
                } else {
                    let content_entries = value.content.iter().map(|(media_type, schema, example)| {
                        let schema_ref = if let Some(schema_name) = schema {
                            quote! {
                                Some(::utocli::RefOr::Ref(::utocli::Ref {
                                    ref_path: format!("#/components/schemas/{}", #schema_name),
                                }))
                            }
                        } else {
                            quote! { None }
                        };

                        let example_value = if let Some(ex) = example {
                            quote! {
                                Some(
                                    serde_json::from_str(#ex)
                                        .unwrap_or_else(|_| serde_json::Value::String(#ex.to_string()))
                                )
                            }
                        } else {
                            quote! { None }
                        };

                        quote! {
                            (#media_type.to_string(), ::utocli::MediaType {
                                schema: #schema_ref,
                                example: #example_value,
                            })
                        }
                    });

                    quote! {
                        Some(::utocli::Map::from_iter(vec![
                            #(#content_entries),*
                        ]))
                    }
                };

                tokens.extend(quote! {
                    ::utocli::Response {
                        description: #description,
                        content: #content,
                    }
                });
            }
            None => {
                tokens.extend(quote! {
                    ::utocli::Response {
                        description: None,
                        content: None,
                    }
                });
            }
        }

        Ok(())
    }
}

/// Parsed response attributes from `#[response(...)]` for ToResponse derive.
///
/// This is used for the `ToResponse` derive macro (no status field).
/// Matches utoipa-gen/src/path/response/derive.rs lines 37989-37997
#[derive(Default)]
struct DeriveToResponseValue {
    content_type: Option<String>,
    description: Option<String>,
    /// Example value paired with the Ident for better error messages.
    /// Matches utoipa pattern from line 37995
    example: Option<(AnyValue, Ident)>,
}

impl DeriveResponseValue for DeriveToResponseValue {
    fn merge_from(mut self, other: Self) -> Self {
        if other.content_type.is_some() {
            self.content_type = other.content_type;
        }
        if other.description.is_some() {
            self.description = other.description;
        }
        if other.example.is_some() {
            self.example = other.example;
        }
        self
    }
}

impl syn::parse::Parse for DeriveToResponseValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut response = DeriveToResponseValue::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "description" => {
                    input.parse::<Token![=]>()?;
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        response.description = Some(s.value());
                    }
                }
                "content_type" => {
                    input.parse::<Token![=]>()?;
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        response.content_type = Some(s.value());
                    }
                }
                "example" => {
                    // Matches utoipa-gen/src/path/response/derive.rs line 38041
                    response.example = Some((parse::example(input)?, ident));
                }
                _ => {
                    return Err(Diagnostics::with_span(
                        ident.span(),
                        format!("unexpected attribute: {attribute_name}"),
                    )
                    .help("Valid attributes are: description, content_type, example")
                    .note("Example: #[response(description = \"Success\", content_type = \"application/json\")]")
                    .into());
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(response)
    }
}

/// Response generator for the ToResponse derive macro.
pub struct ToResponse {
    ident: Ident,
    generics: Generics,
    response: ResponseTuple<'static>,
}

impl ToResponse {
    pub fn new(
        attributes: Vec<Attribute>,
        data: &Data,
        generics: Generics,
        ident: Ident,
    ) -> Result<ToResponse, Diagnostics> {
        let response = match &data {
            Data::Struct(struct_value) => match &struct_value.fields {
                Fields::Named(fields) => {
                    ToResponseNamedStructResponse::new(&attributes, &ident, &fields.named)?.0
                }
                Fields::Unnamed(_fields) => ToResponseUnitStructResponse::new(&attributes)?.0,
                Fields::Unit => ToResponseUnitStructResponse::new(&attributes)?.0,
            },
            Data::Enum(_) => {
                return Err(
                    Diagnostics::new("ToResponse cannot be derived for enums yet")
                        .help("Use IntoResponses derive instead for enum types with multiple response variants")
                        .note("ToResponse is for single response types only"),
                );
            }
            Data::Union(_) => {
                return Err(Diagnostics::new("ToResponse cannot be derived for unions")
                    .help("ToResponse can only be derived for structs"));
            }
        };

        Ok(Self {
            ident,
            generics,
            response,
        })
    }
}

impl ToTokensDiagnostics for ToResponse {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        let name = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let response_tokens = self.response.try_to_token_stream()?;

        tokens.extend(quote! {
            impl<'r> #impl_generics ::utocli::ToResponse<'r> for #name #ty_generics #where_clause {
                fn response() -> (&'r str, ::utocli::RefOr<::utocli::Response>) {
                    (stringify!(#name), ::utocli::RefOr::T(#response_tokens))
                }
            }
        });

        Ok(())
    }
}

#[allow(dead_code)]
trait ResponseHelper {
    fn validate_attributes<'a, I: IntoIterator<Item = &'a Attribute>>(
        attributes: I,
        validate: impl Fn(&Attribute) -> (bool, &'static str) + 'a,
    ) -> impl Iterator<Item = Diagnostics> + 'a
    where
        <I as IntoIterator>::IntoIter: 'a,
    {
        attributes.into_iter().filter_map(move |attribute| {
            let (valid, error_message) = validate(attribute);
            if !valid {
                Some(Diagnostics::with_span(attribute.span(), error_message))
            } else {
                None
            }
        })
    }
}

struct ToResponseNamedStructResponse<'n>(ResponseTuple<'n>);

impl ResponseHelper for ToResponseNamedStructResponse<'_> {}

impl ToResponseNamedStructResponse<'_> {
    fn new(
        attributes: &[Attribute],
        _ident: &Ident,
        fields: &Punctuated<Field, Comma>,
    ) -> Result<Self, Diagnostics> {
        let derive_value = DeriveToResponseValue::from_attributes(attributes)?;
        let description = parse_doc_comments(attributes);

        // Parse field-level #[content(...)] attributes
        let mut content = Vec::new();
        for field in fields {
            let content_attrs = ContentAttributes::parse(&field.attrs)?;
            if let Some(media_type) = content_attrs.media_type {
                content.push((media_type, content_attrs.schema, content_attrs.example));
            }
        }

        let response_value = if content.is_empty() {
            ResponseValue::from_derive_to_response_value(
                derive_value.unwrap_or_default(),
                description,
            )
        } else {
            ResponseValue::from_derive_to_response_value_with_content(
                derive_value.unwrap_or_default(),
                description,
                content,
            )
        };

        Ok(Self(response_value.into()))
    }
}

struct ToResponseUnitStructResponse<'u>(ResponseTuple<'u>);

impl ResponseHelper for ToResponseUnitStructResponse<'_> {}

impl ToResponseUnitStructResponse<'_> {
    fn new(attributes: &[Attribute]) -> Result<Self, Diagnostics> {
        let derive_value = DeriveToResponseValue::from_attributes(attributes)?;
        let description = parse_doc_comments(attributes);

        let response_value = ResponseValue::from_derive_to_response_value(
            derive_value.unwrap_or_default(),
            description,
        );

        Ok(Self(response_value.into()))
    }
}

/// Parsed response attributes from `#[response(...)]` for IntoResponses derive.
///
/// This is used for the `IntoResponses` derive macro (has status field).
/// Matches utoipa-gen/src/path/response/derive.rs lines 38063-38071
#[derive(Default)]
struct DeriveIntoResponsesValue {
    status: ResponseStatus,
    content_type: Option<String>,
    description: Option<String>,
    /// Example value paired with the Ident for better error messages.
    /// Matches utoipa pattern from line 38069
    example: Option<(AnyValue, Ident)>,
}

impl DeriveResponseValue for DeriveIntoResponsesValue {
    fn merge_from(mut self, other: Self) -> Self {
        self.status = other.status;

        if other.content_type.is_some() {
            self.content_type = other.content_type;
        }
        if other.description.is_some() {
            self.description = other.description;
        }
        if other.example.is_some() {
            self.example = other.example;
        }

        self
    }
}

impl syn::parse::Parse for DeriveIntoResponsesValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut response = DeriveIntoResponsesValue::default();
        let first_span = input.span();

        let status_ident = input.parse::<Ident>().map_err(|error| {
            Diagnostics::with_span(error.span(), "missing expected `status` attribute")
                .help("IntoResponses requires a status code for each response variant")
                .note("Example: #[response(status = \"0\", description = \"Success\")]")
        })?;

        if status_ident == "status" {
            input.parse::<Token![=]>()?;
            response.status = input.parse::<ResponseStatus>()?;
        } else {
            return Err(Diagnostics::with_span(
                status_ident.span(),
                "missing expected `status` attribute",
            )
            .help("The first attribute must be `status = \"...\"`")
            .note("Example: #[response(status = \"0\", description = \"Success\")]")
            .into());
        }

        if response.status.to_token_stream().is_empty() {
            return Err(Diagnostics::with_span(
                first_span,
                "missing expected `status` attribute",
            )
            .help("Response status code cannot be empty")
            .note("CLI exit codes are typically: \"0\" (success), \"1\" (error), \"2\" (usage error)")
            .into());
        }

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "description" => {
                    input.parse::<Token![=]>()?;
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        response.description = Some(s.value());
                    }
                }
                "content_type" => {
                    input.parse::<Token![=]>()?;
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        response.content_type = Some(s.value());
                    }
                }
                "example" => {
                    // Matches utoipa-gen/src/path/response/derive.rs line 38137
                    response.example = Some((parse::example(input)?, ident));
                }
                _ => {
                    return Err(Diagnostics::with_span(
                        ident.span(),
                        format!("unexpected attribute: {attribute_name}"),
                    )
                    .help("Valid attributes are: description, content_type, example")
                    .note("Example: #[response(description = \"Success\", content_type = \"application/json\")]")
                    .into());
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(response)
    }
}

/// IntoResponses generator for the IntoResponses derive macro.
pub struct IntoResponses {
    pub attributes: Vec<Attribute>,
    pub data: Data,
    pub generics: Generics,
    pub ident: Ident,
}

impl ToTokensDiagnostics for IntoResponses {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        let responses = match &self.data {
            Data::Struct(struct_value) => match &struct_value.fields {
                Fields::Named(fields) => {
                    let response =
                        NamedStructResponse::new(&self.attributes, &self.ident, &fields.named)?.0;
                    let status = &response.status_code;
                    let response_tokens = response.try_to_token_stream()?;

                    vec![
                        quote!((#status.to_string(), ::utocli::opencli::RefOr::T(#response_tokens))),
                    ]
                }
                Fields::Unnamed(fields) => {
                    let field = fields
                        .unnamed
                        .iter()
                        .next()
                        .expect("Unnamed struct must have 1 field");

                    let response =
                        UnnamedStructResponse::new(&self.attributes, &field.ty, &field.attrs)?.0;
                    let status = &response.status_code;
                    let response_tokens = response.try_to_token_stream()?;

                    vec![
                        quote!((#status.to_string(), ::utocli::opencli::RefOr::T(#response_tokens))),
                    ]
                }
                Fields::Unit => {
                    let response = UnitStructResponse::new(&self.attributes)?.0;
                    let status = &response.status_code;
                    let response_tokens = response.try_to_token_stream()?;

                    vec![
                        quote!((#status.to_string(), ::utocli::opencli::RefOr::T(#response_tokens))),
                    ]
                }
            },
            Data::Enum(enum_value) => enum_value
                .variants
                .iter()
                .map(|variant| match &variant.fields {
                    Fields::Named(fields) => Ok(NamedStructResponse::new(
                        &variant.attrs,
                        &variant.ident,
                        &fields.named,
                    )?
                    .0),
                    Fields::Unnamed(fields) => {
                        let field = fields
                            .unnamed
                            .iter()
                            .next()
                            .expect("Unnamed enum variant must have 1 field");
                        UnnamedStructResponse::new(&variant.attrs, &field.ty, &field.attrs)
                            .map(|r| r.0)
                    }
                    Fields::Unit => Ok(UnitStructResponse::new(&variant.attrs)?.0),
                })
                .collect::<Result<Vec<ResponseTuple>, Diagnostics>>()?
                .iter()
                .map(|response| {
                    let status = &response.status_code;
                    let response_tokens = response.try_to_token_stream()?;
                    Ok(quote!((#status.to_string(), ::utocli::opencli::RefOr::T(#response_tokens))))
                })
                .collect::<Result<Vec<_>, Diagnostics>>()?,
            Data::Union(_) => {
                return Err(Diagnostics::with_span(
                    self.ident.span(),
                    "`IntoResponses` does not support `Union` type",
                )
                .help("IntoResponses can only be derived for enums with response variants")
                .note("Each enum variant represents a different response type with its own status code"));
            }
        };

        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        tokens.extend(quote! {
            impl #impl_generics ::utocli::IntoResponses for #ident #ty_generics #where_clause {
                fn responses() -> std::collections::BTreeMap<String, ::utocli::opencli::RefOr<::utocli::opencli::Response>> {
                    std::collections::BTreeMap::from_iter(vec![
                        #(#responses),*
                    ])
                }
            }
        });

        Ok(())
    }
}

#[allow(dead_code)]
trait Response {
    fn has_no_field_attributes(attribute: &Attribute) -> (bool, &'static str) {
        const ERROR: &str =
            "Unexpected field attribute, field attributes are only supported at unnamed fields";

        if let Some(ident) = attribute.path().get_ident() {
            match &*ident.to_string() {
                "to_schema" => (false, ERROR),
                "ref_response" => (false, ERROR),
                "content" => (false, ERROR),
                "to_response" => (false, ERROR),
                _ => (true, ERROR),
            }
        } else {
            (true, ERROR)
        }
    }

    fn validate_attributes<'a, I: IntoIterator<Item = &'a Attribute>>(
        attributes: I,
        validate: impl Fn(&Attribute) -> (bool, &'static str) + 'a,
    ) -> impl Iterator<Item = Diagnostics> + 'a
    where
        <I as IntoIterator>::IntoIter: 'a,
    {
        attributes.into_iter().filter_map(move |attribute| {
            let (valid, error_message) = validate(attribute);
            if !valid {
                Some(Diagnostics::with_span(attribute.span(), error_message))
            } else {
                None
            }
        })
    }
}

struct UnnamedStructResponse<'u>(ResponseTuple<'u>);

impl Response for UnnamedStructResponse<'_> {}

impl<'u> UnnamedStructResponse<'u> {
    fn new(
        attributes: &[Attribute],
        ty: &'u Type,
        inner_attributes: &[Attribute],
    ) -> Result<Self, Diagnostics> {
        let is_inline = inner_attributes.iter().any(|attribute| {
            attribute
                .path()
                .get_ident()
                .map(|i| i == "to_schema")
                .unwrap_or(false)
        });
        let ref_response = inner_attributes.iter().any(|attribute| {
            attribute
                .path()
                .get_ident()
                .map(|i| i == "ref_response")
                .unwrap_or(false)
        });
        let to_response = inner_attributes.iter().any(|attribute| {
            attribute
                .path()
                .get_ident()
                .map(|i| i == "to_response")
                .unwrap_or(false)
        });

        if is_inline && (ref_response || to_response) {
            return Err(Diagnostics::with_span(
                ty.span(),
                "Attribute `to_schema` cannot be used with `ref_response` and `to_response` attribute",
            )
            .help("Remove either `to_schema` or the conflicting attribute")
            .note("`to_schema` inlines the schema, while `ref_response` and `to_response` reference it"));
        }
        let mut derive_value = DeriveIntoResponsesValue::from_attributes(attributes)?
            .expect("`IntoResponses` must have `#[response(...)]` attribute");
        let description = parse_doc_comments(attributes);
        let status_code = mem::take(&mut derive_value.status);

        let response = match (ref_response, to_response) {
            (false, false) => Self(
                (
                    status_code,
                    ResponseValue::from_derive_into_responses_value(derive_value, description),
                )
                    .into(),
            ),
            (true, false) => Self(ResponseTuple {
                inner: Some(ResponseTupleInner::Ref(ParsedType {
                    ty: Cow::Borrowed(ty),
                    is_inline: false,
                })),
                status_code,
            }),
            (false, true) => Self(ResponseTuple {
                inner: Some(ResponseTupleInner::Ref(ParsedType {
                    ty: Cow::Borrowed(ty),
                    is_inline: true,
                })),
                status_code,
            }),
            (true, true) => {
                return Err(Diagnostics::with_span(
                    ty.span(),
                    "Cannot define `ref_response` and `to_response` attribute simultaneously",
                )
                .help("Choose either `ref_response` (schema reference) or `to_response` (inline schema)")
                .note("These attributes are mutually exclusive"));
            }
        };

        Ok(response)
    }
}

struct NamedStructResponse<'n>(ResponseTuple<'n>);

impl Response for NamedStructResponse<'_> {}

impl NamedStructResponse<'_> {
    fn new(
        attributes: &[Attribute],
        _ident: &Ident,
        _fields: &Punctuated<Field, Comma>,
    ) -> Result<Self, Diagnostics> {
        let mut derive_value = DeriveIntoResponsesValue::from_attributes(attributes)?
            .expect("`IntoResponses` must have `#[response(...)]` attribute");
        let description = parse_doc_comments(attributes);
        let status_code = mem::take(&mut derive_value.status);

        let response_value =
            ResponseValue::from_derive_into_responses_value(derive_value, description);

        Ok(Self((status_code, response_value).into()))
    }
}

struct UnitStructResponse<'u>(ResponseTuple<'u>);

impl Response for UnitStructResponse<'_> {}

impl UnitStructResponse<'_> {
    fn new(attributes: &[Attribute]) -> Result<Self, Diagnostics> {
        let mut derive_value = DeriveIntoResponsesValue::from_attributes(attributes)?
            .expect("`IntoResponses` must have `#[response(...)]` attribute");
        let status_code = mem::take(&mut derive_value.status);
        let description = parse_doc_comments(attributes);

        let response_value =
            ResponseValue::from_derive_into_responses_value(derive_value, description);

        Ok(Self((status_code, response_value).into()))
    }
}
