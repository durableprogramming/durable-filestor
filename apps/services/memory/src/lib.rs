//! In-memory storage service for durable-filestor
//!
//! This crate provides an in-memory storage service implementation with a client-server
//! architecture. The service runs as a daemon and stores data in memory with optional TTL.
//! Supports all standard durable-filestor operations: list, upload, download, and head.

pub mod cli;
pub mod operations;
pub mod server;
pub mod client;

pub use cli::*;
pub use operations::*;
pub use server::*;
pub use client::*;