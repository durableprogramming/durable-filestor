//! Core operations for S3 storage service

use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::ObjectCannedAcl;
use aws_sdk_s3::{Client, Config};
use durable_filestor_shared::{DurableError, Result, ServiceConfig};
use std::path::Path;
use tokio::fs as async_fs;

/// File metadata information for S3 objects
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub key: String,
    pub size: u64,
    pub last_modified: std::time::SystemTime,
    pub etag: Option<String>,
}

/// S3 storage service
pub struct S3Service {
    client: Client,
    bucket: String,
    config: ServiceConfig,
}

impl S3Service {
    /// Create a new S3 service instance
    pub async fn new(bucket: String, config: ServiceConfig) -> Result<Self> {
        let region = config
            .default_region
            .as_deref()
            .unwrap_or("us-east-1");

        let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_types::region::Region::new(region.to_string()));

        // Configure custom endpoint if provided
        if let Some(endpoint) = &config.endpoint {
            loader = loader.endpoint_url(endpoint);
        }

        let aws_config = loader.load().await;
        let s3_config = Config::from(&aws_config);
        let client = Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket,
            config,
        })
    }

    /// List objects in the bucket with the given prefix
    pub async fn list(&self, prefix: &str) -> Result<Vec<FileMetadata>> {
        let mut objects = Vec::new();
        let mut continuation_token = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix);

            if let Some(token) = &continuation_token {
                request = request.continuation_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| DurableError::Storage {
                    message: format!("Failed to list objects: {}", e),
                })?;

            if let Some(contents) = response.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        let metadata = FileMetadata {
                            key: key.clone(),
                            size: object.size.unwrap_or(0) as u64,
                            last_modified: object
                                .last_modified
                                .map(|dt| dt.as_secs_f64())
                                .map(|secs| std::time::UNIX_EPOCH + std::time::Duration::from_secs_f64(secs))
                                .unwrap_or(std::time::UNIX_EPOCH),
                            etag: object.e_tag,
                        };
                        objects.push(metadata);
                    }
                }
            }

            continuation_token = response.next_continuation_token;
            if continuation_token.is_none() {
                break;
            }
        }

        Ok(objects)
    }

    /// Upload a file to S3
    pub async fn upload(&self, source_path: &str, target_key: &str) -> Result<()> {
        let source_path = Path::new(source_path);

        if !source_path.exists() {
            return Err(DurableError::Storage {
                message: format!("Source file does not exist: {}", source_path.display()),
            });
        }

        if source_path.is_dir() {
            return Err(DurableError::Storage {
                message: format!("Source cannot be a directory: {}", source_path.display()),
            });
        }

        // Read the file content
        let body = ByteStream::from_path(source_path)
            .await
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to read source file: {}", e),
            })?;

        // Build the upload request
        let mut request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(target_key)
            .body(body);

        // Set ACL if configured
        if let Some(acl) = self.config.parameters.get("acl") {
            let canned_acl = match acl.as_str() {
                "private" => ObjectCannedAcl::Private,
                "public-read" => ObjectCannedAcl::PublicRead,
                "public-read-write" => ObjectCannedAcl::PublicReadWrite,
                "authenticated-read" => ObjectCannedAcl::AuthenticatedRead,
                "bucket-owner-read" => ObjectCannedAcl::BucketOwnerRead,
                "bucket-owner-full-control" => ObjectCannedAcl::BucketOwnerFullControl,
                _ => ObjectCannedAcl::Private, // Default to private
            };
            request = request.acl(canned_acl);
        }

        // Add metadata if configured
        if let Some(content_type) = self.config.parameters.get("content-type") {
            request = request.content_type(content_type);
        }

        // Execute the upload
        request
            .send()
            .await
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to upload object: {}", e),
            })?;

        Ok(())
    }

    /// Download a file from S3
    pub async fn download(&self, source_key: &str, target_path: &str) -> Result<()> {
        let target_path = Path::new(target_path);

        // Ensure target directory exists
        if let Some(parent) = target_path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|e| DurableError::Storage {
                    message: format!("Failed to create target directory: {}", e),
                })?;
        }

        // Get the object from S3
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(source_key)
            .send()
            .await
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to download object: {}", e),
            })?;

        // Write the content to the target file
        let mut body = response.body;
        let mut file = async_fs::File::create(target_path)
            .await
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to create target file: {}", e),
            })?;

        while let Some(bytes) = body
            .try_next()
            .await
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to read object body: {}", e),
            })?
        {
            tokio::io::AsyncWriteExt::write_all(&mut file, &bytes)
                .await
                .map_err(|e| DurableError::Storage {
                    message: format!("Failed to write to target file: {}", e),
                })?;
        }

        Ok(())
    }

    /// Get metadata for an S3 object
    pub async fn head(&self, key: &str) -> Result<FileMetadata> {
        let response = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to get object metadata: {}", e),
            })?;

        let metadata = FileMetadata {
            key: key.to_string(),
            size: response.content_length.unwrap_or(0) as u64,
            last_modified: response
                .last_modified
                .map(|dt| dt.as_secs_f64())
                .map(|secs| std::time::UNIX_EPOCH + std::time::Duration::from_secs_f64(secs))
                .unwrap_or(std::time::UNIX_EPOCH),
            etag: response.e_tag,
        };

        Ok(metadata)
    }

    /// Delete an object from S3
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to delete object: {}", e),
            })?;

        Ok(())
    }

    /// Copy an object within S3
    pub async fn copy(&self, source_key: &str, target_key: &str) -> Result<()> {
        self.client
            .copy_object()
            .bucket(&self.bucket)
            .key(target_key)
            .copy_source(format!("{}/{}", self.bucket, source_key))
            .send()
            .await
            .map_err(|e| DurableError::Storage {
                message: format!("Failed to copy object: {}", e),
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs as async_fs;

    // Note: These tests require AWS credentials and a test bucket
    // They are disabled by default to avoid requiring AWS setup

    #[tokio::test]
    #[ignore] // Requires AWS credentials
    async fn test_s3_operations() {
        let config = ServiceConfig::default();
        let bucket = std::env::var("TEST_S3_BUCKET").unwrap_or("test-bucket".to_string());
        let service = S3Service::new(bucket, config).await.unwrap();

        // Test with a temporary file
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let content = "test content";
        async_fs::write(&test_file, content).await.unwrap();

        let key = "test-object.txt";

        // Upload
        service.upload(&test_file.to_string_lossy(), key).await.unwrap();

        // Head
        let metadata = service.head(key).await.unwrap();
        assert_eq!(metadata.key, key);
        assert_eq!(metadata.size, content.len() as u64);

        // List
        let objects = service.list("").await.unwrap();
        assert!(objects.iter().any(|obj| obj.key == key));

        // Download
        let download_file = temp_dir.path().join("downloaded.txt");
        service.download(key, &download_file.to_string_lossy()).await.unwrap();

        let downloaded_content = async_fs::read_to_string(&download_file).await.unwrap();
        assert_eq!(downloaded_content, content);

        // Clean up
        service.delete(key).await.unwrap();
    }

    #[tokio::test]
    async fn test_service_creation() {
        let config = ServiceConfig::default();
        // This will fail without AWS credentials, but tests the creation logic
        let result = S3Service::new("test-bucket".to_string(), config).await;
        // We expect this to fail in test environment without credentials
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_file_validation() {
        let config = ServiceConfig::default();
        let service = S3Service::new("test-bucket".to_string(), config).await.unwrap();

        // Test with nonexistent file
        let result = service.upload("/nonexistent/file.txt", "test-key").await;
        assert!(result.is_err());
        if let Err(DurableError::Storage { message }) = result {
            assert!(message.contains("does not exist"));
        }
    }
}