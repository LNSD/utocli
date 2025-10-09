//! Kitchen sink test for clap derive integration with utocli.
//!
//! This test demonstrates comprehensive clap + OpenCLI integration covering:
//! - Root command with global flags
//! - Subcommand generation
//! - All parameter types (arguments, flags, options)
//! - Type mapping (bool, String, PathBuf, Vec<T>, Option<T>)
//! - Enum constraints from value_parser
//! - Default values
//! - Arity from num_args
//! - Format hints (path, etc.)
//! - Aliases (short flags, command aliases)
//! - Scope (global → inherited)

#[test]
fn generate_opencli_spec_from_clap_parser_succeeds() {
    //* When
    let spec = Cli::opencli();

    //* Then
    let json_output =
        serde_json::to_string_pretty(&spec).expect("should serialize OpenCLI to JSON");

    // Validate against the OpenCLI JSON schema
    let value = serde_json::from_str(&json_output).expect("should parse generated JSON");
    assert_is_schema_compliant(&value);

    // Check against the stored snapshot
    insta::assert_snapshot!(json_output);
}

#[test]
fn serialize_opencli_spec_from_clap_parser_to_yaml_succeeds() {
    //* When
    let spec = Cli::opencli();

    //* Then
    let yaml_output = serde_norway::to_string(&spec).expect("should serialize OpenCLI to YAML");

    // Check against the stored snapshot
    insta::assert_snapshot!(yaml_output);
}

