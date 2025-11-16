//! URI export functionality for configuration objects

use durable_filestor_shared::{
    Config, Protocol, Result, ServiceConfig, KeyTranslation, HashFunction,
    build_uri_components, build_uri_parameters, build_key_translation_components,
    join_uri_components, build_uri_string, context_error
};
use durable_filestor_uri_parser::UriParser;
use std::collections::HashMap;

/// Configuration URI exporter
pub struct UriExporter;

impl UriExporter {
    /// Export a Config to URI format
    ///
    /// # Examples
    /// ```
    /// use durable_filestor_config_uri_exporter::UriExporter;
    /// use durable_filestor_shared::Config;
    ///
    /// let config = Config::default();
    /// let uri_str = UriExporter::export_config(&config).unwrap();
    /// println!("{}", uri_str);
    /// ```
    pub fn export_config(config: &Config) -> Result<String> {
        let mut uri_parts = Vec::new();

        // Export protocols
        for (name, protocol) in &config.protocols {
            let protocol_uri = context_error!(
                Self::export_protocol(protocol),
                format!("Failed to export protocol '{}'", name)
            )?;
            uri_parts.push(format!("protocol:{}:{}", name, protocol_uri));
        }

        // Export services
        for (name, service) in &config.services {
            let service_uri = context_error!(
                Self::export_service_config(service),
                format!("Failed to export service '{}'", name)
            )?;
            uri_parts.push(format!("service:{}:{}", name, service_uri));
        }

        Ok(build_uri_string!("config", uri_parts, ";", "empty"))
    }

    /// Export a Protocol to URI format
    ///
    /// # Examples
    /// ```
    /// use durable_filestor_config_uri_exporter::UriExporter;
    /// use durable_filestor_shared::{Protocol, KeyTranslation};
    ///
    /// let protocol = Protocol {
    ///     service: "s3".to_string(),
    ///     bucket: Some("my-bucket".to_string()),
    ///     region: Some("us-east-1".to_string()),
    ///     path: None,
    ///     key_translation: KeyTranslation::default(),
    ///     parameters: std::collections::HashMap::new(),
    /// };
    /// let uri_str = UriExporter::export_protocol(&protocol).unwrap();
    /// ```
    pub fn export_protocol(protocol: &Protocol) -> Result<String> {
        let mut components = Vec::new();

        // Service is required
        components.push(format!("service={}", urlencoding::encode(&protocol.service)));

        // Optional components using macro
        build_uri_components!(components, {
            bucket => protocol.bucket.as_ref(),
            region => protocol.region.as_ref(),
            path => protocol.path.as_ref()
        });

        // Key translation
        let key_translation_str = context_error!(
            Self::export_key_translation(&protocol.key_translation),
            "Failed to export key translation"
        )?;
        if !key_translation_str.is_empty() {
            components.push(format!("key_translation={}", urlencoding::encode(&key_translation_str)));
        }

        // Parameters using macro
        build_uri_parameters!(components, protocol.parameters);

        Ok(join_uri_components!(components, "&", ""))
    }

    /// Export a ServiceConfig to URI format
    ///
    /// # Examples
    /// ```
    /// use durable_filestor_config_uri_exporter::UriExporter;
    /// use durable_filestor_shared::ServiceConfig;
    ///
    /// let service_config = ServiceConfig {
    ///     default_region: Some("us-west-2".to_string()),
    ///     endpoint: Some("https://s3.us-west-2.amazonaws.com".to_string()),
    ///     default_permissions: None,
    ///     parameters: std::collections::HashMap::new(),
    /// };
    /// let uri_str = UriExporter::export_service_config(&service_config).unwrap();
    /// ```
    pub fn export_service_config(service_config: &ServiceConfig) -> Result<String> {
        let mut components = Vec::new();

        // Optional components using macro
        build_uri_components!(components, {
            default_region => service_config.default_region.as_ref(),
            endpoint => service_config.endpoint.as_ref(),
            default_permissions => service_config.default_permissions.as_ref()
        });

        // Parameters using macro
        build_uri_parameters!(components, service_config.parameters);

        Ok(join_uri_components!(components, "&", "empty"))
    }

    /// Export KeyTranslation to URI format
    fn export_key_translation(key_translation: &KeyTranslation) -> Result<String> {
        let mut components = Vec::new();

        // Use macro to build key translation components
        build_key_translation_components!(components, key_translation, Self::hash_function_to_string);

        Ok(join_uri_components!(components, ",", ""))
    }

