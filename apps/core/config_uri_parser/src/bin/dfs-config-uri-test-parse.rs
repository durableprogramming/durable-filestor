//! Test binary for URI-based configuration parsing with extensive integration testing

use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use clap::{Arg, Command, ArgMatches};
use durable_filestor_config_uri_parser::ConfigUriParser;
use durable_filestor_shared::{
    Config,
    test_utils::*,
    cli_utils::{display_test_results_summary, display_test_case_result, display_protocols}
};

fn main() -> Result<()> {
    let matches = Command::new("dfs-config-uri-test-parse")
        .version("0.1.0")
        .about("Test URI-based configuration parsing for durable-filestor with extensive integration testing")
        .arg(
            Arg::new("uri")
                .help("Configuration URI to parse (if not provided, runs integration tests)")
                .index(1),
        )
        .arg(
            Arg::new("validate")
                .long("validate")
                .short('v')
                .help("Validate the parsed configuration")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("run-tests")
                .long("run-tests")
                .short('t')
                .help("Run comprehensive integration tests")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .help("Enable verbose output for debugging")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .value_name("FORMAT")
                .help("Output format: toml, json, yaml")
                .default_value("toml"),
        )
        .get_matches();

    let verbose = matches.get_flag("verbose");
    
    if matches.get_flag("run-tests") || matches.get_one::<String>("uri").is_none() {
        run_integration_tests(verbose)?;
    } else {
        let uri = matches.get_one::<String>("uri").unwrap();
        parse_single_uri(&matches, uri, verbose)?;
    }

    Ok(())
}

fn parse_single_uri(matches: &ArgMatches, uri: &str, verbose: bool) -> Result<()> {
    let validate = matches.get_flag("validate");
    let format = matches.get_one::<String>("format").unwrap();
    
    let parser = ConfigUriParser::new();

    if verbose {
        println!("=== URI Parsing ===");
        println!("Input URI: {}", uri);
        println!("Validation enabled: {}", validate);
        println!("Output format: {}", format);
        println!();
    }

    let config = if validate {
        parser.parse_and_validate(uri)
            .with_context(|| format!("Failed to parse and validate URI: {}", uri))?
    } else {
        parser.parse_from_uri(uri)
            .with_context(|| format!("Failed to parse URI: {}", uri))?
    };

    if verbose {
        println!("=== Parsed Configuration ===");
        if let Err(e) = display_protocols(&config) {
            eprintln!("Failed to display protocols: {}", e);
        }
        println!();
    }

    match format.as_str() {
        "json" => println!("{}", serde_json::to_string_pretty(&config)?),
        "yaml" => println!("{}", serde_yaml::to_string(&config)?),
        _ => println!("{}", toml::to_string_pretty(&config)?),
    }

    Ok(())
}

fn run_integration_tests(verbose: bool) -> Result<()> {
    println!("=== Running Comprehensive URI Configuration Parser Integration Tests ===");
    println!();

    let mut total_tests = 0;
    let mut passed_tests = 0;

    // TOML Protocol Tests
    println!("Testing TOML Protocol URIs...");
    let toml_results = test_toml_protocol(verbose);
    total_tests += toml_results.0;
    passed_tests += toml_results.1;

    println!();

    // DFS Protocol Tests
    println!("Testing DFS Protocol URIs...");
    let dfs_results = test_dfs_protocol(verbose);
    total_tests += dfs_results.0;
    passed_tests += dfs_results.1;

    println!();

    // Error Handling Tests
    println!("Testing Error Handling...");
    let error_results = test_error_handling(verbose);
    total_tests += error_results.0;
    passed_tests += error_results.1;

    println!();

    // Validation Tests
    println!("Testing Validation...");
    let validation_results = test_validation(verbose);
    total_tests += validation_results.0;
    passed_tests += validation_results.1;

    println!();

    // Edge Case Tests
    println!("Testing Edge Cases...");
    let edge_results = test_edge_cases(verbose);
    total_tests += edge_results.0;
    passed_tests += edge_results.1;

    println!();

    // Summary
    display_test_results_summary(passed_tests, total_tests - passed_tests, total_tests);

    Ok(())
}

