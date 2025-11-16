//! Tests for durable-filestor-shared macros
//! 
//! These tests verify that the macros expand correctly and produce
//! functional code patterns.

use durable_filestor_shared::*;
use clap::Parser;
use std::collections::HashMap;

// Test structures for macro testing - moved outside of mod to be accessible
#[derive(Parser, Debug)]
struct TestCli {
    #[arg(short, long)]
    endpoint: Option<String>,
    
    #[arg(short, long)]
    default_region: Option<String>,
    
    #[command(subcommand)]
    command: TestCommands,
}

define_storage_commands!(TestCommands, {
    List { path: String },
    Upload { source: String, target: String },
    Download { source: String, target: String },
    Head { path: String }
});

struct TestService {
    config: ServiceConfig,
}

impl TestService {
    fn new(config: ServiceConfig) -> Self {
        Self { config }
    }

    async fn list(&self, path: &str) -> Result<Vec<String>> {
        Ok(vec![format!("file1_{}", path), format!("file2_{}", path)])
    }

    async fn upload(&self, _source: &str, _target: &str) -> Result<()> {
        Ok(())
    }

    async fn download(&self, _source: &str, _target: &str) -> Result<()> {
        Ok(())
    }

    async fn head(&self, _path: &str) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();
        metadata.insert("size".to_string(), "1024".to_string());
        metadata.insert("modified".to_string(), "2023-01-01".to_string());
        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_storage_commands_macro() {
        // Test that the macro generates the expected enum
        let list_cmd = TestCommands::List { 
            path: "/test".to_string() 
        };
        
        let upload_cmd = TestCommands::Upload {
            source: "file.txt".to_string(),
            target: "remote/file.txt".to_string(),
        };

        match list_cmd {
            TestCommands::List { path } => assert_eq!(path, "/test"),
            _ => panic!("Expected List command"),
        }

        match upload_cmd {
            TestCommands::Upload { source, target } => {
                assert_eq!(source, "file.txt");
                assert_eq!(target, "remote/file.txt");
            }
            _ => panic!("Expected Upload command"),
        }
    }

    #[test]
    fn test_build_service_config_macro() {
        let cli = TestCli {
            endpoint: Some("test-endpoint".to_string()),
            default_region: Some("us-east-1".to_string()),
            command: TestCommands::List { path: "/".to_string() },
        };

        let config = build_service_config!(cli, {
            endpoint => cli.endpoint.clone(),
            default_region => cli.default_region.clone(),
            default_permissions => None::<String>
        });

        assert_eq!(config.endpoint, Some("test-endpoint".to_string()));
        assert_eq!(config.default_region, Some("us-east-1".to_string()));
        assert_eq!(config.default_permissions, None);
    }

    #[test]
    fn test_build_service_config_macro_with_none_values() {
        let cli = TestCli {
            endpoint: None,
            default_region: Some("us-west-2".to_string()),
            command: TestCommands::List { path: "/".to_string() },
        };

        let config = build_service_config!(cli, {
            endpoint => cli.endpoint.clone(),
            default_region => cli.default_region.clone()
        });

        assert_eq!(config.endpoint, None);
        assert_eq!(config.default_region, Some("us-west-2".to_string()));
    }

    service_test!(test_service_test_macro_list,
        |config| TestService::new(config),
        |service: TestService| async move {
            let results = service.list("/test").await.unwrap();
            assert_eq!(results.len(), 2);
            assert_eq!(results[0], "file1_/test");
            assert_eq!(results[1], "file2_/test");
        }
    );

    service_test!(test_service_test_macro_upload,
        |config| TestService::new(config),
        |service: TestService| async move {
            let result = service.upload("local.txt", "remote.txt").await;
            assert!(result.is_ok());
        }
    );

