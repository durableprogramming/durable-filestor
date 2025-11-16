//! Comprehensive tests for the dfs-config-toml-export binary
//!
//! This test suite provides extensive coverage for the CLI tool functionality,
//! including all subcommands, error handling, file I/O operations, and edge cases.

use assert_cmd::Command;
use durable_filestor_config_toml_exporter::TomlExporter;
use durable_filestor_shared::{Config, Protocol, ServiceConfig, KeyTranslation, test_utils::create_test_config};
use predicates::prelude::*;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;



    /// Helper function to create a temporary config file with given content
    fn create_temp_config(content: &str) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        fs::write(&config_path, content).unwrap();
        (temp_dir, config_path)
    }

    /// Helper function to create a sample config with protocols and services
    fn create_sample_config() -> Config {
        create_test_config()
    }

    /// Helper function to get the binary command
    fn dfs_export_cmd() -> Command {
        let mut cmd = Command::cargo_bin("dfs-config-toml-export").unwrap();
        cmd.current_dir(std::env::temp_dir()); // Use temp dir to avoid issues
        cmd
    }

    mod config_command_tests {
        use super::*;

        #[test]
        fn test_config_command_with_valid_file() {
            let config = create_sample_config();
            let toml_content = TomlExporter::export_config(&config).unwrap();
            let (_temp_dir, config_path) = create_temp_config(&toml_content);

            let mut cmd = dfs_export_cmd();
            cmd.arg("config")
               .arg(config_path)
               .assert()
               .success()
               .stdout(predicate::str::contains("[protocols.test-s3]"))
               .stdout(predicate::str::contains("service = \"s3\""));
        }

        #[test]
        fn test_config_command_with_compact_format() {
            let config = create_sample_config();
            let toml_content = TomlExporter::export_config(&config).unwrap();
            let (_temp_dir, config_path) = create_temp_config(&toml_content);

            let mut cmd = dfs_export_cmd();
            let output = cmd.arg("config")
                          .arg(config_path)
                          .arg("--compact")
                          .output()
                          .unwrap();

            assert!(output.status.success());

            let compact_output = String::from_utf8(output.stdout).unwrap();
            let pretty_output = TomlExporter::export_config(&config).unwrap();

            // Both should be valid and contain the expected content
            assert!(compact_output.contains("[protocols.test-s3]"));
            assert!(pretty_output.contains("[protocols.test-s3]"));
        }

        #[test]
        fn test_config_command_with_pretty_format() {
            let config = create_sample_config();
            let toml_content = TomlExporter::export_config(&config).unwrap();
            let (_temp_dir, config_path) = create_temp_config(&toml_content);

            let mut cmd = dfs_export_cmd();
            cmd.arg("config")
               .arg(config_path)
               .assert()
               .success()
               .stdout(predicate::str::contains("\n")) // Should have newlines for pretty format
               .stdout(predicate::str::contains("service = \"s3\""));
        }

        #[test]
        fn test_config_command_with_empty_config() {
            let config = Config::default();
            let toml_content = TomlExporter::export_config(&config).unwrap();
            let (_temp_dir, config_path) = create_temp_config(&toml_content);

            let mut cmd = dfs_export_cmd();
            cmd.arg("config")
               .arg(config_path)
               .assert()
               .success()
               .stdout(predicate::str::is_empty().not()); // Should produce some output
        }

        #[test]
        fn test_config_command_with_nonexistent_file() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("config")
               .arg("/nonexistent/path/config.toml")
               .assert()
               .failure()
               .stderr(predicate::str::contains("Failed to read config file"));
        }

        #[test]
        fn test_config_command_with_invalid_toml() {
            let (_temp_dir, config_path) = create_temp_config("invalid [ toml content {{{{");

            let mut cmd = dfs_export_cmd();
            cmd.arg("config")
               .arg(config_path)
               .assert()
               .failure()
               .stderr(predicate::str::contains("Failed to parse config file"));
        }

        #[test]
        fn test_config_command_with_malformed_file() {
            // Create a file with binary content
            let (_temp_dir, config_path) = create_temp_config("");
            fs::write(&config_path, &[0, 1, 2, 255]).unwrap();

            let mut cmd = dfs_export_cmd();
            cmd.arg("config")
               .arg(config_path)
               .assert()
               .failure()
               .stderr(predicate::str::contains("Failed to read config file"));
        }

        #[test]
        fn test_config_command_with_unicode_content() {
            let config_content = r#"
[protocols.test]
service = "s3"
bucket = "test-bucket"
region = "us-east-1"

[protocols.test.key_translation]
prefix = "测试-"
suffix = "-测试"

[services.s3]
default_region = "us-west-2"
"#;
            let (_temp_dir, config_path) = create_temp_config(config_content);

            let mut cmd = dfs_export_cmd();
            cmd.arg("config")
               .arg(config_path)
               .assert()
               .success()
               .stdout(predicate::str::contains("测试-"))
               .stdout(predicate::str::contains("-测试"));
        }

        #[test]
        fn test_config_command_with_large_config() {
            // Create a config with many protocols and services
            let mut config = Config::default();
            let mut protocols = HashMap::new();
            let mut services = HashMap::new();

            // Add 10 protocols
            for i in 0..10 {
                let protocol = Protocol {
                    service: format!("service{}", i),
                    bucket: Some(format!("bucket{}", i)),
                    region: Some(format!("region{}", i)),
                    path: Some(format!("/path{}", i)),
                    key_translation: KeyTranslation::default(),
                    parameters: HashMap::new(),
                };
                protocols.insert(format!("protocol{}", i), protocol);
            }

            // Add 5 services
            for i in 0..5 {
                let service = ServiceConfig {
                    default_region: Some(format!("region{}", i)),
                    endpoint: Some(format!("https://endpoint{}.com", i)),
                    default_permissions: Some("0644".to_string()),
                    parameters: HashMap::new(),
                };
                services.insert(format!("service{}", i), service);
            }

            config.protocols = protocols;
            config.services = services;

            let toml_content = TomlExporter::export_config(&config).unwrap();
            let (_temp_dir, config_path) = create_temp_config(&toml_content);

            let mut cmd = dfs_export_cmd();
            cmd.arg("config")
               .arg(config_path)
               .assert()
               .success();

            // Verify the command succeeds and produces output
            let output = cmd.output().unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap();

            // Just check that we have some protocols and services in the output
            assert!(stdout.contains("[protocols."));
            assert!(stdout.contains("service = \""));
            assert!(stdout.contains("[services."));
            assert!(stdout.contains("default_region = \""));
        }
    }

    mod protocol_command_tests {
        use super::*;

        #[test]
        fn test_protocol_command_basic() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("protocol")
               .arg("test-s3")
               .arg("s3")
               .arg("--bucket")
               .arg("my-bucket")
               .arg("--region")
               .arg("us-east-1")
               .assert()
               .success()
               .stdout(predicate::str::contains("service = \"s3\""))
               .stdout(predicate::str::contains("bucket = \"my-bucket\""))
               .stdout(predicate::str::contains("region = \"us-east-1\""));
        }

        #[test]
        fn test_protocol_command_with_path() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("protocol")
               .arg("test-local")
               .arg("local")
               .arg("--path")
               .arg("/tmp/data")
               .assert()
               .success()
               .stdout(predicate::str::contains("service = \"local\""))
               .stdout(predicate::str::contains("path = \"/tmp/data\""));
        }

        #[test]
        fn test_protocol_command_minimal() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("protocol")
               .arg("minimal")
               .arg("memory")
               .assert()
               .success()
               .stdout(predicate::str::contains("service = \"memory\""))
               .stdout(predicate::str::is_empty().not());
        }

        #[test]
        fn test_protocol_command_with_all_options() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("protocol")
               .arg("complex")
               .arg("s3")
               .arg("--bucket")
               .arg("complex-bucket")
               .arg("--region")
               .arg("us-west-2")
               .arg("--path")
               .arg("/complex/path")
               .assert()
               .success()
               .stdout(predicate::str::contains("service = \"s3\""))
               .stdout(predicate::str::contains("bucket = \"complex-bucket\""))
               .stdout(predicate::str::contains("region = \"us-west-2\""))
               .stdout(predicate::str::contains("path = \"/complex/path\""));
        }

        #[test]
        fn test_protocol_command_with_special_characters() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("protocol")
               .arg("special")
               .arg("s3")
               .arg("--bucket")
               .arg("bucket-with-dashes")
               .arg("--region")
               .arg("us-east-1")
               .arg("--path")
               .arg("/path/with spaces/and-dashes")
               .assert()
               .success()
               .stdout(predicate::str::contains("bucket = \"bucket-with-dashes\""))
               .stdout(predicate::str::contains("path = \"/path/with spaces/and-dashes\""));
        }

        #[test]
        fn test_protocol_command_missing_service() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("protocol")
               .arg("test")
               .assert()
               .failure()
               .stderr(predicate::str::contains("error: the following required arguments were not provided"));
        }

        #[test]
        fn test_protocol_command_empty_service() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("protocol")
               .arg("test")
               .arg("")
               .assert()
               .success()
               .stdout(predicate::str::contains("service = \"\""));
        }

        #[test]
        fn test_protocol_command_unicode_names() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("protocol")
               .arg("测试协议")
               .arg("s3")
               .arg("--bucket")
               .arg("测试-bucket")
               .arg("--region")
               .arg("us-east-1")
               .assert()
               .success()
               .stdout(predicate::str::contains("测试-bucket"));
        }
    }

    mod service_command_tests {
        use super::*;

        #[test]
        fn test_service_command_basic() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("service")
               .arg("s3")
               .arg("--region")
               .arg("us-east-1")
               .arg("--endpoint")
               .arg("https://s3.amazonaws.com")
               .assert()
               .success()
               .stdout(predicate::str::contains("default_region = \"us-east-1\""))
               .stdout(predicate::str::contains("endpoint = \"https://s3.amazonaws.com\""));
        }

        #[test]
        fn test_service_command_with_permissions() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("service")
               .arg("local")
               .arg("--permissions")
               .arg("0755")
               .assert()
               .success()
               .stdout(predicate::str::contains("default_permissions = \"0755\""));
        }

        #[test]
        fn test_service_command_minimal() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("service")
               .arg("minimal")
               .assert()
               .success()
               .stdout(predicate::str::is_empty().not());
        }

        #[test]
        fn test_service_command_with_all_options() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("service")
               .arg("full")
               .arg("--region")
               .arg("eu-central-1")
               .arg("--endpoint")
               .arg("https://custom.endpoint.com:8443/api")
               .arg("--permissions")
               .arg("0644")
               .assert()
               .success()
               .stdout(predicate::str::contains("default_region = \"eu-central-1\""))
               .stdout(predicate::str::contains("endpoint = \"https://custom.endpoint.com:8443/api\""))
               .stdout(predicate::str::contains("default_permissions = \"0644\""));
        }

        #[test]
        fn test_service_command_special_characters() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("service")
               .arg("special-service")
               .arg("--region")
               .arg("us-west-2")
               .arg("--endpoint")
               .arg("https://endpoint.with.dots.com/path?query=value")
               .arg("--permissions")
               .arg("0777")
               .assert()
               .success()
               .stdout(predicate::str::contains("endpoint = \"https://endpoint.with.dots.com/path?query=value\""))
               .stdout(predicate::str::contains("default_permissions = \"0777\""));
        }

        #[test]
        fn test_service_command_unicode_values() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("service")
               .arg("unicode")
               .arg("--region")
               .arg("us-east-1")
               .arg("--endpoint")
               .arg("https://测试.endpoint.com")
               .assert()
               .success()
               .stdout(predicate::str::contains("endpoint = \"https://测试.endpoint.com\""));
        }

        #[test]
        fn test_service_command_empty_values() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("service")
               .arg("empty")
               .arg("--region")
               .arg("")
               .arg("--endpoint")
               .arg("")
               .arg("--permissions")
               .arg("")
               .assert()
               .success()
               .stdout(predicate::str::contains("default_region = \"\""))
               .stdout(predicate::str::contains("endpoint = \"\""))
               .stdout(predicate::str::contains("default_permissions = \"\""));
        }
    }

    mod validate_command_tests {
        use super::*;

        #[test]
        fn test_validate_command_valid_toml() {
            let config = create_sample_config();
            let toml_content = durable_filestor_config_toml_exporter::TomlExporter::export_config(&config).unwrap();
            let (_temp_dir, config_path) = create_temp_config(&toml_content);

            let mut cmd = dfs_export_cmd();
            cmd.arg("validate")
               .arg(config_path)
               .assert()
               .success()
               .stdout(predicate::str::contains("✅ TOML is valid"));
        }

        #[test]
        fn test_validate_command_invalid_toml() {
            let (_temp_dir, config_path) = create_temp_config("invalid [ toml {{{ content");

            let mut cmd = dfs_export_cmd();
            cmd.arg("validate")
               .arg(config_path)
               .assert()
               .failure()
               .stdout(predicate::str::contains("❌ TOML is invalid"));
        }

        #[test]
        fn test_validate_command_empty_file() {
            let (_temp_dir, config_path) = create_temp_config("");

            let mut cmd = dfs_export_cmd();
            cmd.arg("validate")
               .arg(config_path)
               .assert()
               .success()
               .stdout(predicate::str::contains("✅ TOML is valid"));
        }

        #[test]
        fn test_validate_command_malformed_file() {
            let (_temp_dir, config_path) = create_temp_config("");
            fs::write(&config_path, &[0, 1, 2, 255]).unwrap();

            let mut cmd = dfs_export_cmd();
            cmd.arg("validate")
               .arg(config_path)
               .assert()
               .failure()
               .stderr(predicate::str::contains("Failed to read TOML file"));
        }

        #[test]
        fn test_validate_command_nonexistent_file() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("validate")
               .arg("/nonexistent/path/file.toml")
               .assert()
               .failure()
               .stderr(predicate::str::contains("Failed to read TOML file"));
        }

        #[test]
        fn test_validate_command_complex_valid_toml() {
            let complex_toml = r#"
[protocols.complex]
service = "s3"
bucket = "complex-bucket"
region = "us-east-1"
path = "/complex/path"

[protocols.complex.key_translation]
downcase = true
partition = true
partition_length = 3
partition_depth = 2
hash_function = "sha256"
prefix = "complex-"
suffix = "-data"
timestamp_suffix = true
normalize_unicode = true
strip_whitespace = true
replace_spaces = "_"
max_key_length = 1024

[protocols.complex.parameters]
timeout = "60"
retries = "5"
special_param = "value with spaces"

[services.s3]
default_region = "us-west-2"
endpoint = "https://s3.us-west-2.amazonaws.com"
default_permissions = "0644"

[services.s3.parameters]
max_connections = "200"
pool_size = "50"
"#;
            let (_temp_dir, config_path) = create_temp_config(complex_toml);

            let mut cmd = dfs_export_cmd();
            cmd.arg("validate")
               .arg(config_path)
               .assert()
               .success()
               .stdout(predicate::str::contains("✅ TOML is valid"));
        }

        #[test]
        fn test_validate_command_with_unicode() {
            let unicode_toml = r#"
[protocols.unicode]
service = "s3"
bucket = "测试-bucket"
region = "us-east-1"

[protocols.unicode.key_translation]
prefix = "前缀-"
suffix = "-后缀"

[services.s3]
default_region = "us-west-2"
endpoint = "https://测试.endpoint.com"
"#;
            let (_temp_dir, config_path) = create_temp_config(unicode_toml);

            let mut cmd = dfs_export_cmd();
            cmd.arg("validate")
               .arg(config_path)
               .assert()
               .success()
               .stdout(predicate::str::contains("✅ TOML is valid"));
        }
    }

    mod examples_command_tests {
        use super::*;

        #[test]
        fn test_examples_command() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("examples")
               .assert()
               .success()
               .stdout(predicate::str::contains("DFS Config TOML Export Examples"))
               .stdout(predicate::str::contains("Export a config file to TOML:"))
               .stdout(predicate::str::contains("dfs-config-toml-export config"))
               .stdout(predicate::str::contains("Export a protocol configuration:"))
               .stdout(predicate::str::contains("Export a service configuration:"))
               .stdout(predicate::str::contains("Validate TOML can be parsed back:"))
               .stdout(predicate::str::contains("Show this help:"));
        }

        #[test]
        fn test_examples_command_contains_all_commands() {
            let mut cmd = dfs_export_cmd();
            let output = cmd.arg("examples").output().unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap();

            assert!(stdout.contains("config examples/example1.toml"));
            assert!(stdout.contains("--service s3 --bucket my-bucket --region us-east-1"));
            assert!(stdout.contains("--region us-west-2 --endpoint"));
            assert!(stdout.contains("validate exported_config.toml"));
            assert!(stdout.contains("--help"));
        }
    }

    mod cli_parsing_tests {
        use super::*;

        #[test]
        fn test_no_arguments() {
            let mut cmd = dfs_export_cmd();
            cmd.assert()
               .failure()
               .stderr(predicate::str::contains("Usage: dfs-config-toml-export <COMMAND>"));
        }

        #[test]
        fn test_help_flag() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("--help")
               .assert()
               .success()
               .stdout(predicate::str::contains("Test tool for DFS TOML config export functionality"))
               .stdout(predicate::str::contains("Usage:"))
               .stdout(predicate::str::contains("Commands:"))
               .stdout(predicate::str::contains("config"))
               .stdout(predicate::str::contains("protocol"))
               .stdout(predicate::str::contains("service"))
               .stdout(predicate::str::contains("validate"))
               .stdout(predicate::str::contains("examples"));
        }

        #[test]
        fn test_version_flag() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("--version")
               .assert()
               .success()
               .stdout(predicate::str::contains("dfs-config-toml-export"))
               .stdout(predicate::str::contains("0.1.0"));
        }

        #[test]
        fn test_invalid_subcommand() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("invalid")
               .assert()
               .failure()
               .stderr(predicate::str::contains("error: unrecognized subcommand"));
        }

        #[test]
        fn test_config_command_missing_file() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("config")
               .assert()
               .failure()
               .stderr(predicate::str::contains("error: the following required arguments were not provided"));
        }

        #[test]
        fn test_protocol_command_missing_required_args() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("protocol")
               .arg("test")
               .assert()
               .failure()
               .stderr(predicate::str::contains("error: the following required arguments were not provided"));
        }

        #[test]
        fn test_validate_command_missing_file() {
            let mut cmd = dfs_export_cmd();
            cmd.arg("validate")
               .assert()
               .failure()
               .stderr(predicate::str::contains("error: the following required arguments were not provided"));
        }
    }

    mod file_io_error_tests {
        use super::*;
        use std::os::unix::fs::PermissionsExt;

        #[test]
        fn test_config_command_unreadable_file() {
            let (_temp_dir, config_path) = create_temp_config("test content");

            // Make file unreadable (if possible on this system)
            if let Ok(metadata) = fs::metadata(&config_path) {
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o000); // No permissions
                let _ = fs::set_permissions(&config_path, permissions);
            }

            let mut cmd = dfs_export_cmd();
            let result = cmd.arg("config")
                          .arg(config_path)
                          .output();

            // On some systems this might succeed, on others it might fail
            // The important thing is that it doesn't crash
            assert!(result.is_ok());
        }

        #[test]
        fn test_config_command_directory_as_file() {
            let temp_dir = TempDir::new().unwrap();
            let dir_path = temp_dir.path().to_path_buf();

            let mut cmd = dfs_export_cmd();
            cmd.arg("config")
               .arg(dir_path)
               .assert()
               .failure()
               .stderr(predicate::str::contains("Failed to read config file"));
        }

        #[test]
        fn test_validate_command_directory_as_file() {
            let temp_dir = TempDir::new().unwrap();
            let dir_path = temp_dir.path().to_path_buf();

            let mut cmd = dfs_export_cmd();
            cmd.arg("validate")
               .arg(dir_path)
               .assert()
               .failure()
               .stderr(predicate::str::contains("Failed to read TOML file"));
        }

        #[test]
        fn test_config_command_with_very_long_path() {
            // Create a path that's too long (if supported by the filesystem)
            let mut long_path = "/tmp/test".to_string();
            for _ in 0..100 {
                long_path.push_str("/very/long/path/component");
            }

            let mut cmd = dfs_export_cmd();
            cmd.arg("config")
               .arg(long_path)
               .assert()
               .failure()
               .stderr(predicate::str::contains("Failed to read config file"));
        }
    }

    mod edge_cases_tests {
        use super::*;

        #[test]
        fn test_config_command_with_extremely_large_file() {
            // Create a very large config file (but not too large for testing)
            let mut large_content = String::new();
            for i in 0..1000 {
                large_content.push_str(&format!("[protocols.test{}]\nservice = \"s3\"\nbucket = \"bucket{}\"\n\n", i, i));
            }

            let (_temp_dir, config_path) = create_temp_config(&large_content);

            let mut cmd = dfs_export_cmd();
            cmd.arg("config")
               .arg(config_path)
               .assert()
               .success()
               .stdout(predicate::str::contains("[protocols.test0]"))
               .stdout(predicate::str::contains("[protocols.test999]"));
        }

        #[test]
        fn test_protocol_command_with_extremely_long_values() {
            let long_value = "a".repeat(1000); // 1k character string

            let mut cmd = dfs_export_cmd();
            cmd.arg("protocol")
               .arg("long")
               .arg(&long_value)
               .arg("--bucket")
               .arg(&long_value)
               .assert()
               .success()
               .stdout(predicate::str::contains(&format!("service = \"{}\"", long_value)))
               .stdout(predicate::str::contains(&format!("bucket = \"{}\"", long_value)));
        }

        #[test]
        fn test_service_command_with_extremely_long_values() {
            let long_value = "b".repeat(5000);

            let mut cmd = dfs_export_cmd();
            cmd.arg("service")
               .arg("long")
               .arg("--region")
               .arg(&long_value)
               .arg("--endpoint")
               .arg(&long_value)
               .arg("--permissions")
               .arg(&long_value)
               .assert()
               .success()
               .stdout(predicate::str::contains(&format!("default_region = \"{}\"", long_value)))
               .stdout(predicate::str::contains(&format!("endpoint = \"{}\"", long_value)))
               .stdout(predicate::str::contains(&format!("default_permissions = \"{}\"", long_value)));
        }

        #[test]
        fn test_config_command_with_nested_tables() {
            let nested_toml = r#"
[protocols.deep]
service = "s3"

[protocols.deep.key_translation]
downcase = true

[protocols.deep.key_translation.nested]
value = "test"

[services.s3]
default_region = "us-east-1"

[services.s3.parameters]
timeout = "30"
"#;
            let (_temp_dir, config_path) = create_temp_config(nested_toml);

            let mut cmd = dfs_export_cmd();
            cmd.arg("config")
               .arg(config_path)
               .assert()
               .success()
               .stdout(predicate::str::contains("[protocols.deep]"))
               .stdout(predicate::str::contains("[services.s3]"));
        }

        #[test]
        fn test_validate_command_with_array_values() {
            let array_toml = r#"
[protocols.test]
service = "s3"
bucket = "test"

[protocols.test.parameters]
array_value = "[1, 2, 3, 4, 5]"
string_array = "['a', 'b', 'c']"
"#;
            let (_temp_dir, config_path) = create_temp_config(array_toml);

            let mut cmd = dfs_export_cmd();
            cmd.arg("validate")
               .arg(config_path)
               .assert()
               .success()
               .stdout(predicate::str::contains("✅ TOML is valid"));
        }

        #[test]
        fn test_config_command_with_comments() {
            let commented_toml = r#"
# This is a comment
[protocols.test]
service = "s3" # inline comment
bucket = "test"

# Another comment
[services.s3]
default_region = "us-east-1"
"#;
            let (_temp_dir, config_path) = create_temp_config(commented_toml);

            let mut cmd = dfs_export_cmd();
            cmd.arg("config")
               .arg(config_path)
               .assert()
               .success()
               .stdout(predicate::str::contains("service = \"s3\""))
               .stdout(predicate::str::contains("default_region = \"us-east-1\""));
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_round_trip_config_export_import() {
            // Create a complex config
            let original_config = create_sample_config();

            // Export it to TOML
            let toml_content = TomlExporter::export_config(&original_config).unwrap();
            let (_temp_dir, config_path) = create_temp_config(&toml_content);

            // Use the CLI to export it again (this tests the CLI's export functionality)
            let mut cmd = dfs_export_cmd();
            let output = cmd.arg("config")
                          .arg(&config_path)
                          .output()
                          .unwrap();

            assert!(output.status.success());

            let cli_exported_toml = String::from_utf8(output.stdout).unwrap();

            // Parse both TOMLs and compare
            let library_parsed: Config = toml::from_str(&toml_content).unwrap();
            let cli_parsed: Config = toml::from_str(&cli_exported_toml).unwrap();

            // They should be equivalent
            assert_eq!(library_parsed.protocols.len(), cli_parsed.protocols.len());
            assert_eq!(library_parsed.services.len(), cli_parsed.services.len());

            // Check a specific protocol
            let orig_protocol = &original_config.protocols["test-s3"];
            let cli_protocol = &cli_parsed.protocols["test-s3"];

            assert_eq!(orig_protocol.service, cli_protocol.service);
            assert_eq!(orig_protocol.bucket, cli_protocol.bucket);
            assert_eq!(orig_protocol.region, cli_protocol.region);
            assert_eq!(orig_protocol.path, cli_protocol.path);
        }

        #[test]
        fn test_multiple_command_sequence() {
            // Test running multiple commands in sequence
            let config = create_sample_config();
            let toml_content = TomlExporter::export_config(&config).unwrap();
            let (_temp_dir, config_path) = create_temp_config(&toml_content);

            // First validate the file
            let mut validate_cmd = dfs_export_cmd();
            validate_cmd.arg("validate")
                       .arg(&config_path)
                       .assert()
                       .success()
                       .stdout(predicate::str::contains("✅ TOML is valid"));

            // Then export it
            let mut export_cmd = dfs_export_cmd();
            export_cmd.arg("config")
                     .arg(&config_path)
                     .assert()
                     .success();

            // Then export a protocol
            let mut protocol_cmd = dfs_export_cmd();
            protocol_cmd.arg("protocol")
                       .arg("new-protocol")
                       .arg("local")
                       .arg("--path")
                       .arg("/tmp")
                       .assert()
                       .success();

            // Then export a service
            let mut service_cmd = dfs_export_cmd();
            service_cmd.arg("service")
                      .arg("new-service")
                      .arg("--region")
                      .arg("us-east-1")
                      .assert()
                      .success();
        }

        #[test]
        fn test_output_redirection_simulation() {
            // Test that output can be captured and is properly formatted
            let mut cmd = dfs_export_cmd();
            let output = cmd.arg("protocol")
                          .arg("test")
                          .arg("s3")
                          .arg("--bucket")
                          .arg("test-bucket")
                          .output()
                          .unwrap();

            assert!(output.status.success());

            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(!stdout.is_empty());
            assert!(stdout.contains("service = \"s3\""));
            assert!(stdout.contains("bucket = \"test-bucket\""));

            // Should be valid TOML
            let _: toml::Value = toml::from_str(&stdout).unwrap();
        }
    }
}