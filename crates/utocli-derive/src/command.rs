//! Command attribute macro for generating OpenCLI command definitions.
//!
//! This module implements the `#[command(...)]` attribute macro, which is parallel
//! to utoipa's `#[utoipa::path(...)]` macro. It allows decorating functions to generate
//! OpenCLI command specifications.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Ident, ItemFn, Lit, Result as SynResult, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
};

use crate::{diagnostics::Diagnostics, doc_comment::parse_doc_comments};

/// Parsed command attributes from `#[command(...)]`.
#[derive(Default)]
struct CommandAttributes {
    name: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    operation_id: Option<String>,
    aliases: Vec<String>,
    tags: Vec<String>,
    parameters: Vec<ParameterDef>,
    responses: Vec<ResponseDef>,
    extensions: Vec<(String, String)>,
}

#[derive(Clone, Default)]
struct ParameterDef {
    name: String,
    in_: Option<String>, // None means use default (option)
    position: Option<u32>,
    description: Option<String>,
    required: bool,
    scope: String,
    schema_type: String,
    schema_format: Option<String>,
    enum_values: Vec<String>,
    default_value: Option<String>,
    example: Option<String>,
    arity_min: Option<u32>,
    arity_max: Option<u32>,
    alias: Vec<String>,
    extensions: Vec<(String, String)>,
}

