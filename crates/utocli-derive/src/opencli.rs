//! OpenCli generation for OpenCli derive macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Lit};

use crate::{
    diagnostics::{Diagnostics, ToTokensDiagnostics},
    doc_comment::parse_doc_comments,
};

/// Parsed OpenCli attributes from `#[opencli(...)]`.
#[derive(Default)]
struct OpenCliAttributes {
    info_title: Option<String>,
    info_version: Option<String>,
    info_description: Option<String>,
    info_contact: Option<ContactDef>,
    info_license: Option<LicenseDef>,
    external_docs: Option<ExternalDocsDef>,
    commands: Vec<syn::Path>,
    component_schemas: Vec<syn::Path>,
    component_parameters: Vec<syn::Path>,
    component_responses: Vec<syn::Path>,
    tags: Vec<TagDef>,
    platforms: Vec<PlatformDef>,
    environment: Vec<EnvVarDef>,
}

#[derive(Clone)]
struct TagDef {
    name: String,
    description: Option<String>,
}

#[derive(Clone)]
struct ContactDef {
    name: Option<String>,
    url: Option<String>,
    email: Option<String>,
}

#[derive(Clone)]
struct LicenseDef {
    name: String,
    url: Option<String>,
}

#[derive(Clone)]
struct ExternalDocsDef {
    url: String,
    description: Option<String>,
}

#[derive(Clone)]
struct PlatformDef {
    name: String,
    architectures: Vec<String>,
}

#[allow(dead_code)]
#[derive(Clone)]
struct EnvVarDef {
    name: String,
    description: Option<String>,
}

