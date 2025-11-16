//! URI-based configuration parser for durable-filestor
//!
//! This module provides functionality to parse configuration from URI strings:
//! 1. TOML protocol: `toml://base64-encoded-toml-config`
//! 2. DFS protocol: `dfs://s3?access_key_id=xxx&secret_access_key=yyy&bucket_name=zzz`

pub mod parser;
pub mod protocols;

pub use parser::*;
pub use protocols::*;