impl Parse for ParameterDef {
    fn parse(input: ParseStream) -> SynResult<Self> {
        const EXPECTED_ATTRIBUTE: &str = "unexpected attribute, expected any of: name, in, position, description, required, scope, schema_type, schema_format, enum_values, default, example, arity_min, arity_max, alias, extend";

        let mut param = ParameterDef {
            required: false,                   // default
            scope: "local".to_string(),        // default
            schema_type: "string".to_string(), // default
            ..Default::default()
        };

        // Parse parameter tuple: (name = "file", in = "argument", ...)
        let content;
        syn::parenthesized!(content in input);

        while !content.is_empty() {
            // Check for the 'in' keyword first (it's a Rust keyword)
            let attribute_name = if content.peek(Token![in]) {
                content.parse::<Token![in]>()?;
                "in".to_string()
            } else {
                let ident = content.parse::<Ident>().map_err(|error| -> syn::Error {
                    Diagnostics::with_span(
                        error.span(),
                        format!("{EXPECTED_ATTRIBUTE}, {error}"),
                    )
                    .help("Valid parameter attributes: name, in, position, description, required, alias, extend")
                    .note("Example: (name = \"file\", in = \"argument\", position = 1)")
                    .into()
                })?;
                ident.to_string()
            };

            match attribute_name.as_str() {
                "name" => {
                    content.parse::<Token![=]>()?;
                    let lit: Lit = content.parse()?;
                    if let Lit::Str(s) = lit {
                        param.name = s.value();
                    }
                }
                "in" => {
                    content.parse::<Token![=]>()?;
                    let lit: Lit = content.parse()?;
                    if let Lit::Str(s) = lit {
                        param.in_ = Some(s.value());
                    }
                }
                "position" => {
                    content.parse::<Token![=]>()?;
                    let lit: Lit = content.parse()?;
                    if let Lit::Int(i) = lit {
                        param.position = Some(i.base10_parse()?);
                    }
                }
                "description" => {
                    content.parse::<Token![=]>()?;
                    let lit: Lit = content.parse()?;
                    if let Lit::Str(s) = lit {
                        param.description = Some(s.value());
                    }
                }
                "required" => {
                    content.parse::<Token![=]>()?;
                    let lit: Lit = content.parse()?;
                    if let Lit::Bool(b) = lit {
                        param.required = b.value();
                    }
                }
                "scope" => {
                    content.parse::<Token![=]>()?;
                    let lit: Lit = content.parse()?;
                    if let Lit::Str(s) = lit {
                        param.scope = s.value();
                    }
                }
                "schema_type" => {
                    content.parse::<Token![=]>()?;
                    let lit: Lit = content.parse()?;
                    if let Lit::Str(s) = lit {
                        param.schema_type = s.value();
                    }
                }
                "schema_format" => {
                    content.parse::<Token![=]>()?;
                    let lit: Lit = content.parse()?;
                    if let Lit::Str(s) = lit {
                        param.schema_format = Some(s.value());
                    }
                }
                "default" => {
                    content.parse::<Token![=]>()?;
                    let lit: Lit = content.parse()?;
                    param.default_value = Some(match lit {
                        Lit::Str(s) => s.value(),
                        Lit::Bool(b) => b.value().to_string(),
                        Lit::Int(i) => i.base10_digits().to_string(),
                        _ => String::new(),
                    });
                }
                "alias" => {
                    // Parse alias: alias("s") or alias = "s"
                    if content.peek(syn::token::Paren) {
                        let alias_content;
                        syn::parenthesized!(alias_content in content);
                        let items: Punctuated<Lit, Comma> =
                            alias_content.parse_terminated(Lit::parse, Token![,])?;
                        for item in items {
                            if let Lit::Str(s) = item {
                                param.alias.push(s.value());
                            }
                        }
                    } else {
                        content.parse::<Token![=]>()?;
                        let lit: Lit = content.parse()?;
                        if let Lit::Str(s) = lit {
                            param.alias.push(s.value());
                        }
                    }
                }
                "enum_values" => {
                    // Parse enum_values("json", "yaml", "text")
                    let enum_content;
                    syn::parenthesized!(enum_content in content);
                    let items: Punctuated<Lit, Comma> =
                        enum_content.parse_terminated(Lit::parse, Token![,])?;
                    for item in items {
                        if let Lit::Str(s) = item {
                            param.enum_values.push(s.value());
                        }
                    }
                }
                "example" => {
                    content.parse::<Token![=]>()?;
                    let lit: Lit = content.parse()?;
                    if let Lit::Str(s) = lit {
                        param.example = Some(s.value());
                    }
                }
                "arity_min" => {
                    content.parse::<Token![=]>()?;
                    let lit: Lit = content.parse()?;
                    if let Lit::Int(i) = lit {
                        param.arity_min = Some(i.base10_parse()?);
                    }
                }
                "arity_max" => {
                    content.parse::<Token![=]>()?;
                    let lit: Lit = content.parse()?;
                    if let Lit::Int(i) = lit {
                        param.arity_max = Some(i.base10_parse()?);
                    }
                }
                "extend" => {
                    // Parse extensions: extend(x_completion = "file")
                    let ext_content;
                    syn::parenthesized!(ext_content in content);
                    while !ext_content.is_empty() {
                        let key: Ident = ext_content.parse()?;
                        ext_content.parse::<Token![=]>()?;
                        let value: Lit = ext_content.parse()?;
                        if let Lit::Str(s) = value {
                            let ext_key = if key.to_string().starts_with("x_") {
                                key.to_string().replace('_', "-")
                            } else {
                                format!("x-{}", key.to_string().replace('_', "-"))
                            };
                            param.extensions.push((ext_key, s.value()));
                        }
                        if !ext_content.is_empty() {
                            ext_content.parse::<Token![,]>()?;
                        }
                    }
                }
                _ => {
                    return Err(Diagnostics::with_span(content.span(), EXPECTED_ATTRIBUTE)
                        .help("Valid parameter attributes are listed above")
                        .note("Example: (name = \"file\", in = \"argument\", position = 1)")
                        .into());
                }
            }

            // Check for comma separator
            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(param)
    }
}

#[derive(Clone, Default)]
struct ResponseDef {
    status: String,
    description: String,
    content: Vec<ContentDef>,
}

impl Parse for ResponseDef {
    fn parse(input: ParseStream) -> SynResult<Self> {
        const EXPECTED_ATTRIBUTE: &str =
            "unexpected attribute, expected any of: status, description, content";
        const EXPECTED_ATTRIBUTE_MESSAGE: &str = EXPECTED_ATTRIBUTE;
        let mut response = ResponseDef::default();

        // Parse response tuple: (status = "0", description = "Success", content(...))
        let content;
        syn::parenthesized!(content in input);

        while !content.is_empty() {
            let ident = content.parse::<Ident>().map_err(|error| -> syn::Error {
                Diagnostics::with_span(
                    error.span(),
                    format!("{EXPECTED_ATTRIBUTE_MESSAGE}, {error}"),
                )
                .help("Valid response attributes: status, description, content")
                .note("Example: (status = \"0\", description = \"Success\")")
                .into()
            })?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "status" => {
                    content.parse::<Token![=]>()?;
                    let lit: Lit = content.parse()?;
                    if let Lit::Str(s) = lit {
                        response.status = s.value();
                    }
                }
                "description" => {
                    content.parse::<Token![=]>()?;
                    let lit: Lit = content.parse()?;
                    if let Lit::Str(s) = lit {
                        response.description = s.value();
                    }
                }
                "content" => {
                    // Parse content: content((media_type = "...", ...), ...)
                    let content_list;
                    syn::parenthesized!(content_list in content);
                    let contents: Punctuated<ContentDef, Token![,]> =
                        Punctuated::parse_terminated(&content_list)?;
                    response.content = contents.into_iter().collect();
                }
                _ => {
                    return Err(Diagnostics::with_span(ident.span(), EXPECTED_ATTRIBUTE)
                        .help("Valid response attributes are listed above")
                        .note("Example: (status = \"0\", description = \"Success\", content(...))")
                        .into());
                }
            }

            // Check for comma separator
            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(response)
    }
}

#[derive(Clone, Default)]
struct ContentDef {
    media_type: String,
    schema_ref: Option<String>,
    example: Option<String>,
    inline_props: Vec<(String, String)>, // (property_name, property_type)
}

impl Parse for ContentDef {
    fn parse(input: ParseStream) -> SynResult<Self> {
        const EXPECTED_ATTRIBUTE: &str =
            "unexpected attribute, expected any of: media_type, schema, example, inline_properties";
        let mut content = ContentDef::default();

        // Parse content tuple: (media_type = "application/json", schema = "...", example = "...")
        let content_inner;
        syn::parenthesized!(content_inner in input);

        while !content_inner.is_empty() {
            let ident = content_inner
                .parse::<Ident>()
                .map_err(|error| -> syn::Error {
                    Diagnostics::with_span(error.span(), format!("{EXPECTED_ATTRIBUTE}, {error}"))
                    .help(
                        "Valid content attributes: media_type, schema, example, inline_properties",
                    )
                    .note("Example: (media_type = \"application/json\", schema = \"OutputSchema\")")
                    .into()
                })?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "media_type" => {
                    content_inner.parse::<Token![=]>()?;
                    let lit: Lit = content_inner.parse()?;
                    if let Lit::Str(s) = lit {
                        content.media_type = s.value();
                    }
                }
                "schema" => {
                    content_inner.parse::<Token![=]>()?;
                    let lit: Lit = content_inner.parse()?;
                    if let Lit::Str(s) = lit {
                        content.schema_ref = Some(s.value());
                    }
                }
                "example" => {
                    content_inner.parse::<Token![=]>()?;
                    let lit: Lit = content_inner.parse()?;
                    if let Lit::Str(s) = lit {
                        content.example = Some(s.value());
                    }
                }
                "inline_properties" => {
                    // Parse inline_properties(("prop1", "type1"), ("prop2", "type2"))
                    let props_content;
                    syn::parenthesized!(props_content in content_inner);

                    while !props_content.is_empty() {
                        // Parse each property tuple
                        let prop_tuple;
                        syn::parenthesized!(prop_tuple in props_content);

                        // Parse property name
                        let name_lit: Lit = prop_tuple.parse()?;
                        let prop_name = if let Lit::Str(s) = name_lit {
                            s.value()
                        } else {
                            String::new()
                        };

                        prop_tuple.parse::<Token![,]>()?;

                        // Parse property type
                        let type_lit: Lit = prop_tuple.parse()?;
                        let prop_type = if let Lit::Str(s) = type_lit {
                            s.value()
                        } else {
                            String::new()
                        };

                        content.inline_props.push((prop_name, prop_type));

                        // Check for comma separator between properties
                        if !props_content.is_empty() {
                            props_content.parse::<Token![,]>()?;
                        }
                    }
                }
                _ => {
                    return Err(Diagnostics::with_span(
                        ident.span(),
                        EXPECTED_ATTRIBUTE,
                    )
                    .help("Valid content attributes are listed above")
                    .note("Example: content((media_type = \"application/json\", schema = \"OutputSchema\"))")
                    .into());
                }
            }

            // Check for comma separator
            if !content_inner.is_empty() {
                content_inner.parse::<Token![,]>()?;
            }
        }

