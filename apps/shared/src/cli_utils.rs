//! Common CLI utilities and command handlers for durable-filestor

use crate::{Config, DurableError, Result};

/// Common trait for CLI command handlers
pub trait CommandHandler {
    /// Handle a command with the given configuration
    fn handle_command(&self, config: &Config) -> Result<()>;
}

/// Display configuration information
pub fn display_config_info(config: &Config) -> Result<()> {
    println!("Current configuration:");
    let toml_content = toml::to_string_pretty(config)
        .map_err(|e| DurableError::Config {
            message: format!("Failed to serialize config: {}", e),
        })?;
    println!("{}", toml_content);
    Ok(())
}

/// Display protocol information
pub fn display_protocols(config: &Config) -> Result<()> {
    if config.protocols.is_empty() {
        println!("No protocols configured.");
    } else {
        println!("Configured protocols:");
        for (name, protocol) in &config.protocols {
            println!("  {} -> {} service", name, protocol.service);
            if let Some(ref bucket) = protocol.bucket {
                println!("    Bucket: {}", bucket);
            }
            if let Some(ref region) = protocol.region {
                println!("    Region: {}", region);
            }
            if let Some(ref path) = protocol.path {
                println!("    Path: {}", path);
            }
        }
    }
    Ok(())
}

/// Display operation parameters for debugging
pub fn display_operation_info(
    operation: &str,
    uri: &str,
    service: &Option<String>,
    bucket: &Option<String>,
    region: &Option<String>,
    path: &Option<String>,
) {
    println!("Would {} for URI: {}", operation, uri);
    if let Some(service) = service {
        println!("  Service: {}", service);
    }
    if let Some(bucket) = bucket {
        println!("  Bucket: {}", bucket);
    }
    if let Some(region) = region {
        println!("  Region: {}", region);
    }
    if let Some(path) = path {
        println!("  Path: {}", path);
    }
}

/// Display file operation information
pub fn display_file_operation_info(
    operation: &str,
    source: &str,
    target: &str,
    service: &Option<String>,
    bucket: &Option<String>,
    region: &Option<String>,
    path: &Option<String>,
) {
    println!("Would {} '{}' to '{}'", operation, source, target);
    if let Some(service) = service {
        println!("  Service: {}", service);
    }
    if let Some(bucket) = bucket {
        println!("  Bucket: {}", bucket);
    }
    if let Some(region) = region {
        println!("  Region: {}", region);
    }
    if let Some(path) = path {
        println!("  Path: {}", path);
    }
}

/// Display not implemented message
pub fn display_not_implemented(feature: &str) {
    println!("{} functionality not yet implemented.", feature);
}



/// Handle a list command with shared logic
pub fn handle_list_command_shared(
    uri: &str,
    service: &Option<String>,
    bucket: &Option<String>,
    region: &Option<String>,
    path: &Option<String>,
) -> Result<()> {
    display_operation_info("list files", uri, service, bucket, region, path);
    display_not_implemented("Listing");
    Ok(())
}

/// Handle an upload command with shared logic
pub fn handle_upload_command_shared(
    source: &str,
    target: &str,
    service: &Option<String>,
    bucket: &Option<String>,
    region: &Option<String>,
    path: &Option<String>,
) -> Result<()> {
    display_file_operation_info("upload", source, target, service, bucket, region, path);
    display_not_implemented("Upload");
    Ok(())
}

/// Handle a download command with shared logic
pub fn handle_download_command_shared(
    source: &str,
    target: &str,
    service: &Option<String>,
    bucket: &Option<String>,
    region: &Option<String>,
    path: &Option<String>,
) -> Result<()> {
    display_file_operation_info("download", source, target, service, bucket, region, path);
    display_not_implemented("Download");
    Ok(())
}

/// Handle a head command with shared logic
pub fn handle_head_command_shared(
    uri: &str,
    service: &Option<String>,
    bucket: &Option<String>,
    region: &Option<String>,
    path: &Option<String>,
) -> Result<()> {
    display_operation_info("get metadata", uri, service, bucket, region, path);
    display_not_implemented("Head");
    Ok(())
}

/// Handle config commands with shared logic
pub fn handle_config_command_shared(
    action: &crate::ConfigAction,
    config: &Config,
) -> Result<()> {
    match action {
        crate::ConfigAction::AddProtocol { name, service, bucket, region, path } => {
            // This would add a protocol to the configuration
            // For now, just print what would be done
            println!("Would add protocol '{}' with service '{}'", name, service);
            if let Some(bucket) = bucket {
                println!("  Bucket: {}", bucket);
            }
            if let Some(region) = region {
                println!("  Region: {}", region);
            }
            if let Some(path) = path {
                println!("  Path: {}", path);
            }
        }
        crate::ConfigAction::RemoveProtocol { name } => {
            println!("Would remove protocol '{}'", name);
        }
        crate::ConfigAction::ListProtocols => {
            display_protocols(config)?;
        }
        crate::ConfigAction::Show => {
            display_config_info(config)?;
        }
    }
    Ok(())
}

