//! Integration tests for building the OpenCLI kitchen-sink example.
//!
//! This test suite builds a comprehensive OpenCLI specification using the builder API,
//! based on the official OpenCLI specification example.

use utocli::opencli::{
    Architecture, Arity, Array, Command, Commands, Components, Contact, EnvironmentVariable,
    ExternalDocs, Info, License, Map, MediaType, Object, OpenCliBuilder, Parameter, ParameterScope,
    Platform, PlatformName, Ref, RefOr, Response, Schema, SchemaFormat, SchemaType, Tag,
};

#[test]
fn build_opencli_with_complete_spec_succeeds() {
    //* Given
    let info = build_info();
    let external_docs = build_external_docs();
    let platforms = build_platforms();
    let environment = build_environment_variables();
    let tags = build_tags();
    let components = build_components();
    let commands = build_commands();

    //* When
    let opencli = OpenCliBuilder::new()
        .info(info)
        .commands(commands)
        .components(components)
        .tags(tags)
        .platforms(platforms)
        .environment(environment)
        .external_docs(external_docs)
        .build();
    let json_output =
        serde_json::to_string_pretty(&opencli).expect("should serialize OpenCLI to JSON");

    //* Then
    // Validate against the OpenCLI JSON schema
    let value = serde_json::from_str(&json_output).expect("should parse generated JSON");
    assert_is_schema_compliant(&value);

    // Check against the stored snapshot
    insta::assert_snapshot!(json_output);
}

#[test]
fn serialize_opencli_to_yaml_succeeds() {
    //* Given
    let info = build_info();
    let external_docs = build_external_docs();
    let platforms = build_platforms();
    let environment = build_environment_variables();
    let tags = build_tags();
    let components = build_components();
    let commands = build_commands();

    //* When
    let opencli = OpenCliBuilder::new()
        .info(info)
        .commands(commands)
        .components(components)
        .tags(tags)
        .platforms(platforms)
        .environment(environment)
        .external_docs(external_docs)
        .build();
    let yaml_output = serde_norway::to_string(&opencli).expect("should serialize OpenCLI to YAML");

    //* Then
    // Check against the stored snapshot
    insta::assert_snapshot!(yaml_output);
}

/// Builds the Info section with contact and license information.
fn build_info() -> Info {
    Info::new("Open Command-Line Interface Specification", "1.0.0")
        .description("Standard for defining command-line interfaces")
        .contact(
            Contact::new()
                .name("OpenCLI Working Group")
                .url("https://github.com/nrranjithnr/open-cli-specification"),
        )
        .license(License::new("Apache 2.0").url("https://www.apache.org/licenses/LICENSE-2.0"))
}

/// Builds the external documentation reference.
fn build_external_docs() -> ExternalDocs {
    ExternalDocs::new("https://www.openclispec.org").description("Find out more about OpenCLI")
}

/// Builds platform support definitions.
fn build_platforms() -> Vec<Platform> {
    vec![
        Platform::new(PlatformName::Linux)
            .architectures(vec![Architecture::Amd64, Architecture::Arm64]),
        Platform::new(PlatformName::Darwin)
            .architectures(vec![Architecture::Amd64, Architecture::Arm64]),
        Platform::new(PlatformName::Windows)
            .architectures(vec![Architecture::Amd64, Architecture::Arm64]),
    ]
}

/// Builds environment variable definitions.
fn build_environment_variables() -> Vec<EnvironmentVariable> {
    vec![
        EnvironmentVariable::new("OCS_CONFIG_PATH")
            .description("Override default configuration file path"),
        EnvironmentVariable::new("OCS_VERBOSE").description("Enable verbose output globally"),
        EnvironmentVariable::new("OCS_QUIET").description("Suppress non-essential output globally"),
    ]
}

/// Builds tag definitions for command organization.
fn build_tags() -> Vec<Tag> {
    vec![
        Tag::new("core").description("Core commands and utilities"),
        Tag::new("data").description("Data processing commands"),
        Tag::new("auth").description("Authentication and user management"),
        Tag::new("system").description("System-level commands and utilities"),
    ]
}

