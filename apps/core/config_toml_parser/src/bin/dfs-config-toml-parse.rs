use clap::Parser;
use durable_filestor_shared::{config_utils::{toml_to_json, ConfigUtils}, DurableError, Result};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dfs-config-toml-parse")]
#[command(about = "Parse TOML configuration files and output them as JSON")]
struct Args {
    /// Input TOML file path
    #[arg(help = "Path to the TOML configuration file to parse")]
    input: PathBuf,
    
    /// Pretty print JSON output
    #[arg(short, long, help = "Format JSON output with indentation")]
    pretty: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Parsing TOML file: {}", args.input.display());
    println!("Output format: {}", if args.pretty { "pretty JSON" } else { "compact JSON" });
    println!();

    // Read and parse the TOML file content
    let content = std::fs::read_to_string(&args.input)
        .map_err(|e| DurableError::Io { source: e })?;

    let toml_value: toml::Value = toml::from_str(&content)
        .map_err(|e| DurableError::Toml { source: e })?;

    let json_value = toml_to_json(toml_value);

    let output = if args.pretty {
        serde_json::to_string_pretty(&json_value)
            .map_err(|e| DurableError::Json { source: e })?
    } else {
        serde_json::to_string(&json_value)
            .map_err(|e| DurableError::Json { source: e })?
    };

    println!("✅ Successfully parsed TOML file");
    println!("{}", output);

    Ok(())
}

