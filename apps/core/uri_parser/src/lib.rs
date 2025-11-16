//! URI parser for durable-filestor
//!
//! This module provides functionality to parse custom protocol URIs
//! and resolve them to their corresponding service configurations.
//!
//! This crate now serves as a thin wrapper around the shared URI parsing
//! functionality to maintain backward compatibility.

pub mod parser;

pub use parser::*;

// Re-export shared types and resolver for convenience
pub use durable_filestor_shared::{ParsedUri, ResolvedUri, UriResolver};