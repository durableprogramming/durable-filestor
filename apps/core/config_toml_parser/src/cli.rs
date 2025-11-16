//! CLI utilities and command handlers for config TOML parser
//!
//! This module provides CLI-specific functionality that builds on the shared
//! CLI utilities while adding config parser-specific features.

use durable_filestor_shared::{ConfigAction, Config, Result, cli_utils::*};

/// Enhanced CLI utilities for config TOML parser
pub struct ConfigCliUtils;

/// Implementation of CLI utilities for config parsing
impl ConfigCliUtils {
    /// Handle configuration commands with enhanced error handling
    pub fn handle_config_command(config: &Config, action: &ConfigAction) -> Result<()> {
        match action {
            ConfigAction::AddProtocol { name, service, bucket, region, path } => {
                println!("Would add protocol '{}' with service '{}'", name, service);
                if let Some(bucket) = bucket {
                    println!("  Bucket: {}", bucket);
                }
                if let Some(region) = region {
                    println!("  Region: {}", region);
                }
                if let Some(path) = path {
                    println!("  Path: {}", path);
                }
                println!("Note: This is a demonstration. Use the main CLI for actual configuration management.");
            }
            ConfigAction::RemoveProtocol { name } => {
                println!("Would remove protocol '{}'", name);
                println!("Note: This is a demonstration. Use the main CLI for actual configuration management.");
            }
            ConfigAction::ListProtocols => {
                display_protocols(config)?;
            }
            ConfigAction::Show => {
                display_config_info(config)?;
            }
        }
        Ok(())
    }

    /// Display configuration parsing information
    pub fn display_config_parsing_info(config_path: &std::path::Path, config: &Config) -> Result<()> {
        println!("Configuration loaded from: {}", config_path.display());
        println!("Protocols found: {}", config.protocol_count());
        println!("Services found: {}", config.service_count());

        if !config.protocols.is_empty() {
            println!("\nAvailable protocols:");
            for (name, protocol) in &config.protocols {
                println!("  • {} (service: {})", name, protocol.service);
            }
        }

        Ok(())
    }

    /// Validate configuration and display results
    pub fn validate_and_display_config(config: &Config) -> Result<()> {
        match config.validate_basic() {
            Ok(_) => {
                println!("✅ Configuration is valid");
                println!("   Protocols: {}", config.protocol_count());
                println!("   Services: {}", config.service_count());
            }
            Err(e) => {
                println!("❌ Configuration validation failed: {}", e);
                return Err(e);
            }
        }
        Ok(())
    }
}

// Re-export shared CLI types for convenience
pub use durable_filestor_shared::{Cli, Commands};