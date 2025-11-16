//! Main binary for durable-filestor-s3

use clap::Parser;
use durable_filestor_s3::S3Cli;
use durable_filestor_shared::Result;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = S3Cli::parse();
    durable_filestor_s3::cli::execute(cli).await
}