/// Builds the components section with reusable schemas.
fn build_components() -> Components {
    let mut schemas = Map::new();

    // Error schema
    let error_schema = Schema::Object(Box::new(
        Object::new()
            .schema_type(SchemaType::Object)
            .properties({
                let mut props = Map::new();
                props.insert(
                    "code".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new()
                            .schema_type(SchemaType::Integer)
                            .format(SchemaFormat::Int32),
                    ))),
                );
                props.insert(
                    "message".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new().schema_type(SchemaType::String),
                    ))),
                );
                props.insert(
                    "details".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new().schema_type(SchemaType::String),
                    ))),
                );
                props
            })
            .required(vec!["code".to_string(), "message".to_string()]),
    ));
    schemas.insert("Error".to_string(), RefOr::T(error_schema));

    // ValidationError schema
    let validation_error_schema = Schema::Object(Box::new(
        Object::new()
            .schema_type(SchemaType::Object)
            .properties({
                let mut props = Map::new();
                props.insert(
                    "line".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new().schema_type(SchemaType::Integer),
                    ))),
                );
                props.insert(
                    "message".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new().schema_type(SchemaType::String),
                    ))),
                );
                props.insert(
                    "severity".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new()
                            .schema_type(SchemaType::String)
                            .enum_values(vec![
                                serde_json::Value::String("error".to_string()),
                                serde_json::Value::String("warning".to_string()),
                            ]),
                    ))),
                );
                props
            })
            .required(vec![
                "line".to_string(),
                "message".to_string(),
                "severity".to_string(),
            ]),
    ));
    schemas.insert(
        "ValidationError".to_string(),
        RefOr::T(validation_error_schema),
    );

    // ValidationResult schema
    let validation_result_schema = Schema::Object(Box::new(
        Object::new()
            .schema_type(SchemaType::Object)
            .properties({
                let mut props = Map::new();
                props.insert(
                    "valid".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new().schema_type(SchemaType::Boolean),
                    ))),
                );
                props.insert(
                    "file".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new().schema_type(SchemaType::String),
                    ))),
                );
                props.insert(
                    "errors".to_string(),
                    RefOr::T(Schema::Array(Array::new().items(RefOr::Ref(Ref {
                        ref_path: "#/components/schemas/ValidationError".to_string(),
                    })))),
                );
                props.insert(
                    "warnings".to_string(),
                    RefOr::T(Schema::Array(Array::new().items(RefOr::T(Schema::Object(
                        Box::new(Object::new().schema_type(SchemaType::String)),
                    ))))),
                );
                props
            })
            .required(vec!["valid".to_string(), "file".to_string()]),
    ));
    schemas.insert(
        "ValidationResult".to_string(),
        RefOr::T(validation_result_schema),
    );

    // GeneratedFile schema
    let generated_file_schema = Schema::Object(Box::new(
        Object::new()
            .schema_type(SchemaType::Object)
            .properties({
                let mut props = Map::new();
                props.insert(
                    "path".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new().schema_type(SchemaType::String),
                    ))),
                );
                props.insert(
                    "size".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new().schema_type(SchemaType::Integer),
                    ))),
                );
                props.insert(
                    "type".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new()
                            .schema_type(SchemaType::String)
                            .enum_values(vec![
                                serde_json::Value::String("source".to_string()),
                                serde_json::Value::String("documentation".to_string()),
                                serde_json::Value::String("configuration".to_string()),
                            ]),
                    ))),
                );
                props
            })
            .required(vec!["path".to_string(), "type".to_string()]),
    ));
    schemas.insert("GeneratedFile".to_string(), RefOr::T(generated_file_schema));

    // GenerationResult schema
    let generation_result_schema = Schema::Object(Box::new(
        Object::new()
            .schema_type(SchemaType::Object)
            .properties({
                let mut props = Map::new();
                props.insert(
                    "success".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new().schema_type(SchemaType::Boolean),
                    ))),
                );
                props.insert(
                    "output_directory".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new().schema_type(SchemaType::String),
                    ))),
                );
                props.insert(
                    "language".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new().schema_type(SchemaType::String),
                    ))),
                );
                props.insert(
                    "template".to_string(),
                    RefOr::T(Schema::Object(Box::new(
                        Object::new().schema_type(SchemaType::String),
                    ))),
                );
                props.insert(
                    "files_generated".to_string(),
                    RefOr::T(Schema::Array(Array::new().items(RefOr::Ref(Ref {
                        ref_path: "#/components/schemas/GeneratedFile".to_string(),
                    })))),
                );
                props
            })
            .required(vec!["success".to_string(), "output_directory".to_string()]),
    ));
    schemas.insert(
        "GenerationResult".to_string(),
        RefOr::T(generation_result_schema),
    );

    // Build parameters
    let mut parameters = Map::new();

    parameters.insert(
        "ConfigFile".to_string(),
        RefOr::T(
            Parameter::new("config")
                .alias(vec!["c".to_string()])
                .description("Path to configuration file")
                .scope(ParameterScope::Inherited)
                .schema(RefOr::T(Schema::Object(Box::new(
                    Object::new()
                        .schema_type(SchemaType::String)
                        .format(SchemaFormat::Path)
                        .example(serde_json::Value::String(
                            "~/.config/ocs/config.yaml".to_string(),
                        )),
                )))),
        ),
    );

    parameters.insert(
        "OutputFormat".to_string(),
        RefOr::T(
            Parameter::new("output")
                .alias(vec!["o".to_string()])
                .description("Output format for results")
                .scope(ParameterScope::Local)
                .schema(RefOr::T(Schema::Object(Box::new(
                    Object::new()
                        .schema_type(SchemaType::String)
                        .enum_values(vec![
                            serde_json::Value::String("json".to_string()),
                            serde_json::Value::String("yaml".to_string()),
                            serde_json::Value::String("text".to_string()),
                        ])
                        .default_value(serde_json::Value::String("text".to_string())),
                )))),
        ),
    );

    // Build responses
    let mut responses = Map::new();

    responses.insert(
        "ValidationSuccess".to_string(),
        RefOr::T(
            Response::new()
                .description("Validation completed successfully")
                .content({
                    let mut content = Map::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType::new()
                            .schema(RefOr::Ref(Ref {
                                ref_path: "#/components/schemas/ValidationResult".to_string(),
                            }))
                            .example(serde_json::json!({
                                "valid": true,
                                "file": "spec.yaml",
                                "errors": [],
                                "warnings": []
                            })),
                    );
                    content.insert(
                        "text/plain".to_string(),
                        MediaType::new().example(serde_json::Value::String(
                            "✓ Validation successful\nNo errors found\n".to_string(),
                        )),
                    );
                    content
                }),
        ),
    );

    responses.insert(
        "ValidationFailed".to_string(),
        RefOr::T(
            Response::new()
                .description("Validation failed with errors")
                .content({
                    let mut content = Map::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType::new()
                            .schema(RefOr::Ref(Ref {
                                ref_path: "#/components/schemas/ValidationResult".to_string(),
                            }))
                            .example(serde_json::json!({
                                "valid": false,
                                "file": "invalid-spec.yaml",
                                "errors": [
                                    {
                                        "line": 5,
                                        "message": "Missing required field",
                                        "severity": "error"
                                    }
                                ],
                                "warnings": []
                            })),
                    );
                    content.insert(
                        "text/plain".to_string(),
                        MediaType::new().example(serde_json::Value::String(
                            "✗ Validation failed\n1 error found\n".to_string(),
                        )),
                    );
                    content
                }),
        ),
    );

    responses.insert(
        "FileNotFound".to_string(),
        RefOr::T(
            Response::new()
                .description("File not found or not readable")
                .content({
                    let mut content = Map::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType::new()
                            .schema(RefOr::Ref(Ref {
                                ref_path: "#/components/schemas/Error".to_string(),
                            }))
                            .example(serde_json::json!({
                                "code": 2,
                                "message": "File not found",
                                "details": "Could not read the specified file"
                            })),
                    );
                    content.insert(
                        "text/plain".to_string(),
                        MediaType::new()
                            .example(serde_json::Value::String(
                                "✗ Error: File not found\nCould not read the specified file\nPlease check the file path and permissions\n".to_string(),
                            )),
                    );
                    content
                }),
        ),
    );

    Components::new()
        .schemas(schemas)
        .parameters(parameters)
        .responses(responses)
}

