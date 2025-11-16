//! S3 storage service for durable-filestor
//!
//! This crate provides a storage service implementation for Amazon S3 and S3-compatible
//! services, supporting all standard durable-filestor operations: list, upload, download, and head.

pub mod cli;
pub mod operations;

pub use cli::*;
pub use operations::*;