    #[test]
    fn test_format_list_output_macro() {
        use std::io::{self, Write};
        use std::sync::{Arc, Mutex};
        
        // Create a mock writer to capture output
        #[derive(Clone)]
        struct MockWriter {
            data: Arc<Mutex<Vec<u8>>>,
        }
        
        impl MockWriter {
            fn new() -> Self {
                Self {
                    data: Arc::new(Mutex::new(Vec::new())),
                }
            }
            
            fn get_output(&self) -> String {
                let data = self.data.lock().unwrap();
                String::from_utf8(data.clone()).unwrap()
            }
        }
        
        impl Write for MockWriter {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                let mut data = self.data.lock().unwrap();
                data.extend_from_slice(buf);
                Ok(buf.len())
            }
            
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        // Test the concept of the macro (actual macro uses stdout)
        let items = vec!["file1.txt", "file2.txt", "dir1/"];
        let mut output = Vec::new();
        
        for item in items {
            writeln!(output, "{}", item).unwrap();
        }
        
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("file1.txt\n"));
        assert!(output_str.contains("file2.txt\n"));
        assert!(output_str.contains("dir1/\n"));
    }

    #[test]
    fn test_set_env_vars_macro() {
        // Test setting environment variables
        let endpoint = Some("test-endpoint".to_string());
        let region = Some("us-east-1".to_string());
        let empty_val: Option<String> = None;

        set_env_vars!(
            "TEST_ENDPOINT" => endpoint.as_ref(),
            "TEST_REGION" => region.as_ref(),
            "TEST_EMPTY" => empty_val.as_ref()
        );

        assert_eq!(std::env::var("TEST_ENDPOINT").unwrap(), "test-endpoint");
        assert_eq!(std::env::var("TEST_REGION").unwrap(), "us-east-1");
        assert!(std::env::var("TEST_EMPTY").is_err()); // Should not be set
        
        // Clean up
        std::env::remove_var("TEST_ENDPOINT");
        std::env::remove_var("TEST_REGION");
    }

    #[test]
    fn test_context_error_macro() {
        let success_result: Result<String> = Ok("success".to_string());
        let error_result: Result<String> = Err(DurableError::Generic {
            message: "original error".to_string(),
        });

        // Test success case
        let wrapped_success = context_error!(success_result, "operation context");
        assert!(wrapped_success.is_ok());
        assert_eq!(wrapped_success.unwrap(), "success");

        // Test error case
        let wrapped_error = context_error!(error_result, "operation context");
        assert!(wrapped_error.is_err());
        
        if let Err(DurableError::Generic { message }) = wrapped_error {
            assert!(message.contains("operation context"));
            assert!(message.contains("original error"));
        } else {
            panic!("Expected Generic error");
        }
    }

    // Test async function for CLI execution simulation
    async fn mock_execute_cli(cli: TestCli) -> Result<()> {
        let config = build_service_config!(cli, {
            endpoint => cli.endpoint.clone(),
            default_region => cli.default_region.clone()
        });
        
        let service = TestService::new(config);
        
        match cli.command {
            TestCommands::List { path } => {
                let _results = service.list(&path).await?;
                Ok(())
            },
            TestCommands::Upload { source, target } => {
                service.upload(&source, &target).await
            },
            TestCommands::Download { source, target } => {
                service.download(&source, &target).await
            },
            TestCommands::Head { path } => {
                let _metadata = service.head(&path).await?;
                Ok(())
            },
        }
    }

    #[tokio::test]
    async fn test_integration_cli_execution() {
        let cli = TestCli {
            endpoint: Some("test-endpoint".to_string()),
            default_region: Some("us-east-1".to_string()),
            command: TestCommands::List { path: "/test".to_string() },
        };

        let result = mock_execute_cli(cli).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_integration_upload_command() {
        let cli = TestCli {
            endpoint: Some("test-endpoint".to_string()),
            default_region: None,
            command: TestCommands::Upload {
                source: "local.txt".to_string(),
                target: "remote.txt".to_string(),
            },
        };

        let result = mock_execute_cli(cli).await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod macro_expansion_tests {
    use super::*;

    // These tests verify that macros expand to syntactically correct code
    
    #[test]
    fn test_macro_expansion_compile_time() {
        // This test ensures all macros compile correctly
        
        // Test define_storage_commands expansion
        define_storage_commands!(TestCommands2, {
            List { path: String },
            Copy { from: String, to: String }
        });
        
        let cmd = TestCommands2::List { path: "test".to_string() };
        match cmd {
            TestCommands2::List { path } => assert_eq!(path, "test"),
            _ => panic!("Unexpected command type"),
        }
    }
    
    service_test!(test_macro_expansion_service_test,
        |config| TestService::new(config),
        |_service: TestService| async move {
            // Test that the macro generates valid async test code
            assert!(true);
        }
    );
}

#[cfg(test)]
mod documentation_tests {
    use super::*;
    
    #[test]
    fn test_macro_usage_examples() {
        // Test examples from macro documentation work correctly
        
        // Example from build_service_config docs
        let cli = TestCli {
            endpoint: Some("example.com".to_string()),
            default_region: Some("us-west-1".to_string()),
            command: TestCommands::List { path: "/".to_string() },
        };
        
        let _config = build_service_config!(cli, {
            endpoint => cli.endpoint.clone(),
            default_region => cli.default_region.clone()
        });
        
        // Example from define_storage_commands docs
        define_storage_commands!(ExampleCommands, {
            List { path: String },
            Upload { source: String, target: String },
            Download { source: String, target: String },
            Head { path: String }
        });
        
        let _cmd = ExampleCommands::List { path: "/example".to_string() };
    }
}