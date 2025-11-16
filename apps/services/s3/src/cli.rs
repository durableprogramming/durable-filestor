//! CLI interface for S3 storage service

use clap::{Parser, Subcommand};
use durable_filestor_shared::{Result, ServiceConfig};
use std::io::{self, Write};

use crate::operations::S3Service;

/// CLI structure for S3 service
#[derive(Parser, Debug)]
#[command(name = "durable-filestor-s3")]
#[command(about = "S3 storage operations for durable-filestor")]
pub struct S3Cli {
    /// S3 bucket name
    #[arg(short, long)]
    pub bucket: String,

    /// AWS region
    #[arg(short, long, default_value = "us-east-1")]
    pub region: String,

    /// Custom S3 endpoint URL
    #[arg(short = 'e', long)]
    pub endpoint: Option<String>,

    /// AWS access key ID (can also be set via AWS_ACCESS_KEY_ID env var)
    #[arg(long, env = "AWS_ACCESS_KEY_ID")]
    pub access_key_id: Option<String>,

    /// AWS secret access key (can also be set via AWS_SECRET_ACCESS_KEY env var)
    #[arg(long, env = "AWS_SECRET_ACCESS_KEY")]
    pub secret_access_key: Option<String>,

    /// Object ACL (private, public-read, public-read-write, etc.)
    #[arg(long)]
    pub acl: Option<String>,

    /// Content type for uploaded objects
    #[arg(long)]
    pub content_type: Option<String>,

    /// Subcommands
    #[command(subcommand)]
    pub command: S3Commands,
}

#[derive(Subcommand, Debug)]
pub enum S3Commands {
    /// List objects in the bucket
    List {
        /// Prefix to filter objects
        prefix: Option<String>,
    },

    /// Upload a file to S3
    Upload {
        /// Source file path
        source: String,

        /// Target object key
        target: String,
    },

    /// Download a file from S3
    Download {
        /// Source object key
        source: String,

        /// Target file path
        target: String,
    },

    /// Get object metadata
    Head {
        /// Object key to check
        key: String,
    },

    /// Delete an object from S3
    Delete {
        /// Object key to delete
        key: String,
    },

    /// Copy an object within S3
    Copy {
        /// Source object key
        source: String,

        /// Target object key
        target: String,
    },
}

/// Execute the CLI command
pub async fn execute(cli: S3Cli) -> Result<()> {
    // Build service config from CLI args
    let mut config = ServiceConfig::default();

    config.default_region = Some(cli.region);

    if let Some(endpoint) = &cli.endpoint {
        config.endpoint = Some(endpoint.clone());
    }

    if let Some(acl) = &cli.acl {
        config.parameters.insert("acl".to_string(), acl.clone());
    }

    if let Some(content_type) = &cli.content_type {
        config.parameters.insert("content-type".to_string(), content_type.clone());
    }

    // Set AWS credentials as environment variables if provided
    if let Some(access_key_id) = &cli.access_key_id {
        std::env::set_var("AWS_ACCESS_KEY_ID", access_key_id);
    }

    if let Some(secret_access_key) = &cli.secret_access_key {
        std::env::set_var("AWS_SECRET_ACCESS_KEY", secret_access_key);
    }

    let bucket_name = cli.bucket.clone();
    let service = S3Service::new(cli.bucket, config).await?;

    match cli.command {
        S3Commands::List { prefix } => {
            let prefix = prefix.as_deref().unwrap_or("");
            let objects = service.list(prefix).await?;
            let stdout = io::stdout();
            let mut handle = stdout.lock();

            writeln!(handle, "Found {} objects:", objects.len())?;
            for obj in objects {
                writeln!(
                    handle,
                    "  {} ({} bytes, modified: {:?})",
                    obj.key, obj.size, obj.last_modified
                )?;
            }
        }

        S3Commands::Upload { source, target } => {
            service.upload(&source, &target).await?;
            println!("Successfully uploaded {} to s3://{}/{}", source, bucket_name, target);
        }

        S3Commands::Download { source, target } => {
            service.download(&source, &target).await?;
            println!("Successfully downloaded s3://{}/{} to {}", bucket_name, source, target);
        }

        S3Commands::Head { key } => {
            let metadata = service.head(&key).await?;
            println!("Object: s3://{}/{}", bucket_name, metadata.key);
            println!("Size: {} bytes", metadata.size);
            println!("Last Modified: {:?}", metadata.last_modified);
            if let Some(etag) = metadata.etag {
                println!("ETag: {}", etag);
            }
        }

        S3Commands::Delete { key } => {
            service.delete(&key).await?;
            println!("Successfully deleted s3://{}/{}", bucket_name, key);
        }

        S3Commands::Copy { source, target } => {
            service.copy(&source, &target).await?;
            println!("Successfully copied s3://{}/{} to s3://{}/{}", bucket_name, source, bucket_name, target);
        }
    }

    Ok(())
}