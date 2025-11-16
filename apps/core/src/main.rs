//! Main executable for durable-filestor core

use durable_filestor_core::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await?;
    Ok(())
}