//! Shared test utilities for URI parsing and configuration testing
//!
//! This module provides common test data, configurations, and utilities
//! that can be reused across different test binaries and integration tests.

use crate::{Config, Protocol, KeyTranslation, Cli, Commands, ConfigAction};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a standard test configuration with common protocols for testing
pub fn create_test_config() -> Config {
    let mut protocols = HashMap::new();

    // S3 protocol with key translation
    protocols.insert(
        "test-s3".to_string(),
        Protocol {
            service: "s3".to_string(),
            bucket: Some("test-bucket".to_string()),
            region: Some("us-east-1".to_string()),
            path: None,
            key_translation: KeyTranslation {
                downcase: true,
                prefix: Some("files/".to_string()),
                ..Default::default()
            },
            parameters: std::collections::HashMap::new(),
        },
    );

    // Local protocol
    protocols.insert(
        "test-local".to_string(),
        Protocol {
            service: "local".to_string(),
            bucket: None,
            region: None,
            path: Some("/tmp/dfs-test".to_string()),
            key_translation: Default::default(),
            parameters: std::collections::HashMap::new(),
        },
    );

    // Protocol with partitioning
    protocols.insert(
        "partitioned".to_string(),
        Protocol {
            service: "s3".to_string(),
            bucket: Some("partitioned-bucket".to_string()),
            region: Some("us-west-2".to_string()),
            path: None,
            key_translation: KeyTranslation {
                partition: true,
                partition_length: 2,
                partition_depth: 2,
                ..Default::default()
            },
            parameters: std::collections::HashMap::new(),
        },
    );

    // Custom backup protocol
    protocols.insert(
        "backup".to_string(),
        Protocol {
            service: "local".to_string(),
            bucket: None,
            region: None,
            path: Some("/var/backups/dfs".to_string()),
            key_translation: KeyTranslation {
                upcase: true,
                prefix: Some("backup-".to_string()),
                ..Default::default()
            },
            parameters: std::collections::HashMap::new(),
        },
    );

    Config {
        protocols,
        services: HashMap::new(),
    }
}

/// Get standard test URIs for comprehensive testing
pub fn get_standard_test_uris() -> Vec<(&'static str, &'static str)> {
    vec![
        // Basic URIs
        ("s3://bucket/file.txt", "Standard S3 URI"),
        ("my-s3://mybucket/path/to/file.json", "Custom S3-like protocol"),
        ("local:///absolute/path/file.txt", "Local file URI"),
        ("backup://documents/report.pdf", "Custom backup protocol"),

        // URIs with query parameters
        ("s3://bucket/file.txt?version=1", "URI with single query param"),
        ("custom://path/file?a=1&b=2&c=hello", "URI with multiple query params"),
        ("protocol://path?encoded=%2Fhome%2Fuser", "URI with URL-encoded params"),

        // Complex paths
        ("protocol://deep/nested/path/structure/file.ext", "Deeply nested path"),
        ("protocol://path/with spaces/file name.txt", "Path with spaces"),
        ("protocol://path/with-dashes_and_underscores.txt", "Path with special chars"),

        // Edge cases
        ("single://", "URI with empty path"),
        ("minimal://a", "Minimal valid URI"),
    ]
}

/// Get test URIs for resolution testing
pub fn get_resolution_test_uris() -> Vec<&'static str> {
    vec![
        "test-s3://Documents/MyFile.TXT",
        "test-local://data/file.json",
        "partitioned://abcdefghijklmnop",
        "backup://important/document.pdf",
    ]
}

/// Get example URIs for demonstration
pub fn get_example_uris() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Basic S3 URI", "s3://my-bucket/path/to/file.txt"),
        ("Custom protocol", "my-storage://documents/report.pdf"),
        ("Local file", "local:///home/user/data.json"),
        ("With query params", "backup://files/archive.zip?version=2&compressed=true"),
        ("Complex path", "cdn://assets/images/2024/product-photos/item-123.jpg"),
        ("Encoded characters", "protocol://path/file%20name.txt?param=%2Fhome%2Fuser"),
    ]
}

