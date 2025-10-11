//! Integration tests for building the OpenCLI kitchen-sink example using the OpenCli derive macro.
//!
//! This test suite builds a comprehensive OpenCLI specification using the OpenCli derive macro,
//! demonstrating how to generate a complete specification from types with minimal boilerplate.

#![allow(dead_code)]

use utocli::{
    Array, Command, Commands, Map, MediaType, Object, OpenCli, Parameter, ParameterScope, RefOr,
    Response, Schema, SchemaFormat, SchemaType, ToSchema,
};

#[test]
fn generate_opencli_spec_using_derive_succeeds() {
    //* When
    let opencli = CliDoc::opencli();

    //* Then
    let json_output =
        serde_json::to_string_pretty(&opencli).expect("should serialize OpenCLI to JSON");

    // Validate against the OpenCLI JSON schema
    let value = serde_json::from_str(&json_output).expect("should parse generated JSON");
    assert_is_schema_compliant(&value);

    // Check against the stored snapshot
    insta::assert_snapshot!(json_output);
}

#[test]
fn serialize_opencli_spec_using_derive_to_yaml_succeeds() {
    //* When
    let opencli = CliDoc::opencli();

    //* Then
    let yaml_output = serde_norway::to_string(&opencli).expect("should serialize OpenCLI to YAML");

    // Check against the stored snapshot
    insta::assert_snapshot!(yaml_output);
}

#[derive(utocli::OpenCli)]
#[opencli(
    info(
        title = "Open Command-Line Interface Specification",
        version = "1.0.0",
        description = "Standard for defining command-line interfaces",
        contact(
            name = "OpenCLI Working Group",
            url = "https://github.com/nrranjithnr/open-cli-specification"
        ),
        license(
            name = "Apache 2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0"
        )
    ),
    commands(root_command, validate_command, generate_command, lint_command),
    components(
        schemas(Error, Severity, ValidationError, ValidationResult, GeneratedFileType, GeneratedFile, GenerationResult),
        parameters(ConfigFileParam, OutputFormatParam),
        responses(ValidationSuccess, ValidationFailed, FileNotFound)
    ),
    tags(
        (name = "core", description = "Core commands and utilities"),
        (name = "data", description = "Data processing commands"),
        (name = "auth", description = "Authentication and user management"),
        (name = "system", description = "System-level commands and utilities")
    ),
    platforms(
        (name = "linux", architectures(amd64, arm64)),
        (name = "darwin", architectures(amd64, arm64)),
        (name = "windows", architectures(amd64, arm64))
    ),
    environment(
        (name = "OCS_CONFIG_PATH", description = "Override default configuration file path"),
        (name = "OCS_VERBOSE", description = "Enable verbose output globally"),
        (name = "OCS_QUIET", description = "Suppress non-essential output globally")
    ),
    external_docs(
        url = "https://www.openclispec.org",
        description = "Find out more about OpenCLI"
    )
)]
struct CliDoc;

#[derive(utocli::ToSchema)]
struct Error {
    code: i32,
    message: String,
    details: Option<String>,
}

#[derive(utocli::ToSchema)]
enum Severity {
    #[allow(dead_code)]
    Error,
    #[allow(dead_code)]
    Warning,
}

#[derive(utocli::ToSchema)]
struct ValidationError {
    line: i32,
    message: String,
    severity: Severity,
}

#[derive(utocli::ToSchema)]
enum GeneratedFileType {
    #[allow(dead_code)]
    Source,
    #[allow(dead_code)]
    Documentation,
    #[allow(dead_code)]
    Configuration,
}

#[derive(utocli::ToSchema)]
struct GeneratedFile {
    path: String,
    size: Option<i64>,
    #[schema(rename = "type")]
    file_type: GeneratedFileType,
}

#[derive(utocli::ToSchema)]
struct ValidationResult {
    valid: bool,
    file: String,
    errors: Vec<ValidationError>,
    warnings: Option<Vec<String>>,
}

#[derive(utocli::ToSchema)]
struct GenerationResult {
    success: bool,
    output_directory: String,
    language: Option<String>,
    template: Option<String>,
    files_generated: Option<Vec<GeneratedFile>>,
}

#[derive(utocli::ToSchema)]
struct CommandInfo {
    name: String,
    description: String,
}

#[derive(utocli::ToSchema)]
struct RootCommandResponse {
    cli_version: String,
    spec_version: String,
    platform: String,
    commands: Vec<CommandInfo>,
}

