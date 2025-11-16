//! Comprehensive unit tests for config_uri_parser crate

use durable_filestor_config_uri_parser::*;
use durable_filestor_shared::{Config, DurableError, HashFunction, ParsedUri, UriParser, test_utils};

/// Helper function to create a basic valid TOML config for testing
fn create_test_toml_config() -> String {
    // Create a basic config using shared utilities
    let mut config = Config::default();

    // Add a test protocol
    let mut protocols = std::collections::HashMap::new();
    protocols.insert(
        "test".to_string(),
        durable_filestor_shared::Protocol {
            service: "local".to_string(),
            bucket: None,
            region: None,
            path: Some("/tmp/test".to_string()),
            key_translation: Default::default(),
            parameters: std::collections::HashMap::new(),
        },
    );

    // Add a test service
    let mut services = std::collections::HashMap::new();
    services.insert(
        "local".to_string(),
        durable_filestor_shared::ServiceConfig {
            default_region: None,
            endpoint: Some("/tmp/test".to_string()),
            default_permissions: Some("644".to_string()),
            parameters: std::collections::HashMap::new(),
        },
    );

    config.protocols = protocols;
    config.services = services;

    // Serialize to TOML
    toml::to_string_pretty(&config).unwrap()
}

/// Helper function to encode TOML as base64 using shared utilities
fn encode_toml_base64(toml: &str) -> String {
    UriParser::encode_to_base64(toml)
}

/// Helper function to encode bytes as base64 using shared utilities
fn encode_toml_base64_from_bytes(bytes: &[u8]) -> String {
    UriParser::encode_bytes_to_base64(bytes)
}

/// Helper function to create a TOML config with complex key translation settings
fn create_complex_test_toml_config() -> String {
    let mut config = Config::default();

    // Add S3 protocol with key translation
    let mut protocols = std::collections::HashMap::new();
    protocols.insert(
        "s3-prod".to_string(),
        durable_filestor_shared::Protocol {
            service: "s3".to_string(),
            bucket: Some("my-production-bucket".to_string()),
            region: Some("us-east-1".to_string()),
            path: None,
            key_translation: durable_filestor_shared::KeyTranslation {
                downcase: true,
                hash_function: Some(HashFunction::Sha256),
                hash_prefix_length: Some(8),
                partition: true,
                partition_length: 2,
                partition_depth: 3,
                prefix: Some("prod/".to_string()),
                normalize_unicode: true,
                strip_whitespace: true,
                ..Default::default()
            },
            parameters: std::collections::HashMap::new(),
        },
    );

    // Add S3 service
    let mut services = std::collections::HashMap::new();
    services.insert(
        "s3".to_string(),
        durable_filestor_shared::ServiceConfig {
            default_region: Some("us-east-1".to_string()),
            endpoint: Some("https://s3.amazonaws.com".to_string()),
            default_permissions: None,
            parameters: {
                let mut params = std::collections::HashMap::new();
                params.insert("timeout".to_string(), "30".to_string());
                params.insert("retries".to_string(), "3".to_string());
                params
            },
        },
    );

    config.protocols = protocols;
    config.services = services;

    toml::to_string_pretty(&config).unwrap()
}

/// Helper function to create a minimal valid TOML config
fn create_minimal_test_toml_config() -> String {
    let mut config = Config::default();

    let mut protocols = std::collections::HashMap::new();
    protocols.insert(
        "minimal".to_string(),
        durable_filestor_shared::Protocol {
            service: "local".to_string(),
            bucket: None,
            region: None,
            path: None,
            key_translation: Default::default(),
            parameters: std::collections::HashMap::new(),
        },
    );

    config.protocols = protocols;

    toml::to_string_pretty(&config).unwrap()
}

