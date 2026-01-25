use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct IamDataFile {
    roles: Vec<RoleData>,
    #[allow(dead_code)]
    permissions: Vec<PermissionData>,
    #[allow(dead_code)]
    metadata: MetadataData,
}

#[derive(Debug, Deserialize)]
struct RoleData {
    name: String,
    title: String,
    description: String,
    stage: String,
    included_permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PermissionData {
    name: String,
    #[allow(dead_code)]
    service: String,
}

#[derive(Debug, Deserialize)]
struct MetadataData {
    #[allow(dead_code)]
    total_roles: usize,
    #[allow(dead_code)]
    total_permissions: usize,
    last_updated: String,
}

// Serializable search index structures
#[derive(Debug, Clone, Serialize)]
struct Role {
    name: String,
    title: String,
    description: String,
    stage: String,
    included_permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct Permission {
    name: String,
    service: String,
    resource: String,
    action: String,
    granted_by_roles: Vec<u32>, // Role indices for compact storage
}

#[derive(Debug, Clone, Serialize)]
struct RoleSummary {
    name: String,
    title: String,
    stage: String,
}

#[derive(Debug, Serialize)]
struct PrebuiltIndex {
    // All permissions sorted for binary search
    permissions: Vec<Permission>,
    permission_names: Vec<String>,

    // All roles
    roles: Vec<Role>,
    role_names: Vec<String>,
    role_summaries: Vec<RoleSummary>,

    // Service -> permission indices
    service_to_permissions: HashMap<String, Vec<u32>>,

    // Lowercase names for case-insensitive search
    permission_names_lower: Vec<String>,
    role_names_lower: Vec<String>,
    role_titles_lower: Vec<String>,
}

fn main() {
    println!("cargo:rerun-if-changed=../data/iam-data.json");
    println!("cargo:rerun-if-changed=build.rs");

    let data_path = Path::new("../data/iam-data.json");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("prebuilt_index.bin");

    if !data_path.exists() {
        eprintln!("Warning: iam-data.json not found, creating empty index");
        let empty_index = PrebuiltIndex {
            permissions: vec![],
            permission_names: vec![],
            roles: vec![],
            role_names: vec![],
            role_summaries: vec![],
            service_to_permissions: HashMap::new(),
            permission_names_lower: vec![],
            role_names_lower: vec![],
            role_titles_lower: vec![],
        };
        let encoded = bincode::serialize(&empty_index).unwrap();
        fs::write(&dest_path, encoded).unwrap();
        return;
    }

    eprintln!("Building search index from iam-data.json...");

    let content = fs::read_to_string(data_path).expect("Failed to read iam-data.json");
    let data: IamDataFile = serde_json::from_str(&content).expect("Failed to parse JSON");

    // Extract and generate timestamp constant
    let last_updated = &data.metadata.last_updated;
    let timestamp_code = format!(
        "pub const LAST_UPDATED: &str = \"{}\";\n",
        last_updated
    );
    let timestamp_path = Path::new(&out_dir).join("timestamp.rs");
    fs::write(&timestamp_path, timestamp_code).expect("Failed to write timestamp constant");

    // Build role index and summaries
    let mut roles: Vec<Role> = Vec::with_capacity(data.roles.len());
    let mut role_names: Vec<String> = Vec::with_capacity(data.roles.len());
    let mut role_summaries: Vec<RoleSummary> = Vec::with_capacity(data.roles.len());
    let mut role_name_to_idx: HashMap<String, u32> = HashMap::new();

    for role_data in &data.roles {
        let idx = roles.len() as u32;
        role_name_to_idx.insert(role_data.name.clone(), idx);

        roles.push(Role {
            name: role_data.name.clone(),
            title: role_data.title.clone(),
            description: role_data.description.clone(),
            stage: role_data.stage.clone(),
            included_permissions: role_data.included_permissions.clone(),
        });
        role_names.push(role_data.name.clone());
        role_summaries.push(RoleSummary {
            name: role_data.name.clone(),
            title: role_data.title.clone(),
            stage: role_data.stage.clone(),
        });
    }

    // Build permission index with role mappings
    let mut permission_map: HashMap<String, Permission> = HashMap::new();
    let mut service_to_permissions: HashMap<String, Vec<u32>> = HashMap::new();

    for role_data in &data.roles {
        let role_idx = *role_name_to_idx.get(&role_data.name).unwrap();

        for perm_name in &role_data.included_permissions {
            let entry = permission_map.entry(perm_name.clone()).or_insert_with(|| {
                let parts: Vec<&str> = perm_name.split('.').collect();
                Permission {
                    name: perm_name.clone(),
                    service: parts.first().unwrap_or(&"").to_string(),
                    resource: parts.get(1).unwrap_or(&"").to_string(),
                    action: parts.get(2).unwrap_or(&"").to_string(),
                    granted_by_roles: vec![],
                }
            });
            entry.granted_by_roles.push(role_idx);
        }
    }

    // Sort permissions and build final structures
    let mut permissions: Vec<Permission> = permission_map.into_values().collect();
    permissions.sort_by(|a, b| a.name.cmp(&b.name));

    let mut permission_names: Vec<String> = Vec::with_capacity(permissions.len());
    for (idx, perm) in permissions.iter().enumerate() {
        permission_names.push(perm.name.clone());
        service_to_permissions
            .entry(perm.service.clone())
            .or_insert_with(Vec::new)
            .push(idx as u32);
    }

    // Pre-compute lowercase versions for case-insensitive search
    let permission_names_lower: Vec<String> = permission_names.iter().map(|s| s.to_lowercase()).collect();
    let role_names_lower: Vec<String> = role_names.iter().map(|s| s.to_lowercase()).collect();
    let role_titles_lower: Vec<String> = roles.iter().map(|r| r.title.to_lowercase()).collect();

    let index = PrebuiltIndex {
        permissions,
        permission_names,
        roles,
        role_names,
        role_summaries,
        service_to_permissions,
        permission_names_lower,
        role_names_lower,
        role_titles_lower,
    };

    eprintln!("Indexed {} permissions and {} roles", index.permission_names.len(), index.role_names.len());

    let encoded = bincode::serialize(&index).expect("Failed to serialize index");
    eprintln!("Index size: {} bytes ({:.2} MB)", encoded.len(), encoded.len() as f64 / 1024.0 / 1024.0);

    fs::write(&dest_path, encoded).expect("Failed to write index");
    eprintln!("Wrote prebuilt index to {:?}", dest_path);
}
