//! Configuration parser implementation

use durable_filestor_shared::{Cli, Config, ConfigParserTrait, ConfigUtils, Result};
use std::path::PathBuf;

/// Configuration parser that handles multiple sources with precedence
pub struct ConfigParser {
    config_file_path: Option<PathBuf>,
}

impl ConfigParser {
    /// Create a new configuration parser
    pub fn new() -> Self {
        Self {
            config_file_path: None,
        }
    }

    /// Set custom configuration file path
    pub fn with_config_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config_file_path = Some(path.into());
        self
    }

    /// Load configuration from all sources with proper precedence
    pub fn load_config(&self, cli: &Cli) -> Result<Config> {
        // Determine config file path
        let config_path = ConfigUtils::determine_config_path(cli.config.as_ref(), self.config_file_path.as_ref())?;

        // Load configurations from different sources
        let file_config = <Self as ConfigParserTrait>::load_from_file(self, &config_path)?;
        let env_config = ConfigUtils::load_from_env()?;

        // Merge configurations with proper precedence
        let config = ConfigUtils::load_config_with_precedence(file_config, env_config, None);

        Ok(config)
    }

    /// Load configuration from file
    pub fn load_from_file(&self, cli: &Cli) -> Result<Option<Config>> {
        let config_path = ConfigUtils::determine_config_path(cli.config.as_ref(), self.config_file_path.as_ref())?;
        <Self as ConfigParserTrait>::load_from_file(self, &config_path)
    }

    /// Load configuration from environment variables
    pub fn load_from_env(&self) -> Result<Config> {
        ConfigUtils::load_from_env()
    }

    /// Save configuration to file
    pub fn save_config(&self, config: &Config, cli: &Cli) -> Result<()> {
        let config_path = ConfigUtils::determine_config_path(cli.config.as_ref(), self.config_file_path.as_ref())?;
        <Self as ConfigParserTrait>::save_config(self, config, &config_path)
    }
}

impl ConfigParserTrait for ConfigParser {}

impl Default for ConfigParser {
    fn default() -> Self {
        Self::new()
    }
}