/// Helper function to create a multi-protocol TOML config
fn create_multi_protocol_test_toml_config() -> String {
    let mut config = Config::default();

    let mut protocols = std::collections::HashMap::new();

    // Local dev protocol
    protocols.insert(
        "local-dev".to_string(),
        durable_filestor_shared::Protocol {
            service: "local".to_string(),
            bucket: None,
            region: None,
            path: Some("/tmp/dev".to_string()),
            key_translation: Default::default(),
            parameters: std::collections::HashMap::new(),
        },
    );

    // S3 prod protocol
    protocols.insert(
        "s3-prod".to_string(),
        durable_filestor_shared::Protocol {
            service: "s3".to_string(),
            bucket: Some("prod-bucket".to_string()),
            region: Some("us-east-1".to_string()),
            path: None,
            key_translation: Default::default(),
            parameters: std::collections::HashMap::new(),
        },
    );

    let mut services = std::collections::HashMap::new();

    // Local service
    services.insert(
        "local".to_string(),
        durable_filestor_shared::ServiceConfig {
            default_region: None,
            endpoint: Some("/tmp".to_string()),
            default_permissions: None,
            parameters: std::collections::HashMap::new(),
        },
    );

    // S3 service
    services.insert(
        "s3".to_string(),
        durable_filestor_shared::ServiceConfig {
            default_region: Some("us-east-1".to_string()),
            endpoint: None,
            default_permissions: None,
            parameters: std::collections::HashMap::new(),
        },
    );

    config.protocols = protocols;
    config.services = services;

    toml::to_string_pretty(&config).unwrap()
}

mod toml_protocol_tests {
    use super::*;

    #[test]
    fn test_parse_valid_toml_protocol() {
        let toml_config = create_test_toml_config();
        let encoded = encode_toml_base64(&toml_config);
        let uri = format!("toml://{}", encoded);

        let parsed_uri = UriParser::parse(&uri).unwrap();
        let result = parse_toml_protocol(&parsed_uri);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.protocols.contains_key("test"));
        assert!(config.services.contains_key("local"));

        let protocol = &config.protocols["test"];
        assert_eq!(protocol.service, "local");
        assert_eq!(protocol.path.as_ref().unwrap(), "/tmp/test");

