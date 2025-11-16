//! CLI interface for memory storage service

use clap::{Parser, Subcommand};
use durable_filestor_shared::{Result, ServiceConfig};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

use crate::client::MemoryClient;
use crate::server::{MemoryServer, ServerConfig};

/// CLI structure for memory service
#[derive(Parser, Debug)]
#[command(name = "durable-filestor-memory")]
#[command(about = "In-memory storage operations for durable-filestor")]
pub struct MemoryCli {
    /// Socket path for server communication
    #[arg(long, default_value = "/tmp/durable-filestor-memory.sock")]
    pub socket_path: PathBuf,

    /// TTL for keys in seconds (0 = no TTL)
    #[arg(long)]
    pub ttl: Option<u64>,

    /// Server timeout in seconds (default: 3600 = 1 hour)
    #[arg(long, default_value = "3600")]
    pub timeout: u64,

    /// Run as server
    #[arg(long)]
    pub server: bool,

    /// Run in daemon mode (default: true)
    #[arg(long)]
    pub daemon: bool,

    /// Don't run in daemon mode
    #[arg(long)]
    pub no_daemon: bool,

    /// Subcommands (when not running as server)
    #[command(subcommand)]
    pub command: Option<MemoryCommands>,
}

#[derive(Subcommand, Debug)]
pub enum MemoryCommands {
    /// List keys in memory storage
    List {
        /// Prefix to filter keys
        prefix: Option<String>,
    },

    /// Upload data to memory storage
    Upload {
        /// Key to store data under
        key: String,

        /// Data to store (as string)
        data: String,
    },

    /// Download data from memory storage
    Download {
        /// Key to retrieve
        key: String,
    },

    /// Get metadata for a key
    Head {
        /// Key to get metadata for
        key: String,
    },

    /// Delete a key from memory storage
    Delete {
        /// Key to delete
        key: String,
    },

    /// Copy data within memory storage
    Copy {
        /// Source key
        source: String,

        /// Target key
        target: String,
    },

    /// Get memory storage statistics
    Stats,

    /// Trigger cleanup of expired keys
    Cleanup,
}

/// Execute the CLI command
pub async fn execute(cli: MemoryCli) -> Result<()> {
    let socket_path_str = cli.socket_path.to_string_lossy().to_string();

    // If running as server
    if cli.server {
        let mut server_config = ServerConfig::default();
        let socket_path_str = cli.socket_path.to_string_lossy().to_string();
        server_config.socket_path = cli.socket_path;
        server_config.timeout_duration = Duration::from_secs(cli.timeout);
        server_config.daemon_mode = cli.daemon && !cli.no_daemon;

        // Set up service config with TTL
        let mut service_config = ServiceConfig::default();
        if let Some(ttl) = cli.ttl {
            service_config.parameters.insert("ttl".to_string(), ttl.to_string());
        }
        service_config.parameters.insert("socket_path".to_string(), socket_path_str.clone());

        server_config.service_config = service_config;

        let mut server = MemoryServer::new(server_config);
        server.start().await?;
        return Ok(());
    }

    // Running as client
    let mut service_config = ServiceConfig::default();
    if let Some(ttl) = cli.ttl {
        service_config.parameters.insert("ttl".to_string(), ttl.to_string());
    }
    service_config.parameters.insert("socket_path".to_string(), socket_path_str);

    let client = MemoryClient::new(service_config);

    match cli.command {
        Some(MemoryCommands::List { prefix }) => {
            let prefix = prefix.as_deref().unwrap_or("");
            let keys = client.list(prefix).await?;
            let stdout = io::stdout();
            let mut handle = stdout.lock();

            writeln!(handle, "Found {} keys:", keys.len())?;
            for key in keys {
                writeln!(
                    handle,
                    "  {} ({} bytes, created: {}, accessed: {})",
                    key.key, key.size, key.created, key.last_accessed
                )?;
            }
        }

        Some(MemoryCommands::Upload { key, data }) => {
            let data_bytes = data.into_bytes();
            client.upload(&key, data_bytes).await?;
            println!("Successfully uploaded data to key '{}'", key);
        }

        Some(MemoryCommands::Download { key }) => {
            let data = client.download(&key).await?;
            let data_str = String::from_utf8_lossy(&data);
            println!("Data for key '{}': {}", key, data_str);
        }

        Some(MemoryCommands::Head { key }) => {
            let metadata = client.head(&key).await?;
            println!("Key: {}", metadata.key);
            println!("Size: {} bytes", metadata.size);
            println!("Created: {}", metadata.created);
            println!("Last Accessed: {}", metadata.last_accessed);
        }

        Some(MemoryCommands::Delete { key }) => {
            client.delete(&key).await?;
            println!("Successfully deleted key '{}'", key);
        }

        Some(MemoryCommands::Copy { source, target }) => {
            client.copy(&source, &target).await?;
            println!("Successfully copied '{}' to '{}'", source, target);
        }

        Some(MemoryCommands::Stats) => {
            let stats = client.stats().await?;
            println!("Memory Storage Statistics:");
            println!("  Total Keys: {}", stats.total_keys);
            println!("  Total Size: {} bytes", stats.total_size);
            println!("  Expired Keys: {}", stats.expired_keys);
        }

        Some(MemoryCommands::Cleanup) => {
            let removed = client.cleanup().await?;
            println!("Cleaned up {} expired keys", removed);
        }

        None => {
            // No subcommand provided, show help
            println!("No subcommand provided. Use --help for usage information.");
        }
    }

    Ok(())
}