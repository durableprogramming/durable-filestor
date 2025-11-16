//! Macros to reduce code duplication across durable-filestor services

/// Creates a standard tokio main function for service binaries.
/// 
/// This macro eliminates duplication in service main.rs files by providing
/// a standard pattern for CLI initialization and execution.
/// 
/// # Arguments
/// * `$cli_type` - The CLI struct type (e.g., LocalCli, S3Cli)
/// * `$execute_fn` - The function to call with the parsed CLI (e.g., durable_filestor_local::cli::execute)
/// 
/// # Example
/// ```rust,ignore
/// use durable_filestor_shared::service_main;
/// use clap::Parser;
/// 
/// #[derive(Parser)]
/// struct LocalCli;
/// 
/// async fn execute_cli(_cli: LocalCli) -> durable_filestor_shared::Result<()> { 
///     Ok(()) 
/// }
/// 
/// service_main!(LocalCli, execute_cli);
/// ```
#[macro_export]
macro_rules! service_main {
    ($cli_type:ty, $execute_fn:path) => {
        #[tokio::main]
        async fn main() -> durable_filestor_shared::Result<()> {
            tracing_subscriber::fmt::init();
            
            let cli = <$cli_type>::parse();
            $execute_fn(cli).await
        }
    };
}

/// Builds a ServiceConfig from CLI arguments with field mappings.
/// 
/// This macro reduces boilerplate when converting CLI options to ServiceConfig.
/// Each field mapping can be an Option<T> expression that will be checked and
/// assigned if Some().
/// 
/// # Arguments
/// * `$cli` - The CLI struct instance
/// * Field mappings in the form `field_name => expression`
/// 
/// # Example
/// ```rust
/// use durable_filestor_shared::{build_service_config, ServiceConfig};
/// 
/// struct MockCli {
///     path: Option<String>,
///     permissions: Option<String>,
/// }
/// 
/// let cli = MockCli {
///     path: Some("test".to_string()),
///     permissions: Some("644".to_string()),
/// };
/// 
/// let config = build_service_config!(cli, {
///     endpoint => cli.path.clone(),
///     default_permissions => cli.permissions.clone()
/// });
/// ```
#[macro_export]
macro_rules! build_service_config {
    ($cli:expr, { $($field:ident => $expr:expr),* $(,)? }) => {
        {
            let mut config = durable_filestor_shared::ServiceConfig::default();
            $(
                if let Some(value) = $expr {
                    config.$field = Some(value);
                }
            )*
            config
        }
    };
}

/// Creates standard storage service commands enum.
/// 
/// This macro generates a Subcommand enum with common storage operations.
/// Each variant can have custom fields specified.
/// 
/// # Arguments
/// * `$name` - The enum name
/// * Command variants with their fields
/// 
/// # Example
/// ```rust
/// use durable_filestor_shared::define_storage_commands;
/// 
/// define_storage_commands!(LocalCommands, {
///     List { path: String },
///     Upload { source: String, target: String },
///     Download { source: String, target: String },
///     Head { path: String }
/// });
/// ```
#[macro_export]
macro_rules! define_storage_commands {
    ($name:ident, { $($variant:ident { $($field:ident: $field_type:ty),* $(,)? }),* $(,)? }) => {
        #[derive(clap::Subcommand, Debug)]
        pub enum $name {
            $(
                $variant {
                    $(
                        $field: $field_type,
                    )*
                }
            ),*
        }
    };
}

/// Creates a tokio test function with standard service setup.
/// 
/// This macro reduces boilerplate in service tests by providing standard
/// async test setup with service configuration.
/// 
/// # Arguments
/// * `$test_name` - Name of the test function
/// * `$setup` - Expression to create the service (receives ServiceConfig)
/// * `$test_body` - The test body expression (receives the service)
/// 
/// # Example
/// ```rust,no_run
/// use durable_filestor_shared::{service_test, ServiceConfig, Result};
/// 
/// struct MockService;
/// impl MockService {
///     fn new(_config: ServiceConfig) -> Self { Self }
///     async fn list(&self, _path: &str) -> Result<Vec<String>> {
///         Ok(vec!["file1".to_string()])
///     }
/// }
/// 
/// service_test!(test_list_operation,
///     |config| MockService::new(config), 
///     |service: MockService| async move {
///         let results = service.list("/").await.unwrap();
///         assert!(!results.is_empty());
///     }
/// );
/// ```
#[macro_export]
macro_rules! service_test {
    ($test_name:ident, $setup:expr, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let config = durable_filestor_shared::ServiceConfig::default();
            let service = ($setup)(config);
            let test_fn = $test_body;
            test_fn(service).await;
        }
    };
}

