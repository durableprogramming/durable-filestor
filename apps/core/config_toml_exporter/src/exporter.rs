//! TOML export functionality for configuration objects

use durable_filestor_shared::{Config, Protocol, Result, ServiceConfig, toml_export, impl_toml_exporter};
use std::collections::HashMap;

/// Configuration TOML exporter
pub struct TomlExporter;

impl TomlExporter {
    impl_toml_exporter!(Config, "config");
    impl_toml_exporter!(Protocol, "protocol");
    impl_toml_exporter!(ServiceConfig, "service_config");
    
    /// Export protocols map to TOML string
    ///
    /// # Examples
    /// ```
    /// use durable_filestor_config_toml_exporter::TomlExporter;
    /// use durable_filestor_shared::{Protocol, KeyTranslation};
    /// use std::collections::HashMap;
    ///
    /// let mut protocols = HashMap::new();
    /// protocols.insert("my-s3".to_string(), Protocol {
    ///     service: "s3".to_string(),
    ///     bucket: Some("my-bucket".to_string()),
    ///     region: Some("us-east-1".to_string()),
    ///     path: None,
    ///     key_translation: KeyTranslation::default(),
    ///     parameters: HashMap::new(),
    /// });
    ///
    /// let toml_str = TomlExporter::export_protocols(&protocols).unwrap();
    /// ```
    pub fn export_protocols(protocols: &HashMap<String, Protocol>) -> Result<String> {
        toml_export!(protocols, "protocols")
    }

    /// Export services map to TOML string
    ///
    /// # Examples
    /// ```
    /// use durable_filestor_config_toml_exporter::TomlExporter;
    /// use durable_filestor_shared::ServiceConfig;
    /// use std::collections::HashMap;
    ///
    /// let mut services = HashMap::new();
    /// services.insert("s3".to_string(), ServiceConfig {
    ///     default_region: Some("us-west-2".to_string()),
    ///     endpoint: Some("https://s3.us-west-2.amazonaws.com".to_string()),
    ///     default_permissions: None,
    ///     parameters: HashMap::new(),
    /// });
    ///
    /// let toml_str = TomlExporter::export_services(&services).unwrap();
    /// ```
    pub fn export_services(services: &HashMap<String, ServiceConfig>) -> Result<String> {
        toml_export!(services, "services")
    }

    /// Export protocols map with formatting options
    pub fn export_protocols_with_options(protocols: &HashMap<String, Protocol>, pretty: bool) -> Result<String> {
        toml_export!(protocols, "protocols", pretty)
    }

    /// Export services map with formatting options  
    pub fn export_services_with_options(services: &HashMap<String, ServiceConfig>, pretty: bool) -> Result<String> {
        toml_export!(services, "services", pretty)
    }