/// Builds all commands for the CLI.
fn build_commands() -> Commands {
    let mut commands = Commands::new();
    commands.insert("ocs".to_string(), build_root_command());
    commands.insert("/validate".to_string(), build_validate_command());
    commands.insert("/generate".to_string(), build_generate_command());
    commands.insert("/lint".to_string(), build_lint_command());
    commands
}

/// Builds the root 'ocs' command.
fn build_root_command() -> Command {
    Command::new()
        .summary("Open CLI Spec tool")
        .description("Main entry point for the Open CLI Specification tool")
        .operation_id("rootCommand")
        .aliases(vec!["opencli".to_string()])
        .tags(vec!["core".to_string()])
        .parameters(build_root_command_parameters())
        .responses(build_root_command_responses())
}

/// Builds parameters for the root command.
fn build_root_command_parameters() -> Vec<Parameter> {
    vec![
        Parameter::new("config")
            .alias(vec!["c".to_string()])
            .description("Path to configuration file")
            .scope(ParameterScope::Inherited)
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new()
                    .schema_type(SchemaType::String)
                    .format(SchemaFormat::Path)
                    .example(serde_json::Value::String(
                        "~/.config/ocs/config.yaml".to_string(),
                    )),
            )))),
        Parameter::new_flag("verbose")
            .alias(vec!["v".to_string()])
            .description("Enable verbose output")
            .scope(ParameterScope::Inherited)
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new()
                    .schema_type(SchemaType::Boolean)
                    .default_value(serde_json::Value::Bool(false)),
            )))),
        Parameter::new_flag("quiet")
            .alias(vec!["q".to_string()])
            .description("Suppress non-essential output")
            .scope(ParameterScope::Inherited)
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new()
                    .schema_type(SchemaType::Boolean)
                    .default_value(serde_json::Value::Bool(false)),
            )))),
        Parameter::new_flag("version")
            .alias(vec!["V".to_string()])
            .description("Show CLI version")
            .scope(ParameterScope::Local)
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new().schema_type(SchemaType::Boolean),
            )))),
        Parameter::new_flag("help")
            .alias(vec!["h".to_string()])
            .description("Show help information")
            .scope(ParameterScope::Local)
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new().schema_type(SchemaType::Boolean),
            )))),
    ]
}

