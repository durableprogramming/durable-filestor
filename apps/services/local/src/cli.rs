//! CLI interface for local filesystem service

use clap::{Parser, Subcommand};
use durable_filestor_shared::{Result, ServiceConfig};
use std::io::{self, Write};

use crate::operations::LocalService;

/// CLI structure for local service
#[derive(Parser, Debug)]
#[command(name = "durable-filestor-local")]
#[command(about = "Local filesystem operations for durable-filestor")]
pub struct LocalCli {
    /// Base path for operations
    #[arg(short, long)]
    pub path: Option<String>,

    /// Default file permissions (octal)
    #[arg(short = 'P', long)]
    pub permissions: Option<String>,

    /// Subcommands
    #[command(subcommand)]
    pub command: LocalCommands,
}

#[derive(Subcommand, Debug)]
pub enum LocalCommands {
    /// List files and directories
    List {
        /// Path to list (relative to base path)
        path: String,
    },

    /// Upload a file
    Upload {
        /// Source file path
        source: String,

        /// Target path (relative to base path)
        target: String,
    },

    /// Download a file
    Download {
        /// Source path (relative to base path)
        source: String,

        /// Target file path
        target: String,
    },

    /// Get file metadata
    Head {
        /// Path to check (relative to base path)
        path: String,
    },
}

/// Execute the CLI command
pub async fn execute(cli: LocalCli) -> Result<()> {
    // Build service config from CLI args
    let mut config = ServiceConfig::default();

    if let Some(path) = &cli.path {
        config.endpoint = Some(path.clone());
    }

    if let Some(perms) = &cli.permissions {
        config.default_permissions = Some(perms.clone());
    }

    let service = LocalService::new(config);

    match cli.command {
        LocalCommands::List { path } => {
            let results = service.list(&path).await?;
            let stdout = io::stdout();
            let mut handle = stdout.lock();

            for item in results {
                let item_type = if item.is_dir { "DIR" } else { "FILE" };
                writeln!(handle, "{}\t{}\t{}", item_type, item.size, item.path)?;
            }
        }

        LocalCommands::Upload { source, target } => {
            service.upload(&source, &target).await?;
            println!("Successfully uploaded {} to {}", source, target);
        }

        LocalCommands::Download { source, target } => {
            service.download(&source, &target).await?;
            println!("Successfully downloaded {} to {}", source, target);
        }

        LocalCommands::Head { path } => {
            let metadata = service.head(&path).await?;
            let item_type = if metadata.is_dir { "directory" } else { "file" };
            println!("Path: {}", metadata.path);
            println!("Type: {}", item_type);
            println!("Size: {} bytes", metadata.size);
            println!("Modified: {:?}", metadata.modified);
        }
    }

    Ok(())
}