impl OpenCliAttributes {
    fn parse(attrs: &[syn::Attribute]) -> Result<Self, Diagnostics> {
        let mut result = Self::default();

        for attr in attrs {
            if attr.path().is_ident("opencli") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("info") {
                        // Parse info attributes
                        let content;
                        syn::parenthesized!(content in meta.input);

                        while !content.is_empty() {
                            let ident: syn::Ident = content.parse()?;

                            // Check if it's a nested object (contact, license) or a simple value
                            if content.peek(syn::token::Paren) {
                                // Nested object like contact(...) or license(...)
                                let nested_content;
                                syn::parenthesized!(nested_content in content);

                                if ident == "contact" {
                                    let mut contact_name: Option<String> = None;
                                    let mut contact_url: Option<String> = None;
                                    let mut contact_email: Option<String> = None;

                                    while !nested_content.is_empty() {
                                        let field: syn::Ident = nested_content.parse()?;
                                        let _: syn::Token![=] = nested_content.parse()?;
                                        let lit: Lit = nested_content.parse()?;

                                        if field == "name"
                                            && let Lit::Str(ref s) = lit
                                        {
                                            contact_name = Some(s.value());
                                        } else if field == "url"
                                            && let Lit::Str(ref s) = lit
                                        {
                                            contact_url = Some(s.value());
                                        } else if field == "email"
                                            && let Lit::Str(ref s) = lit
                                        {
                                            contact_email = Some(s.value());
                                        }

                                        if !nested_content.is_empty() {
                                            let _: syn::Token![,] = nested_content.parse()?;
                                        }
                                    }

                                    result.info_contact = Some(ContactDef {
                                        name: contact_name,
                                        url: contact_url,
                                        email: contact_email,
                                    });
                                } else if ident == "license" {
                                    let mut license_name: Option<String> = None;
                                    let mut license_url: Option<String> = None;

                                    while !nested_content.is_empty() {
                                        let field: syn::Ident = nested_content.parse()?;
                                        let _: syn::Token![=] = nested_content.parse()?;
                                        let lit: Lit = nested_content.parse()?;

                                        if field == "name"
                                            && let Lit::Str(ref s) = lit
                                        {
                                            license_name = Some(s.value());
                                        } else if field == "url"
                                            && let Lit::Str(ref s) = lit
                                        {
                                            license_url = Some(s.value());
                                        }

                                        if !nested_content.is_empty() {
                                            let _: syn::Token![,] = nested_content.parse()?;
                                        }
                                    }

                                    if let Some(name) = license_name {
                                        result.info_license = Some(LicenseDef {
                                            name,
                                            url: license_url,
                                        });
                                    }
                                }
                            } else {
                                // Simple key = value
                                let _: syn::Token![=] = content.parse()?;
                                let lit: Lit = content.parse()?;

                                if ident == "title"
                                    && let Lit::Str(ref s) = lit
                                {
                                    result.info_title = Some(s.value());
                                } else if ident == "version"
                                    && let Lit::Str(ref s) = lit
                                {
                                    result.info_version = Some(s.value());
                                } else if ident == "description"
                                    && let Lit::Str(ref s) = lit
                                {
                                    result.info_description = Some(s.value());
                                }
                            }

                            if !content.is_empty() {
                                let _: syn::Token![,] = content.parse()?;
                            }
                        }
                    } else if meta.path.is_ident("commands") {
                        // Parse command function paths
                        let content;
                        syn::parenthesized!(content in meta.input);

                        while !content.is_empty() {
                            let path: syn::Path = content.parse()?;
                            result.commands.push(path);

                            if !content.is_empty() {
                                let _: syn::Token![,] = content.parse()?;
                            }
                        }
                    } else if meta.path.is_ident("components") {
                        // Parse components
                        let content;
                        syn::parenthesized!(content in meta.input);

                        while !content.is_empty() {
                            let ident: syn::Ident = content.parse()?;
                            let inner_content;
                            syn::parenthesized!(inner_content in content);

                            if ident == "schemas" {
                                while !inner_content.is_empty() {
                                    let path: syn::Path = inner_content.parse()?;
                                    result.component_schemas.push(path);

                                    if !inner_content.is_empty() {
                                        let _: syn::Token![,] = inner_content.parse()?;
                                    }
                                }
                            } else if ident == "parameters" {
                                while !inner_content.is_empty() {
                                    let path: syn::Path = inner_content.parse()?;
                                    result.component_parameters.push(path);

                                    if !inner_content.is_empty() {
                                        let _: syn::Token![,] = inner_content.parse()?;
                                    }
                                }
                            } else if ident == "responses" {
                                while !inner_content.is_empty() {
                                    let path: syn::Path = inner_content.parse()?;
                                    result.component_responses.push(path);

                                    if !inner_content.is_empty() {
                                        let _: syn::Token![,] = inner_content.parse()?;
                                    }
                                }
                            }

                            if !content.is_empty() {
                                let _: syn::Token![,] = content.parse()?;
                            }
                        }
                    } else if meta.path.is_ident("tags") {
                        // Parse tags
                        let content;
                        syn::parenthesized!(content in meta.input);

                        while !content.is_empty() {
                            let tag_content;
                            syn::parenthesized!(tag_content in content);

                            let mut tag_name: Option<String> = None;
                            let mut tag_desc: Option<String> = None;

                            while !tag_content.is_empty() {
                                let ident: syn::Ident = tag_content.parse()?;
                                let _: syn::Token![=] = tag_content.parse()?;
                                let lit: Lit = tag_content.parse()?;

                                if ident == "name"
                                    && let Lit::Str(ref s) = lit
                                {
                                    tag_name = Some(s.value());
                                } else if ident == "description"
                                    && let Lit::Str(ref s) = lit
                                {
                                    tag_desc = Some(s.value());
                                }

                                if !tag_content.is_empty() {
                                    let _: syn::Token![,] = tag_content.parse()?;
                                }
                            }

                            if let Some(name) = tag_name {
                                result.tags.push(TagDef {
                                    name,
                                    description: tag_desc,
                                });
                            }

                            if !content.is_empty() {
                                let _: syn::Token![,] = content.parse()?;
                            }
                        }
                    } else if meta.path.is_ident("platforms") {
                        // Parse platforms
                        let content;
                        syn::parenthesized!(content in meta.input);

                        while !content.is_empty() {
                            // Check if it's a nested object with architectures or just a platform name
                            if content.peek(syn::token::Paren) {
                                // Platform with architectures: (name = "linux", architectures(amd64, arm64))
                                let platform_content;
                                syn::parenthesized!(platform_content in content);

                                let mut platform_name: Option<String> = None;
                                let mut architectures: Vec<String> = Vec::new();

                                while !platform_content.is_empty() {
                                    let ident: syn::Ident = platform_content.parse()?;

                                    if ident == "name" {
                                        let _: syn::Token![=] = platform_content.parse()?;
                                        let lit: Lit = platform_content.parse()?;
                                        if let Lit::Str(ref s) = lit {
                                            platform_name = Some(s.value());
                                        }
                                    } else if ident == "architectures" {
                                        let arch_content;
                                        syn::parenthesized!(arch_content in platform_content);

                                        while !arch_content.is_empty() {
                                            let arch: syn::Ident = arch_content.parse()?;
                                            architectures.push(arch.to_string());

                                            if !arch_content.is_empty() {
                                                let _: syn::Token![,] = arch_content.parse()?;
                                            }
                                        }
                                    }

                                    if !platform_content.is_empty() {
                                        let _: syn::Token![,] = platform_content.parse()?;
                                    }
                                }

                                if let Some(name) = platform_name {
                                    result.platforms.push(PlatformDef {
                                        name,
                                        architectures,
                                    });
                                }
                            } else {
                                // Simple platform name: linux, darwin, windows
                                let ident: syn::Ident = content.parse()?;
                                result.platforms.push(PlatformDef {
                                    name: ident.to_string(),
                                    architectures: Vec::new(),
                                });
                            }

                            if !content.is_empty() {
                                let _: syn::Token![,] = content.parse()?;
                            }
                        }
                    } else if meta.path.is_ident("external_docs") {
                        // Parse external docs
                        let content;
                        syn::parenthesized!(content in meta.input);

                        let mut url: Option<String> = None;
                        let mut description: Option<String> = None;

                        while !content.is_empty() {
                            let ident: syn::Ident = content.parse()?;
                            let _: syn::Token![=] = content.parse()?;
                            let lit: Lit = content.parse()?;

                            if ident == "url"
                                && let Lit::Str(ref s) = lit
                            {
                                url = Some(s.value());
                            } else if ident == "description"
                                && let Lit::Str(ref s) = lit
                            {
                                description = Some(s.value());
                            }

                            if !content.is_empty() {
                                let _: syn::Token![,] = content.parse()?;
                            }
                        }

                        if let Some(url_str) = url {
                            result.external_docs = Some(ExternalDocsDef {
                                url: url_str,
                                description,
                            });
                        }
                    } else if meta.path.is_ident("environment") {
                        // Parse environment variables
                        let content;
                        syn::parenthesized!(content in meta.input);

                        while !content.is_empty() {
                            let env_content;
                            syn::parenthesized!(env_content in content);

                            let mut env_name: Option<String> = None;
                            let mut env_desc: Option<String> = None;

                            while !env_content.is_empty() {
                                let ident: syn::Ident = env_content.parse()?;
                                let _: syn::Token![=] = env_content.parse()?;
                                let lit: Lit = env_content.parse()?;

                                if ident == "name"
                                    && let Lit::Str(ref s) = lit
                                {
                                    env_name = Some(s.value());
                                } else if ident == "description"
                                    && let Lit::Str(ref s) = lit
                                {
                                    env_desc = Some(s.value());
                                }

                                if !env_content.is_empty() {
                                    let _: syn::Token![,] = env_content.parse()?;
                                }
                            }

                            if let Some(name) = env_name {
                                result.environment.push(EnvVarDef {
                                    name,
                                    description: env_desc,
                                });
                            }

                            if !content.is_empty() {
                                let _: syn::Token![,] = content.parse()?;
                            }
                        }
                    }
                    Ok(())
                })?;
            }
        }

        Ok(result)
    }
}

