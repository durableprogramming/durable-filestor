//! Configuration parser for durable-filestor
//!
//! This module provides functionality to parse configuration from multiple sources:
//! 1. Command line arguments (highest priority)
//! 2. Environment variables
//! 3. Configuration file (~/.config/durable-filestor/filestor.toml)
//! 4. Custom configuration file specified via --config flag

pub mod parser;
pub mod cli;

pub use parser::*;
pub use cli::*;