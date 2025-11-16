use durable_filestor_config_uri_parser::*;
use durable_filestor_shared::{
    Config,
    DurableError,
    UriParser,
    cli_utils::{display_parsed_uri, display_validation_result},
    test_utils::create_test_config,
};
use toml;

fn main() {
    println!("=== Config URI Parser Debug Test ===\n");

    // Test empty TOML
    let empty_toml = "";
    let encoded = UriParser::encode_to_base64(empty_toml);
    let uri = format!("toml://{}", encoded);

    println!("Testing empty TOML URI: {}", uri);
    match UriParser::parse(&uri) {
        Ok(parsed_uri) => {
            display_parsed_uri(&uri, &parsed_uri);
            match parse_toml_protocol(&parsed_uri) {
                Ok(config) => {
                    println!("✅ Empty TOML parsed successfully: protocols={}, services={}",
                             config.protocols.len(), config.services.len());
                }
                Err(e) => println!("❌ Empty TOML failed: {:?}", e),
            }
        }
        Err(e) => println!("❌ URI parse failed: {:?}", e),
    }
    println!();

    // Test validation
    let parser = ConfigUriParser::new();
    let is_valid = parser.parse_and_validate(&uri).is_ok();
    display_validation_result(&uri, is_valid, None);
    println!();

    // Test with shared test configuration
    println!("Testing with shared test configuration:");
    let test_config = create_test_config();
    let test_toml = toml::to_string_pretty(&test_config)
        .expect("Failed to serialize test config");
    let encoded_test = UriParser::encode_to_base64(&test_toml);
    let test_uri = format!("toml://{}", encoded_test);

    println!("Test configuration URI: {}", test_uri);
    match UriParser::parse(&test_uri) {
        Ok(parsed_uri) => {
            display_parsed_uri(&test_uri, &parsed_uri);
            match parse_toml_protocol(&parsed_uri) {
                Ok(config) => {
                    println!("✅ Test config parsed successfully: protocols={}, services={}",
                             config.protocols.len(), config.services.len());
                    // Show some protocol names
                    let protocol_names: Vec<&String> = config.protocols.keys().collect();
                    println!("   Available protocols: {:?}", protocol_names);
                }
                Err(e) => println!("❌ Test config failed: {:?}", e),
            }
        }
        Err(e) => println!("❌ Test URI parse failed: {:?}", e),
    }
    println!();

    // Test validation of test config
    let test_is_valid = parser.parse_and_validate(&test_uri).is_ok();
    display_validation_result(&test_uri, test_is_valid, None);
}
