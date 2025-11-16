//! Local filesystem service for durable-filestor
//!
//! This crate provides a storage service implementation for local filesystem operations,
//! supporting all standard durable-filestor operations: list, upload, download, and head.

pub mod cli;
pub mod operations;

pub use cli::*;
pub use operations::*;