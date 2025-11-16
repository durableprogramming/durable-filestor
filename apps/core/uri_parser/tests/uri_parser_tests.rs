//! Comprehensive tests for URI parsing and resolution

use durable_filestor_uri_parser::{UriParser, UriResolver};
use durable_filestor_shared::{create_test_config, get_standard_test_uris, get_validation_test_uris, get_protocol_extraction_test_cases, get_path_extraction_test_cases};

#[test]
fn test_uri_parser_basic_functionality() {
    // Test standard S3 URI
    let parsed = UriParser::parse("s3://my-bucket/path/to/file.txt").unwrap();
    assert_eq!(parsed.protocol, "s3");
    assert_eq!(parsed.path, "my-bucket/path/to/file.txt");
    assert!(parsed.query_params.is_empty());

    // Test custom protocol URI
    let parsed = UriParser::parse("my-custom://data/file.json").unwrap();
    assert_eq!(parsed.protocol, "my-custom");
    assert_eq!(parsed.path, "data/file.json");

    // Test URI with query parameters
    let parsed = UriParser::parse("protocol://path/file.txt?version=1&encoding=utf8").unwrap();
    assert_eq!(parsed.protocol, "protocol");
    assert_eq!(parsed.path, "path/file.txt");
    assert_eq!(parsed.query_params.get("version"), Some(&"1".to_string()));
    assert_eq!(parsed.query_params.get("encoding"), Some(&"utf8".to_string()));
}

#[test]
fn test_uri_parser_edge_cases() {
    // Test empty URI
    assert!(UriParser::parse("").is_err());

    // Test URI without protocol separator
    assert!(UriParser::parse("no-protocol").is_err());

    // Test empty protocol
    assert!(UriParser::parse("://path").is_err());

    // Test protocol with invalid characters
    assert!(UriParser::parse("proto/col://path").is_err());
    assert!(UriParser::parse("proto?col://path").is_err());
    assert!(UriParser::parse("proto#col://path").is_err());

    // Test URL-encoded paths and parameters
    let parsed = UriParser::parse("protocol://folder/file%20name.txt?path=%2Fhome%2Fuser").unwrap();
    assert_eq!(parsed.path, "folder/file%20name.txt");
    assert_eq!(parsed.query_params.get("path"), Some(&"/home/user".to_string()));
}

#[test]
fn test_uri_parser_validation() {
    // Test all valid URIs from shared test utilities
    for uri in get_validation_test_uris() {
        assert!(UriParser::validate(uri).is_ok(), "URI '{}' should be valid", uri);
    }

    // Test invalid URIs
    assert!(UriParser::validate("").is_err());
    assert!(UriParser::validate("no-protocol").is_err());
    assert!(UriParser::validate("://empty-protocol").is_err());
}

#[test]
fn test_uri_parser_extract_methods() {
    // Test protocol extraction using shared test cases
    for (uri, expected_protocol) in get_protocol_extraction_test_cases() {
        assert_eq!(UriParser::extract_protocol(uri).unwrap(), expected_protocol,
                   "Protocol extraction failed for URI: {}", uri);
    }

    // Test path extraction using shared test cases
    for (uri, expected_path) in get_path_extraction_test_cases() {
        assert_eq!(UriParser::extract_path(uri).unwrap(), expected_path,
                   "Path extraction failed for URI: {}", uri);
    }

    // Test with invalid URI
    assert!(UriParser::extract_protocol("invalid").is_err());
    assert!(UriParser::extract_path("invalid").is_err());
}

#[test]
fn test_parsed_uri_methods() {
    let parsed = UriParser::parse("test://bucket/file.txt").unwrap();

    assert_eq!(parsed.full_path(), "test://bucket/file.txt");
    assert_eq!(parsed.original, "test://bucket/file.txt");
}

#[test]
fn test_uri_resolver_basic_resolution() {
    let config = create_test_config();
    let resolver = UriResolver::new(&config);

    // Test S3 protocol resolution
    let resolved = resolver.resolve_uri("test-s3://documents/MyFile.txt").unwrap();
    assert_eq!(resolved.service, "s3");
    assert_eq!(resolved.bucket(), Some("test-bucket"));
    assert_eq!(resolved.region(), Some("us-east-1"));
    // Should be downcased and prefixed due to key translation
    assert_eq!(resolved.translated_key, "files/documents/myfile.txt");

    // Test local protocol resolution
    let resolved = resolver.resolve_uri("test-local://files/data.txt").unwrap();
    assert_eq!(resolved.service, "local");
    assert_eq!(resolved.path(), Some("/tmp/dfs-test"));
    assert_eq!(resolved.translated_key, "files/data.txt");
}

