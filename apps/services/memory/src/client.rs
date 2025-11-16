//! Memory storage client for communicating with the server

use crate::operations::MemoryStats;
use crate::server::{ServerRequest, ServerResponse};
use durable_filestor_shared::{DurableError, Result, ServiceConfig};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::process::Command;
use tracing::{info, warn};

/// Memory storage client
#[derive(Clone)]
pub struct MemoryClient {
    socket_path: String,
    service_config: ServiceConfig,
}

impl MemoryClient {
    /// Create a new client instance
    pub fn new(service_config: ServiceConfig) -> Self {
        let socket_path = service_config
            .parameters
            .get("socket_path")
            .map(|s| s.clone())
            .unwrap_or_else(|| "/tmp/durable-filestor-memory.sock".to_string());

        Self {
            socket_path,
            service_config,
        }
    }

    /// Ensure the server is running, starting it if necessary
    pub async fn ensure_server(&self) -> Result<()> {
        // Check if socket exists
        if std::path::Path::new(&self.socket_path).exists() {
            // Try to ping the server
            if self.ping().await.is_ok() {
                return Ok(());
            }
            // Socket exists but server not responding, remove it
            if let Err(e) = tokio::fs::remove_file(&self.socket_path).await {
                warn!("Failed to remove stale socket: {:?}", e);
            }
        }

        // Start the server
        info!("Starting memory server...");
        self.start_server().await
    }