/// Helper function to run a test case with common logic
fn run_test_case<F>(
    parser: &ConfigUriParser,
    uri: &str,
    test_name: &str,
    verbose: bool,
    validator: F,
) -> bool
where
    F: Fn(&Config) -> bool,
{
    match parser.parse_from_uri(uri) {
        Ok(config) => {
            if validator(&config) {
                if verbose { println!("✅ {}", test_name); }
                true
            } else {
                if verbose { println!("❌ {} - validation failed", test_name); }
                false
            }
        }
        Err(e) => {
            if verbose { println!("❌ {} - {}", test_name, e); }
            false
        }
    }
}

/// Helper function to run a validation test case
fn run_validation_test_case(
    parser: &ConfigUriParser,
    uri: &str,
    test_name: &str,
    verbose: bool,
    should_pass: bool,
) -> bool {
    let result = parser.parse_and_validate(uri);

    match (result, should_pass) {
        (Ok(_), true) => {
            if verbose { println!("✅ {}", test_name); }
            true
        }
        (Err(_), false) => {
            if verbose { println!("✅ {}", test_name); }
            true
        }
        (Ok(_), false) => {
            if verbose { println!("❌ {} - should have failed validation", test_name); }
            false
        }
        (Err(e), true) => {
            if verbose { println!("❌ {} - {}", test_name, e); }
            false
        }
    }
}

/// Helper function to run an error handling test case
fn run_error_test_case(
    parser: &ConfigUriParser,
    uri: &str,
    test_name: &str,
    verbose: bool,
) -> bool {
    match parser.parse_from_uri(uri) {
        Err(_) => {
            if verbose { println!("✅ {}", test_name); }
            true
        }
        Ok(_) => {
            if verbose { println!("❌ {} - should have failed", test_name); }
            false
        }
    }
}

fn test_toml_protocol(verbose: bool) -> (usize, usize) {
    let mut total = 0;
    let mut passed = 0;
    let parser = ConfigUriParser::new();

    // Test 1: Basic TOML configuration
    total += 1;
    let basic_toml = r#"
[protocols]
test-s3 = { service = "s3", region = "us-west-2", bucket = "test-bucket" }

[services.s3]
default_region = "us-west-2"
"#;
    let encoded = general_purpose::STANDARD.encode(basic_toml);
    let uri = format!("toml://{}", encoded);

    if run_test_case(&parser, &uri, "Basic TOML parsing", verbose, |config| {
        config.protocols.len() == 1 && config.services.len() == 1
    }) {
        passed += 1;
    }

    // Test 2: Complex TOML with multiple services and protocols
    total += 1;
    let complex_toml = r#"
[protocols]
my-s3 = { service = "s3", region = "us-east-1", bucket = "prod-bucket", path = "uploads/" }
my-local = { service = "local", path = "/tmp/storage" }

[services.s3]
default_region = "us-east-1"
endpoint = "https://s3.amazonaws.com"

[services.local]
default_permissions = "755"

[services.s3.parameters]
retry_count = "3"
timeout = "30"
"#;
    let encoded = general_purpose::STANDARD.encode(complex_toml);
    let uri = format!("toml://{}", encoded);

    if run_test_case(&parser, &uri, "Complex TOML parsing", verbose, |config| {
        config.protocols.len() == 2 && config.services.len() == 2
    }) {
        passed += 1;
    }

    // Test 3: TOML with key translation settings
    total += 1;
    let kt_toml = r#"
[protocols.my-hashed]
service = "s3"
bucket = "hashed-bucket"

[protocols.my-hashed.key_translation]
hash_algorithm = "SHA256"
case_conversion = "lower"
normalize_unicode = true

[services.s3]
default_region = "us-west-1"
"#;
    let encoded = general_purpose::STANDARD.encode(kt_toml);
    let uri = format!("toml://{}", encoded);
    
    match parser.parse_from_uri(&uri) {
        Ok(config) => {
            if let Some(protocol) = config.protocols.get("my-hashed") {
                if protocol.key_translation.hash_function.is_some() {
                    passed += 1;
                    if verbose { println!("✅ TOML with key translation"); }
                } else {
                    if verbose { println!("❌ TOML key translation - missing hash function"); }
                }
            } else {
                if verbose { println!("❌ TOML key translation - missing protocol"); }
            }
        }
        Err(e) => {
            if verbose { println!("❌ TOML key translation - {}", e); }
        }
    }

    (total, passed)
}