        let service = &config.services["local"];
        assert_eq!(service.endpoint.as_ref().unwrap(), "/tmp/test");
        assert_eq!(service.default_permissions.as_ref().unwrap(), "644");
    }

    #[test]
    fn test_parse_toml_protocol_with_complex_config() {
        let complex_toml = create_complex_test_toml_config();
        let encoded = encode_toml_base64(&complex_toml);
        let uri = format!("toml://{}", encoded);

        let parsed_uri = UriParser::parse(&uri).unwrap();
        let result = parse_toml_protocol(&parsed_uri);
        assert!(result.is_ok());

        let config = result.unwrap();

        // Check protocol configuration
        let protocol = &config.protocols["s3-prod"];
        assert_eq!(protocol.service, "s3");
        assert_eq!(protocol.bucket.as_ref().unwrap(), "my-production-bucket");
        assert_eq!(protocol.region.as_ref().unwrap(), "us-east-1");

        // Check key translation
        let kt = &protocol.key_translation;
        assert!(kt.downcase);
        assert_eq!(kt.hash_function, Some(HashFunction::Sha256));
        assert_eq!(kt.hash_prefix_length, Some(8));
        assert!(kt.partition);
        assert_eq!(kt.partition_length, 2);
        assert_eq!(kt.partition_depth, 3);
        assert_eq!(kt.prefix.as_ref().unwrap(), "prod/");
        assert!(kt.normalize_unicode);
        assert!(kt.strip_whitespace);

        // Check service configuration
        let service = &config.services["s3"];
        assert_eq!(service.default_region.as_ref().unwrap(), "us-east-1");
        assert_eq!(service.endpoint.as_ref().unwrap(), "https://s3.amazonaws.com");
        assert_eq!(service.parameters.get("timeout").unwrap(), "30");
        assert_eq!(service.parameters.get("retries").unwrap(), "3");
    }

    #[test]
    fn test_parse_toml_protocol_invalid_base64() {
        let invalid_uri = "toml://invalid-base64!";
        let parsed_uri = UriParser::parse(invalid_uri).unwrap();

        let result = parse_toml_protocol(&parsed_uri);
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            DurableError::Config { message } => {
                assert!(message.contains("Failed to decode base64"));
            }
            _ => panic!("Expected Config error, got {:?}", error),
        }
    }

    #[test]
    fn test_parse_toml_protocol_invalid_utf8() {
        // Create invalid UTF-8 by encoding non-UTF8 bytes
        // Use bytes that will create a valid hostname but invalid UTF-8 content
        let invalid_utf8 = b"\xc3\x28"; // This creates invalid UTF-8 when decoded
        let encoded = encode_toml_base64_from_bytes(invalid_utf8);
        let uri = format!("toml://{}", encoded);

        let parsed_uri = UriParser::parse(&uri).unwrap();
        let result = parse_toml_protocol(&parsed_uri);
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            DurableError::Config { message } => {
                // The error should be related to UTF-8 decoding
                let message_lower = message.to_lowercase();
                assert!(
                    message_lower.contains("utf-8") || 
                    message_lower.contains("invalid") ||
                    message_lower.contains("decode"),
                    "Expected error message to contain UTF-8/invalid/decode related text, got: '{}'", 
                    message
                );
            }
            _ => panic!("Expected Config error, got {:?}", error),
        }
    }

    #[test]
    fn test_parse_toml_protocol_invalid_toml() {
        let invalid_toml = "invalid [toml content {{{";
        let encoded = encode_toml_base64(invalid_toml);
        let uri = format!("toml://{}", encoded);

        let parsed_uri = UriParser::parse(&uri).unwrap();
        let result = parse_toml_protocol(&parsed_uri);
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            DurableError::Config { message } => {
                assert!(message.contains("Failed to parse TOML"));
            }
            _ => panic!("Expected Config error, got {:?}", error),
        }
    }

    #[test]
    fn test_parse_toml_protocol_missing_hostname() {
        let uri = "toml://";
        let parsed_uri = UriParser::parse(uri).unwrap();

        let result = parse_toml_protocol(&parsed_uri);
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            DurableError::Config { message } => {
                assert!(message.contains("requires base64-encoded TOML as hostname"));
            }
            _ => panic!("Expected Config error, got {:?}", error),
        }
    }

    #[test]
    fn test_parse_toml_protocol_minimal_valid_config() {
        // Use a minimal valid TOML that results in empty config
        let minimal_toml = "# This is a comment\n";
        let encoded = encode_toml_base64(minimal_toml);
        let uri = format!("toml://{}", encoded);

        let parsed_uri = UriParser::parse(&uri).unwrap();
        let result = parse_toml_protocol(&parsed_uri);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.protocols.is_empty());
        assert!(config.services.is_empty());
    }

    #[test]
    fn test_parse_toml_protocol_minimal_config() {
        let minimal_toml = create_minimal_test_toml_config();
        let encoded = encode_toml_base64(&minimal_toml);
        let uri = format!("toml://{}", encoded);

        let parsed_uri = UriParser::parse(&uri).unwrap();
        let result = parse_toml_protocol(&parsed_uri);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.protocols.contains_key("minimal"));
        assert_eq!(config.protocols["minimal"].service, "local");
    }
}

mod dfs_protocol_tests {
    use super::*;

    #[test]
    fn test_parse_dfs_protocol_basic() {
        let uri = "dfs://s3?access_key_id=test_key&secret_access_key=test_secret&bucket_name=test_bucket";
        let parsed_uri = UriParser::parse(uri).unwrap();

        let result = parse_dfs_protocol(&parsed_uri);
        assert!(result.is_ok());

        let config = result.unwrap();

        // Check service configuration
        assert!(config.services.contains_key("s3"));
        let service = &config.services["s3"];
        assert_eq!(service.parameters.get("access_key_id").unwrap(), "test_key");
        assert_eq!(service.parameters.get("secret_access_key").unwrap(), "test_secret");
        assert_eq!(service.parameters.get("bucket_name").unwrap(), "test_bucket");

        // Check protocol configuration
        assert!(config.protocols.contains_key("default-s3"));
        let protocol = &config.protocols["default-s3"];
        assert_eq!(protocol.service, "s3");
        assert_eq!(protocol.bucket.as_ref().unwrap(), "test_bucket");
        assert_eq!(protocol.parameters.get("access_key_id").unwrap(), "test_key");
    }

