//! Common types and data structures

use serde::{Deserialize, Serialize};

/// Supported hash functions for key translation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HashFunction {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Blake2b,
}

/// Key translation configuration options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct KeyTranslation {
    /// Convert keys to lowercase
    pub downcase: bool,
    /// Convert keys to uppercase
    pub upcase: bool,
    /// Hash function to use
    pub hash_function: Option<HashFunction>,
    /// Use only first N characters of hash
    pub hash_prefix_length: Option<usize>,
    /// Enable key partitioning
    pub partition: bool,
    /// Characters per partition segment
    pub partition_length: usize,
    /// Number of partition levels
    pub partition_depth: usize,
    /// Directory separator for partitions
    pub partition_separator: String,
    /// Add prefix to all keys
    pub prefix: Option<String>,
    /// Add suffix to all keys
    pub suffix: Option<String>,
    /// Add timestamp suffix
    pub timestamp_suffix: bool,
    /// Normalize unicode characters
    pub normalize_unicode: bool,
    /// Remove leading/trailing whitespace
    pub strip_whitespace: bool,
    /// Replace spaces with character
    pub replace_spaces: Option<String>,
    /// Truncate long keys
    pub max_key_length: Option<usize>,
}

impl Default for HashFunction {
    fn default() -> Self {
        HashFunction::Sha256
    }
}

/// Protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    /// Storage service type
    pub service: String,
    /// Bucket name (for S3-like services)
    pub bucket: Option<String>,
    /// Region (for S3-like services)
    pub region: Option<String>,
    /// Path (for local filesystem)
    pub path: Option<String>,
    /// Key translation options
    #[serde(default)]
    pub key_translation: KeyTranslation,
    /// Additional protocol-specific parameters
    #[serde(default)]
    pub parameters: std::collections::HashMap<String, String>,
}

/// Service-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceConfig {
    /// Default region for S3 services
    pub default_region: Option<String>,
    /// Custom endpoint URL
    pub endpoint: Option<String>,
    /// Default permissions for local files
    pub default_permissions: Option<String>,
    /// Additional service-specific parameters
    #[serde(default)]
    pub parameters: std::collections::HashMap<String, String>,
}

/// Complete application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Protocol definitions
    #[serde(default)]
    pub protocols: std::collections::HashMap<String, Protocol>,
    /// Service-specific configurations
    #[serde(default)]
    pub services: std::collections::HashMap<String, ServiceConfig>,
}

/// Represents a parsed URI with protocol information
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedUri {
    /// The protocol name (e.g., "my-s3", "backup")
    pub protocol: String,
    /// The path component of the URI
    pub path: String,
    /// Query parameters from the URI
    pub query_params: std::collections::HashMap<String, String>,
    /// The original URI string
    pub original: String,
}

impl ParsedUri {
    /// Create a new ParsedUri
    pub fn new(protocol: String, path: String, query_params: std::collections::HashMap<String, String>, original: String) -> Self {
        Self {
            protocol,
            path,
            query_params,
            original,
        }
    }

    /// Get the full path including protocol
    pub fn full_path(&self) -> String {
        format!("{}://{}", self.protocol, self.path)
    }
}

/// Represents a resolved URI with full service configuration
#[derive(Debug, Clone)]
pub struct ResolvedUri {
    /// The original parsed URI
    pub parsed: ParsedUri,
    /// The resolved protocol configuration
    pub protocol: Protocol,
    /// The translated key (after applying key translation rules)
    pub translated_key: String,
    /// The service type
    pub service: String,
}

impl ResolvedUri {
    /// Create a new ResolvedUri
    pub fn new(parsed: ParsedUri, protocol: Protocol, translated_key: String, service: String) -> Self {
        Self {
            parsed,
            protocol,
            translated_key,
            service,
        }
    }

    /// Get the bucket name (if applicable)
    pub fn bucket(&self) -> Option<&str> {
        self.protocol.bucket.as_deref()
    }

    /// Get the region (if applicable)
    pub fn region(&self) -> Option<&str> {
        self.protocol.region.as_deref()
    }

    /// Get the path (if applicable)
    pub fn path(&self) -> Option<&str> {
        self.protocol.path.as_deref()
    }
}

impl Config {
    /// Validate that the configuration contains at least one protocol or service
    pub fn validate_basic(&self) -> crate::Result<()> {
        if self.protocols.is_empty() && self.services.is_empty() {
            return Err(crate::DurableError::Config {
                message: "Configuration must contain at least one protocol or service".to_string(),
            });
        }
        Ok(())
    }

