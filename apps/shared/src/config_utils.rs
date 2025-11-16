//! Shared configuration utilities for durable-filestor
//!
//! This module provides common functionality for configuration parsing,
//! file handling, and environment variable processing that can be reused
//! across different configuration parsers.

use crate::{Config, DurableError, Protocol, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Configuration utilities for handling common config operations
pub struct ConfigUtils;

impl ConfigUtils {
    /// Determine the default configuration file path
    ///
    /// Returns the path ~/.config/durable-filestor/filestor.toml
    pub fn get_default_config_path() -> Result<PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| DurableError::Config {
                message: "Could not determine config directory".to_string(),
            })?;

        path.push("durable-filestor");
        path.push("filestor.toml");

        Ok(path)
    }

    /// Determine the configuration file path with precedence:
    /// 1. Custom path from CLI (highest priority)
    /// 2. Parser-specific config file path
    /// 3. Default config path (lowest priority)
    pub fn determine_config_path(cli_config: Option<&PathBuf>, parser_config: Option<&PathBuf>) -> Result<PathBuf> {
        // Check if custom config path is specified via CLI
        if let Some(path) = cli_config {
            return Ok(path.to_path_buf());
        }

        // Check if custom config path is set in the parser
        if let Some(path) = parser_config {
            return Ok(path.to_path_buf());
        }

        // Use default config path
        Self::get_default_config_path()
    }

    /// Load configuration from TOML file
    pub fn load_from_toml_file(config_path: &PathBuf) -> Result<Config> {
        if !config_path.exists() {
            return Err(DurableError::Config {
                message: format!("Configuration file does not exist: {}", config_path.display()),
            });
        }

        let content = fs::read_to_string(config_path)
            .map_err(|e| DurableError::Config {
                message: format!("Failed to read config file {}: {}", config_path.display(), e),
            })?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| DurableError::Config {
                message: format!("Failed to parse config file {}: {}", config_path.display(), e),
            })?;

        Ok(config)
    }

    /// Save configuration to TOML file
    pub fn save_to_toml_file(config: &Config, config_path: &PathBuf) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| DurableError::Config {
                    message: format!("Failed to create config directory: {}", e),
                })?;
        }

        let content = toml::to_string_pretty(config)
            .map_err(|e| DurableError::Config {
                message: format!("Failed to serialize config: {}", e),
            })?;

        fs::write(config_path, content)
            .map_err(|e| DurableError::Config {
                message: format!("Failed to write config file {}: {}", config_path.display(), e),
            })?;

        Ok(())
    }

    /// Load configuration from environment variables
    pub fn load_from_env() -> Result<Config> {
        let mut config = Config::default();

        // Parse DURABLE_FILESTOR_PROTOCOLS environment variable
        if let Ok(protocols_str) = env::var("DURABLE_FILESTOR_PROTOCOLS") {
            if let Ok(protocols) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&protocols_str) {
                // Convert JSON protocols to our Protocol struct
                for (name, value) in protocols {
                    if let Some(obj) = value.as_object() {
                        let service = obj.get("service").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                        let bucket = obj.get("bucket").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let region = obj.get("region").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let path = obj.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let protocol = Protocol {
                            service,
                            bucket,
                            region,
                            path,
                            key_translation: Default::default(),
                            parameters: HashMap::new(),
                        };
                        config.protocols.insert(name, protocol);
                    }
                }
            }
        }

        // Parse service-specific environment variables
        Self::load_service_env_vars(&mut config)?;

        Ok(config)
    }

    /// Load service-specific environment variables
    fn load_service_env_vars(config: &mut Config) -> Result<()> {
        for (key, value) in env::vars() {
            if key.starts_with("DURABLE_FILESTOR_") && key.ends_with("_REGION") {
                let service_name = key
                    .strip_prefix("DURABLE_FILESTOR_")
                    .and_then(|s| s.strip_suffix("_REGION"))
                    .unwrap_or("")
                    .to_lowercase();

                if !service_name.is_empty() {
                    config.services
                        .entry(service_name)
                        .or_insert_with(Default::default)
                        .default_region = Some(value.clone());
                }
            }

            if key.starts_with("DURABLE_FILESTOR_") && key.ends_with("_ENDPOINT") {
                let service_name = key
                    .strip_prefix("DURABLE_FILESTOR_")
                    .and_then(|s| s.strip_suffix("_ENDPOINT"))
                    .unwrap_or("")
                    .to_lowercase();

                if !service_name.is_empty() {
                    config.services
                        .entry(service_name)
                        .or_insert_with(Default::default)
                        .endpoint = Some(value);
                }
            }
        }

        Ok(())
    }

    /// Merge two configurations, with values from `other` taking precedence
    pub fn merge_configs(mut base: Config, other: Config) -> Config {
        // Merge protocols
        for (name, protocol) in other.protocols {
            base.protocols.insert(name, protocol);
        }

        // Merge services
        for (name, service) in other.services {
            base.services.insert(name, service);
        }

        base
    }

    /// Load configuration with precedence:
    /// 1. File config (lowest priority)
    /// 2. Environment config (medium priority)
    /// 3. CLI overrides (highest priority)
    pub fn load_config_with_precedence(
        file_config: Option<Config>,
        env_config: Config,
        cli_config: Option<Config>,
    ) -> Config {
        let mut config = Config::default();

        // Load from configuration file (lowest priority)
        if let Some(file_cfg) = file_config {
            config = Self::merge_configs(config, file_cfg);
        }

        // Load from environment variables (medium priority)
        config = Self::merge_configs(config, env_config);

        // Apply CLI overrides (highest priority)
        if let Some(cli_cfg) = cli_config {
            config = Self::merge_configs(config, cli_cfg);
        }

        config
    }
}

/// Convert TOML value to JSON value for output formatting
pub fn toml_to_json(toml_value: toml::Value) -> serde_json::Value {
    match toml_value {
        toml::Value::String(s) => serde_json::Value::String(s),
        toml::Value::Integer(i) => serde_json::Value::Number(serde_json::Number::from(i)),
        toml::Value::Float(f) => {
            if let Some(num) = serde_json::Number::from_f64(f) {
                serde_json::Value::Number(num)
            } else {
                serde_json::Value::Null
            }
        }
        toml::Value::Boolean(b) => serde_json::Value::Bool(b),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        toml::Value::Array(arr) => {
            let json_arr: Vec<serde_json::Value> = arr.into_iter().map(toml_to_json).collect();
            serde_json::Value::Array(json_arr)
        }
        toml::Value::Table(table) => {
            let mut json_obj = serde_json::Map::new();
            for (key, value) in table {
                json_obj.insert(key, toml_to_json(value));
            }
            serde_json::Value::Object(json_obj)
        }
    }
}

/// Trait for configuration parsers that provides common functionality
pub trait ConfigParserTrait {
    /// Load configuration from file if it exists
    fn load_from_file(&self, config_path: &PathBuf) -> Result<Option<Config>> {
        if !config_path.exists() {
            return Ok(None);
        }

        let config = ConfigUtils::load_from_toml_file(config_path)?;
        Ok(Some(config))
    }

    /// Save configuration to file
    fn save_config(&self, config: &Config, config_path: &PathBuf) -> Result<()> {
        ConfigUtils::save_to_toml_file(config, config_path)
    }
}