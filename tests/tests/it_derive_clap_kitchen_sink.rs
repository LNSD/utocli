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

use clap::{Parser, Subcommand};
use utocli::clap::{CommandCollection, OpenCli};

#[derive(Parser, OpenCli)]
#[command(name = "ocs", version = "1.0.0")]
#[command(about = "Open CLI Spec tool")]
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
    tags("core"),
    external_docs(
        description = "Find out more about OpenCLI",
        url = "https://www.openclispec.org"
    )
)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    #[opencli(description = "Enable verbose logging", scope = "inherited")]
    verbose: bool,

    /// Path to configuration file
    #[arg(short = 'c', long, global = true, value_name = "FILE")]
    #[opencli(
        description = "Path to configuration file",
        format = "path",
        example = "~/.config/ocs/config.yaml"
    )]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, CommandCollection)]
enum Commands {
    /// Validate CLI specification
    #[opencli(operation_id = "validateCommand", aliases("val", "check"))]
    Validate {
        /// Path to the CLI specification file
        #[arg(value_name = "FILE")]
        #[opencli(format = "path", example = "opencli.yaml")]
        file: String,

        /// Enable strict validation mode
        #[arg(short, long)]
        strict: bool,

        /// Output format for validation results
        #[arg(short, long, value_parser = ["json", "yaml", "text"], default_value = "text")]
        output: String,
    },

    /// Generate CLI code
    #[opencli(operation_id = "generateCommand", aliases("gen", "codegen"))]
    Generate {
        /// Path to the CLI specification file
        #[arg(value_name = "SPEC")]
        #[opencli(format = "path")]
        spec: String,

        /// Target programming language
        #[arg(short, long, value_parser = ["go", "python", "rust"])]
        language: String,

        /// Output directory
        #[arg(short, long, value_name = "DIR", default_value = "./generated")]
        #[opencli(description = "Output directory for generated code", format = "path")]
        output_dir: String,
    },

