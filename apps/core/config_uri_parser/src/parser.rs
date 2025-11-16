//! URI configuration parser implementation

use crate::protocols::{parse_dfs_protocol, parse_toml_protocol};
use durable_filestor_shared::{Config, DurableError, Result, UriParser};

/// URI-based configuration parser
pub struct ConfigUriParser;

impl ConfigUriParser {
    /// Create a new URI configuration parser
    pub fn new() -> Self {
        Self
    }

    /// Parse configuration from a URI string
    pub fn parse_from_uri(&self, uri: &str) -> Result<Config> {
        let parsed_uri = UriParser::parse(uri)
            .map_err(|e| DurableError::Config {
                message: format!("Invalid URI format: {}", e),
            })?;

        match parsed_uri.protocol.as_str() {
            "toml" => parse_toml_protocol(&parsed_uri),
            "dfs" => parse_dfs_protocol(&parsed_uri),
            scheme => Err(DurableError::Config {
                message: format!("Unsupported URI scheme: {}", scheme),
            }),
        }
    }

    /// Parse configuration from a URI string with validation
    pub fn parse_and_validate(&self, uri: &str) -> Result<Config> {
        let config = self.parse_from_uri(uri)?;

        // Basic validation using shared utilities
        config.validate_basic()?;

        Ok(config)
    }
}

impl Default for ConfigUriParser {
    fn default() -> Self {
        Self::new()
    }
}