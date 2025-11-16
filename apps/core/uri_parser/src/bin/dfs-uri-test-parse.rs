//! Binary tool for testing and parsing DFS URIs
//!
//! This tool provides a command-line interface for parsing custom protocol URIs
//! and demonstrating the functionality of the durable-filestor URI parser.

use durable_filestor_uri_parser::{UriParser, UriResolver};
use durable_filestor_shared::{
    create_test_config, get_standard_test_uris, get_resolution_test_uris, get_example_uris,
    get_validation_test_uris, get_protocol_extraction_test_cases, get_path_extraction_test_cases,
    display_parsed_uri, display_validation_result, display_protocol_extraction,
    display_path_extraction, display_uri_resolution, display_test_results_summary,
    display_test_case_result, display_resolution_test_result, display_uri_examples,
    display_available_test_protocols
};
use std::env;
use std::process;

const USAGE: &str = r#"
dfs-uri-test-parse - Test tool for DFS URI parsing

USAGE:
    dfs-uri-test-parse [OPTIONS] <URI>

ARGUMENTS:
    <URI>    The URI to parse (e.g., "my-s3://bucket/file.txt")

OPTIONS:
    -h, --help       Show this help message
    -v, --validate   Only validate the URI without parsing details
    -p, --protocol   Extract only the protocol name
    -P, --path       Extract only the path component
    -r, --resolve    Attempt to resolve URI using test configuration
    -t, --test       Run comprehensive parsing tests
    -e, --examples   Show URI parsing examples

EXAMPLES:
    dfs-uri-test-parse "s3://mybucket/file.txt"
    dfs-uri-test-parse --protocol "custom://path/file.ext"
    dfs-uri-test-parse --resolve "test-s3://data/file.json"
    dfs-uri-test-parse --test
"#;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("{}", USAGE);
        process::exit(1);
    }

    match args[1].as_str() {
        "-h" | "--help" => {
            println!("{}", USAGE);
            process::exit(0);
        }
        "-t" | "--test" => {
            run_tests();
            process::exit(0);
        }
        "-e" | "--examples" => {
            show_examples();
            process::exit(0);
        }
        _ => {}
    }

    if args.len() < 3 && !args[1].starts_with('-') {
        // Simple URI parsing
        parse_and_display(&args[1]);
        return;
    }

    if args.len() < 3 {
        eprintln!("Error: URI argument required");
        eprintln!("{}", USAGE);
        process::exit(1);
    }

    let option = &args[1];
    let uri = &args[2];

    match option.as_str() {
        "-v" | "--validate" => validate_uri(uri),
        "-p" | "--protocol" => extract_protocol(uri),
        "-P" | "--path" => extract_path(uri),
        "-r" | "--resolve" => resolve_uri(uri),
        _ => {
            eprintln!("Unknown option: {}", option);
            eprintln!("{}", USAGE);
            process::exit(1);
        }
    }
}

fn parse_and_display(uri: &str) {
    match UriParser::parse(uri) {
        Ok(parsed) => {
            display_parsed_uri(uri, &parsed);
        }
        Err(e) => {
            eprintln!("❌ Parse Error: {}", e);
            process::exit(1);
        }
    }
}



fn validate_uri(uri: &str) {
    match UriParser::validate(uri) {
        Ok(()) => display_validation_result(uri, true, None),
        Err(e) => {
            display_validation_result(uri, false, Some(&e.to_string()));
            process::exit(1);
        }
    }
}

fn extract_protocol(uri: &str) {
    match UriParser::extract_protocol(uri) {
        Ok(protocol) => display_protocol_extraction(uri, Some(&protocol), None),
        Err(e) => {
            display_protocol_extraction(uri, None, Some(&e.to_string()));
        }
    }
}

fn extract_path(uri: &str) {
    match UriParser::extract_path(uri) {
        Ok(path) => display_path_extraction(uri, Some(&path), None),
        Err(e) => {
            display_path_extraction(uri, None, Some(&e.to_string()));
        }
    }
}

fn resolve_uri(uri: &str) {
    let config = create_test_config();
    let resolver = UriResolver::new(&config);

    match resolver.resolve_uri(uri) {
        Ok(resolved) => {
            display_uri_resolution(uri, Some(&resolved), None);
        }
        Err(e) => {
            display_uri_resolution(uri, None, Some(&e.to_string()));
        }
    }
}