        Ok(content)
    }
}

/// Parser for command attributes
impl Parse for CommandAttributes {
    fn parse(input: ParseStream) -> SynResult<Self> {
        const EXPECTED_ATTRIBUTE: &str = "unexpected attribute, expected any of: name, summary, description, operation_id, aliases, tags, parameters, responses, extend";
        let mut attrs = CommandAttributes::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| -> syn::Error {
                Diagnostics::with_span(
                    error.span(),
                    format!("{EXPECTED_ATTRIBUTE}, {error}"),
                )
                .help("Valid command attributes: name, summary, description, operation_id, aliases, tags, parameters, responses, extend")
                .note("Example: #[command(name = \"build\", summary = \"Build the project\")]")
                .into()
            })?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "name" => {
                    input.parse::<Token![=]>()?;
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        attrs.name = Some(s.value());
                    }
                }
                "summary" => {
                    input.parse::<Token![=]>()?;
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        attrs.summary = Some(s.value());
                    }
                }
                "description" => {
                    input.parse::<Token![=]>()?;
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        attrs.description = Some(s.value());
                    }
                }
                "operation_id" => {
                    input.parse::<Token![=]>()?;
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        attrs.operation_id = Some(s.value());
                    }
                }
                "aliases" => {
                    // Parse list: aliases("val", "check")
                    let content;
                    syn::parenthesized!(content in input);
                    let items: Punctuated<Lit, Comma> =
                        content.parse_terminated(Lit::parse, Token![,])?;
                    for item in items {
                        if let Lit::Str(s) = item {
                            attrs.aliases.push(s.value());
                        }
                    }
                }
                "tags" => {
                    // Parse list: tags("core", "validation")
                    let content;
                    syn::parenthesized!(content in input);
                    let items: Punctuated<Lit, Comma> =
                        content.parse_terminated(Lit::parse, Token![,])?;
                    for item in items {
                        if let Lit::Str(s) = item {
                            attrs.tags.push(s.value());
                        }
                    }
                }
                "extend" => {
                    // Parse extensions: extend(x_cli_category = "validation")
                    let content;
                    syn::parenthesized!(content in input);
                    while !content.is_empty() {
                        let key: Ident = content.parse()?;
                        content.parse::<Token![=]>()?;
                        let value: Lit = content.parse()?;
                        if let Lit::Str(s) = value {
                            let ext_key = if key.to_string().starts_with("x_") {
                                key.to_string().replace('_', "-")
                            } else {
                                format!("x-{}", key.to_string().replace('_', "-"))
                            };
                            attrs.extensions.push((ext_key, s.value()));
                        }
                        if !content.is_empty() {
                            content.parse::<Token![,]>()?;
                        }
                    }
                }
                "parameters" => {
                    // Parse parameters: parameters(...)
                    let content;
                    syn::parenthesized!(content in input);
                    attrs.parameters = Self::parse_parameters_list(&content)?;
                }
                "responses" => {
                    // Parse responses: responses(...)
                    let content;
                    syn::parenthesized!(content in input);
                    attrs.responses = Self::parse_responses_list(&content)?;
                }
                _ => {
                    return Err(Diagnostics::with_span(
                        ident.span(),
                        EXPECTED_ATTRIBUTE,
                    )
                    .help("Valid command attributes are listed above")
                    .note("Example: #[command(name = \"build\", summary = \"Build the project\")]")
                    .into());
                }
            }

            // Check for comma separator
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(attrs)
    }
}