    /// Check if the configuration is empty (no protocols or services)
    pub fn is_empty(&self) -> bool {
        self.protocols.is_empty() && self.services.is_empty()
    }

    /// Get the number of protocols in the configuration
    pub fn protocol_count(&self) -> usize {
        self.protocols.len()
    }

    /// Get the number of services in the configuration
    pub fn service_count(&self) -> usize {
        self.services.len()
    }

    /// Create a generic service configuration from parameters
    pub fn create_generic_service_config(params: &std::collections::HashMap<String, String>) -> ServiceConfig {
        let mut service_config = ServiceConfig::default();

        // Handle common/standard parameters
        if let Some(region) = params.get("region") {
            service_config.default_region = Some(region.clone());
        }

        if let Some(endpoint) = params.get("endpoint") {
            service_config.endpoint = Some(endpoint.clone());
        }

        if let Some(permissions) = params.get("default_permissions") {
            service_config.default_permissions = Some(permissions.clone());
        }

        // Store all parameters including standard ones for complete access
        service_config.parameters = params.clone();

        service_config
    }

    /// Create a generic protocol configuration from parameters
    pub fn create_generic_protocol(service_type: &str, params: &std::collections::HashMap<String, String>) -> Protocol {
        Protocol {
            service: service_type.to_string(),
            region: params.get("region").cloned(),
            bucket: params.get("bucket_name")
                .or_else(|| params.get("bucket"))
                .or_else(|| params.get("container"))
                .cloned(),
            path: params.get("path").cloned(),
            key_translation: Default::default(), // Could be configured via additional parameters if needed
            parameters: params.clone(),
        }
    }
}

/// Main CLI structure for durable-filestor
#[derive(clap::Parser, Debug)]
#[command(name = "durable-filestor")]
#[command(about = "A robust file storage system for reading and writing to various file storage backends")]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<std::path::PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Subcommands
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Configure protocols and services
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// List files and directories
    List {
        /// Target URI (protocol://path or service-specific path)
        uri: String,

        /// Service type (if not using protocol)
        #[arg(short, long)]
        service: Option<String>,

        /// Bucket name (for S3-like services)
        #[arg(short, long)]
        bucket: Option<String>,

        /// Region (for S3-like services)
        #[arg(short, long)]
        region: Option<String>,

        /// Path (for local filesystem)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Upload files
    Upload {
        /// Source file path
        source: String,

        /// Target URI (protocol://path or service-specific path)
        target: String,

        /// Service type (if not using protocol)
        #[arg(short, long)]
        service: Option<String>,

        /// Bucket name (for S3-like services)
        #[arg(short, long)]
        bucket: Option<String>,

        /// Region (for S3-like services)
        #[arg(short, long)]
        region: Option<String>,

        /// Path (for local filesystem)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Download files
    Download {
        /// Source URI (protocol://path or service-specific path)
        source: String,

        /// Target file path
        target: String,

        /// Service type (if not using protocol)
        #[arg(short, long)]
        service: Option<String>,

        /// Bucket name (for S3-like services)
        #[arg(short, long)]
        bucket: Option<String>,

        /// Region (for S3-like services)
        #[arg(short, long)]
        region: Option<String>,

        /// Path (for local filesystem)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Get file metadata
    Head {
        /// Target URI (protocol://path or service-specific path)
        uri: String,

        /// Service type (if not using protocol)
        #[arg(short, long)]
        service: Option<String>,

        /// Bucket name (for S3-like services)
        #[arg(short, long)]
        bucket: Option<String>,

        /// Region (for S3-like services)
        #[arg(short, long)]
        region: Option<String>,

        /// Path (for local filesystem)
        #[arg(short, long)]
        path: Option<String>,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum ConfigAction {
    /// Add a new protocol
    AddProtocol {
        /// Protocol name
        name: String,

        /// Service type
        #[arg(short, long)]
        service: String,

        /// Bucket name (for S3-like services)
        #[arg(short, long)]
        bucket: Option<String>,

        /// Region (for S3-like services)
        #[arg(short, long)]
        region: Option<String>,

        /// Path (for local filesystem)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Remove a protocol
    RemoveProtocol {
        /// Protocol name
        name: String,
    },

    /// List all protocols
    ListProtocols,

    /// Show current configuration
    Show,
}