#[derive(utocli::ToParameter)]
struct ConfigFileParam {
    #[param(
        alias = "c",
        description = "Path to configuration file",
        scope = "inherited",
        format = "path",
        example = "~/.config/ocs/config.yaml"
    )]
    config: String,
}

#[derive(utocli::ToParameter)]
struct OutputFormatParam {
    #[param(
        alias = "o",
        description = "Output format for results",
        scope = "local",
        enum_values("json", "yaml", "text"),
        default = "text"
    )]
    output: String,
}

#[derive(utocli::ToResponse)]
#[response(description = "Validation completed successfully")]
struct ValidationSuccess {
    #[content(
        media_type = "application/json",
        schema = "ValidationResult",
        example = r#"{"valid":true,"file":"spec.yaml","errors":[],"warnings":[]}"#
    )]
    json: (),
    #[content(
        media_type = "text/plain",
        example = "✓ Validation successful\nNo errors found\n"
    )]
    text: (),
}

#[derive(utocli::ToResponse)]
#[response(description = "Validation failed with errors")]
struct ValidationFailed {
    #[content(
        media_type = "application/json",
        schema = "ValidationResult",
        example = r#"{"valid":false,"file":"invalid-spec.yaml","errors":[{"line":5,"message":"Missing required field","severity":"error"}],"warnings":[]}"#
    )]
    json: (),
    #[content(
        media_type = "text/plain",
        example = "✗ Validation failed\n1 error found\n"
    )]
    text: (),
}

#[derive(utocli::ToResponse)]
#[response(description = "File not found or not readable")]
struct FileNotFound {
    #[content(
        media_type = "application/json",
        schema = "Error",
        example = r#"{"code":2,"message":"File not found","details":"Could not read the specified file"}"#
    )]
    json: (),
    #[content(
        media_type = "text/plain",
        example = "✗ Error: File not found\nCould not read the specified file\nPlease check the file path and permissions\n"
    )]
    text: (),
}

/// Builds all commands for the CLI.
fn build_commands() -> Commands {
    let mut commands = Commands::new();
    commands.insert(
        "ocs".to_string(),
        <__command_root_command as utocli::CommandPath>::command(),
    );
    commands.insert(
        "/validate".to_string(),
        <__command_validate_command as utocli::CommandPath>::command(),
    );
    commands
}

/// Root command implementation using the command attribute macro.
#[utocli::command(
    name = "ocs",
    summary = "Open CLI Spec tool",
    description = "Main entry point for the Open CLI Specification tool",
    operation_id = "rootCommand",
    aliases("opencli"),
    tags("core"),
    parameters(
        (
            name = "config",
            alias("c"),
            description = "Path to configuration file",
            scope = "inherited",
            schema_type = "string",
            schema_format = "path",
            example = "~/.config/ocs/config.yaml"
        ),
        (
            name = "verbose",
            in = "flag",
            alias("v"),
            description = "Enable verbose output",
            scope = "inherited",
            schema_type = "boolean",
            default = false
        ),
        (
            name = "quiet",
            in = "flag",
            alias("q"),
            description = "Suppress non-essential output",
            scope = "inherited",
            schema_type = "boolean",
            default = false
        ),
        (
            name = "version",
            in = "flag",
            alias("V"),
            description = "Show CLI version",
            scope = "local",
            schema_type = "boolean"
        ),
        (
            name = "help",
            in = "flag",
            alias("h"),
            description = "Show help information",
            scope = "local",
            schema_type = "boolean"
        )
    ),
    responses(
        (
            status = "0",
            description = "Version information displayed",
            content(
                (
                    media_type = "text/plain",
                    example = "ocs v1.0.0\nOpenCLI Specification v1.0.0\nPlatform: linux-amd64\n\nUsage: ocs [command] [flags]\n\nAvailable Commands:\n  validate    Validate CLI specification files\n  generate    Generate CLI code from specification\n  lint        Lint CLI specification files\n  \nUse \"ocs [command] --help\" for more information about a command.\n"
                ),
                (
                    media_type = "application/json",
                    inline_properties(
                        ("cli_version", "string"),
                        ("spec_version", "string"),
                        ("platform", "string"),
                        ("commands", "array")
                    ),
                    example = "{\"cli_version\":\"1.0.0\",\"spec_version\":\"1.0.0\",\"platform\":\"linux-amd64\",\"commands\":[{\"name\":\"validate\",\"description\":\"Validate CLI specification files\"},{\"name\":\"generate\",\"description\":\"Generate CLI code from specification\"},{\"name\":\"lint\",\"description\":\"Lint CLI specification files\"}]}"
                )
            )
        )
    )
)]
fn root_command() {
    // Command implementation (not used in spec generation)
}

