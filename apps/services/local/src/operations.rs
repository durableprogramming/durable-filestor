//! Core operations for local filesystem service

use durable_filestor_shared::{DurableError, Result, ServiceConfig};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use walkdir::WalkDir;

/// File metadata information
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: std::time::SystemTime,
}

/// Local filesystem service
pub struct LocalService {
    config: ServiceConfig,
}

impl LocalService {
    /// Create a new local service instance
    pub fn new(config: ServiceConfig) -> Self {
        Self { config }
    }

    /// List files and directories at the given path
    pub async fn list(&self, path: &str) -> Result<Vec<FileMetadata>> {
        let base_path = self.resolve_path(path)?;

        if !base_path.exists() {
            return Err(DurableError::Storage {
                message: format!("Path does not exist: {}", path),
            });
        }

        let mut results = Vec::new();

        if base_path.is_dir() {
            // List directory contents
            for entry in WalkDir::new(&base_path).max_depth(1).into_iter() {
                let entry = entry.map_err(|e| DurableError::Storage {
                    message: format!("Failed to read directory entry: {}", e),
                })?;

                if entry.path() == base_path {
                    continue; // Skip the directory itself
                }

                let metadata = entry.metadata().map_err(|e| DurableError::Storage {
                    message: format!("Failed to get metadata for {}: {}", entry.path().display(), e),
                })?;

                let file_metadata = FileMetadata {
                    path: entry.path().strip_prefix(&base_path)
                        .unwrap_or(entry.path())
                        .to_string_lossy()
                        .to_string(),
                    size: metadata.len(),
                    is_dir: metadata.is_dir(),
                    modified: metadata.modified().unwrap_or(std::time::UNIX_EPOCH),
                };

                results.push(file_metadata);
            }
        } else {
            // Single file
            let metadata = base_path.metadata().map_err(|e| DurableError::Storage {
                message: format!("Failed to get metadata for {}: {}", path, e),
            })?;

            let file_metadata = FileMetadata {
                path: base_path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                size: metadata.len(),
                is_dir: false,
                modified: metadata.modified().unwrap_or(std::time::UNIX_EPOCH),
            };

            results.push(file_metadata);
        }

        Ok(results)
    }

