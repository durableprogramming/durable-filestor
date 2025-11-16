//! Main binary for durable-filestor-local

use clap::Parser;
use durable_filestor_local::LocalCli;
use durable_filestor_shared::Result;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = LocalCli::parse();
    durable_filestor_local::cli::execute(cli).await
}