/// Builds responses for the root command.
fn build_root_command_responses() -> Map<String, Response> {
    let mut responses = Map::new();
    responses.insert(
        "0".to_string(),
        Response::new()
            .description("Version information displayed")
            .content({
                let mut content = Map::new();
                content.insert(
                    "text/plain".to_string(),
                    MediaType::new().example(serde_json::Value::String(
                        "ocs v1.0.0\nOpenCLI Specification v1.0.0\nPlatform: linux-amd64\n\nUsage: ocs [command] [flags]\n\nAvailable Commands:\n  validate    Validate CLI specification files\n  generate    Generate CLI code from specification\n  lint        Lint CLI specification files\n  \nUse \"ocs [command] --help\" for more information about a command.\n"
                            .to_string(),
                    )),
                );
                content.insert(
                    "application/json".to_string(),
                    MediaType::new()
                        .schema(RefOr::T(Schema::Object(Box::new(
                            Object::new().schema_type(SchemaType::Object).properties({
                                let mut props = Map::new();
                                props.insert(
                                    "cli_version".to_string(),
                                    RefOr::T(Schema::Object(Box::new(
                                        Object::new().schema_type(SchemaType::String),
                                    ))),
                                );
                                props.insert(
                                    "spec_version".to_string(),
                                    RefOr::T(Schema::Object(Box::new(
                                        Object::new().schema_type(SchemaType::String),
                                    ))),
                                );
                                props.insert(
                                    "platform".to_string(),
                                    RefOr::T(Schema::Object(Box::new(
                                        Object::new().schema_type(SchemaType::String),
                                    ))),
                                );
                                props.insert(
                                    "commands".to_string(),
                                    RefOr::T(Schema::Array(Array::new().items(RefOr::T(
                                        Schema::Object(Box::new(
                                            Object::new()
                                                .schema_type(SchemaType::Object)
                                                .properties({
                                                    let mut cmd_props = Map::new();
                                                    cmd_props.insert(
                                                        "name".to_string(),
                                                        RefOr::T(Schema::Object(Box::new(
                                                            Object::new()
                                                                .schema_type(SchemaType::String),
                                                        ))),
                                                    );
                                                    cmd_props.insert(
                                                        "description".to_string(),
                                                        RefOr::T(Schema::Object(Box::new(
                                                            Object::new()
                                                                .schema_type(SchemaType::String),
                                                        ))),
                                                    );
                                                    cmd_props
                                                }),
                                        )),
                                    )))),
                                );
                                props
                            }),
                        ))))
                        .example(serde_json::json!({
                            "cli_version": "1.0.0",
                            "spec_version": "1.0.0",
                            "platform": "linux-amd64",
                            "commands": [
                                {
                                    "name": "validate",
                                    "description": "Validate CLI specification files"
                                },
                                {
                                    "name": "generate",
                                    "description": "Generate CLI code from specification"
                                },
                                {
                                    "name": "lint",
                                    "description": "Lint CLI specification files"
                                }
                            ]
                        })),
                );
                content
            }),
    );
    responses
}

