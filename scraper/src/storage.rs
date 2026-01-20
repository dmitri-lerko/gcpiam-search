use crate::error::{Result, ScraperError};
use crate::models::IamDataset;
use std::path::Path;
use tokio::fs;
use tracing::debug;

/// Manages saving and loading of IAM data files
pub struct StorageManager {
    output_dir: std::path::PathBuf,
}

impl StorageManager {
    /// Create a new storage manager
    pub fn new(output_dir: &Path) -> Self {
        StorageManager {
            output_dir: output_dir.to_path_buf(),
        }
    }

    /// Save dataset to JSON files
    pub async fn save(&self, dataset: &IamDataset) -> Result<()> {
        // Create output directory
        fs::create_dir_all(&self.output_dir)
            .await
            .map_err(ScraperError::FileIoError)?;

        // Save individual files
        self.save_json("iam-roles.json", &dataset.roles).await?;
        self.save_json("iam-permissions.json", &dataset.permissions).await?;
        self.save_json("metadata.json", &dataset.metadata).await?;
        self.save_json("indexes.json", &dataset.indexes).await?;

        // Save complete dataset
        self.save_json("iam-data-complete.json", dataset).await?;

        // Save minified version
        let minified = serde_json::to_string(dataset)?;
        let minified_path = self.output_dir.join("iam-data-complete.min.json");
        fs::write(&minified_path, minified)
            .await
            .map_err(ScraperError::FileIoError)?;

        debug!("Data saved to {:?}", self.output_dir);
        Ok(())
    }

    /// Load previously saved data
    pub async fn load_previous(&self) -> Result<Option<IamDataset>> {
        let path = self.output_dir.join("iam-data-complete.json");
        match fs::read_to_string(&path).await {
            Ok(content) => {
                let dataset = serde_json::from_str(&content)?;
                Ok(Some(dataset))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(ScraperError::FileIoError(e)),
        }
    }

    /// Save JSON to file
    async fn save_json<T: serde::ser::Serialize>(&self, filename: &str, data: &T) -> Result<()> {
        let path = self.output_dir.join(filename);
        let json = serde_json::to_string_pretty(data)?;
        fs::write(&path, json)
            .await
            .map_err(ScraperError::FileIoError)?;
        debug!("Wrote {}", filename);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_manager_creation() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path());
        assert_eq!(manager.output_dir, temp_dir.path());
    }

    #[tokio::test]
    async fn test_save_and_load_data() {
        use crate::models::{IamRole, IamPermission, IamStage, IamMetadata, Changes, Indexes};

        let temp_dir = tempfile::TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path());

        // Create test data
        let dataset = IamDataset {
            metadata: IamMetadata {
                last_updated: "2024-01-03T00:00:00Z".to_string(),
                total_roles: 1,
                total_permissions: 1,
                gcp_api_version: "v1".to_string(),
                changes_since_last_run: Changes::default(),
            },
            roles: vec![IamRole {
                name: "roles/test".to_string(),
                title: "Test Role".to_string(),
                description: "Test role for testing".to_string(),
                stage: IamStage::Ga,
                included_permissions: vec!["test.permission".to_string()],
                etag: "test-etag".to_string(),
                deleted: false,
                permission_count: 1,
                keywords: vec![],
            }],
            permissions: vec![IamPermission {
                name: "test.permission".to_string(),
                service: "test".to_string(),
                resource: "permission".to_string(),
                action: "use".to_string(),
                description: None,
                custom_roles_support_level: None,
                stage: None,
                api_disabled: None,
                roles_granting: vec!["roles/test".to_string()],
            }],
            indexes: Indexes {
                roles_by_name: [("roles/test".to_string(), 0)].iter().cloned().collect(),
                permissions_by_name: [("test.permission".to_string(), 0)].iter().cloned().collect(),
                roles_by_stage: [("GA".to_string(), vec![0])].iter().cloned().collect(),
                permissions_by_service: [("test".to_string(), vec![0])].iter().cloned().collect(),
            },
        };

        // Save data
        let save_result = manager.save(&dataset).await;
        assert!(save_result.is_ok());

        // Load data
        let loaded_result = manager.load_previous().await;
        assert!(loaded_result.is_ok());

        let loaded_data = loaded_result.unwrap();
        assert!(loaded_data.is_some());

        let loaded = loaded_data.unwrap();
        assert_eq!(loaded.metadata.total_roles, 1);
        assert_eq!(loaded.metadata.total_permissions, 1);
        assert_eq!(loaded.roles[0].name, "roles/test");
    }

    #[tokio::test]
    async fn test_load_nonexistent_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let manager = StorageManager::new(temp_dir.path());

        let result = manager.load_previous().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