/// Get protocol extraction test cases
pub fn get_protocol_extraction_test_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        ("s3://bucket/file.txt", "s3"),
        ("my-s3://mybucket/path/to/file.json", "my-s3"),
        ("local:///absolute/path/file.txt", "local"),
        ("backup://documents/report.pdf", "backup"),
        ("custom://path/file?a=1&b=2", "custom"),
        ("protocol://path?encoded=%2Fhome%2Fuser", "protocol"),
        ("single://", "single"),
        ("minimal://a", "minimal"),
    ]
}

/// Get path extraction test cases
pub fn get_path_extraction_test_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        ("s3://bucket/file.txt", "bucket/file.txt"),
        ("my-s3://mybucket/path/to/file.json", "mybucket/path/to/file.json"),
        ("local:///absolute/path/file.txt", "/absolute/path/file.txt"),
        ("backup://documents/report.pdf", "documents/report.pdf"),
        ("custom://path/file?a=1&b=2", "path/file"),
        ("protocol://path?encoded=%2Fhome%2Fuser", "path"),
        ("single://", ""),
        ("minimal://a", "a"),
        ("protocol://deep/nested/path/structure/file.ext", "deep/nested/path/structure/file.ext"),
    ]
}

/// Get validation test URIs (all should be valid)
pub fn get_validation_test_uris() -> Vec<&'static str> {
    vec![
        "s3://bucket/file.txt",
        "my-s3://mybucket/path/to/file.json",
        "local:///absolute/path/file.txt",
        "backup://documents/report.pdf",
        "s3://bucket/file.txt?version=1",
        "custom://path/file?a=1&b=2&c=hello",
        "protocol://deep/nested/path/structure/file.ext",
        "protocol://path/with spaces/file name.txt",
        "single://",
        "minimal://a",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_config_has_expected_protocols() {
        let config = create_test_config();

        assert!(config.protocols.contains_key("test-s3"));
        assert!(config.protocols.contains_key("test-local"));
        assert!(config.protocols.contains_key("partitioned"));
        assert!(config.protocols.contains_key("backup"));
    }

    #[test]
    fn test_standard_test_uris_not_empty() {
        let uris = get_standard_test_uris();
        assert!(!uris.is_empty());
        assert!(uris.len() > 5); // Should have a reasonable number of test cases
    }

    #[test]
    fn test_resolution_test_uris_not_empty() {
        let uris = get_resolution_test_uris();
        assert!(!uris.is_empty());
    }

    #[test]
    fn test_example_uris_not_empty() {
        let uris = get_example_uris();
        assert!(!uris.is_empty());
    }
}

/// Test utilities for configuration parser testing
pub mod config_test_utils {
    use super::*;

    /// Create a temporary config file with the given content
    /// Returns (TempDir, PathBuf) where TempDir manages cleanup
    pub fn create_temp_config(content: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        fs::write(&config_path, content).unwrap();
        (temp_dir, config_path)
    }

    /// Create a basic CLI instance for testing
    pub fn create_test_cli(command: Commands) -> Cli {
        Cli {
            config: None,
            verbose: false,
            command,
        }
    }

    /// Create a CLI instance with a specific config path
    pub fn create_test_cli_with_config(config_path: PathBuf, command: Commands) -> Cli {
        Cli {
            config: Some(config_path),
            verbose: false,
            command,
        }
    }

    /// Create a list command CLI for testing
    pub fn create_list_cli(uri: &str) -> Cli {
        create_test_cli(Commands::List {
            uri: uri.to_string(),
            service: None,
            bucket: None,
            region: None,
            path: None,
        })
    }

    /// Create a config show command CLI for testing
    pub fn create_config_show_cli() -> Cli {
        create_test_cli(Commands::Config {
            action: ConfigAction::Show,
        })
    }