/// Builds the '/validate' subcommand.
fn build_validate_command() -> Command {
    let mut extensions = Map::new();
    extensions.insert(
        "x-cli-category".to_string(),
        serde_json::Value::String("validation".to_string()),
    );
    extensions.insert(
        "x-performance".to_string(),
        serde_json::Value::String("fast".to_string()),
    );

    Command::new()
        .summary("Validate CLI specification")
        .description("Validate a CLI specification file against the OpenCLI standard")
        .operation_id("validateCommand")
        .aliases(vec!["val".to_string(), "check".to_string()])
        .tags(vec!["core".to_string()])
        .extensions(extensions)
        .parameters(build_validate_command_parameters())
        .responses(build_validate_command_responses())
}

/// Builds parameters for the validate command.
fn build_validate_command_parameters() -> Vec<Parameter> {
    let mut file_extensions = Map::new();
    file_extensions.insert(
        "x-completion".to_string(),
        serde_json::Value::String("file".to_string()),
    );
    file_extensions.insert(
        "x-validation".to_string(),
        serde_json::Value::String("file-exists".to_string()),
    );

    vec![
        Parameter::new_argument("file", 1)
            .description("Path to the CLI specification file")
            .required(true)
            .scope(ParameterScope::Local)
            .extensions(file_extensions)
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new()
                    .schema_type(SchemaType::String)
                    .format(SchemaFormat::Path)
                    .example(serde_json::Value::String("opencli.yaml".to_string())),
            )))),
        Parameter::new_flag("strict")
            .alias(vec!["s".to_string()])
            .description("Enable strict validation mode")
            .scope(ParameterScope::Local)
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new()
                    .schema_type(SchemaType::Boolean)
                    .default_value(serde_json::Value::Bool(false)),
            )))),
        Parameter::new("output")
            .alias(vec!["o".to_string()])
            .description("Output format for validation results")
            .scope(ParameterScope::Local)
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new()
                    .schema_type(SchemaType::String)
                    .enum_values(vec![
                        serde_json::Value::String("json".to_string()),
                        serde_json::Value::String("yaml".to_string()),
                        serde_json::Value::String("text".to_string()),
                    ])
                    .default_value(serde_json::Value::String("text".to_string())),
            )))),
    ]
}