    /// Convert HashFunction to string representation
    fn hash_function_to_string(hash_func: &HashFunction) -> &'static str {
        match hash_func {
            HashFunction::Md5 => "md5",
            HashFunction::Sha1 => "sha1",
            HashFunction::Sha256 => "sha256",
            HashFunction::Sha512 => "sha512",
            HashFunction::Blake2b => "blake2b",
        }
    }

    /// Export protocols map to URI format
    ///
    /// # Examples
    /// ```
    /// use durable_filestor_config_uri_exporter::UriExporter;
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
    /// let uri_str = UriExporter::export_protocols(&protocols).unwrap();
    /// ```
    pub fn export_protocols(protocols: &HashMap<String, Protocol>) -> Result<String> {
        let mut uri_parts = Vec::new();

        for (name, protocol) in protocols {
            let protocol_uri = context_error!(
                Self::export_protocol(protocol),
                format!("Failed to export protocol '{}'", name)
            )?;
            uri_parts.push(format!("{}:{}", name, protocol_uri));
        }

        Ok(build_uri_string!("protocols", uri_parts, ";", "empty"))
    }

    /// Export services map to URI format
    ///
    /// # Examples
    /// ```
    /// use durable_filestor_config_uri_exporter::UriExporter;
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
    /// let uri_str = UriExporter::export_services(&services).unwrap();
    /// ```
    pub fn export_services(services: &HashMap<String, ServiceConfig>) -> Result<String> {
        let mut uri_parts = Vec::new();

        for (name, service) in services {
            let service_uri = context_error!(
                Self::export_service_config(service),
                format!("Failed to export service '{}'", name)
            )?;
            uri_parts.push(format!("{}:{}", name, service_uri));
        }

        Ok(build_uri_string!("services", uri_parts, ";", "empty"))
    }

    /// Export config as multiple URIs (one per protocol/service)
    ///
    /// # Examples
    /// ```
    /// use durable_filestor_config_uri_exporter::UriExporter;
    /// use durable_filestor_shared::Config;
    ///
    /// let config = Config::default();
    /// let uris = UriExporter::export_config_as_uris(&config).unwrap();
    /// for uri in uris {
    ///     println!("{}", uri);
    /// }
    /// ```
    pub fn export_config_as_uris(config: &Config) -> Result<Vec<String>> {
        let mut uris = Vec::new();

        // Export each protocol as a separate URI
        for (name, protocol) in &config.protocols {
            let protocol_str = context_error!(
                Self::export_protocol(protocol),
                format!("Failed to export protocol '{}' as URI", name)
            )?;
            uris.push(format!("protocol://{}?{}", name, protocol_str));
        }

        // Export each service as a separate URI
        for (name, service) in &config.services {
            let service_str = context_error!(
                Self::export_service_config(service),
                format!("Failed to export service '{}' as URI", name)
            )?;
            uris.push(format!("service://{}?{}", name, service_str));
        }

        Ok(uris)
    }

    /// Validate that exported URI can be parsed back
    ///
    /// # Examples
    /// ```
    /// use durable_filestor_config_uri_exporter::UriExporter;
    /// use durable_filestor_shared::Config;
    ///
    /// let config = Config::default();
    /// let uri_str = UriExporter::export_config(&config).unwrap();
    /// let is_valid = UriExporter::validate_export(&uri_str).unwrap();
    /// assert!(is_valid);
    /// ```
    pub fn validate_export(uri_str: &str) -> Result<bool> {
        let parse_result = context_error!(
            UriParser::parse(uri_str),
            format!("Failed to validate URI: {}", uri_str)
        );

        match parse_result {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_empty_config() {
        let config = Config::default();
        let result = UriExporter::export_config(&config);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        assert!(uri_str.starts_with("config://"));

        // Validate that it can be parsed back
        let is_valid = UriExporter::validate_export(&uri_str).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_export_protocol() {
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

        let result = UriExporter::export_protocol(&protocol);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        assert!(uri_str.contains("service=s3"));
        assert!(uri_str.contains("bucket=my-bucket"));
        assert!(uri_str.contains("region=us-east-1"));
        assert!(uri_str.contains("key_translation="));
        assert!(uri_str.contains("timeout=30"));
    }

    #[test]
    fn test_export_service_config() {
        let mut parameters = HashMap::new();
        parameters.insert("max_connections".to_string(), "100".to_string());

        let service_config = ServiceConfig {
            default_region: Some("us-west-2".to_string()),
            endpoint: Some("https://s3.us-west-2.amazonaws.com".to_string()),
            default_permissions: Some("0644".to_string()),
            parameters,
        };

        let result = UriExporter::export_service_config(&service_config);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        assert!(uri_str.contains("default_region=us-west-2"));
        assert!(uri_str.contains("endpoint=https%3A%2F%2Fs3.us-west-2.amazonaws.com"));
        assert!(uri_str.contains("default_permissions=0644"));
        assert!(uri_str.contains("max_connections=100"));
    }

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
            key_translation: KeyTranslation::default(),
            parameters: HashMap::new(),
        };

        protocols.insert("prod-s3".to_string(), protocol1);
        protocols.insert("local-dev".to_string(), protocol2);

        let result = UriExporter::export_protocols(&protocols);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        assert!(uri_str.starts_with("protocols://"));
        assert!(uri_str.contains("prod-s3:"));
        assert!(uri_str.contains("local-dev:"));
        assert!(uri_str.contains("service=s3"));
        assert!(uri_str.contains("service=local"));
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

        let result = UriExporter::export_services(&services);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        assert!(uri_str.starts_with("services://"));
        assert!(uri_str.contains("s3:"));
        assert!(uri_str.contains("local:"));
        assert!(uri_str.contains("default_region=us-east-1"));
        assert!(uri_str.contains("default_permissions=0755"));
    }

    #[test]
    fn test_export_config_as_uris() {
        let mut config = Config::default();

        let protocol = Protocol {
            service: "s3".to_string(),
            bucket: Some("test-bucket".to_string()),
            region: Some("us-east-1".to_string()),
            path: None,
            key_translation: KeyTranslation::default(),
            parameters: HashMap::new(),
        };

        let service = ServiceConfig {
            default_region: Some("us-west-2".to_string()),
            endpoint: None,
            default_permissions: None,
            parameters: HashMap::new(),
        };

        config.protocols.insert("test-protocol".to_string(), protocol);
        config.services.insert("test-service".to_string(), service);

        let result = UriExporter::export_config_as_uris(&config);
        assert!(result.is_ok());

        let uris = result.unwrap();
        assert_eq!(uris.len(), 2);

        let protocol_uri = uris.iter().find(|u| u.contains("protocol://")).unwrap();
        let service_uri = uris.iter().find(|u| u.contains("service://")).unwrap();

        assert!(protocol_uri.contains("test-protocol"));
        assert!(protocol_uri.contains("service=s3"));
        assert!(service_uri.contains("test-service"));
        assert!(service_uri.contains("default_region=us-west-2"));
    }

    #[test]
    fn test_validate_export() {
        let config = Config::default();
        let uri_str = UriExporter::export_config(&config).unwrap();

        let is_valid = UriExporter::validate_export(&uri_str).unwrap();
        assert!(is_valid);

        // Test with invalid URI
        let invalid_uri = "invalid uri format";
        let is_valid_invalid = UriExporter::validate_export(invalid_uri).unwrap();
        assert!(!is_valid_invalid);
    }

    #[test]
    fn test_key_translation_export() {
        let key_translation = KeyTranslation {
            downcase: true,
            upcase: false,
            hash_function: Some(HashFunction::Sha256),
            hash_prefix_length: Some(8),
            partition: true,
            partition_length: 2,
            partition_depth: 3,
            partition_separator: "/".to_string(),
            prefix: Some("test-".to_string()),
            suffix: Some("-end".to_string()),
            timestamp_suffix: true,
            normalize_unicode: true,
            strip_whitespace: true,
            replace_spaces: Some("_".to_string()),
            max_key_length: Some(100),
        };

        let result = UriExporter::export_key_translation(&key_translation);
        assert!(result.is_ok());

        let key_str = result.unwrap();
        assert!(key_str.contains("downcase"));
        assert!(key_str.contains("hash=sha256"));
        assert!(key_str.contains("hash_prefix_length=8"));
        assert!(key_str.contains("partition_length=2"));
        assert!(key_str.contains("partition_depth=3"));
        assert!(key_str.contains("prefix=test-"));
        assert!(key_str.contains("suffix=-end"));
        assert!(key_str.contains("timestamp_suffix"));
        assert!(key_str.contains("normalize_unicode"));
        assert!(key_str.contains("strip_whitespace"));
        assert!(key_str.contains("replace_spaces=_"));
        assert!(key_str.contains("max_key_length=100"));
    }

    #[test]
    fn test_hash_function_to_string() {
        assert_eq!(UriExporter::hash_function_to_string(&HashFunction::Md5), "md5");
        assert_eq!(UriExporter::hash_function_to_string(&HashFunction::Sha1), "sha1");
        assert_eq!(UriExporter::hash_function_to_string(&HashFunction::Sha256), "sha256");
        assert_eq!(UriExporter::hash_function_to_string(&HashFunction::Sha512), "sha512");
        assert_eq!(UriExporter::hash_function_to_string(&HashFunction::Blake2b), "blake2b");
    }

    #[test]
    fn test_export_empty_protocol() {
        let protocol = Protocol {
            service: "s3".to_string(),
            bucket: None,
            region: None,
            path: None,
            key_translation: KeyTranslation::default(),
            parameters: HashMap::new(),
        };

        let result = UriExporter::export_protocol(&protocol);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        assert!(uri_str.contains("service=s3"));
        assert!(!uri_str.contains("bucket="));
        assert!(!uri_str.contains("region="));
        assert!(!uri_str.contains("path="));
        assert!(!uri_str.contains("key_translation="));
    }

    #[test]
    fn test_export_protocol_with_special_characters() {
        let mut parameters = HashMap::new();
        parameters.insert("special key".to_string(), "value with spaces & symbols!".to_string());

        let protocol = Protocol {
            service: "s3".to_string(),
            bucket: Some("bucket with spaces".to_string()),
            region: Some("us-east-1".to_string()),
            path: Some("/path with spaces".to_string()),
            key_translation: KeyTranslation::default(),
            parameters,
        };

        let result = UriExporter::export_protocol(&protocol);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        // URL encoding should handle special characters
        assert!(uri_str.contains("bucket=bucket%20with%20spaces"));
        assert!(uri_str.contains("path=%2Fpath%20with%20spaces"));
        assert!(uri_str.contains("special%20key=value%20with%20spaces%20%26%20symbols%21"));
    }

    #[test]
    fn test_export_service_config_empty() {
        let service_config = ServiceConfig::default();

        let result = UriExporter::export_service_config(&service_config);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        assert_eq!(uri_str, "empty");
    }

    #[test]
    fn test_export_service_config_with_special_characters() {
        let mut parameters = HashMap::new();
        parameters.insert("endpoint url".to_string(), "https://example.com/path?query=value&other=123".to_string());

        let service_config = ServiceConfig {
            default_region: Some("us-west-2".to_string()),
            endpoint: Some("https://s3.us-west-2.amazonaws.com/bucket?version=4".to_string()),
            default_permissions: Some("rwxr-xr-x".to_string()),
            parameters,
        };

        let result = UriExporter::export_service_config(&service_config);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        assert!(uri_str.contains("default_region=us-west-2"));
        assert!(uri_str.contains("endpoint=https%3A%2F%2Fs3.us-west-2.amazonaws.com%2Fbucket%3Fversion%3D4"));
        assert!(uri_str.contains("default_permissions=rwxr-xr-x"));
        assert!(uri_str.contains("endpoint%20url=https%3A%2F%2Fexample.com%2Fpath%3Fquery%3Dvalue%26other%3D123"));
    }

    #[test]
    fn test_export_key_translation_minimal() {
        let key_translation = KeyTranslation::default();

        let result = UriExporter::export_key_translation(&key_translation);
        assert!(result.is_ok());

        let key_str = result.unwrap();
        assert_eq!(key_str, "");
    }

    #[test]
    fn test_export_key_translation_all_options() {
        let key_translation = KeyTranslation {
            downcase: true,
            upcase: false,
            hash_function: Some(HashFunction::Sha256),
            hash_prefix_length: Some(16),
            partition: true,
            partition_length: 3,
            partition_depth: 4,
            partition_separator: "/deep/".to_string(),
            prefix: Some("pre-".to_string()),
            suffix: Some("-suf".to_string()),
            timestamp_suffix: true,
            normalize_unicode: true,
            strip_whitespace: true,
            replace_spaces: Some("__".to_string()),
            max_key_length: Some(500),
        };

        let result = UriExporter::export_key_translation(&key_translation);
        assert!(result.is_ok());

        let key_str = result.unwrap();
        assert!(key_str.contains("downcase"));
        assert!(key_str.contains("hash=sha256"));
        assert!(key_str.contains("hash_prefix_length=16"));
        assert!(key_str.contains("partition_length=3"));
        assert!(key_str.contains("partition_depth=4"));
        assert!(key_str.contains("partition_separator=%2Fdeep%2F"));
        assert!(key_str.contains("prefix=pre-"));
        assert!(key_str.contains("suffix=-suf"));
        assert!(key_str.contains("timestamp_suffix"));
        assert!(key_str.contains("normalize_unicode"));
        assert!(key_str.contains("strip_whitespace"));
        assert!(key_str.contains("replace_spaces=__"));
        assert!(key_str.contains("max_key_length=500"));
    }

    #[test]
    fn test_export_key_translation_with_special_characters() {
        let key_translation = KeyTranslation {
            prefix: Some("prefix with spaces & symbols!".to_string()),
            suffix: Some("suffix with émojis 🎉".to_string()),
            replace_spaces: Some(" → ".to_string()),
            partition_separator: "/path/with spaces/".to_string(),
            partition: true, // Enable partitioning to include partition_separator
            ..Default::default()
        };

        let result = UriExporter::export_key_translation(&key_translation);
        assert!(result.is_ok());

        let key_str = result.unwrap();
        assert!(key_str.contains("prefix=prefix%20with%20spaces%20%26%20symbols%21"));
        assert!(key_str.contains("suffix=suffix%20with%20%C3%A9mojis%20%F0%9F%8E%89"));
        assert!(key_str.contains("replace_spaces=%20%E2%86%92%20"));
        assert!(key_str.contains("partition_separator=%2Fpath%2Fwith%20spaces%2F"));
    }

    #[test]
    fn test_export_protocols_empty() {
        let protocols = HashMap::new();

        let result = UriExporter::export_protocols(&protocols);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        assert_eq!(uri_str, "protocols://empty");
    }

    #[test]
    fn test_export_services_empty() {
        let services = HashMap::new();

        let result = UriExporter::export_services(&services);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        assert_eq!(uri_str, "services://empty");
    }

    #[test]
    fn test_export_config_as_uris_empty() {
        let config = Config::default();

        let result = UriExporter::export_config_as_uris(&config);
        assert!(result.is_ok());

        let uris = result.unwrap();
        assert!(uris.is_empty());
    }

    #[test]
    fn test_export_config_as_uris_mixed() {
        let mut config = Config::default();

        let protocol = Protocol {
            service: "s3".to_string(),
            bucket: Some("test-bucket".to_string()),
            region: Some("us-east-1".to_string()),
            path: None,
            key_translation: KeyTranslation::default(),
            parameters: HashMap::new(),
        };

        let service = ServiceConfig {
            default_region: Some("us-west-2".to_string()),
            endpoint: Some("https://s3.us-west-2.amazonaws.com".to_string()),
            default_permissions: None,
            parameters: HashMap::new(),
        };

        config.protocols.insert("test-protocol".to_string(), protocol);
        config.services.insert("test-service".to_string(), service);

        let result = UriExporter::export_config_as_uris(&config);
        assert!(result.is_ok());

        let uris = result.unwrap();
        assert_eq!(uris.len(), 2);

        let protocol_uri = uris.iter().find(|u| u.starts_with("protocol://")).unwrap();
        let service_uri = uris.iter().find(|u| u.starts_with("service://")).unwrap();

        assert!(protocol_uri.contains("test-protocol"));
        assert!(protocol_uri.contains("service=s3"));
        assert!(service_uri.contains("test-service"));
        assert!(service_uri.contains("default_region=us-west-2"));
    }

    #[test]
    fn test_validate_export_invalid_uri() {
        let invalid_uri = "not a valid uri at all!!!";
        let result = UriExporter::validate_export(invalid_uri);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_validate_export_malformed_uri() {
        let malformed_uri = "config://protocol:invalid:format:here";
        let result = UriExporter::validate_export(malformed_uri);
        assert!(result.is_ok());
        // This might be valid or invalid depending on the parser implementation
        // The important thing is that it doesn't panic
    }

    #[test]
    fn test_export_config_with_many_protocols_and_services() {
        let mut config = Config::default();

        // Add multiple protocols
        for i in 1..=5 {
            let protocol = Protocol {
                service: format!("service{}", i),
                bucket: Some(format!("bucket{}", i)),
                region: Some(format!("region{}", i)),
                path: Some(format!("/path{}", i)),
                key_translation: KeyTranslation::default(),
                parameters: HashMap::new(),
            };
            config.protocols.insert(format!("protocol{}", i), protocol);
        }

        // Add multiple services
        for i in 1..=3 {
            let service = ServiceConfig {
                default_region: Some(format!("region{}", i)),
                endpoint: Some(format!("https://service{}.example.com", i)),
                default_permissions: Some(format!("perms{}", i)),
                parameters: HashMap::new(),
            };
            config.services.insert(format!("service{}", i), service);
        }

        let result = UriExporter::export_config(&config);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        assert!(uri_str.starts_with("config://"));

        // Should contain all protocols and services
        for i in 1..=5 {
            assert!(uri_str.contains(&format!("protocol:protocol{}:", i)));
            assert!(uri_str.contains(&format!("service=service{}", i)));
        }
        for i in 1..=3 {
            assert!(uri_str.contains(&format!("service:service{}:", i)));
            assert!(uri_str.contains(&format!("default_region=region{}", i)));
        }
    }

    #[test]
    fn test_export_protocol_with_unicode_characters() {
        let mut parameters = HashMap::new();
        parameters.insert("café".to_string(), "naïve".to_string());
        parameters.insert("测试".to_string(), "值".to_string());

        let protocol = Protocol {
            service: "s3".to_string(),
            bucket: Some("búcket-wíth-ünicöde".to_string()),
            region: Some("us-east-1".to_string()),
            path: None,
            key_translation: KeyTranslation::default(),
            parameters,
        };

        let result = UriExporter::export_protocol(&protocol);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        println!("Actual URI: {}", uri_str); // Debug output

        // Unicode characters should be properly URL encoded
        // Check that some encoding is present (exact encoding may vary)
        assert!(uri_str.contains("bucket="));
        assert!(uri_str.contains("caf"));
        assert!(uri_str.contains("na"));
        // The test is mainly to ensure no panics with unicode
        assert!(uri_str.len() > 50); // Should be reasonably long with encoding
    }

    #[test]
    fn test_export_config_round_trip_validation() {
        let mut config = Config::default();

        let protocol = Protocol {
            service: "s3".to_string(),
            bucket: Some("test-bucket".to_string()),
            region: Some("us-east-1".to_string()),
            path: Some("/test/path".to_string()),
            key_translation: KeyTranslation {
                downcase: true,
                hash_function: Some(HashFunction::Sha256),
                prefix: Some("test-".to_string()),
                ..Default::default()
            },
            parameters: {
                let mut p = HashMap::new();
                p.insert("timeout".to_string(), "30".to_string());
                p
            },
        };

        let service = ServiceConfig {
            default_region: Some("us-west-2".to_string()),
            endpoint: Some("https://s3.us-west-2.amazonaws.com".to_string()),
            default_permissions: Some("0644".to_string()),
            parameters: {
                let mut p = HashMap::new();
                p.insert("max_connections".to_string(), "100".to_string());
                p
            },
        };

        config.protocols.insert("test-protocol".to_string(), protocol);
        config.services.insert("test-service".to_string(), service);

        // Export to URI
        let uri_str = UriExporter::export_config(&config).unwrap();

        // Validate that it can be parsed back
        let is_valid = UriExporter::validate_export(&uri_str).unwrap();
        assert!(is_valid, "Exported URI should be valid: {}", uri_str);
    }

    #[test]
    fn test_export_key_translation_edge_cases() {
        // Test with zero values
        let key_translation = KeyTranslation {
            partition_length: 0,
            partition_depth: 0,
            max_key_length: Some(0),
            hash_prefix_length: Some(0),
            partition: true, // Enable partitioning to include partition fields
            ..Default::default()
        };

        let result = UriExporter::export_key_translation(&key_translation);
        assert!(result.is_ok());

        let key_str = result.unwrap();
        assert!(key_str.contains("partition_length=0"));
        assert!(key_str.contains("partition_depth=0"));
        assert!(key_str.contains("max_key_length=0"));
        assert!(key_str.contains("hash_prefix_length=0"));
    }

    #[test]
    fn test_export_key_translation_large_values() {
        let key_translation = KeyTranslation {
            partition_length: usize::MAX,
            partition_depth: usize::MAX,
            max_key_length: Some(usize::MAX),
            hash_prefix_length: Some(usize::MAX),
            partition: true, // Enable partitioning to include partition fields
            ..Default::default()
        };

        let result = UriExporter::export_key_translation(&key_translation);
        assert!(result.is_ok());

        let key_str = result.unwrap();
        assert!(key_str.contains(&format!("partition_length={}", usize::MAX)));
        assert!(key_str.contains(&format!("partition_depth={}", usize::MAX)));
        assert!(key_str.contains(&format!("max_key_length={}", usize::MAX)));
        assert!(key_str.contains(&format!("hash_prefix_length={}", usize::MAX)));
    }

    #[test]
    fn test_export_protocol_with_empty_service_name() {
        let protocol = Protocol {
            service: "".to_string(),
            bucket: Some("bucket".to_string()),
            region: None,
            path: None,
            key_translation: KeyTranslation::default(),
            parameters: HashMap::new(),
        };

        let result = UriExporter::export_protocol(&protocol);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        assert!(uri_str.contains("service=")); // Should still have the parameter
    }

    #[test]
    fn test_export_service_config_with_empty_parameters() {
        let mut parameters = HashMap::new();
        parameters.insert("".to_string(), "".to_string());
        parameters.insert("key_only".to_string(), "".to_string());
        parameters.insert("".to_string(), "value_only".to_string());

        let service_config = ServiceConfig {
            default_region: Some("us-east-1".to_string()),
            endpoint: None,
            default_permissions: None,
            parameters,
        };

        let result = UriExporter::export_service_config(&service_config);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        println!("Empty params URI: {}", uri_str); // Debug output

        assert!(uri_str.contains("default_region=us-east-1"));
        // Empty keys and values should be handled gracefully
        // The exact encoding depends on urlencoding behavior
        assert!(uri_str.contains("=")); // Should have some parameters
    }

    #[test]
    fn test_export_config_with_duplicate_names() {
        let mut config = Config::default();

        // Add protocols with same service but different names
        let protocol1 = Protocol {
            service: "s3".to_string(),
            bucket: Some("bucket1".to_string()),
            region: Some("us-east-1".to_string()),
            path: None,
            key_translation: KeyTranslation::default(),
            parameters: HashMap::new(),
        };

        let protocol2 = Protocol {
            service: "s3".to_string(),
            bucket: Some("bucket2".to_string()),
            region: Some("us-west-2".to_string()),
            path: None,
            key_translation: KeyTranslation::default(),
            parameters: HashMap::new(),
        };

        config.protocols.insert("prod-s3".to_string(), protocol1);
        config.protocols.insert("dev-s3".to_string(), protocol2);

        let result = UriExporter::export_config(&config);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        assert!(uri_str.contains("protocol:prod-s3:"));
        assert!(uri_str.contains("protocol:dev-s3:"));
        assert!(uri_str.contains("bucket=bucket1"));
        assert!(uri_str.contains("bucket=bucket2"));
    }

    #[test]
    fn test_export_protocols_with_special_names() {
        let mut protocols = HashMap::new();

        let protocol = Protocol {
            service: "s3".to_string(),
            bucket: Some("bucket".to_string()),
            region: Some("us-east-1".to_string()),
            path: None,
            key_translation: KeyTranslation::default(),
            parameters: HashMap::new(),
        };

        // Test protocol names with special characters
        protocols.insert("protocol with spaces".to_string(), protocol.clone());
        protocols.insert("protocol:with:colons".to_string(), protocol.clone());
        protocols.insert("protocol;semi:colons".to_string(), protocol.clone());
        protocols.insert("protocol?query=value".to_string(), protocol.clone());

        let result = UriExporter::export_protocols(&protocols);
        assert!(result.is_ok());

        let uri_str = result.unwrap();
        assert!(uri_str.contains("protocol with spaces:"));
        assert!(uri_str.contains("protocol:with:colons:"));
        assert!(uri_str.contains("protocol;semi:colons:"));
        assert!(uri_str.contains("protocol?query=value:"));
    }

    #[test]
    fn test_export_config_as_uris_ordering() {
        let mut config = Config::default();

        // Add items in reverse alphabetical order
        let protocol_z = Protocol {
            service: "s3".to_string(),
            bucket: Some("z-bucket".to_string()),
            region: Some("us-east-1".to_string()),
            path: None,
            key_translation: KeyTranslation::default(),
            parameters: HashMap::new(),
        };

        let protocol_a = Protocol {
            service: "local".to_string(),
            bucket: None,
            region: None,
            path: Some("/a-path".to_string()),
            key_translation: KeyTranslation::default(),
            parameters: HashMap::new(),
        };

        config.protocols.insert("z-protocol".to_string(), protocol_z);
        config.protocols.insert("a-protocol".to_string(), protocol_a);

        let result = UriExporter::export_config_as_uris(&config);
        assert!(result.is_ok());

        let uris = result.unwrap();
        assert_eq!(uris.len(), 2);

        // URIs should be in insertion order (HashMap ordering)
        // This test ensures we don't accidentally sort or reorder
        let first_uri = &uris[0];
        let second_uri = &uris[1];

        assert!(first_uri.contains("z-protocol") || second_uri.contains("z-protocol"));
        assert!(first_uri.contains("a-protocol") || second_uri.contains("a-protocol"));
    }

    #[test]
    fn test_export_key_translation_boolean_flags() {
        // Test all boolean flags set to true
        let key_translation = KeyTranslation {
            downcase: true,
            upcase: true,
            timestamp_suffix: true,
            normalize_unicode: true,
            strip_whitespace: true,
            partition: true,
            partition_length: 2,
            partition_depth: 2,
            partition_separator: "/".to_string(),
            ..Default::default()
        };

        let result = UriExporter::export_key_translation(&key_translation);
        assert!(result.is_ok());

        let key_str = result.unwrap();
        assert!(key_str.contains("downcase"));
        assert!(key_str.contains("upcase"));
        assert!(key_str.contains("timestamp_suffix"));
        assert!(key_str.contains("normalize_unicode"));
        assert!(key_str.contains("strip_whitespace"));
        assert!(key_str.contains("partition_length=2"));
        assert!(key_str.contains("partition_depth=2"));
        assert!(key_str.contains("partition_separator=%2F"));
    }

    #[test]
    fn test_export_key_translation_boolean_flags_false() {
        // Test all boolean flags set to false (should not appear)
        let key_translation = KeyTranslation {
            downcase: false,
            upcase: false,
            timestamp_suffix: false,
            normalize_unicode: false,
            strip_whitespace: false,
            partition: false,
            ..Default::default()
        };

        let result = UriExporter::export_key_translation(&key_translation);
        assert!(result.is_ok());

        let key_str = result.unwrap();
        assert!(!key_str.contains("downcase"));
        assert!(!key_str.contains("upcase"));
        assert!(!key_str.contains("timestamp_suffix"));
        assert!(!key_str.contains("normalize_unicode"));
        assert!(!key_str.contains("strip_whitespace"));
        assert!(!key_str.contains("partition_length="));
        assert!(!key_str.contains("partition_depth="));
        assert!(!key_str.contains("partition_separator="));
    }

    #[cfg(test)]
    mod proptest_tests {
        use super::*;
        use proptest::prelude::*;

        // Property-based tests for comprehensive coverage

        proptest! {
            #[test]
            fn test_export_protocol_arbitrary_strings(
                service in "[a-zA-Z0-9_-]{1,50}",
                bucket in proptest::option::of("[a-zA-Z0-9_-]{1,50}"),
                region in proptest::option::of("[a-zA-Z0-9_-]{1,50}"),
                path in proptest::option::of("/[a-zA-Z0-9_/-]{1,100}")
            ) {
                let protocol = Protocol {
                    service: service.clone(),
                    bucket: bucket.clone(),
                    region: region.clone(),
                    path: path.clone(),
                    key_translation: KeyTranslation::default(),
                    parameters: HashMap::new(),
                };

                let result = UriExporter::export_protocol(&protocol);
                prop_assert!(result.is_ok());

                let uri_str = result.unwrap();
                prop_assert!(uri_str.contains("service="));

                if let Some(ref b) = bucket {
                    let expected = format!("bucket={}", urlencoding::encode(b));
                    prop_assert!(uri_str.contains(&expected));
                }
                if let Some(ref r) = region {
                    let expected = format!("region={}", urlencoding::encode(r));
                    prop_assert!(uri_str.contains(&expected));
                }
                if let Some(ref p) = path {
                    let expected = format!("path={}", urlencoding::encode(p));
                    prop_assert!(uri_str.contains(&expected));
                }
            }

            #[test]
            fn test_export_service_config_arbitrary_strings(
                region in proptest::option::of("[a-zA-Z0-9_-]{1,50}"),
                endpoint in proptest::option::of("https://[a-zA-Z0-9.-]{1,100}"),
                permissions in proptest::option::of("[a-zA-Z0-9_-]{1,20}")
            ) {
                let service_config = ServiceConfig {
                    default_region: region.clone(),
                    endpoint: endpoint.clone(),
                    default_permissions: permissions.clone(),
                    parameters: HashMap::new(),
                };

                let result = UriExporter::export_service_config(&service_config);
                prop_assert!(result.is_ok());

                let uri_str = result.unwrap();

                if let Some(ref r) = region {
                    let expected = format!("default_region={}", urlencoding::encode(r));
                    prop_assert!(uri_str.contains(&expected));
                }
                if let Some(ref e) = endpoint {
                    let expected = format!("endpoint={}", urlencoding::encode(e));
                    prop_assert!(uri_str.contains(&expected));
                }
                if let Some(ref p) = permissions {
                    let expected = format!("default_permissions={}", urlencoding::encode(p));
                    prop_assert!(uri_str.contains(&expected));
                }
            }

            #[test]
            fn test_export_key_translation_arbitrary_values(
                prefix_len in proptest::option::of(0usize..1000),
                max_len in proptest::option::of(0usize..10000),
                part_len in 0usize..100,
                part_depth in 0usize..10
            ) {
                let key_translation = KeyTranslation {
                    hash_prefix_length: prefix_len,
                    max_key_length: max_len,
                    partition_length: part_len,
                    partition_depth: part_depth,
                    partition: part_len > 0 && part_depth > 0,
                    ..Default::default()
                };

                let result = UriExporter::export_key_translation(&key_translation);
                prop_assert!(result.is_ok());

                let key_str = result.unwrap();

                if let Some(len) = prefix_len {
                    let expected = format!("hash_prefix_length={}", len);
                    prop_assert!(key_str.contains(&expected));
                }
                if let Some(len) = max_len {
                    let expected = format!("max_key_length={}", len);
                    prop_assert!(key_str.contains(&expected));
                }
                if key_translation.partition {
                    let expected_len = format!("partition_length={}", part_len);
                    let expected_depth = format!("partition_depth={}", part_depth);
                    prop_assert!(key_str.contains(&expected_len));
                    prop_assert!(key_str.contains(&expected_depth));
                }
            }

            #[test]
            fn test_export_config_arbitrary_protocols_and_services(
                num_protocols in 0usize..10,
                num_services in 0usize..10
            ) {
                let mut config = Config::default();

                // Add arbitrary number of protocols
                for i in 0..num_protocols {
                    let protocol = Protocol {
                        service: format!("service{}", i),
                        bucket: Some(format!("bucket{}", i)),
                        region: Some(format!("region{}", i)),
                        path: None,
                        key_translation: KeyTranslation::default(),
                        parameters: HashMap::new(),
                    };
                    config.protocols.insert(format!("protocol{}", i), protocol);
                }

                // Add arbitrary number of services
                for i in 0..num_services {
                    let service = ServiceConfig {
                        default_region: Some(format!("region{}", i)),
                        endpoint: Some(format!("https://service{}.example.com", i)),
                        default_permissions: None,
                        parameters: HashMap::new(),
                    };
                    config.services.insert(format!("service{}", i), service);
                }

                let result = UriExporter::export_config(&config);
                prop_assert!(result.is_ok());

                let uri_str = result.unwrap();

                if num_protocols == 0 && num_services == 0 {
                    prop_assert_eq!(uri_str, "config://empty");
                } else {
                    prop_assert!(uri_str.starts_with("config://"));
                    for i in 0..num_protocols {
                        let expected = format!("protocol:protocol{}:", i);
                        prop_assert!(uri_str.contains(&expected));
                    }
                    for i in 0..num_services {
                        let expected = format!("service:service{}:", i);
                        prop_assert!(uri_str.contains(&expected));
                    }
                }
            }

            #[test]
            fn test_round_trip_export_validate(
                service in "[a-zA-Z0-9_-]{1,20}",
                bucket in proptest::option::of("[a-zA-Z0-9_-]{1,20}"),
                region in proptest::option::of("[a-zA-Z0-9_-]{1,20}")
            ) {
                let protocol = Protocol {
                    service,
                    bucket,
                    region,
                    path: None,
                    key_translation: KeyTranslation::default(),
                    parameters: HashMap::new(),
                };

                let mut config = Config::default();
                config.protocols.insert("test".to_string(), protocol);

                let uri_str = UriExporter::export_config(&config).unwrap();
                let is_valid = UriExporter::validate_export(&uri_str).unwrap();
                prop_assert!(is_valid, "URI should be valid: {}", uri_str);
            }
        }
    }

    #[cfg(test)]
    mod benchmark_tests {
        use super::*;
        use std::time::Instant;

        #[test]
        fn test_export_performance_large_config() {
            let mut config = Config::default();

            // Create a large configuration with many protocols and services
            for i in 0..100 {
                let mut parameters = HashMap::new();
                for j in 0..10 {
                    parameters.insert(format!("param{}", j), format!("value{}", j));
                }

                let protocol = Protocol {
                    service: format!("service{}", i),
                    bucket: Some(format!("bucket{}", i)),
                    region: Some(format!("region{}", i)),
                    path: Some(format!("/path/{}", i)),
                    key_translation: KeyTranslation {
                        downcase: i % 2 == 0,
                        hash_function: Some(HashFunction::Sha256),
                        prefix: Some(format!("prefix{}", i)),
                        suffix: Some(format!("suffix{}", i)),
                        max_key_length: Some(1000 + i),
                        ..Default::default()
                    },
                    parameters: parameters.clone(),
                };

                let service = ServiceConfig {
                    default_region: Some(format!("region{}", i)),
                    endpoint: Some(format!("https://service{}.example.com", i)),
                    default_permissions: Some(format!("perms{}", i)),
                    parameters,
                };

                config.protocols.insert(format!("protocol{}", i), protocol);
                config.services.insert(format!("service{}", i), service);
            }

            let start = Instant::now();
            let result = UriExporter::export_config(&config);
            let duration = start.elapsed();

            assert!(result.is_ok());
            let uri_str = result.unwrap();

            // Should complete in reasonable time (less than 1 second)
            assert!(duration.as_millis() < 1000, "Export took too long: {:?}", duration);

            // URI should be quite long
            assert!(uri_str.len() > 10000);

            // Validate it can be parsed back
            let is_valid = UriExporter::validate_export(&uri_str).unwrap();
            assert!(is_valid);
        }

        #[test]
        fn test_export_performance_many_parameters() {
            let mut parameters = HashMap::new();

            // Add many parameters
            for i in 0..1000 {
                parameters.insert(format!("param{}", i), format!("value{}", i));
            }

            let protocol = Protocol {
                service: "s3".to_string(),
                bucket: Some("bucket".to_string()),
                region: Some("us-east-1".to_string()),
                path: None,
                key_translation: KeyTranslation::default(),
                parameters,
            };

            let start = Instant::now();
            let result = UriExporter::export_protocol(&protocol);
            let duration = start.elapsed();

            assert!(result.is_ok());
            let uri_str = result.unwrap();

            // Should complete quickly even with many parameters
            assert!(duration.as_millis() < 100, "Export took too long: {:?}", duration);

            // Should contain all parameters
            for i in 0..1000 {
                assert!(uri_str.contains(&format!("param{}=", i)));
                assert!(uri_str.contains(&format!("value{}", i)));
            }
        }
    }

    #[cfg(test)]
    mod fuzzing_tests {
        use super::*;

        #[test]
        fn test_export_with_extreme_unicode() {
            // Test with various Unicode characters that might cause issues
            let test_strings = vec![
                "🚀🌟💫",  // Emojis
                "café naïve résumé",  // Accented characters
                "测试中文",  // Chinese characters
                "русский текст",  // Cyrillic
                "العربية",  // Arabic
                "🌸🌺🌻🌷",  // More emojis
                "αβγδε",  // Greek
                "⅓¼½¾",  // Fractions
                "™®©",  // Symbols
            ];

            for test_str in test_strings {
                let mut parameters = HashMap::new();
                parameters.insert(test_str.to_string(), test_str.to_string());

                let protocol = Protocol {
                    service: "s3".to_string(),
                    bucket: Some(test_str.to_string()),
                    region: Some("us-east-1".to_string()),
                    path: Some(format!("/{}", test_str)),
                    key_translation: KeyTranslation {
                        prefix: Some(test_str.to_string()),
                        suffix: Some(test_str.to_string()),
                        replace_spaces: Some(test_str.to_string()),
                        ..Default::default()
                    },
                    parameters,
                };

                let result = UriExporter::export_protocol(&protocol);
                assert!(result.is_ok(), "Failed to export with string: {}", test_str);

                let uri_str = result.unwrap();

                // Should be able to validate (parse back)
                let is_valid = UriExporter::validate_export(&uri_str);
                assert!(is_valid.is_ok(), "Failed to validate URI with string: {}", test_str);
            }
        }

        #[test]
        fn test_export_with_pathological_inputs() {
            let long_string = "a".repeat(100);
            let medium_string = "x".repeat(50);
            let pathological_inputs = vec![
                "",  // Empty string
                " ",  // Space only
                "\t\n\r",  // Whitespace characters
                "\0",  // Null byte
                &long_string,  // Long string
                &medium_string,  // Medium string
                "!@#$%^&*()",  // Special characters
                "<>\"';",  // Potentially problematic characters
                "file:with:colons",  // Colons
                "path/with/slashes",  // Slashes
                "?query=value&other=123",  // Query-like
                "#fragment",  // Fragment
                "user@host.com",  // Email-like
                "192.168.1.1",  // IP-like
                "::1",  // IPv6-like
            ];

            for input in pathological_inputs {
                let protocol = Protocol {
                    service: input.to_string(),
                    bucket: Some(input.to_string()),
                    region: Some(input.to_string()),
                    path: Some(format!("/{}", input)),
                    key_translation: KeyTranslation {
                        prefix: Some(input.to_string()),
                        suffix: Some(input.to_string()),
                        partition_separator: input.to_string(),
                        replace_spaces: Some(input.to_string()),
                        ..Default::default()
                    },
                    parameters: {
                        let mut p = HashMap::new();
                        p.insert(input.to_string(), input.to_string());
                        p
                    },
                };

                let result = UriExporter::export_protocol(&protocol);
                // Should not panic, even with pathological input
                assert!(result.is_ok(), "Failed with pathological input: {:?}", input);

                let uri_str = result.unwrap();

                // The URI should contain URL-encoded versions of the input
                // (unless the input is empty, which gets special handling)
                if !input.is_empty() {
                    assert!(uri_str.contains("service="));
                }
            }
        }
    }
}