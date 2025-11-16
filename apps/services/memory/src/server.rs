//! Memory storage server implementation

use crate::operations::{MemoryService, MemoryStats};
use durable_filestor_shared::{DurableError, Result, ServiceConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::signal;
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{info, warn, error};

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub socket_path: PathBuf,
    pub timeout_duration: Duration,
    pub daemon_mode: bool,
    pub service_config: ServiceConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            socket_path: PathBuf::from("/tmp/durable-filestor-memory.sock"),
            timeout_duration: Duration::from_secs(3600), // 1 hour
            daemon_mode: true,
            service_config: ServiceConfig::default(),
        }
    }
}

/// Request types that can be sent to the server
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerRequest {
    List { prefix: String },
    Upload { key: String, data: Vec<u8> },
    Download { key: String },
    Head { key: String },
    Delete { key: String },
    Copy { source: String, target: String },
    Stats,
    Cleanup,
    Ping,
    Shutdown,
}

/// Response types from the server
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerResponse {
    List(Vec<crate::operations::MemoryFileMetadata>),
    Upload,
    Download(Vec<u8>),
    Head(crate::operations::MemoryFileMetadata),
    Delete,
    Copy,
    Stats(MemoryStats),
    Cleanup(usize),
    Ping,
    Error(String),
}

/// Memory storage server
pub struct MemoryServer {
    config: ServerConfig,
    service: MemoryService,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl MemoryServer {
    /// Create a new server instance
    pub fn new(config: ServerConfig) -> Self {
        let service = MemoryService::new(config.service_config.clone());
        Self {
            config,
            service,
            shutdown_tx: None,
        }
    }

    /// Start the server
    pub async fn start(&mut self) -> Result<()> {
        // Remove existing socket file if it exists
        if self.config.socket_path.exists() {
            tokio::fs::remove_file(&self.config.socket_path).await
                .map_err(|e| DurableError::Storage {
                    message: format!("Failed to remove existing socket: {}", e),
                })?;
        }

        let listener = UnixListener::bind(&self.config.socket_path)
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to bind to socket: {}", e),
            })?;

        info!("Memory server started on socket: {:?}", self.config.socket_path);

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Clone what we need for the async task
        let service = self.service.clone();
        let timeout_duration = self.config.timeout_duration;
        let socket_path = self.config.socket_path.clone();
        let daemon_mode = self.config.daemon_mode;

        // Spawn cleanup task
        let cleanup_service = service.clone();
        let cleanup_interval = Duration::from_secs(60); // Cleanup every minute
        tokio::spawn(async move {
            let mut interval = interval(cleanup_interval);
            loop {
                interval.tick().await;
                if let Err(e) = cleanup_service.cleanup().await {
                    warn!("Cleanup task error: {:?}", e);
                }
            }
        });

        // Handle incoming connections
        tokio::spawn(async move {
            let mut last_request_time = std::time::Instant::now();

            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _)) => {
                                last_request_time = std::time::Instant::now();
                                let service_clone = service.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = handle_connection(stream, service_clone).await {
                                        error!("Connection error: {:?}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                error!("Accept error: {:?}", e);
                                break;
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Shutdown signal received");
                        break;
                    }
                    _ = tokio::time::sleep(timeout_duration) => {
                        let time_since_last_request = std::time::Instant::now().duration_since(last_request_time);
                        if time_since_last_request >= timeout_duration {
                            info!("Timeout reached, shutting down server");
                            break;
                        }
                    }
                }
            }

            // Cleanup socket file
            if let Err(e) = tokio::fs::remove_file(&socket_path).await {
                warn!("Failed to remove socket file: {:?}", e);
            }
        });

        // If not in daemon mode, wait for shutdown signal
        if !daemon_mode {
            signal::ctrl_c().await
                .map_err(|e| DurableError::Storage {
                    message: format!("Failed to listen for shutdown signal: {}", e),
                })?;
        }

        Ok(())
    }

    /// Stop the server
    pub async fn stop(&self) -> Result<()> {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(()).await;
        }
        Ok(())
    }
}

/// Handle a single client connection
async fn handle_connection(mut stream: UnixStream, service: MemoryService) -> Result<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Read request
    let n = reader.read_line(&mut line).await
        .map_err(|e| DurableError::Storage {
            message: format!("Failed to read request: {}", e),
        })?;

    if n == 0 {
        return Ok(()); // Connection closed
    }

    // Parse request
    let request: ServerRequest = serde_json::from_str(&line.trim())
        .map_err(|e| DurableError::Storage {
            message: format!("Failed to parse request: {}", e),
        })?;

    // Process request
    let response = match request {
        ServerRequest::List { prefix } => {
            match service.list(&prefix).await {
                Ok(metadata) => ServerResponse::List(metadata),
                Err(e) => ServerResponse::Error(format!("{:?}", e)),
            }
        }
        ServerRequest::Upload { key, data } => {
            match service.upload(&key, data).await {
                Ok(()) => ServerResponse::Upload,
                Err(e) => ServerResponse::Error(format!("{:?}", e)),
            }
        }
        ServerRequest::Download { key } => {
            match service.download(&key).await {
                Ok(data) => ServerResponse::Download(data),
                Err(e) => ServerResponse::Error(format!("{:?}", e)),
            }
        }
        ServerRequest::Head { key } => {
            match service.head(&key).await {
                Ok(metadata) => ServerResponse::Head(metadata),
                Err(e) => ServerResponse::Error(format!("{:?}", e)),
            }
        }
        ServerRequest::Delete { key } => {
            match service.delete(&key).await {
                Ok(()) => ServerResponse::Delete,
                Err(e) => ServerResponse::Error(format!("{:?}", e)),
            }
        }
        ServerRequest::Copy { source, target } => {
            match service.copy(&source, &target).await {
                Ok(()) => ServerResponse::Copy,
                Err(e) => ServerResponse::Error(format!("{:?}", e)),
            }
        }
        ServerRequest::Stats => {
            match service.stats().await {
                Ok(stats) => ServerResponse::Stats(stats),
                Err(e) => ServerResponse::Error(format!("{:?}", e)),
            }
        }
        ServerRequest::Cleanup => {
            match service.cleanup().await {
                Ok(count) => ServerResponse::Cleanup(count),
                Err(e) => ServerResponse::Error(format!("{:?}", e)),
            }
        }
        ServerRequest::Ping => ServerResponse::Ping,
        ServerRequest::Shutdown => {
            // This would be handled by the main server loop
            ServerResponse::Error("Shutdown not implemented via request".to_string())
        }
    };

    // Send response
    let response_json = serde_json::to_string(&response)
        .map_err(|e| DurableError::Storage {
            message: format!("Failed to serialize response: {}", e),
        })?;

    writer.write_all(format!("{}\n", response_json).as_bytes()).await
        .map_err(|e| DurableError::Storage {
            message: format!("Failed to write response: {}", e),
        })?;

    writer.flush().await
        .map_err(|e| DurableError::Storage {
            message: format!("Failed to flush response: {}", e),
        })?;

    Ok(())
}