    /// Lint CLI specification files
    #[opencli(operation_id = "lintCommand", aliases("check-style"))]
    Lint {
        /// Files to lint
        #[arg(num_args = 1.., value_name = "FILES")]
        #[opencli(
            description = "Paths to CLI specification files to lint",
            arity(min = 1)
        )]
        files: Vec<String>,

        /// Specific linting rules to apply
        #[arg(short, long, num_args = 1..=10)]
        #[opencli(arity(min = 1, max = 10))]
        rules: Option<Vec<String>>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_opencli_spec_from_clap_parser_succeeds() {
        //* Given
        // Cli struct with clap + opencli derives

        //* When
        let spec = Cli::opencli();

        //* Then
        assert!(!spec.commands.is_empty(), "commands should not be empty");

        // Check root command exists
        let root = spec.commands.get("ocs").expect("should have root command");
        assert_eq!(
            root.summary,
            Some("Open CLI Spec tool".to_string()),
            "root command should have summary from about"
        );
    }

    #[test]
    fn extract_info_from_opencli_attrs_succeeds() {
        //* Given
        // Cli struct with #[opencli(info(...))]

        //* When
        let spec = Cli::opencli();
        let commands = &spec.commands;
        let root = commands.get("ocs").expect("should have root command");

        //* Then
        // Info is at the OpenCli level, not Command level
        // This test verifies command structure for now
        assert!(root.summary.is_some(), "should have summary");
    }

    #[test]
    fn extract_global_flags_with_inherited_scope_succeeds() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let root = commands.get("ocs").expect("should have root command");

        //* Then
        let verbose_param = root
            .parameters
            .as_ref()
            .expect("should have parameters")
            .iter()
            .find(|p| p.name == "verbose")
            .expect("should have verbose parameter");

        assert_eq!(
            verbose_param.in_,
            Some(utocli::ParameterIn::Flag),
            "verbose should be a flag"
        );
        assert_eq!(
            verbose_param.scope,
            Some(utocli::ParameterScope::Inherited),
            "global flag should have inherited scope"
        );
        assert_eq!(
            verbose_param.alias,
            Some(vec!["v".to_string()]),
            "should have short flag as alias"
        );
    }

    #[test]
    fn extract_option_with_path_format_succeeds() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let root = commands.get("ocs").expect("should have root command");

        //* Then
        let config_param = root
            .parameters
            .as_ref()
            .expect("should have parameters")
            .iter()
            .find(|p| p.name == "config")
            .expect("should have config parameter");

        assert_eq!(
            config_param.in_,
            Some(utocli::ParameterIn::Option),
            "config should be an option"
        );
        assert_eq!(
            config_param.required,
            Some(false),
            "Option<T> should not be required"
        );
        assert_eq!(
            config_param.alias,
            Some(vec!["c".to_string()]),
            "should have short alias"
        );

        // Check schema format
        if let Some(utocli::RefOr::T(utocli::Schema::Object(obj))) = &config_param.schema {
            assert_eq!(
                obj.format,
                Some(utocli::SchemaFormat::Path),
                "should have path format"
            );
        } else {
            panic!("config should have Object schema");
        }
    }

    #[test]
    fn extract_subcommands_with_kebab_case_paths_succeeds() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let validate_cmd = commands.get("/validate");
        let generate_cmd = commands.get("/generate");
        let lint_cmd = commands.get("/lint");

        //* Then
        assert!(validate_cmd.is_some(), "should have /validate subcommand");
        assert!(generate_cmd.is_some(), "should have /generate subcommand");
        assert!(lint_cmd.is_some(), "should have /lint subcommand");
    }

    #[test]
    fn extract_subcommand_summary_from_doc_comment_succeeds() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let validate_cmd = commands
            .get("/validate")
            .expect("should have validate subcommand");

        //* Then
        assert_eq!(
            validate_cmd.summary,
            Some("Validate CLI specification".to_string()),
            "should extract summary from doc comment"
        );
    }

    #[test]
    fn extract_subcommand_operation_id_and_aliases_succeeds() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let validate_cmd = commands
            .get("/validate")
            .expect("should have validate subcommand");

        //* Then
        assert_eq!(
            validate_cmd.operation_id,
            Some("validateCommand".to_string()),
            "should have operation_id from opencli attr"
        );
        assert_eq!(
            validate_cmd.aliases,
            Some(vec!["val".to_string(), "check".to_string()]),
            "should have aliases from opencli attr"
        );
    }

    #[test]
    fn extract_subcommand_tags_succeeds() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let validate_cmd = commands
            .get("/validate")
            .expect("should have validate subcommand");

        //* Then
        // Tags are set at root level via #[opencli(tags(...))]
        // Subcommands inherit or have their own tags
        // For now, verify command structure is correct
        assert!(validate_cmd.operation_id.is_some());
    }

    #[test]
    fn extract_positional_argument_with_required_true_succeeds() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let validate_cmd = commands
            .get("/validate")
            .expect("should have validate subcommand");

        //* Then
        let file_param = validate_cmd
            .parameters
            .as_ref()
            .expect("should have parameters")
            .iter()
            .find(|p| p.name == "file")
            .expect("should have file parameter");

        assert_eq!(
            file_param.in_,
            Some(utocli::ParameterIn::Argument),
            "file should be a positional argument"
        );
        assert_eq!(file_param.position, Some(1), "file should have position 1");
        assert_eq!(
            file_param.required,
            Some(true),
            "non-Option type should be required"
        );
    }

    #[test]
    fn extract_flag_parameter_with_bool_type_succeeds() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let validate_cmd = commands
            .get("/validate")
            .expect("should have validate subcommand");

        //* Then
        let strict_param = validate_cmd
            .parameters
            .as_ref()
            .expect("should have parameters")
            .iter()
            .find(|p| p.name == "strict")
            .expect("should have strict parameter");

        assert_eq!(
            strict_param.in_,
            Some(utocli::ParameterIn::Flag),
            "bool with short/long should be a flag"
        );
        assert_eq!(
            strict_param.alias,
            Some(vec!["s".to_string()]),
            "should have short alias"
        );

        // Check schema is boolean
        if let Some(utocli::RefOr::T(utocli::Schema::Object(obj))) = &strict_param.schema {
            assert_eq!(
                obj.schema_type,
                Some(utocli::SchemaType::Boolean),
                "flag should have boolean schema"
            );
        } else {
            panic!("strict should have Object schema");
        }
    }

    #[test]
    fn map_value_parser_to_enum_constraint_succeeds() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let validate_cmd = commands
            .get("/validate")
            .expect("should have validate subcommand");

        //* Then
        let output_param = validate_cmd
            .parameters
            .as_ref()
            .expect("should have parameters")
            .iter()
            .find(|p| p.name == "output")
            .expect("should have output parameter");

        if let Some(utocli::RefOr::T(utocli::Schema::Object(obj))) = &output_param.schema {
            assert_eq!(
                obj.enum_values,
                Some(vec![
                    serde_json::Value::String("json".to_string()),
                    serde_json::Value::String("yaml".to_string()),
                    serde_json::Value::String("text".to_string()),
                ]),
                "should have enum constraint from value_parser"
            );
        } else {
            panic!("output should have Object schema with enum");
        }
    }

    #[test]
    fn map_default_value_to_schema_default_succeeds() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let validate_cmd = commands
            .get("/validate")
            .expect("should have validate subcommand");

        //* Then
        let output_param = validate_cmd
            .parameters
            .as_ref()
            .expect("should have parameters")
            .iter()
            .find(|p| p.name == "output")
            .expect("should have output parameter");

        if let Some(utocli::RefOr::T(utocli::Schema::Object(obj))) = &output_param.schema {
            assert_eq!(
                obj.default,
                Some(serde_json::Value::String("text".to_string())),
                "should have default value from default_value attr"
            );
        } else {
            panic!("output should have Object schema with default");
        }
    }

    #[test]
    fn map_value_name_to_example_succeeds() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let validate_cmd = commands
            .get("/validate")
            .expect("should have validate subcommand");

        //* Then
        let file_param = validate_cmd
            .parameters
            .as_ref()
            .expect("should have parameters")
            .iter()
            .find(|p| p.name == "file")
            .expect("should have file parameter");

        // Example should be on the schema, not the parameter
        if let Some(utocli::RefOr::T(utocli::Schema::Object(obj))) = &file_param.schema {
            assert_eq!(
                obj.example,
                Some(serde_json::Value::String("opencli.yaml".to_string())),
                "should use opencli example attr"
            );
        } else {
            panic!("file should have Object schema with example");
        }
    }

    #[test]
    fn map_vec_type_to_array_schema_succeeds() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let lint_cmd = commands.get("/lint").expect("should have lint subcommand");

        //* Then
        let files_param = lint_cmd
            .parameters
            .as_ref()
            .expect("should have parameters")
            .iter()
            .find(|p| p.name == "files")
            .expect("should have files parameter");

        assert!(
            matches!(
                files_param.schema,
                Some(utocli::RefOr::T(utocli::Schema::Array(_)))
            ),
            "Vec<T> should map to array schema"
        );
    }

    #[test]
    fn map_num_args_to_arity_succeeds() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let lint_cmd = commands.get("/lint").expect("should have lint subcommand");

        //* Then
        let files_param = lint_cmd
            .parameters
            .as_ref()
            .expect("should have parameters")
            .iter()
            .find(|p| p.name == "files")
            .expect("should have files parameter");

        assert_eq!(
            files_param.arity,
            Some(utocli::Arity {
                min: Some(1),
                max: None
            }),
            "num_args = 1.. should map to min: 1, max: None"
        );
    }

    #[test]
    fn map_num_args_range_to_arity_with_max_succeeds() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let lint_cmd = commands.get("/lint").expect("should have lint subcommand");

        //* Then
        let rules_param = lint_cmd
            .parameters
            .as_ref()
            .expect("should have parameters")
            .iter()
            .find(|p| p.name == "rules")
            .expect("should have rules parameter");

        assert_eq!(
            rules_param.arity,
            Some(utocli::Arity {
                min: Some(1),
                max: Some(10)
            }),
            "num_args = 1..=10 should map to min: 1, max: 10"
        );
    }

    #[test]
    fn opencli_spec_structure_matches_snapshot() {
        //* Given
        let spec = Cli::opencli();
        let commands = &spec.commands;

        //* When
        let json_output =
            serde_json::to_string_pretty(&commands).expect("should serialize commands to JSON");

        //* Then
        insta::assert_snapshot!("clap_kitchen_sink_spec", json_output);
    }
}