/// OpenCli generator for the ToOpenCli derive macro.
pub struct OpenCli {
    input: DeriveInput,
    attributes: OpenCliAttributes,
}

impl OpenCli {
    pub fn new(input: DeriveInput) -> Result<Self, Diagnostics> {
        let attributes = OpenCliAttributes::parse(&input.attrs)?;
        Ok(Self { input, attributes })
    }
}

impl ToTokensDiagnostics for OpenCli {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        let name = &self.input.ident;
        let (impl_generics, ty_generics, where_clause) = self.input.generics.split_for_impl();

        // Generate info
        let info_title = self
            .attributes
            .info_title
            .clone()
            .unwrap_or_else(|| "CLI Application".to_string());
        let info_version = self
            .attributes
            .info_version
            .clone()
            .unwrap_or_else(|| "1.0.0".to_string());
        let info_description = self
            .attributes
            .info_description
            .clone()
            .or_else(|| parse_doc_comments(&self.input.attrs));

        let info_desc_tokens = if let Some(desc) = info_description {
            quote! { .description(#desc) }
        } else {
            quote! {}
        };

        let info_contact_tokens = if let Some(contact) = &self.attributes.info_contact {
            let contact_builder = {
                let mut tokens = quote! { ::utocli::Contact::new() };
                if let Some(name) = &contact.name {
                    tokens.extend(quote! { .name(#name) });
                }
                if let Some(url) = &contact.url {
                    tokens.extend(quote! { .url(#url) });
                }
                if let Some(email) = &contact.email {
                    tokens.extend(quote! { .email(#email) });
                }
                tokens
            };
            quote! { .contact(#contact_builder) }
        } else {
            quote! {}
        };

        let info_license_tokens = if let Some(license) = &self.attributes.info_license {
            let name = &license.name;
            let license_builder = if let Some(url) = &license.url {
                quote! { ::utocli::License::new(#name).url(#url) }
            } else {
                quote! { ::utocli::License::new(#name) }
            };
            quote! { .license(#license_builder) }
        } else {
            quote! {}
        };

        // Generate commands
        // Commands are now direct references to functions with #[command] macro
        // We generate structs like __command_{fn_name} that implement CommandPath
        let command_paths = &self.attributes.commands;
        let commands_tokens = if command_paths.is_empty() {
            quote! { ::utocli::Commands::new() }
        } else {
            // For each command path, we need to:
            // 1. Get the generated struct name (__command_{fn_name})
            // 2. Call CommandPath::path() for the key
            // 3. Call CommandPath::command() for the value
            let command_inserts = command_paths.iter().map(|path| {
                // Convert the path to the generated struct name
                // e.g., root_command -> __command_root_command
                let fn_name = path
                    .segments
                    .last()
                    .expect("Path must have at least one segment");
                let struct_name = quote::format_ident!("__command_{}", fn_name.ident);

                quote! {
                    commands.insert(
                        <#struct_name as ::utocli::CommandPath>::path().to_string(),
                        <#struct_name as ::utocli::CommandPath>::command()
                    );
                }
            });

            quote! {{
                let mut commands = ::utocli::Commands::new();
                #(#command_inserts)*
                commands
            }}
        };

        // Generate components
        let schemas = &self.attributes.component_schemas;
        let parameters = &self.attributes.component_parameters;
        let responses = &self.attributes.component_responses;

        let components_tokens =
            if schemas.is_empty() && parameters.is_empty() && responses.is_empty() {
                quote! {}
            } else {
                let schema_inserts = schemas.iter().map(|schema| {
                    quote! {
                        schemas.insert(
                            #schema::schema_name().to_string(),
                            ::utocli::RefOr::T(#schema::schema())
                        );
                    }
                });

                let param_inserts = parameters.iter().map(|param| {
                    quote! {
                        let params = #param::parameters();
                        for p in params {
                            parameters.insert(
                                #param::parameter_name().to_string(),
                                ::utocli::RefOr::T(p)
                            );
                        }
                    }
                });

                let response_inserts = responses.iter().map(|resp| {
                    quote! {
                        responses.insert(
                            #resp::response_name().to_string(),
                            ::utocli::RefOr::T(#resp::response())
                        );
                    }
                });

                quote! {
                    .components({
                        let mut schemas = ::utocli::Map::new();
                        let mut parameters = ::utocli::Map::new();
                        let mut responses = ::utocli::Map::new();

                        #(#schema_inserts)*
                        #(#param_inserts)*
                        #(#response_inserts)*

                        ::utocli::Components::new()
                            .schemas(schemas)
                            .parameters(parameters)
                            .responses(responses)
                    })
                }
            };

        // Generate tags
        let tags = &self.attributes.tags;
        let tags_tokens = if tags.is_empty() {
            quote! {}
        } else {
            let tag_defs = tags.iter().map(|tag| {
                let name = &tag.name;
                let desc_tokens = if let Some(desc) = &tag.description {
                    quote! { .description(#desc) }
                } else {
                    quote! {}
                };
                quote! {
                    ::utocli::Tag::new(#name) #desc_tokens
                }
            });

            quote! {
                .tags(vec![#(#tag_defs),*])
            }
        };

        // Generate platforms
        let platforms = &self.attributes.platforms;
        let platforms_tokens = if platforms.is_empty() {
            quote! {}
        } else {
            let platform_defs = platforms.iter().map(|platform_def| {
                // Convert string name to PlatformName enum
                let platform_enum = match platform_def.name.as_str() {
                    "linux" => quote! { ::utocli::PlatformName::Linux },
                    "darwin" => quote! { ::utocli::PlatformName::Darwin },
                    "windows" => quote! { ::utocli::PlatformName::Windows },
                    "freebsd" => quote! { ::utocli::PlatformName::Freebsd },
                    "netbsd" => quote! { ::utocli::PlatformName::Netbsd },
                    "openbsd" => quote! { ::utocli::PlatformName::Openbsd },
                    "dragonfly" => quote! { ::utocli::PlatformName::Dragonfly },
                    "solaris" => quote! { ::utocli::PlatformName::Solaris },
                    "android" => quote! { ::utocli::PlatformName::Android },
                    "ios" => quote! { ::utocli::PlatformName::Ios },
                    _ => quote! { ::utocli::PlatformName::Linux }, // Default
                };

                // Add architectures if present
                let arch_tokens = if !platform_def.architectures.is_empty() {
                    let archs = platform_def.architectures.iter().map(|arch| {
                        match arch.as_str() {
                            "amd64" | "x86_64" => quote! { ::utocli::Architecture::Amd64 },
                            "arm64" | "aarch64" => quote! { ::utocli::Architecture::Arm64 },
                            "x86" | "i386" => quote! { ::utocli::Architecture::X86 },
                            "arm" => quote! { ::utocli::Architecture::Arm },
                            _ => quote! { ::utocli::Architecture::Amd64 }, // Default
                        }
                    });
                    quote! { .architectures(vec![#(#archs),*]) }
                } else {
                    quote! {}
                };

                quote! {
                    ::utocli::Platform::new(#platform_enum) #arch_tokens
                }
            });

            quote! {
                .platforms(vec![#(#platform_defs),*])
            }
        };

        // Generate environment variables
        let environment = &self.attributes.environment;
        let environment_tokens = if environment.is_empty() {
            quote! {}
        } else {
            let env_defs = environment.iter().map(|env| {
                let name = &env.name;
                let desc_tokens = if let Some(desc) = &env.description {
                    quote! { .description(#desc) }
                } else {
                    quote! {}
                };
                quote! {
                    ::utocli::EnvironmentVariable::new(#name) #desc_tokens
                }
            });

            quote! {
                .environment(vec![#(#env_defs),*])
            }
        };

        // Generate external docs
        let external_docs_tokens = if let Some(ext_docs) = &self.attributes.external_docs {
            let url = &ext_docs.url;
            let desc_tokens = if let Some(desc) = &ext_docs.description {
                quote! { .description(#desc) }
            } else {
                quote! {}
            };
            quote! {
                .external_docs(::utocli::ExternalDocs::new(#url) #desc_tokens)
            }
        } else {
            quote! {}
        };

        tokens.extend(quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                /// Generate the complete OpenCLI specification.
                pub fn opencli() -> ::utocli::OpenCli {
                    let info = ::utocli::Info::new(#info_title, #info_version)
                        #info_desc_tokens
                        #info_contact_tokens
                        #info_license_tokens;

                    ::utocli::OpenCli::new(info)
                        .commands(#commands_tokens)
                        #components_tokens
                        #tags_tokens
                        #platforms_tokens
                        #environment_tokens
                        #external_docs_tokens
                }
            }
        });

        Ok(())
    }
}
