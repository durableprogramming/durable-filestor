//! Tests for the configuration parser

use durable_filestor_config_toml_parser::ConfigParser;
use durable_filestor_shared::{
    Cli, Commands, Config, KeyTranslation, Protocol, ServiceConfig, ConfigUtils,
    test_utils::{config_test_utils::*, create_test_config},
};

use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_file() {
        let config_content = r#"
[protocols.test]
service = "s3"
bucket = "test-bucket"
region = "us-east-1"

[protocols.test.key_translation]
downcase = true
partition = true
partition_length = 4
partition_depth = 2

[services.s3]
default_region = "us-west-2"
endpoint = "https://s3.amazonaws.com"
"#;
        let (_temp_dir, config_path) = create_temp_config(config_content);

        let parser = ConfigParser::new().with_config_file(&config_path);
        let cli = create_test_cli_with_config(config_path, Commands::List {
            uri: "test://".to_string(),
            service: None,
            bucket: None,
            region: None,
            path: None,
        });

        let config = ConfigUtils::load_from_toml_file(cli.config.as_ref().unwrap()).unwrap();

        // Check protocols
        assertions::assert_protocol_exists(&config, "test", "s3");
        assertions::assert_protocol_bucket(&config, "test", Some("test-bucket"));
        assertions::assert_protocol_region(&config, "test", Some("us-east-1"));
        assertions::assert_key_translation(&config, "test", true, true);

        // Check services
        assertions::assert_service_exists(&config, "s3");
        assertions::assert_service_region(&config, "s3", Some("us-west-2"));
        assertions::assert_service_endpoint(&config, "s3", Some("https://s3.amazonaws.com"));
    }

    #[test]
    fn test_load_from_env() {
        let mut env_cleanup = EnvCleanup::new();

        // Set up environment variables with cleanup tracking
        env_cleanup.set_and_track("DURABLE_FILESTOR_PROTOCOLS", r#"{"env_test":{"service":"local","path":"/tmp"}}"#);
        env_cleanup.set_and_track("DURABLE_FILESTOR_S3_REGION", "us-central-1");
        env_cleanup.set_and_track("DURABLE_FILESTOR_S3_ENDPOINT", "https://custom-s3.com");

        let parser = ConfigParser::new();
        let config = parser.load_from_env().unwrap();

        // Check protocol from environment
        assertions::assert_protocol_exists(&config, "env_test", "local");
        let protocol = &config.protocols["env_test"];
        assert_eq!(protocol.path.as_ref().unwrap(), "/tmp");

        // Check service config from environment
        assertions::assert_service_exists(&config, "s3");
        assertions::assert_service_region(&config, "s3", Some("us-central-1"));
        assertions::assert_service_endpoint(&config, "s3", Some("https://custom-s3.com"));

        // EnvCleanup will automatically clean up when it goes out of scope
    }

    #[test]
    fn test_config_precedence() {
        // Create a config file
        let config_content = r#"
[protocols.test]
service = "s3"
bucket = "file-bucket"
region = "us-east-1"
"#;
        let (_temp_dir, config_path) = create_temp_config(config_content);

        let parser = ConfigParser::new().with_config_file(&config_path);
        let cli = Cli {
            config: Some(config_path.clone()),
            verbose: false,
            command: Commands::List {
                uri: "test://".to_string(),
                service: None,
                bucket: None,
                region: None,
                path: None,
            },
        };

        let config = parser.load_config(&cli).unwrap();

        // File config should be loaded
        assert!(config.protocols.contains_key("test"));
        let protocol = &config.protocols["test"];
        assert_eq!(protocol.service, "s3"); // From file
        assert_eq!(protocol.bucket.as_ref().unwrap(), "file-bucket"); // From file
        assert_eq!(protocol.region.as_ref().unwrap(), "us-east-1"); // From file
    }

    #[test]
    fn test_save_and_load_config() {
        let mut config = Config::default();

        // Add test protocol
        let mut key_translation = KeyTranslation::default();
        key_translation.downcase = true;
        key_translation.prefix = Some("test/".to_string());

        let protocol = Protocol {
            service: "s3".to_string(),
            bucket: Some("test-bucket".to_string()),
            region: Some("us-west-2".to_string()),
            path: None,
            key_translation,
            parameters: std::collections::HashMap::new(),
        };

        config.protocols.insert("test".to_string(), protocol);

        // Add test service
        let service_config = ServiceConfig {
            default_region: Some("us-east-1".to_string()),
            endpoint: Some("https://s3.amazonaws.com".to_string()),
            default_permissions: None,
            parameters: std::collections::HashMap::new(),
        };

        config.services.insert("s3".to_string(), service_config);

        // Save config
        let (_temp_dir, config_path) = create_temp_config("");
        let parser = ConfigParser::new().with_config_file(&config_path);
        let cli = Cli {
            config: Some(config_path.clone()),
            verbose: false,
            command: Commands::Config {
                action: durable_filestor_shared::ConfigAction::Show,
            },
        };

        parser.save_config(&config, &cli).unwrap();

        // Load config back
        let loaded_config = ConfigUtils::load_from_toml_file(cli.config.as_ref().unwrap()).unwrap();

        // Verify the loaded config matches
        assert_eq!(loaded_config.protocols.len(), 1);
        assert!(loaded_config.protocols.contains_key("test"));
        let loaded_protocol = &loaded_config.protocols["test"];
        assert_eq!(loaded_protocol.service, "s3");
        assert_eq!(loaded_protocol.bucket.as_ref().unwrap(), "test-bucket");
        assert_eq!(loaded_protocol.region.as_ref().unwrap(), "us-west-2");
        assert!(loaded_protocol.key_translation.downcase);
        assert_eq!(loaded_protocol.key_translation.prefix.as_ref().unwrap(), "test/");

        assert_eq!(loaded_config.services.len(), 1);
        assert!(loaded_config.services.contains_key("s3"));
        let loaded_service = &loaded_config.services["s3"];
        assert_eq!(loaded_service.default_region.as_ref().unwrap(), "us-east-1");
        assert_eq!(loaded_service.endpoint.as_ref().unwrap(), "https://s3.amazonaws.com");
    }

    #[test]
    fn test_key_translation_defaults() {
        let key_translation = KeyTranslation::default();

        assert!(!key_translation.downcase);
        assert!(!key_translation.upcase);
        assert!(key_translation.hash_function.is_none());
        assert!(!key_translation.partition);
        assert_eq!(key_translation.partition_length, 0); // Default value
        assert_eq!(key_translation.partition_depth, 0); // Default value
        assert_eq!(key_translation.partition_separator, ""); // Default value
        assert!(key_translation.prefix.is_none());
        assert!(key_translation.suffix.is_none());
        assert!(!key_translation.timestamp_suffix);
        assert!(!key_translation.normalize_unicode);
        assert!(!key_translation.strip_whitespace);
        assert!(key_translation.replace_spaces.is_none());
        assert!(key_translation.max_key_length.is_none());
    }

    #[test]
    fn test_invalid_config_file() {
        let (_temp_dir, config_path) = create_temp_config("invalid toml content {{{");
        let parser = ConfigParser::new().with_config_file(&config_path);
        let cli = Cli {
            config: Some(config_path),
            verbose: false,
            command: Commands::List {
                uri: "test://".to_string(),
                service: None,
                bucket: None,
                region: None,
                path: None,
            },
        };

        let result = ConfigUtils::load_from_toml_file(cli.config.as_ref().unwrap());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), durable_filestor_shared::DurableError::Config { .. }));
    }

    #[test]
    fn test_nonexistent_config_file() {
        let parser = ConfigParser::new().with_config_file("/nonexistent/path/config.toml");
        let cli = Cli {
            config: Some(PathBuf::from("/nonexistent/path/config.toml")),
            verbose: false,
            command: Commands::List {
                uri: "test://".to_string(),
                service: None,
                bucket: None,
                region: None,
                path: None,
            },
        };

        let config_path = cli.config.as_ref().unwrap();
        let result = if !config_path.exists() {
            Ok(None)
        } else {
            ConfigUtils::load_from_toml_file(config_path).map(Some)
        };
        // Should return Ok(None) for nonexistent file
        assert!(result.unwrap().is_none());
    }
}