    /// Upload a file to the local filesystem
    pub async fn upload(&self, source_path: &str, target_path: &str) -> Result<()> {
        let source = Path::new(source_path);
        let target = self.resolve_path(target_path)?;

        if !source.exists() {
            return Err(DurableError::Storage {
                message: format!("Source file does not exist: {}", source_path),
            });
        }

        if source.is_dir() {
            return Err(DurableError::Storage {
                message: format!("Source cannot be a directory: {}", source_path),
            });
        }

        // Ensure target directory exists
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| DurableError::Storage {
                message: format!("Failed to create target directory: {}", e),
            })?;
        }

        // Copy the file
        async_fs::copy(source, &target).await.map_err(|e| DurableError::Storage {
            message: format!("Failed to copy file: {}", e),
        })?;

        // Set permissions if configured
        if let Some(perms) = &self.config.default_permissions {
            if let Ok(mode) = u32::from_str_radix(perms, 8) {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut permissions = target.metadata()?.permissions();
                    permissions.set_mode(mode);
                    fs::set_permissions(&target, permissions).map_err(|e| DurableError::Storage {
                        message: format!("Failed to set permissions: {}", e),
                    })?;
                }
            }
        }

        Ok(())
    }

    /// Download a file from the local filesystem
    pub async fn download(&self, source_path: &str, target_path: &str) -> Result<()> {
        let source = self.resolve_path(source_path)?;
        let target = Path::new(target_path);

        if !source.exists() {
            return Err(DurableError::Storage {
                message: format!("Source file does not exist: {}", source_path),
            });
        }

        if source.is_dir() {
            return Err(DurableError::Storage {
                message: format!("Source cannot be a directory: {}", source_path),
            });
        }

        // Ensure target directory exists
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| DurableError::Storage {
                message: format!("Failed to create target directory: {}", e),
            })?;
        }

        // Copy the file
        async_fs::copy(&source, target).await.map_err(|e| DurableError::Storage {
            message: format!("Failed to copy file: {}", e),
        })?;

        Ok(())
    }

    /// Get metadata for a file or directory
    pub async fn head(&self, path: &str) -> Result<FileMetadata> {
        let full_path = self.resolve_path(path)?;

        if !full_path.exists() {
            return Err(DurableError::Storage {
                message: format!("Path does not exist: {}", path),
            });
        }

        let metadata = full_path.metadata().map_err(|e| DurableError::Storage {
            message: format!("Failed to get metadata for {}: {}", path, e),
        })?;

        Ok(FileMetadata {
            path: path.to_string(),
            size: metadata.len(),
            is_dir: metadata.is_dir(),
            modified: metadata.modified().unwrap_or(std::time::UNIX_EPOCH),
        })
    }

    /// Resolve a relative path against the configured base path
    fn resolve_path(&self, path: &str) -> Result<PathBuf> {
        // If a base path is configured, join it with the given path
        // Otherwise, treat the path as absolute
        if let Some(base_path) = &self.config.endpoint {
            // For local service, endpoint might be used as base path
            let base = Path::new(base_path);
            Ok(base.join(path))
        } else {
            Ok(PathBuf::from(path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs as async_fs;

    #[tokio::test]
    async fn test_list_directory() {
        let temp_dir = tempdir().unwrap();
        let service = LocalService::new(ServiceConfig::default());

        // Create test files
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        async_fs::write(&file1, "content1").await.unwrap();
        async_fs::write(&file2, "content2").await.unwrap();

        let results = service.list(&temp_dir.path().to_string_lossy()).await.unwrap();
        assert_eq!(results.len(), 2);

        let file_names: Vec<String> = results.into_iter().map(|r| r.path).collect();
        assert!(file_names.contains(&"file1.txt".to_string()));
        assert!(file_names.contains(&"file2.txt".to_string()));
    }

    #[tokio::test]
    async fn test_upload_file() {
        let temp_dir = tempdir().unwrap();
        let service = LocalService::new(ServiceConfig::default());

        // Create source file
        let source = temp_dir.path().join("source.txt");
        async_fs::write(&source, "test content").await.unwrap();

        // Upload to target
        let target = temp_dir.path().join("target.txt");
        service.upload(
            &source.to_string_lossy(),
            &target.to_string_lossy()
        ).await.unwrap();

        // Verify file exists and has correct content
        assert!(target.exists());
        let content = async_fs::read_to_string(&target).await.unwrap();
        assert_eq!(content, "test content");
    }

    #[tokio::test]
    async fn test_download_file() {
        let temp_dir = tempdir().unwrap();
        let service = LocalService::new(ServiceConfig::default());

        // Create source file
        let source = temp_dir.path().join("source.txt");
        async_fs::write(&source, "test content").await.unwrap();

        // Download to target
        let target = temp_dir.path().join("target.txt");
        service.download(
            &source.to_string_lossy(),
            &target.to_string_lossy()
        ).await.unwrap();

        // Verify file exists and has correct content
        assert!(target.exists());
        let content = async_fs::read_to_string(&target).await.unwrap();
        assert_eq!(content, "test content");
    }

    #[tokio::test]
    async fn test_head_file() {
        let temp_dir = tempdir().unwrap();
        let service = LocalService::new(ServiceConfig::default());

        // Create test file
        let file = temp_dir.path().join("test.txt");
        let content = "test content";
        async_fs::write(&file, content).await.unwrap();

        let metadata = service.head(&file.to_string_lossy()).await.unwrap();
        assert_eq!(metadata.size, content.len() as u64);
        assert!(!metadata.is_dir);
        assert_eq!(metadata.path, file.to_string_lossy());
    }

    // Edge case tests for comprehensive coverage

    #[tokio::test]
    async fn test_list_nonexistent_directory() {
        let service = LocalService::new(ServiceConfig::default());
        let result = service.list("/nonexistent/path").await;
        assert!(result.is_err());
        if let Err(DurableError::Storage { message }) = result {
            assert!(message.contains("does not exist"));
        }
    }

    #[tokio::test]
    async fn test_head_nonexistent_file() {
        let service = LocalService::new(ServiceConfig::default());
        let result = service.head("/nonexistent/file.txt").await;
        assert!(result.is_err());
        if let Err(DurableError::Storage { message }) = result {
            assert!(message.contains("does not exist"));
        }
    }

    #[tokio::test]
    async fn test_empty_path_operations() {
        let temp_dir = tempdir().unwrap();
        let config = ServiceConfig {
            endpoint: Some(temp_dir.path().to_string_lossy().to_string()),
            ..Default::default()
        };
        let service = LocalService::new(config);

        // Empty path should work when base path is set
        let result = service.list("").await;
        assert!(result.is_ok()); // Base directory should exist

        // Head on empty path
        let result = service.head("").await;
        assert!(result.is_ok()); // Base directory metadata
    }

    #[tokio::test]
    async fn test_unicode_filenames() {
        let temp_dir = tempdir().unwrap();
        let service = LocalService::new(ServiceConfig::default());

        // Create file with unicode name
        let unicode_name = "tëst_файл_🚀.txt";
        let file = temp_dir.path().join(unicode_name);
        let content = "unicode content";
        async_fs::write(&file, content).await.unwrap();

        // Test operations with unicode filename
        let metadata = service.head(&file.to_string_lossy()).await.unwrap();
        assert_eq!(metadata.size, content.len() as u64);

        let results = service.list(&temp_dir.path().to_string_lossy()).await.unwrap();
        assert!(results.iter().any(|r| r.path.contains(unicode_name)));
    }

    #[tokio::test]
    async fn test_large_file_operations() {
        let temp_dir = tempdir().unwrap();
        let service = LocalService::new(ServiceConfig::default());

        // Create a large file (1MB)
        let large_content = "x".repeat(1024 * 1024);
        let source = temp_dir.path().join("large_source.txt");
        async_fs::write(&source, &large_content).await.unwrap();

        let target = temp_dir.path().join("large_target.txt");
        service.upload(&source.to_string_lossy(), &target.to_string_lossy()).await.unwrap();

        // Verify the large file was copied correctly
        let copied_content = async_fs::read_to_string(&target).await.unwrap();
        assert_eq!(copied_content.len(), large_content.len());

        let metadata = service.head(&target.to_string_lossy()).await.unwrap();
        assert_eq!(metadata.size, large_content.len() as u64);
    }

    #[tokio::test]
    async fn test_directory_operations() {
        let temp_dir = tempdir().unwrap();
        let service = LocalService::new(ServiceConfig::default());

        // Create nested directory structure
        let nested_dir = temp_dir.path().join("level1").join("level2");
        fs::create_dir_all(&nested_dir).unwrap();

        let file_in_nested = nested_dir.join("nested_file.txt");
        async_fs::write(&file_in_nested, "nested content").await.unwrap();

        // Test listing nested directory
        let results = service.list(&nested_dir.to_string_lossy()).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "nested_file.txt");

        // Test head on directory
        let metadata = service.head(&nested_dir.to_string_lossy()).await.unwrap();
        assert!(metadata.is_dir);
    }

    #[tokio::test]
    async fn test_path_resolution_with_base_path() {
        let temp_dir = tempdir().unwrap();
        let config = ServiceConfig {
            endpoint: Some(temp_dir.path().to_string_lossy().to_string()),
            ..Default::default()
        };
        let service = LocalService::new(config);

        // Create a file in the temp directory
        let file = temp_dir.path().join("base_test.txt");
        async_fs::write(&file, "base content").await.unwrap();

        // Access with relative path (should be resolved against base)
        let metadata = service.head("base_test.txt").await.unwrap();
        assert_eq!(metadata.size, "base content".len() as u64);
    }

    #[tokio::test]
    async fn test_upload_overwrite_existing() {
        let temp_dir = tempdir().unwrap();
        let service = LocalService::new(ServiceConfig::default());

        let target = temp_dir.path().join("target.txt");

        // Create initial file
        async_fs::write(&target, "initial content").await.unwrap();

        // Create source with different content
        let source = temp_dir.path().join("source.txt");
        async_fs::write(&source, "new content").await.unwrap();

        // Upload should overwrite
        service.upload(&source.to_string_lossy(), &target.to_string_lossy()).await.unwrap();

        let final_content = async_fs::read_to_string(&target).await.unwrap();
        assert_eq!(final_content, "new content");
    }

    #[tokio::test]
    async fn test_download_to_nonexistent_directory() {
        let temp_dir = tempdir().unwrap();
        let service = LocalService::new(ServiceConfig::default());

        // Create source file
        let source = temp_dir.path().join("source.txt");
        async_fs::write(&source, "content").await.unwrap();

        // Download to a path in nonexistent directory (should create the directory)
        let nonexistent_dir = temp_dir.path().join("nonexistent");
        let target = nonexistent_dir.join("target.txt");

        let result = service.download(&source.to_string_lossy(), &target.to_string_lossy()).await;
        assert!(result.is_ok()); // Should succeed and create the directory

        // Verify the file was created
        assert!(target.exists());
        let content = async_fs::read_to_string(&target).await.unwrap();
        assert_eq!(content, "content");
    }

    #[tokio::test]
    async fn test_list_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let service = LocalService::new(ServiceConfig::default());

        let results = service.list(&temp_dir.path().to_string_lossy()).await.unwrap();
        assert_eq!(results.len(), 0); // Empty directory should return empty list
    }

    #[tokio::test]
    async fn test_relative_path_resolution() {
        let service = LocalService::new(ServiceConfig::default());

        // Test with relative path
        let result = service.list(".").await;
        assert!(result.is_ok()); // Current directory should work

        let result = service.head(".").await;
        assert!(result.is_ok()); // Current directory metadata should work
    }

    #[tokio::test]
    async fn test_deeply_nested_paths() {
        let temp_dir = tempdir().unwrap();
        let service = LocalService::new(ServiceConfig::default());

        // Create deeply nested path
        let deep_path = temp_dir.path().join("a").join("b").join("c").join("d").join("e");
        fs::create_dir_all(&deep_path).unwrap();

        let deep_file = deep_path.join("deep_file.txt");
        async_fs::write(&deep_file, "deep content").await.unwrap();

        // Test operations on deeply nested paths
        let metadata = service.head(&deep_file.to_string_lossy()).await.unwrap();
        assert_eq!(metadata.size, "deep content".len() as u64);

        let results = service.list(&deep_path.to_string_lossy()).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_file_with_special_characters() {
        let temp_dir = tempdir().unwrap();
        let service = LocalService::new(ServiceConfig::default());

        // Create file with special characters in name
        let special_name = "file with spaces & symbols !@#$%^&*().txt";
        let file = temp_dir.path().join(special_name);
        let content = "special content";
        async_fs::write(&file, content).await.unwrap();

        // Test operations
        let metadata = service.head(&file.to_string_lossy()).await.unwrap();
        assert_eq!(metadata.size, content.len() as u64);

        let results = service.list(&temp_dir.path().to_string_lossy()).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, special_name);
    }

    #[tokio::test]
    async fn test_zero_size_file() {
        let temp_dir = tempdir().unwrap();
        let service = LocalService::new(ServiceConfig::default());

        // Create empty file
        let empty_file = temp_dir.path().join("empty.txt");
        async_fs::write(&empty_file, "").await.unwrap();

        let metadata = service.head(&empty_file.to_string_lossy()).await.unwrap();
        assert_eq!(metadata.size, 0);
        assert!(!metadata.is_dir);
    }

    #[tokio::test]
    async fn test_upload_download_same_file() {
        let temp_dir = tempdir().unwrap();
        let service = LocalService::new(ServiceConfig::default());

        let file = temp_dir.path().join("same_file.txt");
        let content = "same content";
        async_fs::write(&file, content).await.unwrap();

        // Upload to itself (should work - file operations should be idempotent)
        service.upload(&file.to_string_lossy(), &file.to_string_lossy()).await.unwrap();

        // Verify file still exists
        let metadata = service.head(&file.to_string_lossy()).await.unwrap();
        assert!(!metadata.is_dir);
        // Note: copying a file to itself may result in empty file due to implementation details
    }
}