/// Display a parsed URI with detailed information
pub fn display_parsed_uri(uri: &str, parsed: &crate::ParsedUri) {
    println!("Parsing URI: {}", uri);
    println!("{}", "=".repeat(50));

    println!("✅ Successfully parsed URI");
    println!("   Protocol: {}", parsed.protocol);
    println!("   Path: {}", parsed.path);
    println!("   Full Path: {}", parsed.full_path());
    println!("   Original: {}", parsed.original);

    if !parsed.query_params.is_empty() {
        println!("   Query Parameters:");
        for (key, value) in &parsed.query_params {
            println!("     {} = {}", key, value);
        }
    } else {
        println!("   Query Parameters: (none)");
    }
}

/// Display URI validation result
pub fn display_validation_result(uri: &str, is_valid: bool, error_msg: Option<&str>) {
    println!("Validating URI: {}", uri);

    if is_valid {
        println!("✅ URI is valid");
    } else {
        println!("❌ URI is invalid: {}", error_msg.unwrap_or("Unknown error"));
    }
}

/// Display protocol extraction result
pub fn display_protocol_extraction(_uri: &str, protocol: Option<&str>, error_msg: Option<&str>) {
    match protocol {
        Some(protocol) => println!("{}", protocol),
        None => {
            eprintln!("❌ Failed to extract protocol: {}", error_msg.unwrap_or("Unknown error"));
            std::process::exit(1);
        }
    }
}

/// Display path extraction result
pub fn display_path_extraction(_uri: &str, path: Option<&str>, error_msg: Option<&str>) {
    match path {
        Some(path) => println!("{}", path),
        None => {
            eprintln!("❌ Failed to extract path: {}", error_msg.unwrap_or("Unknown error"));
            std::process::exit(1);
        }
    }
}

/// Display URI resolution result
pub fn display_uri_resolution(uri: &str, resolved: Option<&crate::ResolvedUri>, error_msg: Option<&str>) {
    println!("Resolving URI: {}", uri);
    println!("{}", "=".repeat(50));

    match resolved {
        Some(resolved) => {
            println!("✅ Successfully resolved URI");
            println!("   Service: {}", resolved.service);
            if let Some(bucket) = resolved.bucket() {
                println!("   Bucket: {}", bucket);
            }
            if let Some(region) = resolved.region() {
                println!("   Region: {}", region);
            }
            if let Some(path) = resolved.path() {
                println!("   Base Path: {}", path);
            }
            println!("   Translated Key: {}", resolved.translated_key);
            println!("   Original URI: {}", resolved.parsed.original);
        }
        None => {
            eprintln!("❌ Resolution Error: {}", error_msg.unwrap_or("Unknown error"));
            std::process::exit(1);
        }
    }
}

/// Display test results summary
pub fn display_test_results_summary(passed: usize, failed: usize, total: usize) {
    println!("{}", "=".repeat(60));
    println!("Test Results:");
    println!("   ✅ Passed: {}", passed);
    println!("   ❌ Failed: {}", failed);
    println!("   Total: {}", total);

    if failed > 0 {
        std::process::exit(1);
    }
}

/// Display test case result
pub fn display_test_case_result(description: &str, uri: &str, success: bool, error_msg: Option<&str>) {
    print!("Testing: {} ... ", description);

    if success {
        println!("✅ PASSED");
        println!("   URI: {}", uri);
    } else {
        println!("❌ FAILED");
        println!("   URI: {}", uri);
        if let Some(error) = error_msg {
            println!("   Error: {}", error);
        }
        println!();
    }
}

/// Display resolution test result
pub fn display_resolution_test_result(uri: &str, success: bool, service: Option<&str>, translated_key: Option<&str>, error_msg: Option<&str>) {
    print!("Resolving {} ... ", uri);

    if success {
        println!("✅ SUCCESS");
        if let (Some(service), Some(key)) = (service, translated_key) {
            println!("   Service: {}", service);
            println!("   Translated key: {}", key);
        }
        println!();
    } else {
        println!("❌ FAILED: {}", error_msg.unwrap_or("Unknown error"));
        println!();
    }
}

/// Display examples with parsing results
pub fn display_uri_examples(examples: &[(&str, &str)]) {
    println!("DFS URI Parsing Examples");
    println!("{}", "=".repeat(50));

    for (description, uri) in examples {
        println!("{}: {}", description, uri);
        match crate::UriParser::parse(uri) {
            Ok(parsed) => {
                println!("   ✅ Protocol: {}", parsed.protocol);
                println!("   ✅ Path: {}", parsed.path);
                if !parsed.query_params.is_empty() {
                    println!("   ✅ Query params: {:?}", parsed.query_params);
                }
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
            }
        }
        println!();
    }
}

/// Display available test protocols
pub fn display_available_test_protocols(config: &Config) {
    println!("Available test protocols for resolution:");
    for protocol in config.protocols.keys() {
        println!("   • {}", protocol);
    }

    println!("\nTry resolving with: dfs-uri-test-parse --resolve \"test-s3://example.txt\"");
}