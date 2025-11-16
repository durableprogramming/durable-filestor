//! Common URI parsing utilities and types
//!
//! This module provides shared functionality for URI parsing and resolution
//! that can be used across different parts of the durable-filestor system.

use crate::{DurableError, Result};
use std::collections::HashMap;

/// URI parser for custom protocols
pub struct UriParser;

impl UriParser {
    /// Parse a URI string into its components
    ///
    /// # Examples
    /// ```
    /// use durable_filestor_shared::UriParser;
    ///
    /// let uri = "my-s3://bucket/file.txt?version=1";
    /// let parsed = UriParser::parse(uri).unwrap();
    /// assert_eq!(parsed.protocol, "my-s3");
    /// assert_eq!(parsed.path, "bucket/file.txt");
    /// ```
    pub fn parse(uri_str: &str) -> Result<crate::ParsedUri> {
        // Handle empty or whitespace-only URIs
        let uri_str = uri_str.trim();
        if uri_str.is_empty() {
            return Err(DurableError::Uri {
                message: "URI cannot be empty".to_string(),
            });
        }

        // Check if it contains the protocol separator
        if uri_str.contains("://") {
            // Try custom protocol parsing first for protocols that might not be standard URLs
            if let Ok(parsed) = Self::parse_custom_protocol(uri_str) {
                return Ok(parsed);
            }
        }

        // Check if it's a standard URL format
        if let Ok(url) = ::url::Url::parse(uri_str) {
            return Self::parse_standard_url(url, uri_str);
        }

        // Handle custom protocol format: protocol://path
        Self::parse_custom_protocol(uri_str)
    }

    /// Parse a standard URL (with scheme://host/path format)
    fn parse_standard_url(url: ::url::Url, original: &str) -> Result<crate::ParsedUri> {
        let protocol = url.scheme().to_string();

        // For standard URLs, the path includes the host
        let path = if let Some(host) = url.host_str() {
            if let Some(url_path) = url.path().strip_prefix('/') {
                format!("{}/{}", host, url_path)
            } else {
                host.to_string()
            }
        } else {
            url.path().to_string()
        };

        // Parse query parameters
        let mut query_params = HashMap::new();
        if let Some(query) = url.query() {
            for (key, value) in ::url::form_urlencoded::parse(query.as_bytes()) {
                query_params.insert(key.to_string(), value.to_string());
            }
        }

        Ok(crate::ParsedUri::new(protocol, path, query_params, original.to_string()))
    }

    /// Parse custom protocol format (protocol://path)
    fn parse_custom_protocol(uri_str: &str) -> Result<crate::ParsedUri> {
        // Find the protocol separator
        let protocol_end = uri_str.find("://").ok_or_else(|| DurableError::Uri {
            message: format!("Invalid URI format: '{}'. Expected format: protocol://path", uri_str),
        })?;

        let protocol = uri_str[..protocol_end].to_string();
        let rest = &uri_str[protocol_end + 3..]; // Skip "://"

        // Validate protocol name
        if protocol.is_empty() {
            return Err(DurableError::Uri {
                message: "Protocol name cannot be empty".to_string(),
            });
        }

        // Check for invalid characters in protocol name
        if protocol.contains('/') || protocol.contains('?') || protocol.contains('#') {
            return Err(DurableError::Uri {
                message: format!("Invalid protocol name: '{}'. Protocol names cannot contain '/', '?', or '#'", protocol),
            });
        }

        // Split path and query parameters
        let (path, query_str) = if let Some(query_start) = rest.find('?') {
            let path_part = &rest[..query_start];
            let query_part = &rest[query_start + 1..];
            (path_part.to_string(), Some(query_part.to_string()))
        } else {
            (rest.to_string(), None)
        };

        // Parse query parameters
        let mut query_params = HashMap::new();
        if let Some(query) = query_str {
            for pair in query.split('&') {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next().unwrap_or("").to_string();
                let value = parts.next().unwrap_or("").to_string();

                if !key.is_empty() {
                    let decoded_key = urlencoding::decode(&key).unwrap_or_else(|_| key.clone().into());
                    let decoded_value = urlencoding::decode(&value).unwrap_or_else(|_| value.clone().into());
                    query_params.insert(decoded_key.to_string(), decoded_value.to_string());
                }
            }
        }

        Ok(crate::ParsedUri::new(protocol, path, query_params, uri_str.to_string()))
    }

    /// Validate that a URI string is properly formatted
    pub fn validate(uri_str: &str) -> Result<()> {
        Self::parse(uri_str).map(|_| ())
    }

    /// Extract just the protocol name from a URI
    pub fn extract_protocol(uri_str: &str) -> Result<String> {
        let parsed = Self::parse(uri_str)?;
        Ok(parsed.protocol)
    }