#[test]
fn test_uri_resolver_partitioning() {
    let config = create_test_config();
    let resolver = UriResolver::new(&config);

    let resolved = resolver.resolve_uri("partitioned://abcdefg").unwrap();
    assert_eq!(resolved.service, "s3");
    assert_eq!(resolved.bucket(), Some("partitioned-bucket"));
    // Should be partitioned: ab/cd/efg
    assert_eq!(resolved.translated_key, "ab/cd/efg");
}

#[test]
fn test_uri_resolver_unknown_protocol() {
    let config = create_test_config();
    let resolver = UriResolver::new(&config);

    let result = resolver.resolve_uri("unknown://file.txt");
    assert!(result.is_err());
    // Should be ProtocolNotFound error
    if let Err(durable_filestor_shared::DurableError::ProtocolNotFound { protocol }) = result {
        assert_eq!(protocol, "unknown");
    } else {
        panic!("Expected ProtocolNotFound error");
    }
}

#[test]
fn test_uri_resolver_protocol_queries() {
    let config = create_test_config();
    let resolver = UriResolver::new(&config);

    // Test protocol existence checks
    assert!(resolver.is_protocol_configured("test-s3"));
    assert!(resolver.is_protocol_configured("test-local"));
    assert!(resolver.is_protocol_configured("partitioned"));
    assert!(!resolver.is_protocol_configured("unknown"));

    // Test getting configured protocols
    let protocols = resolver.configured_protocols();
    assert!(protocols.contains(&"test-s3".to_string()));
    assert!(protocols.contains(&"test-local".to_string()));
    assert!(protocols.contains(&"partitioned".to_string()));

    // Test getting protocol config
    let s3_config = resolver.get_protocol_config("test-s3").unwrap();
    assert_eq!(s3_config.service, "s3");
    assert_eq!(s3_config.bucket, Some("test-bucket".to_string()));

    assert!(resolver.get_protocol_config("unknown").is_none());
}

#[test]
fn test_uri_resolver_with_parsed_uri() {
    let config = create_test_config();
    let resolver = UriResolver::new(&config);

    let parsed = UriParser::parse("test-s3://UPPERCASE.TXT").unwrap();
    let resolved = resolver.resolve(parsed).unwrap();

    // Should apply downcase transformation
    assert_eq!(resolved.translated_key, "files/uppercase.txt");
}

#[test]
fn test_complex_uri_scenarios() {
    let config = create_test_config();
    let resolver = UriResolver::new(&config);

    // Test URI with query parameters
    let resolved = resolver.resolve_uri("test-s3://folder/file.txt?version=1").unwrap();
    assert_eq!(resolved.parsed.query_params.get("version"), Some(&"1".to_string()));
    assert_eq!(resolved.translated_key, "files/folder/file.txt");

    // Test deeply nested paths
    let resolved = resolver.resolve_uri("test-local://a/b/c/d/e/f/file.txt").unwrap();
    assert_eq!(resolved.translated_key, "a/b/c/d/e/f/file.txt");
}

#[test]
fn test_uri_parser_special_characters() {
    // Test URIs with special characters using shared test utilities
    let special_char_uris = vec![
        ("protocol://path/with spaces/file name.txt", "path/with spaces/file name.txt"),
        ("protocol://path/with-dashes_and_underscores.txt", "path/with-dashes_and_underscores.txt"),
        ("protocol://path/with.dots/file.ext", "path/with.dots/file.ext"),
    ];

    for (uri, expected_path) in special_char_uris {
        let parsed = UriParser::parse(uri).unwrap();
        assert_eq!(parsed.path, expected_path, "Failed to parse URI with special characters: {}", uri);
    }
}

#[test]
fn test_resolved_uri_accessors() {
    let config = create_test_config();
    let resolver = UriResolver::new(&config);

    let resolved = resolver.resolve_uri("test-s3://file.txt").unwrap();

    assert_eq!(resolved.bucket(), Some("test-bucket"));
    assert_eq!(resolved.region(), Some("us-east-1"));
    assert_eq!(resolved.path(), None); // S3 doesn't have path

    let resolved = resolver.resolve_uri("test-local://file.txt").unwrap();
    assert_eq!(resolved.bucket(), None); // Local doesn't have bucket
    assert_eq!(resolved.region(), None); // Local doesn't have region
    assert_eq!(resolved.path(), Some("/tmp/dfs-test"));
}