#[allow(dead_code)]
fn build_root_command_old() -> Command {
    Command::new()
        .summary("Open CLI Spec tool")
        .description("Main entry point for the Open CLI Specification tool")
        .operation_id("rootCommand")
        .aliases(vec!["opencli".to_string()])
        .tags(vec!["core".to_string()])
        .parameters(vec![
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
                    Object::new()
                        .schema_type(SchemaType::Boolean),
                )))),
            Parameter::new_flag("help")
                .alias(vec!["h".to_string()])
                .description("Show help information")
                .scope(ParameterScope::Local)
                .schema(RefOr::T(Schema::Object(Box::new(
                    Object::new()
                        .schema_type(SchemaType::Boolean),
                )))),
        ])
        .responses({
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
                                "ocs v1.0.0\nOpenCLI Specification v1.0.0\nPlatform: linux-amd64\n\nUsage: ocs [command] [flags]\n\nAvailable Commands:\n  validate    Validate CLI specification files\n  generate    Generate CLI code from specification\n  lint        Lint CLI specification files\n  \nUse \"ocs [command] --help\" for more information about a command.\n".to_string(),
                            )),
                        );
                        content.insert(
                            "application/json".to_string(),
                            MediaType::new()
                                .schema(RefOr::T(Schema::Object(Box::new(
                                    Object::new()
                                        .schema_type(SchemaType::Object)
                                        .properties({
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
                                                                let mut item_props = Map::new();
                                                                item_props.insert(
                                                                    "name".to_string(),
                                                                    RefOr::T(Schema::Object(Box::new(
                                                                        Object::new().schema_type(SchemaType::String),
                                                                    ))),
                                                                );
                                                                item_props.insert(
                                                                    "description".to_string(),
                                                                    RefOr::T(Schema::Object(Box::new(
                                                                        Object::new().schema_type(SchemaType::String),
                                                                    ))),
                                                                );
                                                                item_props
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
        })
}

/// Validate command implementation using the command attribute macro.
#[utocli::command(
    name = "/validate",
    summary = "Validate CLI specification",
    description = "Validate a CLI specification file against the OpenCLI standard",
    operation_id = "validateCommand",
    aliases("val", "check"),
    tags("core"),
    extend(x_cli_category = "validation", x_performance = "fast"),
    parameters(
        (
            name = "file",
            in = "argument",
            position = 1,
            description = "Path to the CLI specification file",
            required = true,
            scope = "local",
            schema_type = "string",
            schema_format = "path",
            example = "opencli.yaml",
            extend(x_completion = "file", x_validation = "file-exists")
        ),
        (
            name = "strict",
            in = "flag",
            alias("s"),
            description = "Enable strict validation mode",
            scope = "local",
            schema_type = "boolean",
            default = false
        ),
        (
            name = "output",
            alias("o"),
            description = "Output format for validation results",
            scope = "local",
            schema_type = "string",
            enum_values("json", "yaml", "text"),
            default = "text"
        )
    ),
    responses(
        (
            status = "0",
            description = "Validation successful",
            content(
                (
                    media_type = "text/plain",
                    example = "✓ Validation successful\nNo errors found in opencli.yaml\n"
                ),
                (
                    media_type = "application/json",
                    inline_properties(
                        ("valid", "boolean"),
                        ("file", "string"),
                        ("errors", "array<string>"),
                        ("warnings", "array<string>")
                    ),
                    example = "{\"valid\":true,\"file\":\"opencli.yaml\",\"errors\":[],\"warnings\":[]}"
                ),
                (
                    media_type = "application/yaml",
                    example = "valid: true\nfile: opencli.yaml\nerrors: []\nwarnings: []\n"
                )
            )
        ),
        (
            status = "1",
            description = "Validation failed",
            content(
                (
                    media_type = "text/plain",
                    example = "✗ Validation failed\nFound 2 errors in opencli.yaml:\n  - Line 5: Missing required field 'operationId'\n  - Line 12: Invalid enum value 'invalid-type'\n"
                ),
                (
                    media_type = "application/json",
                    schema = "ValidationResult",
                    example = "{\"valid\":false,\"file\":\"opencli.yaml\",\"errors\":[{\"line\":5,\"message\":\"Missing required field 'operationId'\",\"severity\":\"error\"},{\"line\":12,\"message\":\"Invalid enum value 'invalid-type'\",\"severity\":\"error\"}],\"warnings\":[]}"
                )
            )
        ),
        (
            status = "2",
            description = "File not found or not readable",
            content(
                (
                    media_type = "text/plain",
                    example = "✗ Error: File not found\nCould not read 'missing-spec.yaml'\nPlease check the file path and permissions\n"
                ),
                (
                    media_type = "application/json",
                    schema = "Error",
                    example = "{\"code\":2,\"message\":\"File not found\",\"details\":\"Could not read 'missing-spec.yaml'\"}"
                )
            )
        )
    )
)]
fn validate_command() {
    // Command implementation (not used in spec generation)
}

/// Generate command implementation using the command attribute macro.
#[utocli::command(
    name = "/generate",
    summary = "Generate CLI code",
    description = "Generate CLI implementation code from specification",
    operation_id = "generateCommand",
    aliases("gen", "codegen"),
    tags("core"),
    parameters(
        (
            name = "spec",
            in = "argument",
            position = 1,
            description = "Path to the CLI specification file",
            required = true,
            scope = "local",
            schema_type = "string",
            schema_format = "path",
            example = "my-cli.yaml"
        ),
        (
            name = "language",
            alias("l"),
            description = "Target programming language",
            required = true,
            scope = "local",
            schema_type = "string",
            enum_values("go", "python", "javascript", "typescript", "rust", "java"),
            example = "go"
        ),
        (
            name = "output-dir",
            alias("o"),
            description = "Output directory for generated code",
            scope = "local",
            schema_type = "string",
            schema_format = "path",
            default = "./generated"
        ),
        (
            name = "template",
            alias("t"),
            description = "Code generation template",
            scope = "local",
            schema_type = "string",
            enum_values("basic", "advanced", "framework"),
            default = "basic"
        )
    ),
    responses(
        (
            status = "0",
            description = "Code generation successful",
            content(
                (
                    media_type = "text/plain",
                    example = "✓ Code generation successful\nGenerated 5 files in ./generated:\n  - main.go\n  - cmd/root.go\n  - cmd/validate.go\n  - cmd/generate.go\n  - README.md\n"
                ),
                (
                    media_type = "application/json",
                    schema = "GenerationResult",
                    example = "{\"success\":true,\"output_directory\":\"./generated\",\"language\":\"go\",\"template\":\"basic\",\"files_generated\":[{\"path\":\"main.go\",\"size\":1024,\"type\":\"source\"},{\"path\":\"cmd/root.go\",\"size\":2048,\"type\":\"source\"},{\"path\":\"README.md\",\"size\":512,\"type\":\"documentation\"}]}"
                )
            )
        ),
        (
            status = "1",
            description = "Generation failed",
            content(
                (
                    media_type = "text/plain",
                    example = "✗ Code generation failed\nError: Invalid specification file\nPlease run 'ocs validate' first\n"
                ),
                (
                    media_type = "application/json",
                    schema = "Error",
                    example = "{\"code\":1,\"message\":\"Code generation failed\",\"details\":\"Invalid specification file. Please run validation first.\"}"
                )
            )
        )
    )
)]
fn generate_command() {
    // Command implementation (not used in spec generation)
}