fn run_tests() {
    println!("Running comprehensive URI parsing tests...");
    println!("{}", "=".repeat(60));

    let test_cases = get_standard_test_uris();
    let mut passed = 0;
    let mut failed = 0;

    for (uri, description) in test_cases {
        match UriParser::parse(uri) {
            Ok(parsed) => {
                display_test_case_result(description, uri, true, None);
                println!("   Protocol: {}", parsed.protocol);
                println!("   Path: {}", parsed.path);
                if !parsed.query_params.is_empty() {
                    println!("   Query params: {} items", parsed.query_params.len());
                }
                println!();
                passed += 1;
            }
            Err(e) => {
                display_test_case_result(description, uri, false, Some(&e.to_string()));
                failed += 1;
            }
        }
    }

    display_test_results_summary(passed, failed, passed + failed);

    // Test resolution with test config
    println!("\nTesting URI resolution...");
    println!("{}", "-".repeat(40));

    let config = create_test_config();
    let resolver = UriResolver::new(&config);
    let resolution_tests = get_resolution_test_uris();

    for uri in resolution_tests {
        match resolver.resolve_uri(uri) {
            Ok(resolved) => {
                display_resolution_test_result(uri, true, Some(&resolved.service), Some(&resolved.translated_key), None);
            }
            Err(e) => {
                display_resolution_test_result(uri, false, None, None, Some(&e.to_string()));
            }
        }
    }

    println!("All tests completed successfully! 🎉");
}

