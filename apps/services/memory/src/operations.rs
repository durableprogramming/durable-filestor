//! Core operations for in-memory storage service

use durable_filestor_shared::{DurableError, Result, ServiceConfig};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// File metadata information for memory objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFileMetadata {
    pub key: String,
    pub size: u64,
    pub created: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

/// In-memory file data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFile {
    pub data: Vec<u8>,
    pub metadata: MemoryFileMetadata,
}

/// In-memory storage service
#[derive(Debug, Clone)]
pub struct MemoryService {
    data: Arc<RwLock<HashMap<String, MemoryFile>>>,
    ttl_duration: Option<Duration>,
}

impl MemoryService {
    /// Create a new memory service instance
    pub fn new(config: ServiceConfig) -> Self {
        let ttl_duration = config.parameters
            .get("ttl")
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs);

        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            ttl_duration,
        }
    }

    /// List all keys in memory storage
    pub async fn list(&self, prefix: &str) -> Result<Vec<MemoryFileMetadata>> {
        let data = self.data.read().map_err(|_| DurableError::Storage {
            message: "Failed to acquire read lock".to_string(),
        })?;

        let mut results = Vec::new();
        let now = Utc::now();

        for (key, file) in data.iter() {
            // Check TTL
            if let Some(ttl) = self.ttl_duration {
                let time_since_access = now.signed_duration_since(file.metadata.last_accessed);
                if time_since_access.num_seconds() > ttl.as_secs() as i64 {
                    continue; // Skip expired entries
                }
            }

            if key.starts_with(prefix) {
                let mut metadata = file.metadata.clone();
                metadata.last_accessed = now; // Update access time
                results.push(metadata);
            }
        }

        // Update access times in the actual data
        drop(data);
        let mut data = self.data.write().map_err(|_| DurableError::Storage {
            message: "Failed to acquire write lock".to_string(),
        })?;

        for result in &results {
            if let Some(file) = data.get_mut(&result.key) {
                file.metadata.last_accessed = now;
            }
        }

        Ok(results)
    }

    /// Upload data to memory storage
    pub async fn upload(&self, key: &str, data: Vec<u8>) -> Result<()> {
        let mut storage = self.data.write().map_err(|_| DurableError::Storage {
            message: "Failed to acquire write lock".to_string(),
        })?;

        let now = Utc::now();
        let metadata = MemoryFileMetadata {
            key: key.to_string(),
            size: data.len() as u64,
            created: now,
            last_accessed: now,
        };

        let file = MemoryFile { data, metadata };
        storage.insert(key.to_string(), file);

        Ok(())
    }

    /// Download data from memory storage
    pub async fn download(&self, key: &str) -> Result<Vec<u8>> {
        let mut storage = self.data.write().map_err(|_| DurableError::Storage {
            message: "Failed to acquire write lock".to_string(),
        })?;

        let file = storage.get_mut(key).ok_or_else(|| DurableError::Storage {
            message: format!("Key '{}' not found", key),
        })?;

        // Check TTL
        if let Some(ttl) = self.ttl_duration {
            let now = Utc::now();
            let time_since_access = now.signed_duration_since(file.metadata.last_accessed);
            if time_since_access.num_seconds() > ttl.as_secs() as i64 {
                storage.remove(key); // Remove expired entry
                return Err(DurableError::Storage {
                    message: format!("Key '{}' has expired", key),
                });
            }
            file.metadata.last_accessed = now;
        }

        Ok(file.data.clone())
    }

    /// Get metadata for a key
    pub async fn head(&self, key: &str) -> Result<MemoryFileMetadata> {
        let mut storage = self.data.write().map_err(|_| DurableError::Storage {
            message: "Failed to acquire write lock".to_string(),
        })?;

        let file = storage.get_mut(key).ok_or_else(|| DurableError::Storage {
            message: format!("Key '{}' not found", key),
        })?;

        // Check TTL
        if let Some(ttl) = self.ttl_duration {
            let now = Utc::now();
            let time_since_access = now.signed_duration_since(file.metadata.last_accessed);
            if time_since_access.num_seconds() > ttl.as_secs() as i64 {
                storage.remove(key); // Remove expired entry
                return Err(DurableError::Storage {
                    message: format!("Key '{}' has expired", key),
                });
            }
            file.metadata.last_accessed = now;
        }

        Ok(file.metadata.clone())
    }

    /// Delete a key from memory storage
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut storage = self.data.write().map_err(|_| DurableError::Storage {
            message: "Failed to acquire write lock".to_string(),
        })?;

        if storage.remove(key).is_none() {
            return Err(DurableError::Storage {
                message: format!("Key '{}' not found", key),
            });
        }

        Ok(())
    }

    /// Copy data within memory storage
    pub async fn copy(&self, source_key: &str, target_key: &str) -> Result<()> {
        let mut storage = self.data.write().map_err(|_| DurableError::Storage {
            message: "Failed to acquire write lock".to_string(),
        })?;

        let source_file = storage.get(source_key).ok_or_else(|| DurableError::Storage {
            message: format!("Source key '{}' not found", source_key),
        })?;

        // Check TTL for source
        if let Some(ttl) = self.ttl_duration {
            let now = Utc::now();
            let time_since_access = now.signed_duration_since(source_file.metadata.last_accessed);
            if time_since_access.num_seconds() > ttl.as_secs() as i64 {
                storage.remove(source_key); // Remove expired entry
                return Err(DurableError::Storage {
                    message: format!("Source key '{}' has expired", source_key),
                });
            }
        }

        let now = Utc::now();
        let mut target_metadata = source_file.metadata.clone();
        target_metadata.key = target_key.to_string();
        target_metadata.created = now;
        target_metadata.last_accessed = now;

        let target_file = MemoryFile {
            data: source_file.data.clone(),
            metadata: target_metadata,
        };

        storage.insert(target_key.to_string(), target_file);
        Ok(())
    }

    /// Get statistics about the memory storage
    pub async fn stats(&self) -> Result<MemoryStats> {
        let storage = self.data.read().map_err(|_| DurableError::Storage {
            message: "Failed to acquire read lock".to_string(),
        })?;

        let mut total_size = 0u64;
        let mut key_count = 0u64;
        let mut expired_count = 0u64;
        let now = Utc::now();

        for (_key, file) in storage.iter() {
            key_count += 1;
            total_size += file.data.len() as u64;

            // Check if expired
            if let Some(ttl) = self.ttl_duration {
                let time_since_access = now.signed_duration_since(file.metadata.last_accessed);
                if time_since_access.num_seconds() > ttl.as_secs() as i64 {
                    expired_count += 1;
                }
            }
        }

        Ok(MemoryStats {
            total_keys: key_count,
            total_size,
            expired_keys: expired_count,
        })
    }

    /// Clean up expired entries
    pub async fn cleanup(&self) -> Result<usize> {
        let mut storage = self.data.write().map_err(|_| DurableError::Storage {
            message: "Failed to acquire write lock".to_string(),
        })?;

        if self.ttl_duration.is_none() {
            return Ok(0);
        }

        let ttl = self.ttl_duration.unwrap();
        let now = Utc::now();
        let mut removed = 0;

        storage.retain(|_, file| {
            let time_since_access = now.signed_duration_since(file.metadata.last_accessed);
            if time_since_access.num_seconds() > ttl.as_secs() as i64 {
                removed += 1;
                false
            } else {
                true
            }
        });

        Ok(removed)
    }
}