    /// Environment variable cleanup utility for tests
    pub struct EnvCleanup {
        vars_to_restore: Vec<(String, Option<String>)>,
        vars_to_remove: Vec<String>,
    }

    impl EnvCleanup {
        /// Create a new cleanup utility
        pub fn new() -> Self {
            Self {
                vars_to_restore: Vec::new(),
                vars_to_remove: Vec::new(),
            }
        }

        /// Track a variable to be removed after test
        pub fn track_removal(&mut self, var: &str) {
            if let Ok(value) = std::env::var(var) {
                self.vars_to_restore.push((var.to_string(), Some(value)));
            } else {
                self.vars_to_remove.push(var.to_string());
            }
        }

        /// Set a variable and track it for cleanup
        pub fn set_and_track(&mut self, var: &str, value: &str) {
            self.track_removal(var);
            std::env::set_var(var, value);
        }

        /// Remove a variable and track it for cleanup
        pub fn remove_and_track(&mut self, var: &str) {
            self.track_removal(var);
            std::env::remove_var(var);
        }
    }

    impl Drop for EnvCleanup {
        fn drop(&mut self) {
            // Restore original values
            for (var, value) in &self.vars_to_restore {
                if let Some(val) = value {
                    std::env::set_var(var, val);
                } else {
                    std::env::remove_var(var);
                }
            }

            // Remove variables that didn't exist originally
            for var in &self.vars_to_remove {
                std::env::remove_var(var);
            }
        }
    }

    /// Assertion utilities for config testing
    pub mod assertions {
        use super::*;

        /// Assert that a protocol exists and has expected properties
        pub fn assert_protocol_exists(config: &Config, name: &str, expected_service: &str) {
            assert!(config.protocols.contains_key(name), "Protocol '{}' not found", name);
            let protocol = &config.protocols[name];
            assert_eq!(protocol.service, expected_service, "Protocol '{}' has wrong service", name);
        }

        /// Assert that a protocol has expected bucket
        pub fn assert_protocol_bucket(config: &Config, name: &str, expected_bucket: Option<&str>) {
            let protocol = &config.protocols[name];
            assert_eq!(protocol.bucket.as_deref(), expected_bucket, "Protocol '{}' has wrong bucket", name);
        }

        /// Assert that a protocol has expected region
        pub fn assert_protocol_region(config: &Config, name: &str, expected_region: Option<&str>) {
            let protocol = &config.protocols[name];
            assert_eq!(protocol.region.as_deref(), expected_region, "Protocol '{}' has wrong region", name);
        }

        /// Assert that a service exists and has expected properties
        pub fn assert_service_exists(config: &Config, name: &str) {
            assert!(config.services.contains_key(name), "Service '{}' not found", name);
        }

        /// Assert that a service has expected region
        pub fn assert_service_region(config: &Config, name: &str, expected_region: Option<&str>) {
            let service = &config.services[name];
            assert_eq!(service.default_region.as_deref(), expected_region, "Service '{}' has wrong region", name);
        }

        /// Assert that a service has expected endpoint
        pub fn assert_service_endpoint(config: &Config, name: &str, expected_endpoint: Option<&str>) {
            let service = &config.services[name];
            assert_eq!(service.endpoint.as_deref(), expected_endpoint, "Service '{}' has wrong endpoint", name);
        }

        /// Assert that key translation has expected properties
        pub fn assert_key_translation(config: &Config, protocol_name: &str, expected_downcase: bool, expected_partition: bool) {
            let protocol = &config.protocols[protocol_name];
            assert_eq!(protocol.key_translation.downcase, expected_downcase, "Protocol '{}' key translation downcase mismatch", protocol_name);
            assert_eq!(protocol.key_translation.partition, expected_partition, "Protocol '{}' key translation partition mismatch", protocol_name);
        }
    }
}