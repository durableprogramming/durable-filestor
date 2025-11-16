//! Test binary for URI config exporter
//!
//! This binary provides command-line interface for testing the URI exporter functionality.

use clap::{Parser, Subcommand};
use durable_filestor_config_uri_exporter::UriExporter;
use durable_filestor_shared::{Config, Protocol, ServiceConfig, KeyTranslation};
use std::collections::HashMap;
use std::fs;

#[derive(Parser)]
#[command(name = "dfs-config-uri-export")]
#[command(about = "Test tool for DFS URI config export functionality")]
#[command(version = "0.1.0")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Export a config file to URI format
    Config {
        /// Path to the config file to export
        file: String,
    },
    /// Export a protocol to URI format
    Protocol {
        /// Protocol name
        name: String,
        /// Storage service type
        service: String,
        /// Bucket name (for S3-like services)
        #[arg(long)]
        bucket: Option<String>,
        /// Region (for S3-like services)
        #[arg(long)]
        region: Option<String>,
        /// Path (for local filesystem)
        #[arg(long)]
        path: Option<String>,
    },
    /// Export a service configuration to URI format
    Service {
        /// Service name
        name: String,
        /// Default region
        #[arg(long)]
        region: Option<String>,
        /// Endpoint URL
        #[arg(long)]
        endpoint: Option<String>,
        /// Default permissions
        #[arg(long)]
        permissions: Option<String>,
    },
    /// Export config as multiple URIs (one per protocol/service)
    MultiUri {
        /// Path to the config file to export
        file: String,
    },
    /// Validate that exported URI can be parsed
    Validate {
        /// URI string to validate
        uri: String,
    },
    /// Show examples of usage
    Examples,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Config { file } => {
            let content = fs::read_to_string(&file)
                .map_err(|e| format!("Failed to read config file '{}': {}", file, e))?;

            let config: Config = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse config file '{}': {}", file, e))?;

            let exported = UriExporter::export_config(&config)?;
            println!("{}", exported);
        }
        Commands::Protocol { name, service, bucket, region, path } => {
            let protocol = Protocol {
                service,
                bucket,
                region,
                path,
                key_translation: KeyTranslation::default(),
                parameters: HashMap::new(),
            };

            let exported = UriExporter::export_protocol(&protocol)?;
            println!("{}:{}", name, exported);
        }
        Commands::Service { name, region, endpoint, permissions } => {
            let service_config = ServiceConfig {
                default_region: region,
                endpoint,
                default_permissions: permissions,
                parameters: HashMap::new(),
            };

            let exported = UriExporter::export_service_config(&service_config)?;
            println!("{}:{}", name, exported);
        }
        Commands::MultiUri { file } => {
            let content = fs::read_to_string(&file)
                .map_err(|e| format!("Failed to read config file '{}': {}", file, e))?;

            let config: Config = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse config file '{}': {}", file, e))?;

            let uris = UriExporter::export_config_as_uris(&config)?;
            for uri in uris {
                println!("{}", uri);
            }
        }
        Commands::Validate { uri } => {
            let is_valid = UriExporter::validate_export(&uri)?;
            if is_valid {
                println!("✅ URI is valid and can be parsed");
                std::process::exit(0);
            } else {
                println!("❌ URI is invalid or cannot be parsed");
                std::process::exit(1);
            }
        }
        Commands::Examples => {
            show_examples();
        }
    }

    Ok(())
}

fn show_examples() {
    println!("DFS Config URI Export Examples");
    println!("==============================");
    println!();
    println!("Export a config file to URI format:");
    println!("  dfs-config-uri-export config examples/example1.toml");
    println!();
    println!("Export a protocol configuration:");
    println!("  dfs-config-uri-export protocol my-s3 --service s3 --bucket my-bucket --region us-east-1");
    println!();
    println!("Export a service configuration:");
    println!("  dfs-config-uri-export service s3 --region us-west-2 --endpoint https://s3.us-west-2.amazonaws.com");
    println!();
    println!("Export config as multiple URIs:");
    println!("  dfs-config-uri-export multi-uri examples/example1.toml");
    println!();
    println!("Validate a URI can be parsed:");
    println!("  dfs-config-uri-export validate \"protocol://service=s3&bucket=my-bucket\"");
    println!();
    println!("Show this help:");
    println!("  dfs-config-uri-export --help");
}