/// Lint command implementation using the command attribute macro.
#[utocli::command(
    name = "/lint",
    summary = "Lint multiple CLI specification files",
    description = "Check multiple CLI specification files for style and best practices",
    operation_id = "lintCommand",
    aliases("check-style"),
    tags("core"),
    parameters(
        (
            name = "files",
            in = "argument",
            position = 1,
            description = "Paths to CLI specification files to lint",
            required = true,
            scope = "local",
            schema_type = "string",
            schema_format = "path",
            example = "spec1.yaml spec2.yaml",
            arity_min = 1
        ),
        (
            name = "rules",
            alias("r"),
            description = "Specific linting rules to apply",
            scope = "local",
            schema_type = "string",
            example = "naming-convention parameter-validation",
            arity_min = 1,
            arity_max = 10
        ),
        (
            name = "exclude",
            alias("x"),
            description = "Rules to exclude from linting",
            scope = "local",
            schema_type = "string",
            arity_min = 0,
            arity_max = 5
        )
    ),
    responses(
        (
            status = "0",
            description = "Linting completed successfully",
            content(
                (
                    media_type = "application/json",
                    inline_properties(
                        ("files_checked", "integer"),
                        ("issues_found", "integer"),
                        ("passed", "boolean")
                    )
                )
            )
        )
    )
)]
fn lint_command() {
    // Command implementation (not used in spec generation)
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
