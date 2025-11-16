//! URI parsing implementation
//!
//! This module provides a thin wrapper around the shared URI parsing functionality
//! for backward compatibility with existing code.

use durable_filestor_shared::{ParsedUri, Result, UriParser as SharedUriParser};

/// URI parser for custom protocols
pub struct UriParser;

impl UriParser {
    /// Parse a URI string into its components
    ///
    /// # Examples
    /// ```
    /// use durable_filestor_uri_parser::UriParser;
    ///
    /// let uri = "my-s3://bucket/file.txt?version=1";
    /// let parsed = UriParser::parse(uri).unwrap();
    /// assert_eq!(parsed.protocol, "my-s3");
    /// assert_eq!(parsed.path, "bucket/file.txt");
    /// ```
    pub fn parse(uri_str: &str) -> Result<ParsedUri> {
        SharedUriParser::parse(uri_str)
    }

    /// Validate that a URI string is properly formatted
    pub fn validate(uri_str: &str) -> Result<()> {
        SharedUriParser::validate(uri_str)
    }

    /// Extract just the protocol name from a URI
    pub fn extract_protocol(uri_str: &str) -> Result<String> {
        SharedUriParser::extract_protocol(uri_str)
    }

    /// Extract just the path from a URI
    pub fn extract_path(uri_str: &str) -> Result<String> {
        SharedUriParser::extract_path(uri_str)
    }
}

