//! URI resolver implementation
//!
//! This module provides a thin wrapper around the shared URI resolution functionality
//! for backward compatibility with existing code.

use durable_filestor_shared::{Config, ParsedUri, ResolvedUri, Result, UriResolver as SharedUriResolver};

/// URI resolver that maps protocols to service configurations
pub struct UriResolver<'a> {
    shared_resolver: SharedUriResolver<'a>,
}

impl<'a> UriResolver<'a> {
    /// Create a new URI resolver
    pub fn new(config: &'a Config) -> Self {
        Self {
            shared_resolver: SharedUriResolver::new(config),
        }
    }

    /// Resolve a parsed URI to its service configuration
    ///
    /// # Examples
    /// ```
    /// use durable_filestor_uri_parser::{UriParser, UriResolver};
    /// use durable_filestor_shared::{Config, Protocol};
    /// use std::collections::HashMap;
    ///
    /// let mut protocols = HashMap::new();
    /// protocols.insert("my-s3".to_string(), Protocol {
    ///     service: "s3".to_string(),
    ///     bucket: Some("my-bucket".to_string()),
    ///     region: Some("us-east-1".to_string()),
    ///     path: None,
    ///     key_translation: Default::default(),
    ///     parameters: HashMap::new(),
    /// });
    ///
    /// let config = Config {
    ///     protocols,
    ///     services: HashMap::new(),
    /// };
    /// let resolver = UriResolver::new(&config);
    /// let parsed = UriParser::parse("my-s3://bucket/file.txt").unwrap();
    /// let resolved = resolver.resolve(parsed).unwrap();
    /// assert_eq!(resolved.service, "s3");
    /// ```
    pub fn resolve(&self, parsed_uri: ParsedUri) -> Result<ResolvedUri> {
        self.shared_resolver.resolve(parsed_uri)
    }

    /// Resolve a URI string directly
    pub fn resolve_uri(&self, uri_str: &str) -> Result<ResolvedUri> {
        self.shared_resolver.resolve_uri(uri_str)
    }

    /// Check if a protocol is configured
    pub fn is_protocol_configured(&self, protocol: &str) -> bool {
        self.shared_resolver.is_protocol_configured(protocol)
    }

    /// Get all configured protocols
    pub fn configured_protocols(&self) -> Vec<String> {
        self.shared_resolver.configured_protocols()
    }

    /// Get protocol configuration by name
    pub fn get_protocol_config(&self, protocol: &str) -> Option<&durable_filestor_shared::Protocol> {
        self.shared_resolver.get_protocol_config(protocol)
    }
}

