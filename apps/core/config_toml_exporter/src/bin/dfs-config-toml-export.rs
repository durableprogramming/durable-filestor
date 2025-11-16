//! Test binary for TOML config exporter
//!
//! This binary provides command-line interface for testing the TOML exporter functionality.

use clap::{Parser, Subcommand};
use durable_filestor_config_toml_exporter::TomlExporter;
use durable_filestor_shared::{Config, Protocol, ServiceConfig, KeyTranslation, context_error};
use std::collections::HashMap;
use std::fs;

#[derive(Parser)]
#[command(name = "dfs-config-toml-export")]
#[command(about = "Test tool for DFS TOML config export functionality")]
#[command(version = "0.1.0")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Export a config file to TOML format
    Config {
        /// Path to the config file to export
        file: String,
        /// Use compact format (default: pretty)
        #[arg(long)]
        compact: bool,
    },
    /// Export a protocol to TOML format
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
    /// Export a service configuration to TOML format
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
    /// Validate that TOML content can be parsed back
    Validate {
        /// Path to TOML file to validate
        file: String,
    },
    /// Show examples of usage
    Examples,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Config { file, compact } => {
            let content = context_error!(
                fs::read_to_string(&file),
                format!("Failed to read config file '{}'", file)
            )?;

            let config: Config = context_error!(
                toml::from_str(&content),
                format!("Failed to parse config file '{}'", file)
            )?;

            let exported = if compact {
                TomlExporter::export_config_with_options(&config, false)?
            } else {
                TomlExporter::export_config(&config)?
            };

            println!("{}", exported);
        }
        Commands::Protocol { name: _, service, bucket, region, path } => {
            let protocol = Protocol {
                service,
                bucket,
                region,
                path,
                key_translation: KeyTranslation::default(),
                parameters: HashMap::new(),
            };

            let exported = TomlExporter::export_protocol(&protocol)?;
            println!("{}", exported);
        }
        Commands::Service { name: _, region, endpoint, permissions } => {
            let service_config = ServiceConfig {
                default_region: region,
                endpoint,
                default_permissions: permissions,
                parameters: HashMap::new(),
            };

            let exported = TomlExporter::export_service_config(&service_config)?;
            println!("{}", exported);
        }
        Commands::Validate { file } => {
            let content = context_error!(
                fs::read_to_string(&file),
                format!("Failed to read TOML file '{}'", file)
            )?;

            let is_valid = TomlExporter::validate_export(&content)?;
            if is_valid {
                println!("✅ TOML is valid and can be parsed back");
                std::process::exit(0);
            } else {
                println!("❌ TOML is invalid or cannot be parsed back");
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
    println!("DFS Config TOML Export Examples");
    println!("================================");
    println!();
    println!("Export a config file to TOML:");
    println!("  dfs-config-toml-export config examples/example1.toml");
    println!();
    println!("Export a config file in compact format:");
    println!("  dfs-config-toml-export config examples/example1.toml --compact");
    println!();
    println!("Export a protocol configuration:");
    println!("  dfs-config-toml-export protocol my-s3 --service s3 --bucket my-bucket --region us-east-1");
    println!();
    println!("Export a service configuration:");
    println!("  dfs-config-toml-export service s3 --region us-west-2 --endpoint https://s3.us-west-2.amazonaws.com");
    println!();
    println!("Validate TOML can be parsed back:");
    println!("  dfs-config-toml-export validate exported_config.toml");
    println!();
    println!("Show this help:");
    println!("  dfs-config-toml-export --help");
}