    /// Extract just the path from a URI
    pub fn extract_path(uri_str: &str) -> Result<String> {
        let parsed = Self::parse(uri_str)?;
        Ok(parsed.path)
    }

    /// Decode base64-encoded content
    pub fn decode_base64(encoded: &str) -> Result<Vec<u8>> {
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD.decode(encoded)
            .map_err(|e| DurableError::Uri {
                message: format!("Failed to decode base64: {}", e),
            })
    }

    /// Decode base64-encoded content to UTF-8 string
    pub fn decode_base64_to_string(encoded: &str) -> Result<String> {
        let bytes = Self::decode_base64(encoded)?;
        String::from_utf8(bytes)
            .map_err(|e| DurableError::Uri {
                message: format!("Invalid UTF-8 in decoded content: {}", e),
            })
    }

    /// Encode content to base64
    pub fn encode_base64(data: &[u8]) -> String {
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD.encode(data)
    }

    /// Encode string content to base64
    pub fn encode_string_to_base64(content: &str) -> String {
        Self::encode_base64(content.as_bytes())
    }

    /// Parse TOML content with consistent error handling
    ///
    /// This utility provides standardized TOML parsing with proper error wrapping
    /// for use across the durable-filestor codebase.
    pub fn parse_toml<T>(toml_content: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        toml::from_str(toml_content)
            .map_err(|e| DurableError::Config {
                message: format!("Failed to parse TOML configuration: {}", e),
            })
    }

    /// Encode content to base64 (convenience method for strings)
    ///
    /// This is a convenience method that wraps encode_string_to_base64
    /// for use in tests and other utility functions.
    pub fn encode_to_base64(content: &str) -> String {
        Self::encode_string_to_base64(content)
    }

    /// Encode bytes to base64 (convenience method for raw bytes)
    ///
    /// This is a convenience method for encoding raw bytes to base64,
    /// useful for test fixtures and data preparation.
    pub fn encode_bytes_to_base64(bytes: &[u8]) -> String {
        Self::encode_base64(bytes)
    }
}

/// URI resolver that maps protocols to service configurations
pub struct UriResolver<'a> {
    config: &'a crate::Config,
}

impl<'a> UriResolver<'a> {
    /// Create a new URI resolver
    pub fn new(config: &'a crate::Config) -> Self {
        Self { config }
    }

    /// Resolve a parsed URI to its service configuration
    pub fn resolve(&self, parsed_uri: crate::ParsedUri) -> Result<crate::ResolvedUri> {
        // Find the protocol configuration
        let protocol_config = self.config.protocols.get(&parsed_uri.protocol)
            .ok_or_else(|| DurableError::ProtocolNotFound {
                protocol: parsed_uri.protocol.clone(),
            })?
            .clone();

        let service = protocol_config.service.clone();

        // Apply key translation to the path
        let key_translator = crate::KeyTranslator::new(protocol_config.key_translation.clone());
        let translated_key = key_translator.translate(&parsed_uri.path);

        Ok(crate::ResolvedUri::new(
            parsed_uri,
            protocol_config,
            translated_key,
            service,
        ))
    }

    /// Resolve a URI string directly
    pub fn resolve_uri(&self, uri_str: &str) -> Result<crate::ResolvedUri> {
        let parsed = UriParser::parse(uri_str)?;
        self.resolve(parsed)
    }

    /// Check if a protocol is configured
    pub fn is_protocol_configured(&self, protocol: &str) -> bool {
        self.config.protocols.contains_key(protocol)
    }

    /// Get all configured protocols
    pub fn configured_protocols(&self) -> Vec<String> {
        self.config.protocols.keys().cloned().collect()
    }

