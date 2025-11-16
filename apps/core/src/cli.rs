//! Main CLI implementation for durable-filestor

use durable_filestor_config_toml_parser::ConfigParser;
use durable_filestor_config_uri_parser::ConfigUriParser;
use durable_filestor_shared::{Cli, Commands, Config, Result};
use clap::Parser;

/// Main application structure
pub struct App {
    config_parser: ConfigParser,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        Self {
            config_parser: ConfigParser::new(),
        }
    }

    /// Run the application with the given command line arguments
    pub async fn run(&self, cli: Cli) -> Result<()> {
        // Load configuration
        let config = self.config_parser.load_config(&cli)?;

        // Execute the requested command
        match &cli.command {
            Commands::Config { action } => {
                durable_filestor_shared::handle_config_command_shared(action, &config)
            }
            Commands::List { uri, service, bucket, region, path } => {
                durable_filestor_shared::handle_list_command_shared(uri, service, bucket, region, path)
            }
            Commands::Upload { source, target, service, bucket, region, path } => {
                durable_filestor_shared::handle_upload_command_shared(source, target, service, bucket, region, path)
            }
            Commands::Download { source, target, service, bucket, region, path } => {
                durable_filestor_shared::handle_download_command_shared(source, target, service, bucket, region, path)
            }
            Commands::Head { uri, service, bucket, region, path } => {
                durable_filestor_shared::handle_head_command_shared(uri, service, bucket, region, path)
            }
        }
    }


}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Main entry point for the CLI application
pub async fn run() -> Result<()> {
    let cli = <Cli as clap::Parser>::parse();
    let app = App::new();

    app.run(cli).await
}