/// Builds responses for the validate command.
fn build_validate_command_responses() -> Map<String, Response> {
    let mut responses = Map::new();
    responses.insert(
        "0".to_string(),
        Response::new()
            .description("Validation successful")
            .content({
                let mut content = Map::new();
                content.insert(
                    "text/plain".to_string(),
                    MediaType::new().example(serde_json::Value::String(
                        "✓ Validation successful\nNo errors found in opencli.yaml\n".to_string(),
                    )),
                );
                content.insert(
                    "application/json".to_string(),
                    MediaType::new()
                        .schema(RefOr::T(Schema::Object(Box::new(
                            Object::new().schema_type(SchemaType::Object).properties({
                                let mut props = Map::new();
                                props.insert(
                                    "valid".to_string(),
                                    RefOr::T(Schema::Object(Box::new(
                                        Object::new().schema_type(SchemaType::Boolean),
                                    ))),
                                );
                                props.insert(
                                    "file".to_string(),
                                    RefOr::T(Schema::Object(Box::new(
                                        Object::new().schema_type(SchemaType::String),
                                    ))),
                                );
                                props.insert(
                                    "errors".to_string(),
                                    RefOr::T(Schema::Array(Array::new().items(RefOr::T(
                                        Schema::Object(Box::new(
                                            Object::new().schema_type(SchemaType::String),
                                        )),
                                    )))),
                                );
                                props.insert(
                                    "warnings".to_string(),
                                    RefOr::T(Schema::Array(Array::new().items(RefOr::T(
                                        Schema::Object(Box::new(
                                            Object::new().schema_type(SchemaType::String),
                                        )),
                                    )))),
                                );
                                props
                            }),
                        ))))
                        .example(serde_json::json!({
                            "valid": true,
                            "file": "opencli.yaml",
                            "errors": [],
                            "warnings": []
                        })),
                );
                content.insert(
                    "application/yaml".to_string(),
                    MediaType::new().example(serde_json::Value::String(
                        "valid: true\nfile: opencli.yaml\nerrors: []\nwarnings: []\n".to_string(),
                    )),
                );
                content
            }),
    );
    responses.insert(
        "1".to_string(),
        Response::new().description("Validation failed").content({
            let mut content = Map::new();
            content.insert(
                "text/plain".to_string(),
                MediaType::new().example(serde_json::Value::String(
                    "✗ Validation failed\nFound 2 errors in opencli.yaml:\n  - Line 5: Missing required field 'operationId'\n  - Line 12: Invalid enum value 'invalid-type'\n".to_string(),
                )),
            );
            content.insert(
                "application/json".to_string(),
                MediaType::new()
                    .schema(RefOr::Ref(Ref {
                        ref_path: "#/components/schemas/ValidationResult".to_string(),
                    }))
                    .example(serde_json::json!({
                        "valid": false,
                        "file": "opencli.yaml",
                        "errors": [
                            {
                                "line": 5,
                                "message": "Missing required field 'operationId'",
                                "severity": "error"
                            },
                            {
                                "line": 12,
                                "message": "Invalid enum value 'invalid-type'",
                                "severity": "error"
                            }
                        ],
                        "warnings": []
                    })),
            );
            content
        }),
    );
    responses.insert(
        "2".to_string(),
        Response::new()
            .description("File not found or not readable")
            .content({
                let mut content = Map::new();
                content.insert(
                    "text/plain".to_string(),
                    MediaType::new().example(serde_json::Value::String(
                        "✗ Error: File not found\nCould not read 'missing-spec.yaml'\nPlease check the file path and permissions\n".to_string(),
                    )),
                );
                content.insert(
                    "application/json".to_string(),
                    MediaType::new()
                        .schema(RefOr::Ref(Ref {
                            ref_path: "#/components/schemas/Error".to_string(),
                        }))
                        .example(serde_json::json!({
                            "code": 2,
                            "message": "File not found",
                            "details": "Could not read 'missing-spec.yaml'"
                        })),
                );
                content
            }),
    );
    responses
}