    /// Start the server process
    async fn start_server(&self) -> Result<()> {
        // Try to find the memory service binary
        let exe_path = if let Ok(current_exe) = std::env::current_exe() {
            // If we're running from the memory service binary directly
            if current_exe.to_string_lossy().contains("durable-filestor-memory") {
                current_exe
            } else {
                // Try to find the memory service binary in the same directory
                let mut memory_exe = current_exe.parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf();
                memory_exe.push("durable-filestor-memory");
                if memory_exe.exists() {
                    memory_exe
                } else {
                    // Fallback to current exe (for tests)
                    current_exe
                }
            }
        } else {
            return Err(DurableError::Storage {
                message: "Failed to get current executable path".to_string(),
            });
        };

        let mut cmd = Command::new(exe_path);

        cmd.arg("--server")
            .arg("--socket-path")
            .arg(self.socket_path.as_str());

        // Pass TTL if configured
        if let Some(ttl) = self.service_config.parameters.get("ttl") {
            cmd.arg("--ttl").arg(ttl);
        }

        // Pass timeout if configured
        if let Some(timeout) = self.service_config.parameters.get("timeout") {
            cmd.arg("--timeout").arg(timeout);
        }

        // Start in daemon mode by default
        cmd.arg("--daemon");

        let mut child = cmd.spawn().map_err(|e| DurableError::Storage {
            message: format!("Failed to start server: {}", e),
        })?;

        // Give server time to start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Wait for socket to be created
        let mut attempts = 0;
        while attempts < 50 { // 5 seconds max
            if std::path::Path::new(&self.socket_path).exists() {
                // Try to ping to make sure it's ready
                if self.ping().await.is_ok() {
                    info!("Memory server started successfully");
                    return Ok(());
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            attempts += 1;
        }

        // Clean up child process if it failed
        let _ = child.kill().await;
        let _ = child.wait().await;

        Err(DurableError::Storage {
            message: "Server failed to start within timeout".to_string(),
        })
    }

    /// Ping the server to check if it's alive
    pub async fn ping(&self) -> Result<()> {
        self.send_request(ServerRequest::Ping).await?;
        Ok(())
    }

    /// List keys with optional prefix
    pub async fn list(&self, prefix: &str) -> Result<Vec<crate::operations::MemoryFileMetadata>> {
        self.ensure_server().await?;
        match self.send_request(ServerRequest::List {
            prefix: prefix.to_string(),
        }).await? {
            ServerResponse::List(metadata) => Ok(metadata),
            ServerResponse::Error(msg) => Err(DurableError::Storage { message: msg }),
            _ => Err(DurableError::Storage {
                message: "Unexpected response type".to_string(),
            }),
        }
    }

    /// Upload data to the server
    pub async fn upload(&self, key: &str, data: Vec<u8>) -> Result<()> {
        self.ensure_server().await?;
        match self.send_request(ServerRequest::Upload {
            key: key.to_string(),
            data,
        }).await? {
            ServerResponse::Upload => Ok(()),
            ServerResponse::Error(msg) => Err(DurableError::Storage { message: msg }),
            _ => Err(DurableError::Storage {
                message: "Unexpected response type".to_string(),
            }),
        }
    }

    /// Download data from the server
    pub async fn download(&self, key: &str) -> Result<Vec<u8>> {
        self.ensure_server().await?;
        match self.send_request(ServerRequest::Download {
            key: key.to_string(),
        }).await? {
            ServerResponse::Download(data) => Ok(data),
            ServerResponse::Error(msg) => Err(DurableError::Storage { message: msg }),
            _ => Err(DurableError::Storage {
                message: "Unexpected response type".to_string(),
            }),
        }
    }

    /// Get metadata for a key
    pub async fn head(&self, key: &str) -> Result<crate::operations::MemoryFileMetadata> {
        self.ensure_server().await?;
        match self.send_request(ServerRequest::Head {
            key: key.to_string(),
        }).await? {
            ServerResponse::Head(metadata) => Ok(metadata),
            ServerResponse::Error(msg) => Err(DurableError::Storage { message: msg }),
            _ => Err(DurableError::Storage {
                message: "Unexpected response type".to_string(),
            }),
        }
    }

    /// Delete a key from the server
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.ensure_server().await?;
        match self.send_request(ServerRequest::Delete {
            key: key.to_string(),
        }).await? {
            ServerResponse::Delete => Ok(()),
            ServerResponse::Error(msg) => Err(DurableError::Storage { message: msg }),
            _ => Err(DurableError::Storage {
                message: "Unexpected response type".to_string(),
            }),
        }
    }

    /// Copy data within the server
    pub async fn copy(&self, source: &str, target: &str) -> Result<()> {
        self.ensure_server().await?;
        match self.send_request(ServerRequest::Copy {
            source: source.to_string(),
            target: target.to_string(),
        }).await? {
            ServerResponse::Copy => Ok(()),
            ServerResponse::Error(msg) => Err(DurableError::Storage { message: msg }),
            _ => Err(DurableError::Storage {
                message: "Unexpected response type".to_string(),
            }),
        }
    }

    /// Get server statistics
    pub async fn stats(&self) -> Result<MemoryStats> {
        self.ensure_server().await?;
        match self.send_request(ServerRequest::Stats).await? {
            ServerResponse::Stats(stats) => Ok(stats),
            ServerResponse::Error(msg) => Err(DurableError::Storage { message: msg }),
            _ => Err(DurableError::Storage {
                message: "Unexpected response type".to_string(),
            }),
        }
    }

    /// Trigger cleanup on the server
    pub async fn cleanup(&self) -> Result<usize> {
        self.ensure_server().await?;
        match self.send_request(ServerRequest::Cleanup).await? {
            ServerResponse::Cleanup(count) => Ok(count),
            ServerResponse::Error(msg) => Err(DurableError::Storage { message: msg }),
            _ => Err(DurableError::Storage {
                message: "Unexpected response type".to_string(),
            }),
        }
    }

    /// Send a request to the server and get response
    async fn send_request(&self, request: ServerRequest) -> Result<ServerResponse> {
        let stream = UnixStream::connect(&self.socket_path).await
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to connect to server: {}", e),
            })?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send request
        let request_json = serde_json::to_string(&request)
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to serialize request: {}", e),
            })?;

        writer.write_all(format!("{}\n", request_json).as_bytes()).await
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to send request: {}", e),
            })?;

        writer.flush().await
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to flush request: {}", e),
            })?;

        // Read response
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to read response: {}", e),
            })?;

        let response: ServerResponse = serde_json::from_str(&response_line.trim())
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to parse response: {}", e),
            })?;

        Ok(response)
    }
}

/// Wrapper that provides the same interface as MemoryService using the client
pub struct MemoryClientWrapper {
    client: MemoryClient,
}

impl MemoryClientWrapper {
    pub fn new(service_config: ServiceConfig) -> Self {
        Self {
            client: MemoryClient::new(service_config),
        }
    }

    pub async fn list(&self, prefix: &str) -> Result<Vec<crate::operations::MemoryFileMetadata>> {
        self.client.list(prefix).await
    }

    pub async fn upload(&self, key: &str, data: Vec<u8>) -> Result<()> {
        self.client.upload(key, data).await
    }

    pub async fn download(&self, key: &str) -> Result<Vec<u8>> {
        self.client.download(key).await
    }

    pub async fn head(&self, key: &str) -> Result<crate::operations::MemoryFileMetadata> {
        self.client.head(key).await
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        self.client.delete(key).await
    }

    pub async fn copy(&self, source: &str, target: &str) -> Result<()> {
        self.client.copy(source, target).await
    }

    pub async fn stats(&self) -> Result<MemoryStats> {
        self.client.stats().await
    }

    pub async fn cleanup(&self) -> Result<usize> {
        self.client.cleanup().await
    }
}