    /// Validate that exported TOML can be parsed back
    ///
    /// # Examples
    /// ```
    /// use durable_filestor_config_toml_exporter::TomlExporter;
    /// use durable_filestor_shared::Config;
    ///
    /// let config = Config::default();
    /// let toml_str = TomlExporter::export_config(&config).unwrap();
    /// let is_valid = TomlExporter::validate_export(&toml_str).unwrap();
    /// assert!(is_valid);
    /// ```
    pub fn validate_export(toml_str: &str) -> Result<bool> {
        match toml::from_str::<Config>(toml_str) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Generic validation function for HashMap types
    fn validate_map_export<T>(toml_str: &str) -> Result<bool>
    where
        T: serde::de::DeserializeOwned,
    {
        match toml::from_str::<HashMap<String, T>>(toml_str) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Validate protocols TOML
    pub fn validate_protocols_export(toml_str: &str) -> Result<bool> {
        Self::validate_map_export::<Protocol>(toml_str)
    }

    /// Validate services TOML
    pub fn validate_services_export(toml_str: &str) -> Result<bool> {
        Self::validate_map_export::<ServiceConfig>(toml_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use durable_filestor_shared::{HashFunction, KeyTranslation};
    use std::collections::HashMap;

    mod config_tests {
        use super::*;

        #[test]
        fn test_export_empty_config() {
            let config = Config::default();
            let result = TomlExporter::export_config(&config);
            assert!(result.is_ok());

            let toml_str = result.unwrap();
            assert!(!toml_str.is_empty());

            // Validate that it can be parsed back
            let is_valid = TomlExporter::validate_export(&toml_str).unwrap();
            assert!(is_valid);
        }

        #[test]
        fn test_export_config_with_protocols() {
            let mut protocols = HashMap::new();
            let protocol = Protocol {
                service: "s3".to_string(),
                bucket: Some("test-bucket".to_string()),
                region: Some("us-east-1".to_string()),
                path: None,
                key_translation: KeyTranslation::default(),
                parameters: HashMap::new(),
            };
            protocols.insert("test-s3".to_string(), protocol);

            let mut services = HashMap::new();
            let service = ServiceConfig {
                default_region: Some("us-west-2".to_string()),
                endpoint: None,
                default_permissions: None,
                parameters: HashMap::new(),
            };
            services.insert("s3".to_string(), service);

            let config = Config {
                protocols,
                services,
            };

            let result = TomlExporter::export_config(&config);
            assert!(result.is_ok());

            let toml_str = result.unwrap();
            assert!(toml_str.contains("[protocols.test-s3]"));
            assert!(toml_str.contains("[services.s3]"));
            assert!(toml_str.contains("service = \"s3\""));
            assert!(toml_str.contains("default_region = \"us-west-2\""));
        }

        #[test]
        fn test_export_config_with_options_pretty() {
            let config = Config::default();
            let result = TomlExporter::export_config_with_options(&config, true);
            assert!(result.is_ok());
            let toml_str = result.unwrap();
            
            // Pretty formatting should include newlines and indentation
            assert!(toml_str.contains("\n"));
        }

        #[test]
        fn test_export_config_with_options_compact() {
            let config = Config::default();
            let result = TomlExporter::export_config_with_options(&config, false);
            assert!(result.is_ok());
            let toml_str = result.unwrap();
            
            // Compact formatting should be more condensed
            let pretty_result = TomlExporter::export_config_with_options(&config, true).unwrap();
            assert!(toml_str.len() <= pretty_result.len());
        }

        #[test]
        fn test_config_roundtrip() {
            let mut protocols = HashMap::new();
            let protocol = Protocol {
                service: "local".to_string(),
                bucket: None,
                region: None,
                path: Some("/tmp/test".to_string()),
                key_translation: KeyTranslation {
                    downcase: true,
                    partition: true,
                    partition_length: 3,
                    partition_depth: 2,
                    ..KeyTranslation::default()
                },
                parameters: HashMap::new(),
            };
            protocols.insert("test-local".to_string(), protocol);

            let config = Config {
                protocols,
                services: HashMap::new(),
            };

            let toml_str = TomlExporter::export_config(&config).unwrap();
            let parsed_config: Config = toml::from_str(&toml_str).unwrap();
            
            assert_eq!(config.protocols.len(), parsed_config.protocols.len());
            assert_eq!(config.services.len(), parsed_config.services.len());
            
            let original_protocol = config.protocols.get("test-local").unwrap();
            let parsed_protocol = parsed_config.protocols.get("test-local").unwrap();
            assert_eq!(original_protocol.service, parsed_protocol.service);
            assert_eq!(original_protocol.path, parsed_protocol.path);
        }
    }

    mod protocol_tests {
        use super::*;

        #[test]
        fn test_export_protocol_basic() {
            let protocol = Protocol {
                service: "s3".to_string(),
                bucket: Some("my-bucket".to_string()),
                region: Some("us-east-1".to_string()),
                path: None,
                key_translation: KeyTranslation::default(),
                parameters: HashMap::new(),
            };

            let result = TomlExporter::export_protocol(&protocol);
            assert!(result.is_ok());

            let toml_str = result.unwrap();
            assert!(toml_str.contains("service = \"s3\""));
            assert!(toml_str.contains("bucket = \"my-bucket\""));
            assert!(toml_str.contains("region = \"us-east-1\""));
        }

        #[test]
        fn test_export_protocol_with_key_translation() {
            let mut parameters = HashMap::new();
            parameters.insert("timeout".to_string(), "30".to_string());

            let protocol = Protocol {
                service: "s3".to_string(),
                bucket: Some("my-bucket".to_string()),
                region: Some("us-east-1".to_string()),
                path: None,
                key_translation: KeyTranslation {
                    downcase: true,
                    upcase: false,
                    hash_function: Some(HashFunction::Sha256),
                    hash_prefix_length: Some(8),
                    partition: true,
                    partition_length: 2,
                    partition_depth: 3,
                    partition_separator: "/".to_string(),
                    prefix: Some("prefix-".to_string()),
                    suffix: Some("-suffix".to_string()),
                    timestamp_suffix: false,
                    normalize_unicode: true,
                    strip_whitespace: true,
                    replace_spaces: Some("_".to_string()),
                    max_key_length: Some(255),
                },
                parameters,
            };

            let result = TomlExporter::export_protocol(&protocol);
            assert!(result.is_ok());

            let toml_str = result.unwrap();
            assert!(toml_str.contains("service = \"s3\""));
            assert!(toml_str.contains("bucket = \"my-bucket\""));
            assert!(toml_str.contains("downcase = true"));
            assert!(toml_str.contains("hash_function = \"sha256\""));
            assert!(toml_str.contains("partition_length = 2"));
            assert!(toml_str.contains("prefix = \"prefix-\""));
            assert!(toml_str.contains("timeout = \"30\""));
        }

        #[test]
        fn test_export_protocol_with_options() {
            let protocol = Protocol {
                service: "local".to_string(),
                bucket: None,
                region: None,
                path: Some("/tmp/files".to_string()),
                key_translation: KeyTranslation::default(),
                parameters: HashMap::new(),
            };

            let pretty_result = TomlExporter::export_protocol_with_options(&protocol, true);
            let compact_result = TomlExporter::export_protocol_with_options(&protocol, false);
            
            assert!(pretty_result.is_ok());
            assert!(compact_result.is_ok());
            
            let pretty_str = pretty_result.unwrap();
            let compact_str = compact_result.unwrap();
            
            // Pretty should generally be longer due to formatting
            assert!(pretty_str.len() >= compact_str.len());
            
            // Both should contain the same basic content
            assert!(pretty_str.contains("service = \"local\""));
            assert!(compact_str.contains("service = \"local\""));
        }

        #[test]
        fn test_protocol_roundtrip() {
            let mut parameters = HashMap::new();
            parameters.insert("custom_param".to_string(), "value123".to_string());
            
            let protocol = Protocol {
                service: "memory".to_string(),
                bucket: Some("test-bucket".to_string()),
                region: None,
                path: Some("/custom/path".to_string()),
                key_translation: KeyTranslation {
                    hash_function: Some(HashFunction::Blake2b),
                    hash_prefix_length: Some(16),
                    partition: true,
                    partition_length: 4,
                    partition_depth: 1,
                    normalize_unicode: false,
                    ..KeyTranslation::default()
                },
                parameters,
            };

            let toml_str = TomlExporter::export_protocol(&protocol).unwrap();
            let parsed_protocol: Protocol = toml::from_str(&toml_str).unwrap();
            
            assert_eq!(protocol.service, parsed_protocol.service);
            assert_eq!(protocol.bucket, parsed_protocol.bucket);
            assert_eq!(protocol.path, parsed_protocol.path);
            assert_eq!(protocol.key_translation.hash_function, parsed_protocol.key_translation.hash_function);
            assert_eq!(protocol.parameters, parsed_protocol.parameters);
        }
    }

    mod service_config_tests {
        use super::*;

        #[test]
        fn test_export_service_config_basic() {
            let service_config = ServiceConfig {
                default_region: Some("us-west-2".to_string()),
                endpoint: Some("https://s3.us-west-2.amazonaws.com".to_string()),
                default_permissions: Some("0644".to_string()),
                parameters: HashMap::new(),
            };

            let result = TomlExporter::export_service_config(&service_config);
            assert!(result.is_ok());

            let toml_str = result.unwrap();
            assert!(toml_str.contains("default_region = \"us-west-2\""));
            assert!(toml_str.contains("endpoint = \"https://s3.us-west-2.amazonaws.com\""));
            assert!(toml_str.contains("default_permissions = \"0644\""));
        }

        #[test]
        fn test_export_service_config_with_parameters() {
            let mut parameters = HashMap::new();
            parameters.insert("max_connections".to_string(), "100".to_string());
            parameters.insert("timeout_seconds".to_string(), "30".to_string());
            parameters.insert("retry_attempts".to_string(), "3".to_string());

            let service_config = ServiceConfig {
                default_region: Some("eu-central-1".to_string()),
                endpoint: None,
                default_permissions: None,
                parameters,
            };

            let result = TomlExporter::export_service_config(&service_config);
            assert!(result.is_ok());

            let toml_str = result.unwrap();
            assert!(toml_str.contains("default_region = \"eu-central-1\""));
            assert!(toml_str.contains("max_connections = \"100\""));
            assert!(toml_str.contains("timeout_seconds = \"30\""));
            assert!(toml_str.contains("retry_attempts = \"3\""));
        }

        #[test]
        fn test_export_service_config_minimal() {
            let service_config = ServiceConfig {
                default_region: None,
                endpoint: None,
                default_permissions: None,
                parameters: HashMap::new(),
            };

            let result = TomlExporter::export_service_config(&service_config);
            assert!(result.is_ok());
            
            let toml_str = result.unwrap();
            // Should produce valid TOML even with all None values
            assert!(!toml_str.is_empty());
        }

        #[test]
        fn test_service_config_roundtrip() {
            let mut parameters = HashMap::new();
            parameters.insert("pool_size".to_string(), "50".to_string());
            parameters.insert("buffer_size".to_string(), "4096".to_string());
            
            let service_config = ServiceConfig {
                default_region: Some("ap-southeast-2".to_string()),
                endpoint: Some("https://custom.endpoint.com".to_string()),
                default_permissions: Some("0755".to_string()),
                parameters,
            };

            let toml_str = TomlExporter::export_service_config(&service_config).unwrap();
            let parsed_config: ServiceConfig = toml::from_str(&toml_str).unwrap();
            
            assert_eq!(service_config.default_region, parsed_config.default_region);
            assert_eq!(service_config.endpoint, parsed_config.endpoint);
            assert_eq!(service_config.default_permissions, parsed_config.default_permissions);
            assert_eq!(service_config.parameters, parsed_config.parameters);
        }
    }

    mod map_tests {
        use super::*;

        #[test]
        fn test_export_protocols_map() {
            let mut protocols = HashMap::new();

            let protocol1 = Protocol {
                service: "s3".to_string(),
                bucket: Some("bucket1".to_string()),
                region: Some("us-east-1".to_string()),
                path: None,
                key_translation: KeyTranslation::default(),
                parameters: HashMap::new(),
            };

            let protocol2 = Protocol {
                service: "local".to_string(),
                bucket: None,
                region: None,
                path: Some("/tmp/files".to_string()),
                key_translation: KeyTranslation {
                    downcase: true,
                    partition: true,
                    partition_length: 3,
                    ..KeyTranslation::default()
                },
                parameters: HashMap::new(),
            };

            protocols.insert("prod-s3".to_string(), protocol1);
            protocols.insert("local-dev".to_string(), protocol2);

            let result = TomlExporter::export_protocols(&protocols);
            assert!(result.is_ok());

            let toml_str = result.unwrap();
            assert!(toml_str.contains("[prod-s3]"));
            assert!(toml_str.contains("service = \"s3\""));
            assert!(toml_str.contains("[local-dev]"));
            assert!(toml_str.contains("service = \"local\""));
            assert!(toml_str.contains("bucket = \"bucket1\""));
            assert!(toml_str.contains("path = \"/tmp/files\""));
        }

        #[test]
        fn test_export_services_map() {
            let mut services = HashMap::new();

            let service1 = ServiceConfig {
                default_region: Some("us-east-1".to_string()),
                endpoint: Some("https://s3.amazonaws.com".to_string()),
                default_permissions: None,
                parameters: HashMap::new(),
            };

            let service2 = ServiceConfig {
                default_region: None,
                endpoint: None,
                default_permissions: Some("0755".to_string()),
                parameters: HashMap::new(),
            };

            services.insert("s3".to_string(), service1);
            services.insert("local".to_string(), service2);

            let result = TomlExporter::export_services(&services);
            assert!(result.is_ok());

            let toml_str = result.unwrap();
            assert!(toml_str.contains("[s3]"));
            assert!(toml_str.contains("default_region = \"us-east-1\""));
            assert!(toml_str.contains("[local]"));
            assert!(toml_str.contains("default_permissions = \"0755\""));
        }

        #[test]
        fn test_export_empty_maps() {
            let protocols: HashMap<String, Protocol> = HashMap::new();
            let services: HashMap<String, ServiceConfig> = HashMap::new();

            let protocols_result = TomlExporter::export_protocols(&protocols);
            let services_result = TomlExporter::export_services(&services);
            
            assert!(protocols_result.is_ok());
            assert!(services_result.is_ok());
            
            // Empty maps produce empty TOML strings, which is correct behavior
            let protocols_toml = protocols_result.unwrap();
            let services_toml = services_result.unwrap();
            
            // Empty maps should produce empty strings when serialized to TOML
            assert!(protocols_toml.is_empty());
            assert!(services_toml.is_empty());
            
            // But they should still validate as correct TOML
            assert!(TomlExporter::validate_protocols_export(&protocols_toml).unwrap());
            assert!(TomlExporter::validate_services_export(&services_toml).unwrap());
        }

        #[test]
        fn test_export_maps_with_options() {
            let mut protocols = HashMap::new();
            let protocol = Protocol {
                service: "memory".to_string(),
                bucket: None,
                region: None,
                path: None,
                key_translation: KeyTranslation::default(),
                parameters: HashMap::new(),
            };
            protocols.insert("mem".to_string(), protocol);

            let pretty_result = TomlExporter::export_protocols_with_options(&protocols, true);
            let compact_result = TomlExporter::export_protocols_with_options(&protocols, false);
            
            assert!(pretty_result.is_ok());
            assert!(compact_result.is_ok());
            
            let pretty_str = pretty_result.unwrap();
            let compact_str = compact_result.unwrap();
            
            assert!(pretty_str.contains("[mem]"));
            assert!(compact_str.contains("[mem]"));
        }

        #[test]
        fn test_protocols_map_roundtrip() {
            let mut protocols = HashMap::new();
            
            let mut params1 = HashMap::new();
            params1.insert("region_fallback".to_string(), "us-west-1".to_string());
            
            let protocol1 = Protocol {
                service: "s3".to_string(),
                bucket: Some("primary-bucket".to_string()),
                region: Some("us-east-1".to_string()),
                path: Some("/data".to_string()),
                key_translation: KeyTranslation {
                    hash_function: Some(HashFunction::Sha1),
                    prefix: Some("prod-".to_string()),
                    suffix: Some(".data".to_string()),
                    ..KeyTranslation::default()
                },
                parameters: params1,
            };
            
            protocols.insert("primary".to_string(), protocol1);

            let toml_str = TomlExporter::export_protocols(&protocols).unwrap();
            let parsed_protocols: HashMap<String, Protocol> = toml::from_str(&toml_str).unwrap();
            
            assert_eq!(protocols.len(), parsed_protocols.len());
            
            let original = protocols.get("primary").unwrap();
            let parsed = parsed_protocols.get("primary").unwrap();
            
            assert_eq!(original.service, parsed.service);
            assert_eq!(original.bucket, parsed.bucket);
            assert_eq!(original.parameters, parsed.parameters);
        }
    }

    mod validation_tests {
        use super::*;

        #[test]
        fn test_validate_export() {
            let config = Config::default();
            let toml_str = TomlExporter::export_config(&config).unwrap();

            let is_valid = TomlExporter::validate_export(&toml_str).unwrap();
            assert!(is_valid);

            // Test with invalid TOML
            let invalid_toml = "invalid [ toml content";
            let is_valid_invalid = TomlExporter::validate_export(invalid_toml).unwrap();
            assert!(!is_valid_invalid);
        }

        #[test]
        fn test_validate_protocols_export() {
            let mut protocols = HashMap::new();
            let protocol = Protocol {
                service: "test".to_string(),
                bucket: None,
                region: None,
                path: None,
                key_translation: KeyTranslation::default(),
                parameters: HashMap::new(),
            };
            protocols.insert("test".to_string(), protocol);

            let toml_str = TomlExporter::export_protocols(&protocols).unwrap();
            let is_valid = TomlExporter::validate_protocols_export(&toml_str).unwrap();
            assert!(is_valid);

            // Test with invalid TOML
            let is_valid_invalid = TomlExporter::validate_protocols_export("invalid { toml").unwrap();
            assert!(!is_valid_invalid);
        }

        #[test]
        fn test_validate_services_export() {
            let mut services = HashMap::new();
            let service = ServiceConfig {
                default_region: Some("test".to_string()),
                endpoint: None,
                default_permissions: None,
                parameters: HashMap::new(),
            };
            services.insert("test".to_string(), service);

            let toml_str = TomlExporter::export_services(&services).unwrap();
            let is_valid = TomlExporter::validate_services_export(&toml_str).unwrap();
            assert!(is_valid);

            // Test with invalid TOML
            let is_valid_invalid = TomlExporter::validate_services_export("malformed ] toml").unwrap();
            assert!(!is_valid_invalid);
        }

        #[test]
        fn test_validate_complex_structures() {
            // Create a complex config structure to test validation edge cases
            let mut protocols = HashMap::new();
            let mut parameters = HashMap::new();
            parameters.insert("special_chars".to_string(), "value with \"quotes\" and \\backslashes".to_string());
            parameters.insert("unicode".to_string(), "こんにちは世界".to_string());
            
            let protocol = Protocol {
                service: "test-service".to_string(),
                bucket: Some("bucket-with-dashes".to_string()),
                region: Some("us-east-1".to_string()),
                path: Some("/path/with/slashes".to_string()),
                key_translation: KeyTranslation {
                    downcase: true,
                    upcase: false,
                    hash_function: Some(HashFunction::Sha256),
                    hash_prefix_length: Some(10),
                    partition: true,
                    partition_length: 2,
                    partition_depth: 5,
                    partition_separator: "-".to_string(),
                    prefix: Some("complex-prefix-".to_string()),
                    suffix: Some("-complex.suffix".to_string()),
                    timestamp_suffix: true,
                    normalize_unicode: true,
                    strip_whitespace: false,
                    replace_spaces: Some("_".to_string()),
                    max_key_length: Some(512),
                },
                parameters,
            };
            protocols.insert("complex-protocol".to_string(), protocol);

            let mut services = HashMap::new();
            let mut service_params = HashMap::new();
            service_params.insert("array_like_value".to_string(), "[1,2,3]".to_string());
            service_params.insert("json_like_value".to_string(), "{\"key\": \"value\"}".to_string());
            
            let service = ServiceConfig {
                default_region: Some("complex-region-name".to_string()),
                endpoint: Some("https://complex.endpoint.example.com:8443/path".to_string()),
                default_permissions: Some("0644".to_string()),
                parameters: service_params,
            };
            services.insert("complex-service".to_string(), service);

            let config = Config {
                protocols,
                services,
            };

            let toml_str = TomlExporter::export_config(&config).unwrap();
            let is_valid = TomlExporter::validate_export(&toml_str).unwrap();
            assert!(is_valid);
            
            // Ensure complex structure can be parsed back
            let parsed_config: Config = toml::from_str(&toml_str).unwrap();
            assert_eq!(config.protocols.len(), parsed_config.protocols.len());
            assert_eq!(config.services.len(), parsed_config.services.len());
        }
    }

    mod error_handling_tests {
        use super::*;

        #[test]
        fn test_export_error_messages() {
            // These tests verify that our error messages are helpful and consistent
            let config = Config::default();
            let result = TomlExporter::export_config(&config);
            match result {
                Ok(_) => {}, // Expected success
                Err(e) => {
                    let error_msg = format!("{}", e);
                    assert!(error_msg.contains("config"));
                    assert!(error_msg.contains("TOML"));
                }
            }
        }

        #[test]
        fn test_validation_error_resilience() {
            // Test various malformed TOML strings
            // Note: Empty string is actually valid TOML (empty document)
            let malformed_inputs = vec![
                "[",
                "[section",
                "key = ",
                "key = \"unclosed string",
                "[duplicate]\n[duplicate]",
                "invalid = true\ninvalid = false",
                "\x00\x01\x02", // binary data
            ];

            for input in malformed_inputs {
                let result = TomlExporter::validate_export(input);
                assert!(result.is_ok(), "Validation should not panic on malformed input: {}", input);
                assert!(!result.unwrap(), "Malformed input should not validate as true: {}", input);
            }
            
            // Test that empty string is valid TOML (empty document)
            let empty_result = TomlExporter::validate_export("");
            assert!(empty_result.is_ok());
            assert!(empty_result.unwrap(), "Empty string should validate as true (empty TOML document)");
        }

        #[test]
        fn test_edge_case_data_structures() {
            // Test with edge cases that might cause serialization issues
            let mut protocols = HashMap::new();
            
            // Protocol with empty/extreme values
            let edge_case_protocol = Protocol {
                service: "".to_string(), // Empty service name
                bucket: Some("".to_string()), // Empty bucket name
                region: None,
                path: Some("/".to_string()), // Root path
                key_translation: KeyTranslation {
                    partition_length: 0, // Zero partition length
                    partition_depth: 100, // High partition depth
                    partition_separator: "".to_string(), // Empty separator
                    max_key_length: Some(0), // Zero max length
                    ..KeyTranslation::default()
                },
                parameters: HashMap::new(),
            };
            protocols.insert("".to_string(), edge_case_protocol); // Empty protocol name

            // Should handle edge cases gracefully
            let result = TomlExporter::export_protocols(&protocols);
            assert!(result.is_ok());
            
            let toml_str = result.unwrap();
            let is_valid = TomlExporter::validate_protocols_export(&toml_str).unwrap();
            assert!(is_valid);
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_full_export_import_cycle() {
            // Create a comprehensive config with all features
            let mut protocols = HashMap::new();
            let mut services = HashMap::new();

            // S3 protocol with full configuration
            let mut s3_params = HashMap::new();
            s3_params.insert("multipart_threshold".to_string(), "8388608".to_string());
            s3_params.insert("multipart_chunksize".to_string(), "8388608".to_string());
            
            let s3_protocol = Protocol {
                service: "s3".to_string(),
                bucket: Some("production-data".to_string()),
                region: Some("us-east-1".to_string()),
                path: Some("/application/data".to_string()),
                key_translation: KeyTranslation {
                    downcase: true,
                    upcase: false,
                    hash_function: Some(HashFunction::Sha256),
                    hash_prefix_length: Some(8),
                    partition: true,
                    partition_length: 2,
                    partition_depth: 3,
                    partition_separator: "/".to_string(),
                    prefix: Some("prod-".to_string()),
                    suffix: Some("-v1".to_string()),
                    timestamp_suffix: false,
                    normalize_unicode: true,
                    strip_whitespace: true,
                    replace_spaces: Some("-".to_string()),
                    max_key_length: Some(1024),
                },
                parameters: s3_params,
            };
            protocols.insert("prod-s3".to_string(), s3_protocol);

            // Local protocol for development
            let local_protocol = Protocol {
                service: "local".to_string(),
                bucket: None,
                region: None,
                path: Some("/tmp/dev-storage".to_string()),
                key_translation: KeyTranslation {
                    partition: false,
                    normalize_unicode: false,
                    ..KeyTranslation::default()
                },
                parameters: HashMap::new(),
            };
            protocols.insert("dev-local".to_string(), local_protocol);

            // S3 service configuration
            let mut s3_service_params = HashMap::new();
            s3_service_params.insert("signature_version".to_string(), "s3v4".to_string());
            s3_service_params.insert("addressing_style".to_string(), "virtual".to_string());
            
            let s3_service = ServiceConfig {
                default_region: Some("us-east-1".to_string()),
                endpoint: None,
                default_permissions: Some("0644".to_string()),
                parameters: s3_service_params,
            };
            services.insert("s3".to_string(), s3_service);

            // Local service configuration
            let local_service = ServiceConfig {
                default_region: None,
                endpoint: None,
                default_permissions: Some("0755".to_string()),
                parameters: HashMap::new(),
            };
            services.insert("local".to_string(), local_service);

            let original_config = Config {
                protocols,
                services,
            };

            // Export to TOML
            let toml_str = TomlExporter::export_config(&original_config).unwrap();
            
            // Validate the exported TOML
            assert!(TomlExporter::validate_export(&toml_str).unwrap());
            
            // Import back from TOML
            let imported_config: Config = toml::from_str(&toml_str).unwrap();
            
            // Verify complete round-trip integrity
            assert_eq!(original_config.protocols.len(), imported_config.protocols.len());
            assert_eq!(original_config.services.len(), imported_config.services.len());
            
            // Check protocols
            for (name, original_protocol) in &original_config.protocols {
                let imported_protocol = imported_config.protocols.get(name).unwrap();
                assert_eq!(original_protocol.service, imported_protocol.service);
                assert_eq!(original_protocol.bucket, imported_protocol.bucket);
                assert_eq!(original_protocol.region, imported_protocol.region);
                assert_eq!(original_protocol.path, imported_protocol.path);
                assert_eq!(original_protocol.parameters, imported_protocol.parameters);
                
                // Check key translation
                let orig_kt = &original_protocol.key_translation;
                let imp_kt = &imported_protocol.key_translation;
                assert_eq!(orig_kt.downcase, imp_kt.downcase);
                assert_eq!(orig_kt.hash_function, imp_kt.hash_function);
                assert_eq!(orig_kt.partition, imp_kt.partition);
                assert_eq!(orig_kt.prefix, imp_kt.prefix);
            }
            
            // Check services
            for (name, original_service) in &original_config.services {
                let imported_service = imported_config.services.get(name).unwrap();
                assert_eq!(original_service.default_region, imported_service.default_region);
                assert_eq!(original_service.endpoint, imported_service.endpoint);
                assert_eq!(original_service.default_permissions, imported_service.default_permissions);
                assert_eq!(original_service.parameters, imported_service.parameters);
            }
        }

        #[test]
        fn test_cross_format_compatibility() {
            // Test that both pretty and compact formats produce equivalent results
            let config = Config::default();
            
            let pretty_toml = TomlExporter::export_config_with_options(&config, true).unwrap();
            let compact_toml = TomlExporter::export_config_with_options(&config, false).unwrap();
            
            // Both should be valid
            assert!(TomlExporter::validate_export(&pretty_toml).unwrap());
            assert!(TomlExporter::validate_export(&compact_toml).unwrap());
            
            // Both should parse to the same config
            let pretty_parsed: Config = toml::from_str(&pretty_toml).unwrap();
            let compact_parsed: Config = toml::from_str(&compact_toml).unwrap();
            
            assert_eq!(pretty_parsed.protocols.len(), compact_parsed.protocols.len());
            assert_eq!(pretty_parsed.services.len(), compact_parsed.services.len());
        }
    }
}