fn test_dfs_protocol(verbose: bool) -> (usize, usize) {
    let mut total = 0;
    let mut passed = 0;
    let parser = ConfigUriParser::new();

    // Test 1: Basic S3 DFS URI
    total += 1;
    let uri = "dfs://s3?bucket_name=test-bucket&region=us-west-2&access_key_id=AKIATEST&secret_access_key=secretkey123";
    match parser.parse_from_uri(uri) {
        Ok(config) => {
            if let Some(service) = config.services.get("s3") {
                if service.parameters.get("bucket_name") == Some(&"test-bucket".to_string()) &&
                   service.parameters.get("region") == Some(&"us-west-2".to_string()) {
                    passed += 1;
                    if verbose { println!("✅ Basic S3 DFS URI"); }
                } else {
                    if verbose { println!("❌ Basic S3 DFS URI - missing parameters"); }
                }
            } else {
                if verbose { println!("❌ Basic S3 DFS URI - missing service"); }
            }
        }
        Err(e) => {
            if verbose { println!("❌ Basic S3 DFS URI - {}", e); }
        }
    }

    // Test 2: Local storage DFS URI
    total += 1;
    let uri = "dfs://local?path=/var/storage&permissions=755&create_dirs=true";
    match parser.parse_from_uri(uri) {
        Ok(config) => {
            if let Some(service) = config.services.get("local") {
                if service.parameters.get("path") == Some(&"/var/storage".to_string()) &&
                   service.parameters.get("permissions") == Some(&"755".to_string()) {
                    passed += 1;
                    if verbose { println!("✅ Local storage DFS URI"); }
                } else {
                    if verbose { println!("❌ Local storage DFS URI - missing parameters"); }
                }
            } else {
                if verbose { println!("❌ Local storage DFS URI - missing service"); }
            }
        }
        Err(e) => {
            if verbose { println!("❌ Local storage DFS URI - {}", e); }
        }
    }

    // Test 3: Azure Blob DFS URI
    total += 1;
    let uri = "dfs://azure?container=mycontainer&account_name=myaccount&account_key=mykey&endpoint=https://myaccount.blob.core.windows.net";
    match parser.parse_from_uri(uri) {
        Ok(config) => {
            if let Some(service) = config.services.get("azure") {
                if service.parameters.get("container") == Some(&"mycontainer".to_string()) &&
                   service.endpoint == Some("https://myaccount.blob.core.windows.net".to_string()) {
                    passed += 1;
                    if verbose { println!("✅ Azure Blob DFS URI"); }
                } else {
                    if verbose { println!("❌ Azure Blob DFS URI - missing parameters"); }
                }
            } else {
                if verbose { println!("❌ Azure Blob DFS URI - missing service"); }
            }
        }
        Err(e) => {
            if verbose { println!("❌ Azure Blob DFS URI - {}", e); }
        }
    }

    // Test 4: GCS DFS URI
    total += 1;
    let uri = "dfs://gcs?bucket=my-gcs-bucket&project_id=my-project&credentials_path=/path/to/creds.json";
    match parser.parse_from_uri(uri) {
        Ok(config) => {
            if let Some(service) = config.services.get("gcs") {
                if service.parameters.get("bucket") == Some(&"my-gcs-bucket".to_string()) &&
                   service.parameters.get("project_id") == Some(&"my-project".to_string()) {
                    passed += 1;
                    if verbose { println!("✅ GCS DFS URI"); }
                } else {
                    if verbose { println!("❌ GCS DFS URI - missing parameters"); }
                }
            } else {
                if verbose { println!("❌ GCS DFS URI - missing service"); }
            }
        }
        Err(e) => {
            if verbose { println!("❌ GCS DFS URI - {}", e); }
        }
    }

    // Test 5: Complex DFS URI with many parameters
    total += 1;
    let uri = "dfs://s3?bucket_name=complex-bucket&region=eu-central-1&endpoint=https://s3.eu-central-1.amazonaws.com&retry_count=5&timeout=60&use_ssl=true&validate_certificates=false&chunk_size=8192";
    match parser.parse_from_uri(uri) {
        Ok(config) => {
            if let Some(service) = config.services.get("s3") {
                let param_count = service.parameters.len();
                if param_count >= 7 {
                    passed += 1;
                    if verbose { println!("✅ Complex DFS URI with {} parameters", param_count); }
                } else {
                    if verbose { println!("❌ Complex DFS URI - only {} parameters found", param_count); }
                }
            } else {
                if verbose { println!("❌ Complex DFS URI - missing service"); }
            }
        }
        Err(e) => {
            if verbose { println!("❌ Complex DFS URI - {}", e); }
        }
    }

    (total, passed)
}