impl CommandAttributes {
    fn parse_parameters_list(input: ParseStream) -> SynResult<Vec<ParameterDef>> {
        // Parse list of parameter tuples: ((name = "file", ...), (name = "strict", ...))
        let params: Punctuated<ParameterDef, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(params.into_iter().collect())
    }

    fn parse_responses_list(input: ParseStream) -> SynResult<Vec<ResponseDef>> {
        // Parse list of response tuples: ((status = "0", ...), (status = "1", ...))
        let responses: Punctuated<ResponseDef, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(responses.into_iter().collect())
    }
}

/// Generate tokens for parameter creation
fn generate_parameters_tokens(parameters: &[ParameterDef]) -> TokenStream {
    if parameters.is_empty() {
        return quote! {};
    }

    let param_builders: Vec<TokenStream> = parameters
        .iter()
        .map(|param| {
            let name = &param.name;
            let in_ = &param.in_;
            let scope = &param.scope;
            let schema_type = &param.schema_type;

            let description_tokens = if let Some(desc) = &param.description {
                quote! { .description(#desc) }
            } else {
                quote! {}
            };

            let required_tokens = if param.required {
                quote! { .required(true) }
            } else {
                quote! {}
            };

            let position_tokens = if let Some(pos) = param.position {
                quote! { .position(#pos) }
            } else {
                quote! {}
            };

            let aliases_tokens = if !param.alias.is_empty() {
                let aliases = &param.alias;
                quote! { .alias(vec![#(#aliases.to_string()),*]) }
            } else {
                quote! {}
            };

            let schema_format_tokens = if let Some(format) = &param.schema_format {
                // Convert format string to enum variant (e.g., "path" -> "Path")
                let format_ident = syn::Ident::new(
                    &(format
                        .chars()
                        .next()
                        .unwrap()
                        .to_uppercase()
                        .collect::<String>()
                        + &format[1..]),
                    proc_macro2::Span::call_site(),
                );
                quote! { .format(SchemaFormat::#format_ident) }
            } else {
                quote! {}
            };

            let enum_tokens = if !param.enum_values.is_empty() {
                let enums = &param.enum_values;
                quote! { .enum_values(vec![#(::serde_json::Value::String(#enums.to_string())),*]) }
            } else {
                quote! {}
            };

            let default_tokens = if let Some(default) = &param.default_value {
                // Parse the default value to determine if it's a boolean, number, or string
                let default_value_tokens = if default == "true" || default == "false" {
                    let bool_val = default == "true";
                    quote! { ::serde_json::Value::Bool(#bool_val) }
                } else if default.parse::<i64>().is_ok() {
                    let num: i64 = default.parse().unwrap();
                    quote! { ::serde_json::Value::Number(::serde_json::Number::from(#num)) }
                } else {
                    quote! { ::serde_json::Value::String(#default.to_string()) }
                };
                quote! { .default_value(#default_value_tokens) }
            } else {
                quote! {}
            };

            let example_tokens = if let Some(example) = &param.example {
                quote! {
                    .example(
                        // Try to parse as JSON first, fall back to string
                        match ::serde_json::from_str::<::serde_json::Value>(#example) {
                            Ok(json_value) => json_value,
                            Err(_) => ::serde_json::Value::String(#example.to_string()),
                        }
                    )
                }
            } else {
                quote! {}
            };

            let arity_tokens = if param.arity_min.is_some() || param.arity_max.is_some() {
                let min_tokens = if let Some(min) = param.arity_min {
                    quote! { .min(#min) }
                } else {
                    quote! {}
                };
                let max_tokens = if let Some(max) = param.arity_max {
                    quote! { .max(#max) }
                } else {
                    quote! {}
                };
                quote! {
                    {
                        param = param.arity(
                            ::utocli::Arity::new()
                                #min_tokens
                                #max_tokens
                        );
                    }
                }
            } else {
                quote! {}
            };

            let extensions_tokens = if !param.extensions.is_empty() {
                let ext_keys: Vec<_> = param.extensions.iter().map(|(k, _)| k).collect();
                let ext_values: Vec<_> = param.extensions.iter().map(|(_, v)| v).collect();
                quote! {
                    {
                        let mut exts = ::utocli::Map::new();
                        #(
                            exts.insert(
                                #ext_keys.to_string(),
                                ::serde_json::Value::String(#ext_values.to_string())
                            );
                        )*
                        param = param.extensions(exts);
                    }
                }
            } else {
                quote! {}
            };

            let schema_type_ident = syn::Ident::new(
                &(schema_type
                    .chars()
                    .next()
                    .unwrap()
                    .to_uppercase()
                    .collect::<String>()
                    + &schema_type[1..]),
                proc_macro2::Span::call_site(),
            );

            let scope_ident = syn::Ident::new(
                &(scope
                    .chars()
                    .next()
                    .unwrap()
                    .to_uppercase()
                    .collect::<String>()
                    + &scope[1..]),
                proc_macro2::Span::call_site(),
            );

            // Convert in_ string to enum variant (e.g., "argument" -> "Argument")
            let in_tokens = if let Some(in_str) = in_ {
                let in_ident = syn::Ident::new(
                    &(in_str
                        .chars()
                        .next()
                        .unwrap()
                        .to_uppercase()
                        .collect::<String>()
                        + &in_str[1..]),
                    proc_macro2::Span::call_site(),
                );
                quote! { .in_(::utocli::ParameterIn::#in_ident) }
            } else {
                quote! {}
            };

            quote! {
                {
                    let schema = Schema::Object(Box::new(
                        Object::new()
                            .schema_type(SchemaType::#schema_type_ident)
                            #schema_format_tokens
                            #enum_tokens
                            #default_tokens
                            #example_tokens
                    ));

                    let mut param = Parameter::new(#name)
                        #in_tokens
                        .scope(ParameterScope::#scope_ident)
                        .schema(RefOr::T(schema))
                        #description_tokens
                        #required_tokens
                        #position_tokens
                        #aliases_tokens;

                    #arity_tokens
                    #extensions_tokens

                    param
                }
            }
        })
        .collect();

    quote! {
        command = command.parameters(vec![
            #(#param_builders),*
        ]);
    }
}

/// Generate tokens for response creation
fn generate_responses_tokens(responses: &[ResponseDef]) -> TokenStream {
    if responses.is_empty() {
        return quote! {};
    }

    let response_builders: Vec<TokenStream> = responses.iter().map(|resp| {
        let status = &resp.status;
        let description = &resp.description;

        let content_tokens = if !resp.content.is_empty() {
            let content_builders: Vec<TokenStream> = resp.content.iter().map(|content| {
                let media_type = &content.media_type;

                let schema_tokens = if !content.inline_props.is_empty() {
                    // Generate inline schema with properties
                    // First, try to infer schemas from example if available
                    let (example_json_opt, example_str_var) = if let Some(ref example_str) = content.example {
                        // Try to parse example as JSON to infer schemas
                        (quote! {
                            let example_str = #example_str;
                            let example_json_opt = ::serde_json::from_str::<::serde_json::Value>(example_str).ok();
                        }, quote! { example_str })
                    } else {
                        (quote! {
                            let example_json_opt: Option<::serde_json::Value> = None;
                            let example_str = "";
                        }, quote! { example_str })
                    };

                    let prop_builders: Vec<TokenStream> = content.inline_props.iter().map(|(name, type_str)| {
                        // Check if it's an array type (e.g., "array<string>")
                        if type_str.starts_with("array<") && type_str.ends_with(">") {
                            let item_type = &type_str[6..type_str.len()-1]; // Extract type between < and >
                            let item_type_ident = syn::Ident::new(
                                &(item_type
                                    .chars()
                                    .next()
                                    .unwrap()
                                    .to_uppercase()
                                    .collect::<String>()
                                    + &item_type[1..]),
                                proc_macro2::Span::call_site(),
                            );

                            quote! {
                                props.insert(
                                    #name.to_string(),
                                    RefOr::T(Schema::Array(
                                        Array::new()
                                            .items(RefOr::T(Schema::Object(Box::new(
                                                Object::new().schema_type(SchemaType::#item_type_ident)
                                            ))))
                                    ))
                                );
                            }
                        } else if type_str == "array" {
                            // Plain array - try to infer items from example
                            quote! {
                                props.insert(
                                    #name.to_string(),
                                    if let Some(ref example_json) = example_json_opt {
                                        // Try to infer array items schema from example
                                        if let Some(array_val) = example_json.get(#name) {
                                            if let Some(array) = array_val.as_array() {
                                                if let Some(first_item) = array.first() {
                                                    if let Some(obj) = first_item.as_object() {
                                                        // Infer object properties from first array element
                                                        // Parse original JSON string to extract property order
                                                        let mut item_props = ::utocli::Map::new();

                                                        // Extract property names in order from original JSON string
                                                        if let Some(array_start) = #example_str_var.find('[') {
                                                            if let Some(obj_start) = #example_str_var[array_start..].find('{') {
                                                                let obj_str = &#example_str_var[array_start + obj_start..];
                                                                if let Some(obj_end) = obj_str.find('}') {
                                                                    let obj_content = &obj_str[1..obj_end];
                                                                    // Simple regex-like extraction of "key":value pairs to preserve order
                                                                    let mut property_order = Vec::new();
                                                                    for part in obj_content.split(',') {
                                                                        if let Some(colon_pos) = part.find(':') {
                                                                            let key_part = &part[..colon_pos].trim();
                                                                            if let Some(quote_start) = key_part.find('"') {
                                                                                if let Some(quote_end) = key_part[quote_start+1..].find('"') {
                                                                                    let key = &key_part[quote_start+1..quote_start+1+quote_end];
                                                                                    property_order.push(key.to_string());
                                                                                }
                                                                            }
                                                                        }
                                                                    }

                                                                    // Insert properties in the order they appear in JSON
                                                                    for key in property_order {
                                                                        if let Some(value) = obj.get(&key) {
                                                                            let schema_type = match value {
                                                                                ::serde_json::Value::String(_) => SchemaType::String,
                                                                                ::serde_json::Value::Number(_) => SchemaType::Number,
                                                                                ::serde_json::Value::Bool(_) => SchemaType::Boolean,
                                                                                ::serde_json::Value::Array(_) => SchemaType::Array,
                                                                                ::serde_json::Value::Object(_) => SchemaType::Object,
                                                                                ::serde_json::Value::Null => SchemaType::String,
                                                                            };
                                                                            item_props.insert(
                                                                                key,
                                                                                RefOr::T(Schema::Object(Box::new(
                                                                                    Object::new().schema_type(schema_type)
                                                                                )))
                                                                            );
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        RefOr::T(Schema::Array(
                                                            Array::new()
                                                                .items(RefOr::T(Schema::Object(Box::new(
                                                                    Object::new()
                                                                        .schema_type(SchemaType::Object)
                                                                        .properties(item_props)
                                                                ))))
                                                        ))
                                                    } else {
                                                        // First item is not an object, create generic array
                                                        RefOr::T(Schema::Array(Array::new()))
                                                    }
                                                } else {
                                                    // Empty array, create generic array
                                                    RefOr::T(Schema::Array(Array::new()))
                                                }
                                            } else {
                                                // Not an array in example, create generic array
                                                RefOr::T(Schema::Array(Array::new()))
                                            }
                                        } else {
                                            // Property not found in example, create generic array
                                            RefOr::T(Schema::Array(Array::new()))
                                        }
                                    } else {
                                        // No example available, create generic array
                                        RefOr::T(Schema::Array(Array::new()))
                                    }
                                );
                            }
                        } else {
                            // Convert type string to SchemaType enum variant
                            let type_ident = syn::Ident::new(
                                &(type_str
                                    .chars()
                                    .next()
                                    .unwrap()
                                    .to_uppercase()
                                    .collect::<String>()
                                    + &type_str[1..]),
                                proc_macro2::Span::call_site(),
                            );

                            quote! {
                                props.insert(
                                    #name.to_string(),
                                    RefOr::T(Schema::Object(Box::new(
                                        Object::new().schema_type(SchemaType::#type_ident)
                                    )))
                                );
                            }
                        }
                    }).collect();

                    quote! {
                        media_type = media_type.schema(RefOr::T(Schema::Object(Box::new({
                            #example_json_opt
                            let mut props = ::utocli::Map::new();
                            #(#prop_builders)*
                            Object::new()
                                .schema_type(SchemaType::Object)
                                .properties(props)
                        }))));
                    }
                } else if let Some(schema_ref) = &content.schema_ref {
                    let ref_path = format!("#/components/schemas/{}", schema_ref);
                    quote! {
                        media_type = media_type.schema(RefOr::new_ref(#ref_path));
                    }
                } else {
                    quote! {}
                };

                let example_tokens = if let Some(example) = &content.example {
                    quote! {
                        media_type = media_type.example(
                            // Try to parse as JSON first, fall back to string
                            match ::serde_json::from_str::<::serde_json::Value>(#example) {
                                Ok(json_value) => json_value,
                                Err(_) => ::serde_json::Value::String(#example.to_string()),
                            }
                        );
                    }
                } else {
                    quote! {}
                };

                quote! {
                    {
                        let mut media_type = MediaType::new();
                        #schema_tokens
                        #example_tokens
                        (#media_type.to_string(), media_type)
                    }
                }
            }).collect();

            quote! {
                let response = {
                    let mut content = ::utocli::Map::new();
                    #(
                        let (key, value) = #content_builders;
                        content.insert(key, value);
                    )*
                    response.content(content)
                };
            }
        } else {
            quote! {}
        };

        quote! {
            {
                let response = Response::new()
                    .description(#description);
                #content_tokens
                (#status.to_string(), response)
            }
        }
    }).collect();

    quote! {
        {
            let mut responses = ::utocli::Map::new();
            #(
                let (status, response) = #response_builders;
                responses.insert(status, response);
            )*
            command = command.responses(responses);
        }
    }
}

/// Command attribute macro implementation.
pub fn command(args: TokenStream, input: ItemFn) -> Result<TokenStream, Diagnostics> {
    let attributes: CommandAttributes = syn::parse2(args).map_err(Diagnostics::from)?;

    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_asyncness = &input.sig.asyncness;
    let fn_generics = &input.sig.generics;
    let fn_attrs = &input.attrs;

    // Parse doc comments
    let doc_comments = parse_doc_comments(fn_attrs);
    let description = attributes.description.clone().or(doc_comments);

    let command_name = attributes
        .name
        .clone()
        .unwrap_or_else(|| fn_name.to_string().trim_end_matches("_command").to_string());

    let summary = attributes.summary.clone().unwrap_or_default();
    let operation_id = attributes.operation_id.clone();
    let aliases = &attributes.aliases;
    let tags = &attributes.tags;
    let extensions = &attributes.extensions;

    let description_tokens = if let Some(desc) = description {
        quote! { command = command.description(#desc); }
    } else {
        quote! {}
    };

    let operation_id_tokens = if let Some(op_id) = operation_id {
        quote! { command = command.operation_id(#op_id); }
    } else {
        quote! {}
    };

    let aliases_tokens = if !aliases.is_empty() {
        quote! {
            command = command.aliases(vec![#(#aliases.to_string()),*]);
        }
    } else {
        quote! {}
    };

    let tags_tokens = if !tags.is_empty() {
        quote! {
            command = command.tags(vec![#(#tags.to_string()),*]);
        }
    } else {
        quote! {}
    };

    let extensions_tokens = if !extensions.is_empty() {
        let ext_keys: Vec<_> = extensions.iter().map(|(k, _)| k).collect();
        let ext_values: Vec<_> = extensions.iter().map(|(_, v)| v).collect();
        quote! {
            {
                let mut exts = ::utocli::Map::new();
                #(
                    exts.insert(
                        #ext_keys.to_string(),
                        ::serde_json::Value::String(#ext_values.to_string())
                    );
                )*
                command = command.extensions(exts);
            }
        }
    } else {
        quote! {}
    };

    // Generate parameters tokens
    let parameters_tokens = generate_parameters_tokens(&attributes.parameters);

    // Generate responses tokens
    let responses_tokens = generate_responses_tokens(&attributes.responses);

    // Generate struct name following utoipa's exact pattern: __path_{fn_name}
    // We use __command_ prefix instead to match our domain
    // e.g., validate_command -> __command_validate_command
    let struct_name = quote::format_ident!("__command_{}", fn_name);

    Ok(quote! {
        #(#fn_attrs)*
        #fn_vis #fn_asyncness fn #fn_name #fn_generics(#fn_inputs) #fn_output {
            #fn_block
        }

        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        struct #struct_name;

        impl ::utocli::CommandPath for #struct_name {
            fn path() -> &'static str {
                #command_name
            }

            fn command() -> ::utocli::opencli::Command {
                use ::utocli::opencli::{Command, Parameter, ParameterScope, RefOr, Schema, Object, SchemaType, SchemaFormat, Response, MediaType, Map};

                let mut command = Command::new();
                command = command.summary(#summary);
                #description_tokens
                #operation_id_tokens
                #aliases_tokens
                #tags_tokens
                #extensions_tokens
                #parameters_tokens
                #responses_tokens

                command
            }
        }
    })
}