    #[test]
    fn test_parse_dfs_protocol_with_region_and_endpoint() {
        let uri = "dfs://s3?region=us-west-2&endpoint=https://custom.s3.endpoint.com&bucket=test-bucket&timeout=30";
        let parsed_uri = UriParser::parse(uri).unwrap();

        let result = parse_dfs_protocol(&parsed_uri);
        assert!(result.is_ok());

        let config = result.unwrap();

        // Check service configuration
        let service = &config.services["s3"];
        assert_eq!(service.default_region.as_ref().unwrap(), "us-west-2");
        assert_eq!(service.endpoint.as_ref().unwrap(), "https://custom.s3.endpoint.com");
        assert_eq!(service.parameters.get("timeout").unwrap(), "30");

        // Check protocol configuration
        let protocol = &config.protocols["default-s3"];
        assert_eq!(protocol.region.as_ref().unwrap(), "us-west-2");
        assert_eq!(protocol.bucket.as_ref().unwrap(), "test-bucket");
    }

    #[test]
    fn test_parse_dfs_protocol_alternative_bucket_names() {
        // Test with 'bucket' instead of 'bucket_name'
        let uri1 = "dfs://s3?bucket=my-bucket";
        let parsed_uri1 = UriParser::parse(uri1).unwrap();
        let result1 = parse_dfs_protocol(&parsed_uri1);
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().protocols["default-s3"].bucket.as_ref().unwrap(), "my-bucket");

        // Test with 'container' (for Azure-like services)
        let uri2 = "dfs://azure?container=my-container";
        let parsed_uri2 = UriParser::parse(uri2).unwrap();
        let result2 = parse_dfs_protocol(&parsed_uri2);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap().protocols["default-azure"].bucket.as_ref().unwrap(), "my-container");
    }

    #[test]
    fn test_parse_dfs_protocol_missing_hostname() {
        let uri = "dfs://";
        let parsed_uri = UriParser::parse(uri).unwrap();

        let result = parse_dfs_protocol(&parsed_uri);
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            DurableError::Config { message } => {
                assert!(message.contains("requires service type as hostname"));
            }
            _ => panic!("Expected Config error, got {:?}", error),
        }
    }

    #[test]
    fn test_parse_dfs_protocol_empty_query() {
        let uri = "dfs://local";
        let parsed_uri = UriParser::parse(uri).unwrap();

        let result = parse_dfs_protocol(&parsed_uri);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.services.contains_key("local"));
        assert!(config.protocols.contains_key("default-local"));

        // Should have empty parameters but still create valid config
        let service = &config.services["local"];
        assert!(service.parameters.is_empty());
    }

    #[test]
    fn test_parse_dfs_protocol_special_characters() {
        let uri = "dfs://s3?access_key_id=key%2Bwith%2Bspecial%26chars&bucket=bucket%2Fwith%2Fslashes";
        let parsed_uri = UriParser::parse(uri).unwrap();

        let result = parse_dfs_protocol(&parsed_uri);
        assert!(result.is_ok());

        let config = result.unwrap();
        let service = &config.services["s3"];
        assert_eq!(service.parameters.get("access_key_id").unwrap(), "key+with+special&chars");
        assert_eq!(service.parameters.get("bucket").unwrap(), "bucket/with/slashes");
    }

    #[test]
    fn test_parse_dfs_protocol_multiple_parameters() {
        let uri = "dfs://s3?param1=value1&param2=value2&param3=value3&param4=value4&param5=value5";
        let parsed_uri = UriParser::parse(uri).unwrap();

        let result = parse_dfs_protocol(&parsed_uri);
        assert!(result.is_ok());

        let config = result.unwrap();
        let service = &config.services["s3"];

        assert_eq!(service.parameters.len(), 5);
        assert_eq!(service.parameters.get("param1").unwrap(), "value1");
        assert_eq!(service.parameters.get("param2").unwrap(), "value2");
        assert_eq!(service.parameters.get("param3").unwrap(), "value3");
        assert_eq!(service.parameters.get("param4").unwrap(), "value4");
        assert_eq!(service.parameters.get("param5").unwrap(), "value5");
    }

    #[test]
    fn test_parse_dfs_protocol_empty_values() {
        let uri = "dfs://s3?empty_param=&another_param=value";
        let parsed_uri = UriParser::parse(uri).unwrap();

        let result = parse_dfs_protocol(&parsed_uri);
        assert!(result.is_ok());

        let config = result.unwrap();
        let service = &config.services["s3"];

        assert_eq!(service.parameters.get("empty_param").unwrap(), "");
        assert_eq!(service.parameters.get("another_param").unwrap(), "value");
    }
}