fn test_error_handling(verbose: bool) -> (usize, usize) {
    let mut total = 0;
    let mut passed = 0;
    let parser = ConfigUriParser::new();

    // Test 1: Invalid URI format
    total += 1;
    if run_error_test_case(&parser, "not-a-valid-uri", "Invalid URI format rejection", verbose) {
        passed += 1;
    }

    // Test 2: Unsupported scheme
    total += 1;
    if run_error_test_case(&parser, "http://example.com", "Unsupported scheme rejection", verbose) {
        passed += 1;
    }

    // Test 3: Invalid base64 in TOML URI
    total += 1;
    if run_error_test_case(&parser, "toml://invalid-base64-!@#$%", "Invalid base64 rejection", verbose) {
        passed += 1;
    }

    // Test 4: Invalid TOML content
    total += 1;
    let invalid_toml = "invalid toml content [[[";
    let encoded = general_purpose::STANDARD.encode(invalid_toml);
    let uri = format!("toml://{}", encoded);
    match parser.parse_from_uri(&uri) {
        Err(_) => {
            passed += 1;
            if verbose { println!("✅ Invalid TOML content rejection"); }
        }
        Ok(_) => {
            if verbose { println!("❌ Invalid TOML content should fail"); }
        }
    }

    // Test 5: Missing hostname in DFS URI
    total += 1;
    match parser.parse_from_uri("dfs://?param=value") {
        Err(_) => {
            passed += 1;
            if verbose { println!("✅ Missing hostname rejection"); }
        }
        Ok(_) => {
            if verbose { println!("❌ Missing hostname should fail"); }
        }
    }

    (total, passed)
}

fn test_validation(verbose: bool) -> (usize, usize) {
    let mut total = 0;
    let mut passed = 0;
    let parser = ConfigUriParser::new();

    // Test 1: Valid configuration passes validation
    total += 1;
    let valid_toml = r#"
[protocols]
test = { service = "s3", bucket = "test" }
[services.s3]
default_region = "us-west-2"
"#;
    let encoded = general_purpose::STANDARD.encode(valid_toml);
    let uri = format!("toml://{}", encoded);
    if run_validation_test_case(&parser, &uri, "Valid configuration passes validation", verbose, true) {
        passed += 1;
    }

    // Test 2: Empty configuration fails validation
    total += 1;
    let empty_toml = "";
    let encoded = general_purpose::STANDARD.encode(empty_toml);
    let uri = format!("toml://{}", encoded);
    if run_validation_test_case(&parser, &uri, "Empty configuration rejected by validation", verbose, false) {
        passed += 1;
    }

    // Test 3: DFS URI passes validation
    total += 1;
    let uri = "dfs://s3?bucket_name=test&region=us-west-2";
    if run_validation_test_case(&parser, uri, "DFS URI passes validation", verbose, true) {
        passed += 1;
    }

    (total, passed)
}