/// Memory storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_keys: u64,
    pub total_size: u64,
    pub expired_keys: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[tokio::test]
    async fn test_basic_operations() {
        let service = MemoryService::new(ServiceConfig::default());

        // Test upload
        let data = b"Hello, World!".to_vec();
        service.upload("test-key", data.clone()).await.unwrap();

        // Test download
        let downloaded = service.download("test-key").await.unwrap();
        assert_eq!(downloaded, data);

        // Test head
        let metadata = service.head("test-key").await.unwrap();
        assert_eq!(metadata.key, "test-key");
        assert_eq!(metadata.size, data.len() as u64);

        // Test list
        let keys = service.list("").await.unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].key, "test-key");
    }

    #[tokio::test]
    async fn test_delete_operation() {
        let service = MemoryService::new(ServiceConfig::default());

        service.upload("delete-test", b"data".to_vec()).await.unwrap();
        service.delete("delete-test").await.unwrap();

        let result = service.download("delete-test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_copy_operation() {
        let service = MemoryService::new(ServiceConfig::default());

        let data = b"copy test data".to_vec();
        service.upload("source", data.clone()).await.unwrap();
        service.copy("source", "target").await.unwrap();

        let copied = service.download("target").await.unwrap();
        assert_eq!(copied, data);
    }

    #[tokio::test]
    async fn test_ttl_functionality() {
        let mut config = ServiceConfig::default();
        config.parameters.insert("ttl".to_string(), "2".to_string()); // 2 seconds TTL
        let service = MemoryService::new(config);

        service.upload("ttl-test", b"ttl data".to_vec()).await.unwrap();

        // Should work immediately
        let _ = service.head("ttl-test").await.unwrap();

        // Wait for TTL to expire
        sleep(TokioDuration::from_secs(3)).await;

        // Should fail after TTL
        let result = service.head("ttl-test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cleanup() {
        let mut config = ServiceConfig::default();
        config.parameters.insert("ttl".to_string(), "1".to_string()); // 1 second TTL
        let service = MemoryService::new(config);

        service.upload("cleanup1", b"data1".to_vec()).await.unwrap();
        service.upload("cleanup2", b"data2".to_vec()).await.unwrap();

        // Wait for TTL to expire
        sleep(TokioDuration::from_secs(2)).await;

        let removed = service.cleanup().await.unwrap();
        assert_eq!(removed, 2);

        let stats = service.stats().await.unwrap();
        assert_eq!(stats.total_keys, 0);
    }

    #[tokio::test]
    async fn test_stats() {
        let service = MemoryService::new(ServiceConfig::default());

        service.upload("stats1", b"short".to_vec()).await.unwrap();
        service.upload("stats2", b"longer data here".to_vec()).await.unwrap();

        let stats = service.stats().await.unwrap();
        assert_eq!(stats.total_keys, 2);
        assert_eq!(stats.total_size, 5 + 16); // "short" + "longer data here"
    }
}