/// Builds the '/generate' subcommand.
fn build_generate_command() -> Command {
    Command::new()
        .summary("Generate CLI code")
        .description("Generate CLI implementation code from specification")
        .operation_id("generateCommand")
        .aliases(vec!["gen".to_string(), "codegen".to_string()])
        .tags(vec!["core".to_string()])
        .parameters(build_generate_command_parameters())
        .responses(build_generate_command_responses())
}

/// Builds parameters for the generate command.
fn build_generate_command_parameters() -> Vec<Parameter> {
    vec![
        Parameter::new_argument("spec", 1)
            .description("Path to the CLI specification file")
            .required(true)
            .scope(ParameterScope::Local)
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new()
                    .schema_type(SchemaType::String)
                    .format(SchemaFormat::Path)
                    .example(serde_json::Value::String("my-cli.yaml".to_string())),
            )))),
        Parameter::new("language")
            .alias(vec!["l".to_string()])
            .description("Target programming language")
            .required(true)
            .scope(ParameterScope::Local)
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new()
                    .schema_type(SchemaType::String)
                    .enum_values(vec![
                        serde_json::Value::String("go".to_string()),
                        serde_json::Value::String("python".to_string()),
                        serde_json::Value::String("javascript".to_string()),
                        serde_json::Value::String("typescript".to_string()),
                        serde_json::Value::String("rust".to_string()),
                        serde_json::Value::String("java".to_string()),
                    ])
                    .example(serde_json::Value::String("go".to_string())),
            )))),
        Parameter::new("output-dir")
            .alias(vec!["o".to_string()])
            .description("Output directory for generated code")
            .scope(ParameterScope::Local)
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new()
                    .schema_type(SchemaType::String)
                    .format(SchemaFormat::Path)
                    .default_value(serde_json::Value::String("./generated".to_string())),
            )))),
        Parameter::new("template")
            .alias(vec!["t".to_string()])
            .description("Code generation template")
            .scope(ParameterScope::Local)
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new()
                    .schema_type(SchemaType::String)
                    .enum_values(vec![
                        serde_json::Value::String("basic".to_string()),
                        serde_json::Value::String("advanced".to_string()),
                        serde_json::Value::String("framework".to_string()),
                    ])
                    .default_value(serde_json::Value::String("basic".to_string())),
            )))),
    ]
}

/// Builds responses for the generate command.
fn build_generate_command_responses() -> Map<String, Response> {
    let mut responses = Map::new();
    responses.insert(
        "0".to_string(),
        Response::new()
            .description("Code generation successful")
            .content({
                let mut content = Map::new();
                content.insert(
                    "text/plain".to_string(),
                    MediaType::new().example(serde_json::Value::String(
                        "✓ Code generation successful\nGenerated 5 files in ./generated:\n  - main.go\n  - cmd/root.go\n  - cmd/validate.go\n  - cmd/generate.go\n  - README.md\n".to_string(),
                    )),
                );
                content.insert(
                    "application/json".to_string(),
                    MediaType::new()
                        .schema(RefOr::Ref(Ref {
                            ref_path: "#/components/schemas/GenerationResult".to_string(),
                        }))
                        .example(serde_json::json!({
                            "success": true,
                            "output_directory": "./generated",
                            "language": "go",
                            "template": "basic",
                            "files_generated": [
                                {
                                    "path": "main.go",
                                    "size": 1024,
                                    "type": "source"
                                },
                                {
                                    "path": "cmd/root.go",
                                    "size": 2048,
                                    "type": "source"
                                },
                                {
                                    "path": "README.md",
                                    "size": 512,
                                    "type": "documentation"
                                }
                            ]
                        })),
                );
                content
            }),
    );
    responses.insert(
        "1".to_string(),
        Response::new().description("Generation failed").content({
            let mut content = Map::new();
            content.insert(
                "text/plain".to_string(),
                MediaType::new().example(serde_json::Value::String(
                    "✗ Code generation failed\nError: Invalid specification file\nPlease run 'ocs validate' first\n".to_string(),
                )),
            );
            content.insert(
                "application/json".to_string(),
                MediaType::new()
                    .schema(RefOr::Ref(Ref {
                        ref_path: "#/components/schemas/Error".to_string(),
                    }))
                    .example(serde_json::json!({
                        "code": 1,
                        "message": "Code generation failed",
                        "details": "Invalid specification file. Please run validation first."
                    })),
            );
            content
        }),
    );
    responses
}