fn test_edge_cases(verbose: bool) -> (usize, usize) {
    let mut total = 0;
    let mut passed = 0;
    let parser = ConfigUriParser::new();

    // Test 1: URI with special characters in parameters
    total += 1;
    let uri = "dfs://s3?bucket_name=test-bucket&path=/special%20path%2Bwith%26chars&key=value%3Dtest";
    match parser.parse_from_uri(uri) {
        Ok(config) => {
            if let Some(service) = config.services.get("s3") {
                if service.parameters.get("path") == Some(&"/special path+with&chars".to_string()) {
                    passed += 1;
                    if verbose { println!("✅ URI with special characters"); }
                } else {
                    if verbose { println!("❌ URI special characters not decoded properly"); }
                }
            } else {
                if verbose { println!("❌ URI special characters - missing service"); }
            }
        }
        Err(e) => {
            if verbose { println!("❌ URI special characters - {}", e); }
        }
    }

    // Test 2: TOML with Unicode characters
    total += 1;
    let unicode_toml = r#"
[protocols]
测试 = { service = "s3", bucket = "测试-bucket" }
[services.s3]
default_region = "测试-region"
"#;
    let encoded = general_purpose::STANDARD.encode(unicode_toml);
    let uri = format!("toml://{}", encoded);
    match parser.parse_from_uri(&uri) {
        Ok(config) => {
            if config.protocols.contains_key("测试") {
                passed += 1;
                if verbose { println!("✅ Unicode TOML content"); }
            } else {
                if verbose { println!("❌ Unicode TOML - missing unicode key"); }
            }
        }
        Err(e) => {
            if verbose { println!("❌ Unicode TOML - {}", e); }
        }
    }

    // Test 3: Very long parameter values
    total += 1;
    let long_value = "a".repeat(1000);
    let uri = format!("dfs://custom?long_param={}", long_value);
    match parser.parse_from_uri(&uri) {
        Ok(config) => {
            if let Some(service) = config.services.get("custom") {
                if service.parameters.get("long_param").map(|v| v.len()) == Some(1000) {
                    passed += 1;
                    if verbose { println!("✅ Long parameter values"); }
                } else {
                    if verbose { println!("❌ Long parameter values - incorrect length"); }
                }
            } else {
                if verbose { println!("❌ Long parameter values - missing service"); }
            }
        }
        Err(e) => {
            if verbose { println!("❌ Long parameter values - {}", e); }
        }
    }

    // Test 4: Empty parameter values
    total += 1;
    let uri = "dfs://test?param1=&param2=value&param3=";
    match parser.parse_from_uri(uri) {
        Ok(config) => {
            if let Some(service) = config.services.get("test") {
                if service.parameters.get("param1") == Some(&"".to_string()) &&
                   service.parameters.get("param2") == Some(&"value".to_string()) &&
                   service.parameters.get("param3") == Some(&"".to_string()) {
                    passed += 1;
                    if verbose { println!("✅ Empty parameter values"); }
                } else {
                    if verbose { println!("❌ Empty parameter values - incorrect handling"); }
                }
            } else {
                if verbose { println!("❌ Empty parameter values - missing service"); }
            }
        }
        Err(e) => {
            if verbose { println!("❌ Empty parameter values - {}", e); }
        }
    }

    // Test 5: Case sensitivity in service names
    total += 1;
    let uri1 = "dfs://S3?bucket=test1";
    let uri2 = "dfs://s3?bucket=test2";
    match (parser.parse_from_uri(uri1), parser.parse_from_uri(uri2)) {
        (Ok(config1), Ok(config2)) => {
            if config1.services.contains_key("S3") && config2.services.contains_key("s3") {
                passed += 1;
                if verbose { println!("✅ Case sensitivity in service names"); }
            } else {
                if verbose { println!("❌ Case sensitivity - services not found"); }
            }
        }
        _ => {
            if verbose { println!("❌ Case sensitivity - parsing failed"); }
        }
    }

    (total, passed)
}

