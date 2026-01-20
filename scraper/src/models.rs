use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// IAM role stage/lifecycle
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum IamStage {
    #[serde(rename = "GA")]
    Ga,
    #[serde(rename = "BETA")]
    Beta,
    #[serde(rename = "ALPHA")]
    Alpha,
    #[serde(rename = "DEPRECATED")]
    Deprecated,
}

impl Default for IamStage {
    fn default() -> Self {
        IamStage::Ga
    }
}

/// GCP IAM Role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamRole {
    pub name: String,
    pub title: String,
    pub description: String,
    pub stage: IamStage,
    #[serde(default)]
    pub included_permissions: Vec<String>,
    pub etag: String,
    #[serde(default)]
    pub deleted: bool,
    #[serde(skip)]
    pub permission_count: usize,
    #[serde(skip)]
    pub keywords: Vec<String>,
}

/// GCP IAM Permission
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct IamPermission {
    pub name: String,
    pub service: String,
    pub resource: String,
    pub action: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub custom_roles_support_level: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub api_disabled: Option<bool>,
    #[serde(skip)]
    pub roles_granting: Vec<String>,
}

/// Metadata about the dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamMetadata {
    pub last_updated: String,
    pub total_roles: usize,
    pub total_permissions: usize,
    pub gcp_api_version: String,
    pub changes_since_last_run: Changes,
}

/// Changes detected in the dataset
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Changes {
    pub roles_added: Vec<String>,
    pub roles_removed: Vec<String>,
    pub roles_modified: Vec<String>,
    pub permissions_added: Vec<String>,
    pub permissions_removed: Vec<String>,
}

/// Indexes for fast lookups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indexes {
    pub roles_by_name: HashMap<String, usize>,
    pub permissions_by_name: HashMap<String, usize>,
    pub roles_by_stage: HashMap<String, Vec<usize>>,
    pub permissions_by_service: HashMap<String, Vec<usize>>,
}

/// Complete IAM dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamDataset {
    pub metadata: IamMetadata,
    pub roles: Vec<IamRole>,
    pub permissions: Vec<IamPermission>,
    pub indexes: Indexes,
}

/// Raw data from GCP API
#[derive(Debug)]
pub struct RawGcpData {
    pub roles: Vec<IamRole>,
    pub permissions: Vec<IamPermission>,
    pub fetched_at: String,
}

/// GCP Role from API (raw response)
#[derive(Debug, Deserialize)]
pub struct GcpRoleResponse {
    pub name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub stage: String,
    #[serde(default)]
    #[serde(rename = "includedPermissions")]
    pub included_permissions: Vec<String>,
    pub etag: String,
    #[serde(default)]
    pub deleted: bool,
}

/// GCP Roles list response
#[derive(Debug, Deserialize)]
pub struct GcpRolesResponse {
    pub roles: Vec<GcpRoleResponse>,
    #[serde(default)]
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

impl IamRole {
    /// Create a new IAM role from raw GCP data
    pub fn from_gcp(role: GcpRoleResponse) -> Self {
        let stage = match role.stage.as_str() {
            "BETA" => IamStage::Beta,
            "ALPHA" => IamStage::Alpha,
            "DEPRECATED" => IamStage::Deprecated,
            _ => IamStage::Ga,
        };

        let permission_count = role.included_permissions.len();
        let keywords = Self::extract_keywords(&role.title, &role.description);

        IamRole {
            name: role.name,
            title: role.title,
            description: role.description,
            stage,
            included_permissions: role.included_permissions,
            etag: role.etag,
            deleted: role.deleted,
            permission_count,
            keywords,
        }
    }

    /// Extract keywords from role title and description
    fn extract_keywords(title: &str, description: &str) -> Vec<String> {
        let text = format!("{} {}", title, description).to_lowercase();
        let stop_words = [
            "the", "and", "for", "you", "all", "not", "but", "can", "her", "was",
            "one", "our", "out", "day", "get", "has", "him", "his", "how", "its",
            "may", "new", "now", "old", "see", "way", "who", "boy", "did",
        ];

        text.split_whitespace()
            .filter_map(|word| {
                let clean = word
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>();

                if clean.len() > 3 && !stop_words.contains(&clean.as_str()) {
                    Some(clean)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }
}

impl IamPermission {
    /// Create a new permission from a permission string
    /// Format: service.resource.action (e.g., "compute.instances.list")
    pub fn from_name(name: String) -> Self {
        let parts: Vec<&str> = name.split('.').collect();

        let service = parts.first().map(|s| s.to_string()).unwrap_or_default();
        let resource = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
        let action = parts.get(2).map(|s| s.to_string()).unwrap_or_default();

        IamPermission {
            name,
            service,
            resource,
            action,
            description: None,
            custom_roles_support_level: None,
            stage: None,
            api_disabled: None,
            roles_granting: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_permission_name() {
        let perm = IamPermission::from_name("compute.instances.list".to_string());
        assert_eq!(perm.service, "compute");
        assert_eq!(perm.resource, "instances");
        assert_eq!(perm.action, "list");
    }

    #[test]
    fn test_extract_keywords() {
        let keywords = IamRole::extract_keywords(
            "Compute Admin",
            "Full control of Google Compute Engine resources",
        );
        assert!(!keywords.is_empty());
        assert!(keywords.iter().any(|k| k.contains("compute")));
    }
}
