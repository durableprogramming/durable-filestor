//! Main binary for durable-filestor-memory

use clap::Parser;
use durable_filestor_memory::MemoryCli;
use durable_filestor_shared::Result;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = MemoryCli::parse();
    durable_filestor_memory::cli::execute(cli).await
}