#[derive(clap::Parser, utocli::clap::OpenCli)]
#[command(name = "ocs", version = "1.0.0")]
#[command(about = "Open CLI Spec tool")]
#[command(long_about = "Main entry point for the Open CLI Specification tool")]
#[opencli(
    info(
        title = "Open Command-Line Interface Specification",
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
    operation_id = "rootCommand",
    aliases("opencli"),
    tags("core"),
    tag_definitions(
        (name = "core", description = "Core commands and utilities"),
        (name = "data", description = "Data processing commands"),
        (name = "auth", description = "Authentication and user management"),
        (name = "system", description = "System-level commands and utilities")
    ),
    platforms(
        (name = "linux", architectures = ["amd64", "arm64"]),
        (name = "darwin", architectures = ["amd64", "arm64"]),
        (name = "windows", architectures = ["amd64", "arm64"])
    ),
    environment(
        (name = "OCS_CONFIG_PATH", description = "Override default configuration file path"),
        (name = "OCS_VERBOSE", description = "Enable verbose output globally"),
        (name = "OCS_QUIET", description = "Suppress non-essential output globally")
    ),
    external_docs(
        description = "Find out more about OpenCLI",
        url = "https://www.openclispec.org"
    ),
    responses(
        (status = "0", description = "Version information displayed", content(
            (media_type = "text/plain", example = "ocs v1.0.0\nOpenCLI Specification v1.0.0\nPlatform: linux-amd64\n\nUsage: ocs [command] [flags]\n\nAvailable Commands:\n  validate    Validate CLI specification files\n  generate    Generate CLI code from specification\n  lint        Lint CLI specification files\n  \nUse \"ocs [command] --help\" for more information about a command.\n"),
            (media_type = "application/json",
             schema(r#type = "object", properties(
                 (name = "cli_version", r#type = "string"),
                 (name = "spec_version", r#type = "string"),
                 (name = "platform", r#type = "string"),
                 (name = "commands", r#type = "array", items_ref = "object")
             )),
             example = "{\"cli_version\":\"1.0.0\",\"spec_version\":\"1.0.0\",\"platform\":\"linux-amd64\",\"commands\":[{\"name\":\"validate\",\"description\":\"Validate CLI specification files\"},{\"name\":\"generate\",\"description\":\"Generate CLI code from specification\"},{\"name\":\"lint\",\"description\":\"Lint CLI specification files\"}]}")
        ))
    ),
    components(
        schemas(
            (name = "Error", schema(r#type = "object", properties(
                (name = "code", r#type = "integer", format = "int32"),
                (name = "message", r#type = "string"),
                (name = "details", r#type = "string")
            )), required("code", "message")),
            (name = "Severity", schema(r#type = "string")),
            (name = "ValidationError", schema(r#type = "object", properties(
                (name = "line", r#type = "integer", format = "int32"),
                (name = "message", r#type = "string"),
                (name = "severity", r#type = "string", items_ref = "#/components/schemas/Severity")
            )), required("line", "message", "severity")),
            (name = "ValidationResult", schema(r#type = "object", properties(
                (name = "valid", r#type = "boolean"),
                (name = "file", r#type = "string"),
                (name = "errors", r#type = "array", items_ref = "#/components/schemas/ValidationError"),
                (name = "warnings", r#type = "array", items_ref = "string")
            )), required("valid", "file", "errors")),
            (name = "GeneratedFileType", schema(r#type = "string")),
            (name = "GeneratedFile", schema(r#type = "object", properties(
                (name = "path", r#type = "string"),
                (name = "size", r#type = "integer", format = "int64"),
                (name = "type", r#type = "string", items_ref = "#/components/schemas/GeneratedFileType")
            )), required("path", "type")),
            (name = "GenerationResult", schema(r#type = "object", properties(
                (name = "success", r#type = "boolean"),
                (name = "output_directory", r#type = "string"),
                (name = "language", r#type = "string"),
                (name = "template", r#type = "string"),
                (name = "files_generated", r#type = "array", items_ref = "#/components/schemas/GeneratedFile")
            )), required("success", "output_directory"))
        ),
        parameters(
            (name = "ConfigFile", param_name = "config", alias("c"), description = "Path to configuration file", scope = "inherited", schema(r#type = "string", format = "path")),
            (name = "OutputFormat", param_name = "output", alias("o"), description = "Output format for results", scope = "local", schema(r#type = "string"))
        ),
        responses(
            (name = "ValidationSuccess", description = "Validation completed successfully", content(
                (media_type = "application/json", schema_ref = "#/components/schemas/ValidationResult", example = "{\"valid\":true,\"file\":\"spec.yaml\",\"errors\":[],\"warnings\":[]}"),
                (media_type = "text/plain", example = "✓ Validation successful\nNo errors found\n")
            )),
            (name = "ValidationFailed", description = "Validation failed with errors", content(
                (media_type = "application/json", schema_ref = "#/components/schemas/ValidationResult", example = "{\"valid\":false,\"file\":\"invalid-spec.yaml\",\"errors\":[{\"line\":5,\"message\":\"Missing required field\",\"severity\":\"error\"}],\"warnings\":[]}"),
                (media_type = "text/plain", example = "✗ Validation failed\n1 error found\n")
            )),
            (name = "FileNotFound", description = "File not found or not readable", content(
                (media_type = "application/json", schema_ref = "#/components/schemas/Error", example = "{\"code\":2,\"message\":\"File not found\",\"details\":\"Could not read the specified file\"}"),
                (media_type = "text/plain", example = "✗ Error: File not found\nCould not read the specified file\nPlease check the file path and permissions\n")
            ))
        )
    )
)]
struct Cli {
    /// Path to configuration file
    #[arg(short = 'c', long, global = true, value_name = "FILE")]
    #[opencli(
        description = "Path to configuration file",
        format = "path",
        example = "~/.config/ocs/config.yaml",
        scope = "inherited"
    )]
    config: Option<String>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    #[opencli(description = "Enable verbose output", scope = "inherited")]
    verbose: bool,

    /// Suppress non-essential output
    #[arg(short = 'q', long, global = true)]
    #[opencli(description = "Suppress non-essential output", scope = "inherited")]
    quiet: bool,

    /// Show CLI version
    #[arg(short = 'V', long)]
    #[opencli(description = "Show CLI version", scope = "local")]
    version: bool,

    /// Show help information
    #[arg(short = 'h', long)]
    #[opencli(description = "Show help information", scope = "local")]
    help: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, utocli::clap::CommandCollection)]
enum Commands {
    /// Validate CLI specification
    #[opencli(
        operation_id = "validateCommand",
        description = "Validate a CLI specification file against the OpenCLI standard",
        aliases("val", "check"),
        tags("core"),
        responses(
            (status = "0", description = "Validation successful", content(
                (media_type = "text/plain", example = "✓ Validation successful\nNo errors found in opencli.yaml\n"),
                (media_type = "application/json",
                 schema(r#type = "object", properties(
                     (name = "valid", r#type = "boolean"),
                     (name = "file", r#type = "string"),
                     (name = "errors", r#type = "array", items_ref = "string"),
                     (name = "warnings", r#type = "array", items_ref = "string")
                 )),
                 example = "{\"valid\":true,\"file\":\"opencli.yaml\",\"errors\":[],\"warnings\":[]}"),
                (media_type = "application/yaml", example = "valid: true\nfile: opencli.yaml\nerrors: []\nwarnings: []\n")
            )),
            (status = "1", description = "Validation failed", content(
                (media_type = "text/plain", example = "✗ Validation failed\nFound 2 errors in opencli.yaml:\n  - Line 5: Missing required field 'operationId'\n  - Line 12: Invalid enum value 'invalid-type'\n"),
                (media_type = "application/json",
                 schema_ref = "#/components/schemas/ValidationResult",
                 example = "{\"valid\":false,\"file\":\"opencli.yaml\",\"errors\":[{\"line\":5,\"message\":\"Missing required field 'operationId'\",\"severity\":\"error\"},{\"line\":12,\"message\":\"Invalid enum value 'invalid-type'\",\"severity\":\"error\"}],\"warnings\":[]}")
            )),
            (status = "2", description = "File not found or not readable", content(
                (media_type = "text/plain", example = "✗ Error: File not found\nCould not read 'missing-spec.yaml'\nPlease check the file path and permissions\n"),
                (media_type = "application/json",
                 schema_ref = "#/components/schemas/Error",
                 example = "{\"code\":2,\"message\":\"File not found\",\"details\":\"Could not read 'missing-spec.yaml'\"}")
            ))
        ),
        x_cli_category = "validation",
        x_performance = "fast"
    )]
    Validate {
        /// Path to the CLI specification file
        #[arg(value_name = "FILE")]
        #[opencli(
            description = "Path to the CLI specification file",
            format = "path",
            example = "opencli.yaml",
            x_completion = "file",
            x_validation = "file-exists"
        )]
        file: String,

        /// Enable strict validation mode
        #[arg(short, long)]
        #[opencli(description = "Enable strict validation mode")]
        strict: bool,

        /// Output format for validation results
        #[arg(short, long, value_parser = ["json", "yaml", "text"], default_value = "text")]
        #[opencli(description = "Output format for validation results")]
        output: String,
    },

    /// Generate CLI code
    #[opencli(
        operation_id = "generateCommand",
        description = "Generate CLI implementation code from specification",
        aliases("gen", "codegen"),
        tags("core"),
        responses(
            (status = "0", description = "Code generation successful", content(
                (media_type = "text/plain", example = "✓ Code generation successful\nGenerated 5 files in ./generated:\n  - main.go\n  - cmd/root.go\n  - cmd/validate.go\n  - cmd/generate.go\n  - README.md\n"),
                (media_type = "application/json",
                 schema_ref = "#/components/schemas/GenerationResult",
                 example = "{\"success\":true,\"output_directory\":\"./generated\",\"language\":\"go\",\"template\":\"basic\",\"files_generated\":[{\"path\":\"main.go\",\"size\":1024,\"type\":\"source\"},{\"path\":\"cmd/root.go\",\"size\":2048,\"type\":\"source\"},{\"path\":\"README.md\",\"size\":512,\"type\":\"documentation\"}]}")
            )),
            (status = "1", description = "Generation failed", content(
                (media_type = "text/plain", example = "✗ Code generation failed\nError: Invalid specification file\nPlease run 'ocs validate' first\n"),
                (media_type = "application/json",
                 schema_ref = "#/components/schemas/Error",
                 example = "{\"code\":1,\"message\":\"Code generation failed\",\"details\":\"Invalid specification file. Please run validation first.\"}")
            ))
        )
    )]
    Generate {
        /// Path to the CLI specification file
        #[arg(value_name = "SPEC")]
        #[opencli(
            description = "Path to the CLI specification file",
            format = "path",
            example = "my-cli.yaml"
        )]
        spec: String,

        /// Target programming language
        #[arg(short, long, value_parser = ["go", "python", "javascript", "typescript", "rust", "java"], required = true)]
        #[opencli(description = "Target programming language", example = "go")]
        language: String,

        /// Output directory
        #[arg(short, long, value_name = "DIR", default_value = "./generated")]
        #[opencli(description = "Output directory for generated code", format = "path")]
        output_dir: String,

        /// Code generation template
        #[arg(short = 't', long, value_parser = ["basic", "advanced", "framework"], default_value = "basic")]
        #[opencli(description = "Code generation template")]
        template: String,
    },

    /// Lint CLI specification files
    #[opencli(
        operation_id = "lintCommand",
        description = "Check multiple CLI specification files for style and best practices",
        aliases("check-style"),
        tags("core"),
        responses(
            (status = "0", description = "Linting completed successfully", content(
                (media_type = "application/json",
                 schema(r#type = "object", properties(
                     (name = "files_checked", r#type = "integer"),
                     (name = "issues_found", r#type = "integer"),
                     (name = "passed", r#type = "boolean")
                 )))
            ))
        )
    )]
    Lint {
        /// Files to lint
        #[arg(num_args = 1.., value_name = "FILES")]
        #[opencli(
            description = "Paths to CLI specification files to lint",
            format = "path",
            example = "spec1.yaml spec2.yaml",
            arity(min = 1)
        )]
        files: Vec<String>,

        /// Specific linting rules to apply
        #[arg(short, long, num_args = 1..=10)]
        #[opencli(
            description = "Specific linting rules to apply",
            example = "naming-convention parameter-validation",
            arity(min = 1, max = 10)
        )]
        rules: Option<Vec<String>>,

        /// Rules to exclude from linting
        #[arg(short = 'x', long, num_args = 0..=5)]
        #[opencli(description = "Rules to exclude from linting", arity(min = 0, max = 5))]
        exclude: Option<Vec<String>>,
    },
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