/// Formats and outputs list results to stdout.
/// 
/// This macro standardizes the output formatting for list operations
/// across all services.
/// 
/// # Arguments
/// * `$items` - Iterator of items to list
/// * `$format_fn` - Function to format each item
/// 
/// # Example
/// ```rust,ignore
/// use durable_filestor_shared::format_list_output;
/// 
/// let results = vec!["file1.txt", "file2.txt"];
/// format_list_output!(results, |item| item)?;
/// ```
#[macro_export]
macro_rules! format_list_output {
    ($items:expr, $format_fn:expr) => {
        {
            use std::io::{self, Write};
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            for item in $items {
                writeln!(handle, "{}", $format_fn(item))?;
            }
        }
    };
}

/// Sets environment variables from optional values.
/// 
/// This macro simplifies setting multiple environment variables
/// where each value might be None.
/// 
/// # Arguments
/// * Key-value pairs where values are Option<T>
/// 
/// # Example
/// ```rust
/// use durable_filestor_shared::set_env_vars;
/// 
/// let region = Some("us-east-1".to_string());
/// let endpoint: Option<String> = None;
/// 
/// set_env_vars!(
///     "AWS_REGION" => region.as_ref(),
///     "AWS_ENDPOINT" => endpoint.as_ref()
/// );
/// ```
#[macro_export]
macro_rules! set_env_vars {
    ($($key:expr => $value:expr),* $(,)?) => {
        $(
            if let Some(value) = $value {
                std::env::set_var($key, value);
            }
        )*
    };
}

/// Wraps a Result with additional error context.
/// 
/// This macro provides a consistent way to add context to errors
/// across the codebase.
/// 
/// # Arguments
/// * `$result` - The Result expression to wrap
/// * `$context` - The context string to add
/// 
/// # Example
/// ```rust
/// use durable_filestor_shared::context_error;
/// 
/// let result = context_error!(
///     std::fs::read_to_string("config.toml"),
///     "Failed to read config file"
/// );
/// ```
#[macro_export]
macro_rules! context_error {
    ($result:expr, $context:expr) => {
        $result.map_err(|e| durable_filestor_shared::DurableError::Generic { 
            message: format!("{}: {}", $context, e) 
        })
    };
}

/// Creates a TOML export operation with standardized error handling.
/// 
/// This macro reduces boilerplate for TOML serialization operations
/// by providing consistent error handling with Config error variant.
/// 
/// # Arguments
/// * `$data` - The data structure to serialize
/// * `$pretty` - Optional boolean for pretty formatting (default: true)
/// 
/// # Example
/// ```rust
/// use durable_filestor_shared::{toml_export, Config};
///
/// let config = Config::default();
/// let result = toml_export!(&config, "config");
/// let pretty_result = toml_export!(&config, "config", true);
/// let compact_result = toml_export!(&config, "config", false);
/// ```
#[macro_export]
macro_rules! toml_export {
    ($data:expr, $operation:expr) => {
        toml::to_string_pretty($data)
            .map_err(|e| durable_filestor_shared::DurableError::Config {
                message: format!("Failed to export {} to TOML: {}", $operation, e),
            })
    };
    ($data:expr, $operation:expr, $pretty:expr) => {
        if $pretty {
            toml::to_string_pretty($data)
                .map_err(|e| durable_filestor_shared::DurableError::Config {
                    message: format!("Failed to export {} to TOML: {}", $operation, e),
                })
        } else {
            toml::to_string($data)
                .map_err(|e| durable_filestor_shared::DurableError::Config {
                    message: format!("Failed to export {} to TOML: {}", $operation, e),
                })
        }
    };
}