    /// Get protocol configuration by name
    pub fn get_protocol_config(&self, protocol: &str) -> Option<&crate::Protocol> {
        self.config.protocols.get(protocol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_standard_s3_uri() {
        let uri = "s3://my-bucket/path/to/file.txt";
        let parsed = UriParser::parse(uri).unwrap();

        assert_eq!(parsed.protocol, "s3");
        assert_eq!(parsed.path, "my-bucket/path/to/file.txt");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_custom_protocol_uri() {
        let uri = "my-s3://bucket/file.txt";
        let parsed = UriParser::parse(uri).unwrap();

        assert_eq!(parsed.protocol, "my-s3");
        assert_eq!(parsed.path, "bucket/file.txt");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_uri_with_query_params() {
        let uri = "my-s3://bucket/file.txt?version=1&encoding=utf8";
        let parsed = UriParser::parse(uri).unwrap();

        assert_eq!(parsed.protocol, "my-s3");
        assert_eq!(parsed.path, "bucket/file.txt");
        assert_eq!(parsed.query_params.get("version"), Some(&"1".to_string()));
        assert_eq!(parsed.query_params.get("encoding"), Some(&"utf8".to_string()));
    }

    #[test]
    fn test_parse_uri_with_encoded_params() {
        let uri = "backup://docs/file%20name.txt?path=%2Fhome%2Fuser";
        let parsed = UriParser::parse(uri).unwrap();

        assert_eq!(parsed.protocol, "backup");
        assert_eq!(parsed.path, "docs/file%20name.txt");
        assert_eq!(parsed.query_params.get("path"), Some(&"/home/user".to_string()));
    }

    #[test]
    fn test_parse_empty_uri() {
        let result = UriParser::parse("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DurableError::Uri { .. }));
    }

    #[test]
    fn test_parse_invalid_protocol() {
        let result = UriParser::parse("invalid-protocol");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_protocol() {
        let result = UriParser::parse("://path");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_protocol() {
        assert_eq!(UriParser::extract_protocol("my-s3://bucket/file.txt").unwrap(), "my-s3");
        assert_eq!(UriParser::extract_protocol("backup://docs/").unwrap(), "backup");
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(UriParser::extract_path("my-s3://bucket/file.txt").unwrap(), "bucket/file.txt");
        assert_eq!(UriParser::extract_path("backup://docs/").unwrap(), "docs/");
    }

    #[test]
    fn test_full_path() {
        let parsed = UriParser::parse("my-s3://bucket/file.txt").unwrap();
        assert_eq!(parsed.full_path(), "my-s3://bucket/file.txt");
    }

    fn create_test_config() -> crate::Config {
        let mut protocols = HashMap::new();

        // Add a test protocol
        protocols.insert(
            "test-s3".to_string(),
            crate::Protocol {
                service: "s3".to_string(),
                bucket: Some("test-bucket".to_string()),
                region: Some("us-east-1".to_string()),
                path: None,
                key_translation: Default::default(),
                parameters: HashMap::new(),
            },
        );

        // Add a local protocol
        protocols.insert(
            "test-local".to_string(),
            crate::Protocol {
                service: "local".to_string(),
                bucket: None,
                region: None,
                path: Some("/tmp/test".to_string()),
                key_translation: Default::default(),
                parameters: HashMap::new(),
            },
        );

        crate::Config {
            protocols,
            services: HashMap::new(),
        }
    }

    #[test]
    fn test_resolve_s3_protocol() {
        let config = create_test_config();
        let resolver = UriResolver::new(&config);
        let parsed = UriParser::parse("test-s3://documents/file.txt").unwrap();

        let resolved = resolver.resolve(parsed).unwrap();

        assert_eq!(resolved.service, "s3");
        assert_eq!(resolved.bucket(), Some("test-bucket"));
        assert_eq!(resolved.region(), Some("us-east-1"));
        assert_eq!(resolved.translated_key, "documents/file.txt");
    }

    #[test]
    fn test_resolve_local_protocol() {
        let config = create_test_config();
        let resolver = UriResolver::new(&config);
        let parsed = UriParser::parse("test-local://files/data.txt").unwrap();

        let resolved = resolver.resolve(parsed).unwrap();

        assert_eq!(resolved.service, "local");
        assert_eq!(resolved.path(), Some("/tmp/test"));
        assert_eq!(resolved.translated_key, "files/data.txt");
    }

    #[test]
    fn test_resolve_unknown_protocol() {
        let config = create_test_config();
        let resolver = UriResolver::new(&config);
        let parsed = UriParser::parse("unknown://file.txt").unwrap();

        let result = resolver.resolve(parsed);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DurableError::ProtocolNotFound { .. }));
    }

    #[test]
    fn test_resolve_uri_directly() {
        let config = create_test_config();
        let resolver = UriResolver::new(&config);

        let resolved = resolver.resolve_uri("test-s3://bucket/file.txt").unwrap();

        assert_eq!(resolved.service, "s3");
        assert_eq!(resolved.bucket(), Some("test-bucket"));
    }

    #[test]
    fn test_is_protocol_configured() {
        let config = create_test_config();
        let resolver = UriResolver::new(&config);

        assert!(resolver.is_protocol_configured("test-s3"));
        assert!(resolver.is_protocol_configured("test-local"));
        assert!(!resolver.is_protocol_configured("unknown"));
    }

    #[test]
    fn test_configured_protocols() {
        let config = create_test_config();
        let resolver = UriResolver::new(&config);

        let protocols = resolver.configured_protocols();
        assert!(protocols.contains(&"test-s3".to_string()));
        assert!(protocols.contains(&"test-local".to_string()));
    }
}