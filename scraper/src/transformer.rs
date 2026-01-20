use crate::error::Result;
use crate::models::{IamDataset, IamMetadata, RawGcpData, Changes, Indexes, IamPermission, IamRole};
use std::collections::HashMap;
use tracing::{debug, info};

/// Transforms raw GCP data into optimized format
pub struct DataTransformer;

impl DataTransformer {
    pub fn new() -> Self {
        DataTransformer
    }

    /// Transform raw data to optimized dataset
    pub fn transform(&self, raw_data: RawGcpData) -> Result<IamDataset> {
        debug!("Starting data transformation");

        let mut roles = raw_data.roles;
        let mut permissions = raw_data.permissions;

        // Build bi-directional role↔permission references
        debug!("Building role-permission references");
        self.build_role_permission_references(&mut roles, &mut permissions);

        // Create indexes for fast lookups
        debug!("Building indexes");
        let indexes = self.build_indexes(&roles, &permissions);

        let metadata = IamMetadata {
            last_updated: raw_data.fetched_at,
            total_roles: roles.len(),
            total_permissions: permissions.len(),
            gcp_api_version: "v1".to_string(),
            changes_since_last_run: Changes::default(),
        };

        info!(
            "Transformation complete: {} roles, {} permissions",
            roles.len(),
            permissions.len()
        );

        Ok(IamDataset {
            metadata,
            roles,
            permissions,
            indexes,
        })
    }

    /// Build bi-directional references between roles and permissions
    fn build_role_permission_references(&self, roles: &mut [IamRole], permissions: &mut [IamPermission]) {
        // Create a map of permission name to index for fast lookups
        let perm_name_to_idx: HashMap<String, usize> = permissions
            .iter()
            .enumerate()
            .map(|(idx, perm)| (perm.name.clone(), idx))
            .collect();

        // For each permission, track which roles grant it
        for role in roles.iter() {
            for perm_name in &role.included_permissions {
                if let Some(&perm_idx) = perm_name_to_idx.get(perm_name) {
                    if perm_idx < permissions.len() {
                        permissions[perm_idx].roles_granting.push(role.name.clone());
                    }
                }
            }
        }

        // Deduplicate and sort roles_granting lists
        for perm in permissions.iter_mut() {
            perm.roles_granting.sort();
            perm.roles_granting.dedup();
        }

        debug!("Built role-permission references");
    }

    /// Build indexes for fast lookups
    fn build_indexes(&self, roles: &[IamRole], permissions: &[IamPermission]) -> Indexes {
        let mut roles_by_name = HashMap::new();
        let mut permissions_by_name = HashMap::new();
        let mut roles_by_stage = HashMap::new();
        let mut permissions_by_service = HashMap::new();

        // Build role indexes
        for (idx, role) in roles.iter().enumerate() {
            roles_by_name.insert(role.name.clone(), idx);
            let stage_str = format!("{:?}", role.stage).to_uppercase();
            roles_by_stage
                .entry(stage_str)
                .or_insert_with(Vec::new)
                .push(idx);
        }

        // Build permission indexes
        for (idx, perm) in permissions.iter().enumerate() {
            permissions_by_name.insert(perm.name.clone(), idx);
            if !perm.service.is_empty() {
                permissions_by_service
                    .entry(perm.service.clone())
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }

        debug!(
            "Built indexes: {} roles, {} permissions, {} services",
            roles_by_name.len(),
            permissions_by_name.len(),
            permissions_by_service.len()
        );

        Indexes {
            roles_by_name,
            permissions_by_name,
            roles_by_stage,
            permissions_by_service,
        }
    }
}

impl Default for DataTransformer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::IamStage;

    fn create_test_role(name: &str, permissions: Vec<String>) -> IamRole {
        IamRole {
            name: name.to_string(),
            title: format!("{} Title", name),
            description: format!("{} Description", name),
            stage: IamStage::Ga,
            included_permissions: permissions,
            etag: "test-etag".to_string(),
            deleted: false,
            permission_count: 0,
            keywords: vec![],
        }
    }

    fn create_test_permission(name: &str) -> IamPermission {
        IamPermission::from_name(name.to_string())
    }

    #[test]
    fn test_transformer_creation() {
        let _transformer = DataTransformer::new();
        let _transformer2 = DataTransformer::default();
        assert_eq!(std::mem::size_of::<DataTransformer>(), 0);
    }

    #[test]
    fn test_build_indexes() {
        let transformer = DataTransformer::new();

        let roles = vec![
            create_test_role("roles/admin", vec!["compute.instances.list".to_string()]),
            create_test_role("roles/viewer", vec!["compute.instances.get".to_string()]),
        ];

        let permissions = vec![
            create_test_permission("compute.instances.list"),
            create_test_permission("compute.instances.get"),
        ];

        let indexes = transformer.build_indexes(&roles, &permissions);

        assert_eq!(indexes.roles_by_name.len(), 2);
        assert_eq!(indexes.permissions_by_name.len(), 2);
        assert_eq!(indexes.permissions_by_service.len(), 1); // "compute" service
        assert!(indexes.permissions_by_service.contains_key("compute"));
    }

    #[test]
    fn test_build_role_permission_references() {
        let transformer = DataTransformer::new();

        let mut roles = vec![
            create_test_role("roles/admin", vec![
                "compute.instances.list".to_string(),
                "compute.instances.create".to_string(),
            ]),
        ];

        let mut permissions = vec![
            create_test_permission("compute.instances.list"),
            create_test_permission("compute.instances.create"),
        ];

        transformer.build_role_permission_references(&mut roles, &mut permissions);

        // Check that permissions know which roles grant them
        assert_eq!(permissions[0].roles_granting.len(), 1);
        assert_eq!(permissions[0].roles_granting[0], "roles/admin");
        assert_eq!(permissions[1].roles_granting.len(), 1);
        assert_eq!(permissions[1].roles_granting[0], "roles/admin");
    }

    #[test]
    fn test_permission_parsing() {
        let perm = IamPermission::from_name("compute.instances.list".to_string());
        assert_eq!(perm.service, "compute");
        assert_eq!(perm.resource, "instances");
        assert_eq!(perm.action, "list");
    }

    #[test]
    fn test_permission_parsing_with_multiple_parts() {
        let perm = IamPermission::from_name("compute.disks.create".to_string());
        assert_eq!(perm.service, "compute");
        assert_eq!(perm.resource, "disks");
        assert_eq!(perm.action, "create");
    }

    #[test]
    fn test_transform_raw_data() {
        let transformer = DataTransformer::new();

        let raw_data = RawGcpData {
            roles: vec![
                create_test_role("roles/compute.admin", vec!["compute.instances.list".to_string()]),
            ],
            permissions: vec![
                create_test_permission("compute.instances.list"),
            ],
            fetched_at: "2024-01-03T00:00:00Z".to_string(),
        };

        let result = transformer.transform(raw_data);
        assert!(result.is_ok());

        let dataset = result.unwrap();
        assert_eq!(dataset.roles.len(), 1);
        assert_eq!(dataset.permissions.len(), 1);
        assert_eq!(dataset.metadata.total_roles, 1);
        assert_eq!(dataset.metadata.total_permissions, 1);
    }
}