/// Creates a TOML validation function with consistent error handling.
/// 
/// This macro generates validation functions that attempt to parse
/// TOML strings back to the original type for round-trip validation.
/// 
/// # Arguments
/// * `$type` - The Rust type to validate against
/// 
/// # Example
/// ```rust
/// use durable_filestor_shared::{toml_validate, Config};
/// 
/// toml_validate!(Config);
/// // This creates a validate_config_toml function
/// ```
#[macro_export]
macro_rules! toml_validate {
    ($type:ty) => {
        paste::paste! {
            pub fn [<validate_ $type:snake _toml>](toml_str: &str) -> durable_filestor_shared::Result<bool> {
                match toml::from_str::<$type>(toml_str) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
        }
    };
}

/// Creates a complete TOML exporter implementation for a type.
/// 
/// This macro generates all common export methods for a given type,
/// including standard export, pretty/compact options, and validation.
/// 
/// # Arguments
/// * `$type` - The Rust type to create exporters for
/// * `$name` - A lowercase name for the type (used in function names)
/// 
/// # Example
/// ```rust
/// use durable_filestor_shared::{impl_toml_exporter, toml_export, Config, Protocol};
///
/// impl_toml_exporter!(Config, "config");
/// impl_toml_exporter!(Protocol, "protocol");
/// ```
#[macro_export]
macro_rules! impl_toml_exporter {
    ($type:ty, $name:expr) => {
        paste::paste! {
            pub fn [<export_ $name>](data: &$type) -> durable_filestor_shared::Result<String> {
                toml_export!(data, $name)
            }
            
            pub fn [<export_ $name _with_options>](data: &$type, pretty: bool) -> durable_filestor_shared::Result<String> {
                toml_export!(data, $name, pretty)
            }
            
            pub fn [<validate_ $name _export>](toml_str: &str) -> durable_filestor_shared::Result<bool> {
                match toml::from_str::<$type>(toml_str) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
        }
    };
}

/// Executes storage service commands with a standard match pattern.
///
/// This macro reduces duplication in command execution by providing
/// a standard pattern for matching commands and calling operations.
///
/// # Arguments
/// * `$service` - The service instance
/// * `$command` - The command to match against
/// * Command patterns and their bodies
///
/// # Example
/// ```rust,ignore
/// use durable_filestor_shared::{execute_storage_command, Result};
/// use clap::Subcommand;
///
/// #[derive(Subcommand)]
/// enum Commands {
///     List { path: String },
///     Upload { source: String, target: String },
/// }
///
/// struct MockService;
/// impl MockService {
///     async fn list(&self, path: &str) -> Result<()> { Ok(()) }
///     async fn upload(&self, source: &str, target: &str) -> Result<()> { Ok(()) }
/// }
///
/// async fn example() -> Result<()> {
///     let service = MockService;
///     let command = Commands::List { path: "/".to_string() };
///
///     execute_storage_command!(service, command, {
///         List(path) => service.list(&path).await?,
///         Upload(source, target) => service.upload(&source, &target).await?
///     });
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! execute_storage_command {
    ($service:expr, $command:expr, {
        $($variant:ident($($param:ident),* $(,)?) => $body:expr),* $(,)?
    }) => {
        match $command {
            $(
                Commands::$variant { $($param),* } => $body,
            )*
        }
    };
}

/// Builds URI components from optional fields with URL encoding.
///
/// This macro reduces duplication when building URI query strings from
/// optional configuration fields. It automatically handles URL encoding
/// and only includes non-empty values.
///
/// # Arguments
/// * `$components` - Mutable vector to collect components
/// * Field mappings in the form `field_name => expression`
///
/// # Example
/// ```rust
/// use durable_filestor_shared::build_uri_components;
/// use urlencoding;
///
/// let mut components = Vec::new();
/// let bucket = Some("my-bucket".to_string());
/// let region: Option<String> = None;
///
/// build_uri_components!(components, {
///     bucket => bucket.as_ref(),
///     region => region.as_ref()
/// });
///
/// assert_eq!(components, vec!["bucket=my-bucket"]);
/// ```
#[macro_export]
macro_rules! build_uri_components {
    ($components:expr, { $($field:ident => $expr:expr),* $(,)? }) => {
        $(
            if let Some(value) = $expr {
                $components.push(format!("{}={}", stringify!($field), urlencoding::encode(value)));
            }
        )*
    };
}

/// Builds URI components from HashMap parameters with URL encoding.
///
/// This macro handles parameter maps commonly used in configuration objects.
/// It automatically URL-encodes both keys and values.
///
/// # Arguments
/// * `$components` - Mutable vector to collect components
/// * `$params` - HashMap containing the parameters
///
/// # Example
/// ```rust
/// use durable_filestor_shared::build_uri_parameters;
/// use std::collections::HashMap;
///
/// let mut components = Vec::new();
/// let mut params = HashMap::new();
/// params.insert("timeout".to_string(), "30".to_string());
/// params.insert("retries".to_string(), "3".to_string());
///
/// build_uri_parameters!(components, params);
///
/// assert!(components.contains(&"timeout=30".to_string()));
/// assert!(components.contains(&"retries=3".to_string()));
/// ```
#[macro_export]
macro_rules! build_uri_parameters {
    ($components:expr, $params:expr) => {
        for (key, value) in &$params {
            $components.push(format!("{}={}",
                urlencoding::encode(key),
                urlencoding::encode(value)
            ));
        }
    };
}

/// Builds URI components from KeyTranslation fields.
///
/// This macro handles the complex key translation configuration by
/// building appropriate URI components for each translation option.
///
/// # Arguments
/// * `$components` - Mutable vector to collect components
/// * `$key_translation` - KeyTranslation instance
/// * `$hash_fn` - Function to convert HashFunction to string
///
/// # Example
/// ```rust,ignore
/// use durable_filestor_shared::{build_key_translation_components, KeyTranslation, HashFunction};
///
/// let mut components = Vec::new();
/// let key_translation = KeyTranslation {
///     downcase: true,
///     hash_function: Some(HashFunction::Sha256),
///     ..Default::default()
/// };
///
/// fn hash_to_string(hf: &HashFunction) -> &'static str {
///     match hf {
///         HashFunction::Sha256 => "sha256",
///         _ => "unknown"
///     }
/// }
///
/// build_key_translation_components!(components, key_translation, hash_to_string);
///
/// assert!(components.contains(&"downcase".to_string()));
/// assert!(components.contains(&"hash=sha256".to_string()));
/// ```
#[macro_export]
macro_rules! build_key_translation_components {
    ($components:expr, $key_translation:expr, $hash_fn:expr) => {
        if $key_translation.downcase {
            $components.push("downcase".to_string());
        }

        if $key_translation.upcase {
            $components.push("upcase".to_string());
        }

        if let Some(ref hash_func) = $key_translation.hash_function {
            $components.push(format!("hash={}", $hash_fn(hash_func)));
        }

        if let Some(length) = $key_translation.hash_prefix_length {
            $components.push(format!("hash_prefix_length={}", length));
        }

        if $key_translation.partition {
            $components.push(format!("partition_length={}", $key_translation.partition_length));
            $components.push(format!("partition_depth={}", $key_translation.partition_depth));
            $components.push(format!("partition_separator={}", urlencoding::encode(&$key_translation.partition_separator)));
        }

        if let Some(ref prefix) = $key_translation.prefix {
            $components.push(format!("prefix={}", urlencoding::encode(prefix)));
        }

        if let Some(ref suffix) = $key_translation.suffix {
            $components.push(format!("suffix={}", urlencoding::encode(suffix)));
        }

        if $key_translation.timestamp_suffix {
            $components.push("timestamp_suffix".to_string());
        }

        if $key_translation.normalize_unicode {
            $components.push("normalize_unicode".to_string());
        }

        if $key_translation.strip_whitespace {
            $components.push("strip_whitespace".to_string());
        }

        if let Some(ref replace_spaces) = $key_translation.replace_spaces {
            $components.push(format!("replace_spaces={}", urlencoding::encode(replace_spaces)));
        }

        if let Some(length) = $key_translation.max_key_length {
            $components.push(format!("max_key_length={}", length));
        }
    };
}

/// Joins URI components into a query string or component string.
///
/// This macro provides consistent formatting for joining URI components
/// with appropriate separators.
///
/// # Arguments
/// * `$components` - Vector of components to join
/// * `$separator` - Separator string (e.g., "&" for query strings, "," for key translation)
/// * `$empty_value` - Value to return if components is empty
///
/// # Example
/// ```rust
/// use durable_filestor_shared::join_uri_components;
///
/// let components = vec!["key1=value1".to_string(), "key2=value2".to_string()];
/// let query = join_uri_components!(components, "&", "");
/// assert_eq!(query, "key1=value1&key2=value2");
///
/// let empty_components: Vec<String> = vec![];
/// let empty_result = join_uri_components!(empty_components, "&", "empty");
/// assert_eq!(empty_result, "empty");
/// ```
#[macro_export]
macro_rules! join_uri_components {
    ($components:expr, $separator:expr, $empty_value:expr) => {
        if $components.is_empty() {
            $empty_value.to_string()
        } else {
            $components.join($separator)
        }
    };
}

/// Creates a URI string with scheme and components.
///
/// This macro standardizes URI construction across different export functions.
///
/// # Arguments
/// * `$scheme` - URI scheme (e.g., "config", "protocol", "service")
/// * `$components` - Components to include in the URI
/// * `$separator` - Separator for components
/// * `$empty_value` - Value when no components
///
/// # Example
/// ```rust
/// use durable_filestor_shared::build_uri_string;
///
/// let components = vec!["key1=value1".to_string(), "key2=value2".to_string()];
/// let uri = build_uri_string!("config", components, ";", "empty");
/// assert_eq!(uri, "config://key1=value1;key2=value2");
///
/// let empty_components: Vec<String> = vec![];
/// let empty_uri = build_uri_string!("config", empty_components, ";", "empty");
/// assert_eq!(empty_uri, "config://empty");
/// ```
#[macro_export]
macro_rules! build_uri_string {
    ($scheme:expr, $components:expr, $separator:expr, $empty_value:expr) => {
        if $components.is_empty() {
            format!("{}://{}", $scheme, $empty_value)
        } else {
            format!("{}://{}", $scheme, $components.join($separator))
        }
    };
}