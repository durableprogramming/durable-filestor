//! Protocol handlers for URI-based configuration

use durable_filestor_shared::{Config, DurableError, Result, Protocol, ServiceConfig, ParsedUri, UriParser};

/// Parse TOML protocol: toml://base64-encoded-toml-config
pub fn parse_toml_protocol(parsed_uri: &ParsedUri) -> Result<Config> {
    // Extract the path part which should be the base64-encoded TOML
    let encoded_toml = &parsed_uri.path;

    // Validate that we have a path (hostname/content)
    if encoded_toml.is_empty() {
        return Err(DurableError::Config {
            message: "TOML protocol requires base64-encoded TOML as hostname".to_string(),
        });
    }

    // Decode the base64-encoded TOML using shared utilities
    let toml_content = UriParser::decode_base64_to_string(encoded_toml)
        .map_err(|e| DurableError::Config {
            message: format!("Failed to decode base64 TOML: {}", e),
        })?;

    // Parse the TOML content using shared TOML parsing utility
    let config: Config = UriParser::parse_toml(&toml_content)?;

    Ok(config)
}

/// Parse DFS protocol: dfs://service?key1=value1&key2=value2&...
/// Creates a flexible configuration that accepts any key-value pairs
pub fn parse_dfs_protocol(parsed_uri: &ParsedUri) -> Result<Config> {
    let service_type = &parsed_uri.path;

    // Validate that we have a service type (hostname/path)
    if service_type.is_empty() {
        return Err(DurableError::Config {
            message: "DFS protocol requires service type as hostname (e.g., dfs://s3)".to_string(),
        });
    }

    // Use the already parsed query parameters
    let params = &parsed_uri.query_params;

    let mut config = Config::default();

    // Create a flexible service configuration with all parameters
    let service_config = Config::create_generic_service_config(&params);
    config.services.insert(service_type.to_string(), service_config);

    // Create a default protocol for this service
    let protocol = Config::create_generic_protocol(service_type, &params);
    config.protocols.insert(format!("default-{}", service_type), protocol);

    Ok(config)
}