/// Builds the '/lint' subcommand.
fn build_lint_command() -> Command {
    Command::new()
        .summary("Lint multiple CLI specification files")
        .description("Check multiple CLI specification files for style and best practices")
        .operation_id("lintCommand")
        .aliases(vec!["check-style".to_string()])
        .tags(vec!["core".to_string()])
        .parameters(build_lint_command_parameters())
        .responses(build_lint_command_responses())
}

/// Builds parameters for the lint command.
fn build_lint_command_parameters() -> Vec<Parameter> {
    vec![
        Parameter::new_argument("files", 1)
            .description("Paths to CLI specification files to lint")
            .required(true)
            .scope(ParameterScope::Local)
            .arity(Arity::new().min(1))
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new()
                    .schema_type(SchemaType::String)
                    .format(SchemaFormat::Path)
                    .example(serde_json::Value::String(
                        "spec1.yaml spec2.yaml".to_string(),
                    )),
            )))),
        Parameter::new("rules")
            .alias(vec!["r".to_string()])
            .description("Specific linting rules to apply")
            .scope(ParameterScope::Local)
            .arity(Arity::new().min(1).max(10))
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new()
                    .schema_type(SchemaType::String)
                    .example(serde_json::Value::String(
                        "naming-convention parameter-validation".to_string(),
                    )),
            )))),
        Parameter::new("exclude")
            .alias(vec!["x".to_string()])
            .description("Rules to exclude from linting")
            .scope(ParameterScope::Local)
            .arity(Arity::new().min(0).max(5))
            .schema(RefOr::T(Schema::Object(Box::new(
                Object::new().schema_type(SchemaType::String),
            )))),
    ]
}

/// Builds responses for the lint command.
fn build_lint_command_responses() -> Map<String, Response> {
    let mut responses = Map::new();
    responses.insert(
        "0".to_string(),
        Response::new()
            .description("Linting completed successfully")
            .content({
                let mut content = Map::new();
                content.insert(
                    "application/json".to_string(),
                    MediaType::new().schema(RefOr::T(Schema::Object(Box::new(
                        Object::new().schema_type(SchemaType::Object).properties({
                            let mut props = Map::new();
                            props.insert(
                                "files_checked".to_string(),
                                RefOr::T(Schema::Object(Box::new(
                                    Object::new().schema_type(SchemaType::Integer),
                                ))),
                            );
                            props.insert(
                                "issues_found".to_string(),
                                RefOr::T(Schema::Object(Box::new(
                                    Object::new().schema_type(SchemaType::Integer),
                                ))),
                            );
                            props.insert(
                                "passed".to_string(),
                                RefOr::T(Schema::Object(Box::new(
                                    Object::new().schema_type(SchemaType::Boolean),
                                ))),
                            );
                            props
                        }),
                    )))),
                );
                content
            }),
    );
    responses
}

/// Validates that the given JSON value complies with the OpenCLI JSON schema.
fn assert_is_schema_compliant(spec_json: &serde_json::Value) {
    let schema_content = include_str!("assets/opencli.spec.json");
    let schema_value = serde_json::from_str(schema_content).expect("should parse OpenCLI schema");
    let schema = jsonschema::validator_for(&schema_value).expect("should compile JSON schema");

    let errors: Vec<String> = schema
        .iter_errors(spec_json)
        .map(|err| format!("- {}: {}", err.instance_path, err))
        .collect();
    if !errors.is_empty() {
        panic!(
            "Generated OpenCLI spec does not comply with schema:\n{}",
            errors.join("\n")
        );
    }
}