fn show_examples() {
    let examples = get_example_uris();
    display_uri_examples(&examples);

    let config = create_test_config();
    display_available_test_protocols(&config);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_config_structure() {
        let config = create_test_config();

        // Test that all expected protocols are present
        assert!(config.protocols.contains_key("test-s3"));
        assert!(config.protocols.contains_key("test-local"));
        assert!(config.protocols.contains_key("partitioned"));
        assert!(config.protocols.contains_key("backup"));

        // Test test-s3 protocol configuration
        let test_s3 = &config.protocols["test-s3"];
        assert_eq!(test_s3.service, "s3");
        assert_eq!(test_s3.bucket, Some("test-bucket".to_string()));
        assert_eq!(test_s3.region, Some("us-east-1".to_string()));
        assert!(test_s3.path.is_none());
        assert!(test_s3.key_translation.downcase);
        assert_eq!(test_s3.key_translation.prefix, Some("files/".to_string()));

        // Test test-local protocol configuration
        let test_local = &config.protocols["test-local"];
        assert_eq!(test_local.service, "local");
        assert!(test_local.bucket.is_none());
        assert!(test_local.region.is_none());
        assert_eq!(test_local.path, Some("/tmp/dfs-test".to_string()));
        assert!(!test_local.key_translation.downcase);

        // Test partitioned protocol configuration
        let partitioned = &config.protocols["partitioned"];
        assert_eq!(partitioned.service, "s3");
        assert_eq!(partitioned.bucket, Some("partitioned-bucket".to_string()));
        assert_eq!(partitioned.region, Some("us-west-2".to_string()));
        assert!(partitioned.key_translation.partition);
        assert_eq!(partitioned.key_translation.partition_length, 2);
        assert_eq!(partitioned.key_translation.partition_depth, 2);

        // Test backup protocol configuration
        let backup = &config.protocols["backup"];
        assert_eq!(backup.service, "local");
        assert!(backup.bucket.is_none());
        assert!(backup.region.is_none());
        assert_eq!(backup.path, Some("/var/backups/dfs".to_string()));
        assert!(backup.key_translation.upcase);
        assert_eq!(backup.key_translation.prefix, Some("backup-".to_string()));
    }

    #[test]
    fn test_uri_parsing_test_cases() {
        let test_cases = get_standard_test_uris();

        for (uri, description) in test_cases {
            match UriParser::parse(uri) {
                Ok(parsed) => {
                    // Verify basic structure
                    assert!(!parsed.protocol.is_empty(), "Protocol should not be empty for {}", description);
                    assert!(!parsed.original.is_empty(), "Original should not be empty for {}", description);

                    // Verify full path construction
                    let expected_full_path = if parsed.path.is_empty() {
                        parsed.protocol.clone() + "://" + &parsed.path
                    } else {
                        parsed.protocol.clone() + "://" + &parsed.path
                    };
                    assert_eq!(parsed.full_path(), expected_full_path, "Full path mismatch for {}", description);
                }
                Err(e) => {
                    panic!("Failed to parse URI '{}' ({}): {}", uri, description, e);
                }
            }
        }
    }

    #[test]
    fn test_uri_resolution_with_test_config() {
        let config = create_test_config();
        let resolver = UriResolver::new(&config);

        let resolution_tests = get_resolution_test_uris();

        for uri in resolution_tests {
            match resolver.resolve_uri(uri) {
                Ok(resolved) => {
                    assert!(!resolved.service.is_empty(), "Service should not be empty for {}", uri);
                    assert!(!resolved.translated_key.is_empty(), "Translated key should not be empty for {}", uri);
                    assert!(!resolved.parsed.original.is_empty(), "Original URI should not be empty for {}", uri);
                }
                Err(e) => {
                    panic!("Failed to resolve URI '{}': {}", uri, e);
                }
            }
        }
    }

    #[test]
    fn test_specific_resolution_transformations() {
        let config = create_test_config();
        let resolver = UriResolver::new(&config);

        // Test test-s3 downcase and prefix transformation
        let resolved = resolver.resolve_uri("test-s3://Documents/MyFile.TXT").unwrap();
        assert_eq!(resolved.service, "s3");
        assert_eq!(resolved.bucket().unwrap(), "test-bucket");
        assert_eq!(resolved.region().unwrap(), "us-east-1");
        // Should be downcased and prefixed
        assert!(resolved.translated_key.to_lowercase() == resolved.translated_key);
        assert!(resolved.translated_key.starts_with("files/"));

        // Test backup upcase and prefix transformation
        let resolved = resolver.resolve_uri("backup://important/document.pdf").unwrap();
        assert_eq!(resolved.service, "local");
        assert_eq!(resolved.path().unwrap(), "/var/backups/dfs");
        // Should be upcased and prefixed
        assert!(resolved.translated_key.starts_with("backup-"));
        // Check that the filename part is uppercase (the path separator remains lowercase)
        assert!(resolved.translated_key.contains("IMPORTANT/DOCUMENT.PDF"));
    }

    #[test]
    fn test_partitioned_key_transformation() {
        let config = create_test_config();
        let resolver = UriResolver::new(&config);

        // Test partitioned protocol with depth 2, length 2
        let resolved = resolver.resolve_uri("partitioned://abcdefghijklmnop").unwrap();
        assert_eq!(resolved.service, "s3");
        assert_eq!(resolved.bucket().unwrap(), "partitioned-bucket");
        assert_eq!(resolved.region().unwrap(), "us-west-2");

        // Should create partitioned structure like ab/cd/abcdefghijklmnop
        let key = resolved.translated_key;
        assert!(key.len() > "abcdefghijklmnop".len()); // Should be longer due to partitioning
        assert!(key.contains('/')); // Should contain path separators
    }

    #[test]
    fn test_query_parameter_parsing() {
        // Test single query parameter
        let parsed = UriParser::parse("s3://bucket/file.txt?version=1").unwrap();
        assert_eq!(parsed.query_params.len(), 1);
        assert_eq!(parsed.query_params.get("version"), Some(&"1".to_string()));

        // Test multiple query parameters
        let parsed = UriParser::parse("custom://path/file?a=1&b=2&c=hello").unwrap();
        assert_eq!(parsed.query_params.len(), 3);
        assert_eq!(parsed.query_params.get("a"), Some(&"1".to_string()));
        assert_eq!(parsed.query_params.get("b"), Some(&"2".to_string()));
        assert_eq!(parsed.query_params.get("c"), Some(&"hello".to_string()));

        // Test URL-encoded parameters
        let parsed = UriParser::parse("protocol://path?encoded=%2Fhome%2Fuser").unwrap();
        assert_eq!(parsed.query_params.len(), 1);
        assert_eq!(parsed.query_params.get("encoded"), Some(&"/home/user".to_string()));

        // Test no query parameters
        let parsed = UriParser::parse("s3://bucket/file.txt").unwrap();
        assert_eq!(parsed.query_params.len(), 0);
    }

    #[test]
    fn test_edge_case_uris() {
        // Test URI with empty path
        let parsed = UriParser::parse("single://").unwrap();
        assert_eq!(parsed.protocol, "single");
        assert_eq!(parsed.path, "");
        assert_eq!(parsed.full_path(), "single://");

        // Test minimal valid URI
        let parsed = UriParser::parse("minimal://a").unwrap();
        assert_eq!(parsed.protocol, "minimal");
        assert_eq!(parsed.path, "a");
        assert_eq!(parsed.full_path(), "minimal://a");

        // Test deep nesting
        let parsed = UriParser::parse("protocol://deep/nested/path/structure/file.ext").unwrap();
        assert_eq!(parsed.protocol, "protocol");
        assert_eq!(parsed.path, "deep/nested/path/structure/file.ext");
        assert_eq!(parsed.full_path(), "protocol://deep/nested/path/structure/file.ext");
    }

    #[test]
    fn test_path_with_special_characters() {
        // Test spaces in path
        let parsed = UriParser::parse("protocol://path/with spaces/file name.txt").unwrap();
        assert_eq!(parsed.path, "path/with spaces/file name.txt");

        // Test dashes and underscores
        let parsed = UriParser::parse("protocol://path/with-dashes_and_underscores.txt").unwrap();
        assert_eq!(parsed.path, "path/with-dashes_and_underscores.txt");

        // Test encoded characters
        let parsed = UriParser::parse("protocol://path/file%20name.txt").unwrap();
        assert_eq!(parsed.path, "path/file%20name.txt");
    }

    #[test]
    fn test_config_services_map_empty() {
        let config = create_test_config();
        // Test config has empty services map as expected
        assert!(config.services.is_empty());
    }

    #[test]
    fn test_protocol_config_parameters_empty() {
        let config = create_test_config();

        // All protocols should have empty parameters maps
        for protocol in config.protocols.values() {
            assert!(protocol.parameters.is_empty());
        }
    }

    #[test]
    fn test_uri_validation_integration() {
        let test_uris = get_validation_test_uris();

        for uri in test_uris {
            // Should validate successfully
            assert!(UriParser::validate(uri).is_ok(), "URI '{}' should validate successfully", uri);
        }
    }

    #[test]
    fn test_protocol_extraction() {
        let test_cases = get_protocol_extraction_test_cases();

        for (uri, expected_protocol) in test_cases {
            let protocol = UriParser::extract_protocol(uri).unwrap();
            assert_eq!(protocol, expected_protocol, "Protocol extraction failed for URI: {}", uri);
        }
    }

    #[test]
    fn test_path_extraction() {
        let test_cases = get_path_extraction_test_cases();

        for (uri, expected_path) in test_cases {
            let path = UriParser::extract_path(uri).unwrap();
            assert_eq!(path, expected_path, "Path extraction failed for URI: {}", uri);
        }
    }

    #[test]
    fn test_resolution_error_handling() {
        let config = create_test_config();
        let resolver = UriResolver::new(&config);

        // Test with non-existent protocol
        let result = resolver.resolve_uri("nonexistent://file.txt");
        assert!(result.is_err(), "Should fail to resolve non-existent protocol");

        // Test with malformed URI
        let result = resolver.resolve_uri("invalid-uri");
        assert!(result.is_err(), "Should fail to resolve malformed URI");
    }

    #[test]
    fn test_config_key_translation_defaults() {
        let config = create_test_config();

        // Test that test-local has default key translation (no transformations)
        let test_local = &config.protocols["test-local"];
        assert!(!test_local.key_translation.downcase);
        assert!(!test_local.key_translation.upcase);
        assert!(!test_local.key_translation.partition);
        assert!(test_local.key_translation.prefix.is_none());
        assert!(test_local.key_translation.suffix.is_none());
        assert!(test_local.key_translation.hash_function.is_none());
    }

    #[test]
    fn test_parsed_uri_original_preservation() {
        let test_uris = vec![
            "s3://bucket/file.txt",
            "my-s3://mybucket/path/to/file.json?version=1",
            "local:///absolute/path/file.txt",
            "protocol://path/with spaces/file name.txt?param=value",
        ];

        for uri in test_uris {
            let parsed = UriParser::parse(uri).unwrap();
            assert_eq!(parsed.original, uri, "Original URI should be preserved exactly");
        }
    }

    #[test]
    fn test_full_path_construction() {
        let test_cases = vec![
            ("s3://bucket/file.txt", "s3://bucket/file.txt"),
            ("my-s3://mybucket/path/to/file.json", "my-s3://mybucket/path/to/file.json"),
            ("local:///absolute/path/file.txt", "local:///absolute/path/file.txt"),
            ("single://", "single://"),
            ("minimal://a", "minimal://a"),
        ];

        for (uri, expected_full_path) in test_cases {
            let parsed = UriParser::parse(uri).unwrap();
            assert_eq!(parsed.full_path(), expected_full_path, "Full path construction failed for URI: {}", uri);
        }
    }
}