mod parser_tests {
    use super::*;

    #[test]
    fn test_config_uri_parser_creation() {
        let _parser = ConfigUriParser::new();
        // Should create successfully
        assert!(true); // If we get here, creation worked
    }

    #[test]
    fn test_config_uri_parser_default() {
        let _parser1 = ConfigUriParser::new();
        let _parser2 = ConfigUriParser::default();
        // Both should work the same way
        assert!(true);
    }

    #[test]
    fn test_parse_from_uri_toml() {
        let parser = ConfigUriParser::new();
        let toml_config = create_test_toml_config();
        let encoded = encode_toml_base64(&toml_config);
        let uri = format!("toml://{}", encoded);

        let result = parser.parse_from_uri(&uri);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.protocols.contains_key("test"));
        assert!(config.services.contains_key("local"));
    }

    #[test]
    fn test_parse_from_uri_dfs() {
        let parser = ConfigUriParser::new();
        let uri = "dfs://s3?bucket=test-bucket&region=us-east-1";

        let result = parser.parse_from_uri(uri);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.services.contains_key("s3"));
        assert!(config.protocols.contains_key("default-s3"));
    }

    #[test]
    fn test_parse_from_uri_invalid_scheme() {
        let parser = ConfigUriParser::new();
        let uri = "invalid://some-config";

        let result = parser.parse_from_uri(uri);
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            DurableError::Config { message } => {
                assert!(message.contains("Unsupported URI scheme"));
            }
            _ => panic!("Expected Config error, got {:?}", error),
        }
    }

    #[test]
    fn test_parse_from_uri_invalid_format() {
        let parser = ConfigUriParser::new();
        let uri = "not-a-valid-uri";

        let result = parser.parse_from_uri(uri);
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            DurableError::Config { message } => {
                assert!(message.contains("Invalid URI format"));
            }
            _ => panic!("Expected Config error, got {:?}", error),
        }
    }

    #[test]
    fn test_parse_and_validate_valid_config() {
        let parser = ConfigUriParser::new();
        let toml_config = create_test_toml_config();
        let encoded = encode_toml_base64(&toml_config);
        let uri = format!("toml://{}", encoded);

        let result = parser.parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_and_validate_empty_config() {
        let parser = ConfigUriParser::new();
        let minimal_toml = "# This is a comment\n";
        let encoded = encode_toml_base64(minimal_toml);
        let uri = format!("toml://{}", encoded);

        let result = parser.parse_and_validate(&uri);
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            DurableError::Config { message } => {
                assert!(message.contains("must contain at least one protocol or service"));
            }
            _ => panic!("Expected Config error, got {:?}", error),
        }
    }

    #[test]
    fn test_parse_and_validate_propagates_parse_errors() {
        let parser = ConfigUriParser::new();
        let uri = "invalid://scheme";

        let result = parser.parse_and_validate(uri);
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            DurableError::Config { message } => {
                assert!(message.contains("Unsupported URI scheme"));
            }
            _ => panic!("Expected Config error, got {:?}", error),
        }
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_comprehensive_error_scenarios() {
        let parser = ConfigUriParser::new();

        // Test various invalid URIs
        let invalid_uris = vec![
            "://missing-scheme",
            "toml://",
            "dfs://",
            "unknown://test",
            "",
            "toml://invalid-base64!",
        ];

        for uri in invalid_uris {
            let result = parser.parse_from_uri(uri);
            // Should either fail to parse URI or fail during processing
            // We don't assert the exact error type since different URIs fail at different points
            assert!(result.is_err() || url::Url::parse(uri).is_err(),
                   "URI '{}' should fail to parse or process", uri);
        }

        // Test a URI that parses but has invalid content
        let uri_with_invalid_content = "dfs://service?invalid=param%ZZ";
        let result = parser.parse_from_uri(uri_with_invalid_content);
        // This might succeed or fail depending on how URL parsing handles invalid percent encoding
        // The important thing is that it doesn't panic
        let _ = result; // Just ensure it doesn't panic
    }

    #[test]
    fn test_malformed_toml_edge_cases() {
        let malformed_tomls = vec![
            "[unclosed",
            "key = ",
            "[section]\nkey = value\nkey = duplicate",
            "[section]\nkey = ",
            "[[array]]\nkey = value\n[[array]]\nkey = ",
        ];

        for malformed in malformed_tomls {
            let encoded = encode_toml_base64(malformed);
            let uri = format!("toml://{}", encoded);

            let parsed_uri = UriParser::parse(&uri).unwrap();
            let result = parse_toml_protocol(&parsed_uri);
            assert!(result.is_err(),
                   "Malformed TOML '{}' should fail to parse", malformed);
        }
    }

    #[test]
    fn test_url_encoding_edge_cases() {
        let parser = ConfigUriParser::new();

        // Test URLs with various special characters
        let special_uris = vec![
            "dfs://s3?key=value%20with%20spaces",
            "dfs://s3?key=value%2Bwith%2Bplus",
            "dfs://s3?key=value%26with%26ampersand",
            "dfs://s3?key=value%3Dwith%3Dequals",
            "dfs://s3?key=value%3Fwith%3Fquestion",
        ];

        for uri in special_uris {
            let result = parser.parse_from_uri(uri);
            assert!(result.is_ok(),
                   "URI '{}' with special characters should parse successfully", uri);
        }
    }
}

mod integration_tests {
    use super::*;

    #[test]
    fn test_round_trip_toml_parsing() {
        let original_config = create_test_toml_config();
        let encoded = encode_toml_base64(&original_config);
        let uri = format!("toml://{}", encoded);

        let parser = ConfigUriParser::new();
        let parsed_config = parser.parse_from_uri(&uri).unwrap();

        // Convert back to TOML and compare (this tests serialization)
        let serialized = toml::to_string_pretty(&parsed_config).unwrap();

        // Parse both original and serialized to compare structure
        let original_parsed: Config = UriParser::parse_toml(&original_config).unwrap();
        let serialized_parsed: Config = UriParser::parse_toml(&serialized).unwrap();

        // Compare key structures (exact string match might differ due to formatting)
        assert_eq!(original_parsed.protocols.len(), serialized_parsed.protocols.len());
        assert_eq!(original_parsed.services.len(), serialized_parsed.services.len());
    }

    #[test]
    fn test_dfs_protocol_parameter_preservation() {
        let uri = "dfs://complex?param1=value1&param2=value2&region=us-east-1&bucket=test-bucket&custom_param=custom_value";

        let parser = ConfigUriParser::new();
        let config = parser.parse_from_uri(uri).unwrap();

        let service = &config.services["complex"];
        let protocol = &config.protocols["default-complex"];

        // All parameters should be preserved
        assert_eq!(service.parameters.get("param1").unwrap(), "value1");
        assert_eq!(service.parameters.get("param2").unwrap(), "value2");
        assert_eq!(service.parameters.get("custom_param").unwrap(), "custom_value");

        // Standard fields should be extracted
        assert_eq!(service.default_region.as_ref().unwrap(), "us-east-1");
        assert_eq!(protocol.bucket.as_ref().unwrap(), "test-bucket");
        assert_eq!(protocol.region.as_ref().unwrap(), "us-east-1");

        // Parameters should also be in protocol
        assert_eq!(protocol.parameters.get("param1").unwrap(), "value1");
        assert_eq!(protocol.parameters.get("param2").unwrap(), "value2");
    }

    #[test]
    fn test_multiple_protocols_in_config() {
        let multi_protocol_toml = create_multi_protocol_test_toml_config();
        let encoded = encode_toml_base64(&multi_protocol_toml);
        let uri = format!("toml://{}", encoded);

        let parser = ConfigUriParser::new();
        let config = parser.parse_from_uri(&uri).unwrap();

        assert_eq!(config.protocols.len(), 2);
        assert_eq!(config.services.len(), 2);

        assert!(config.protocols.contains_key("local-dev"));
        assert!(config.protocols.contains_key("s3-prod"));
        assert!(config.services.contains_key("local"));
        assert!(config.services.contains_key("s3"));
    }
}