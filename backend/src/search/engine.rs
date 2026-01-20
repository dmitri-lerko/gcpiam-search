/// High-performance hybrid search engine with role-permission associations
///
/// Implements multiple search strategies:
/// - Exact: O(1) hash map lookups
/// - Prefix: Trie-based autocomplete
/// - Fuzzy: N-gram based similarity matching

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult<T> {
    pub item: T,
    pub score: f64,
}

/// Role with its permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub title: String,
    pub description: String,
    pub stage: String,
    pub included_permissions: Vec<String>,
}

/// Permission with roles that grant it
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub name: String,
    pub service: String,
    pub resource: String,
    pub action: String,
    pub granted_by_roles: Vec<String>,
}

/// Search result for permissions including associated roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSearchResult {
    pub name: String,
    pub service: String,
    pub resource: String,
    pub action: String,
    pub score: f64,
    pub granted_by_roles: Vec<RoleSummary>,
}

/// Search result for roles including their permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleSearchResult {
    pub name: String,
    pub title: String,
    pub description: String,
    pub stage: String,
    pub score: f64,
    pub permission_count: usize,
    pub sample_permissions: Vec<String>,
}

/// Brief role info for permission results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleSummary {
    pub name: String,
    pub title: String,
    pub stage: String,
}

/// High-performance hybrid search engine
pub struct SearchEngine {
    // Permission data
    permissions: HashMap<String, Permission>,
    all_permission_names: Vec<String>,

    // Role data
    roles: HashMap<String, Role>,
    all_role_names: Vec<String>,

    // Indexes
    permission_to_roles: HashMap<String, Vec<String>>,
    service_to_permissions: HashMap<String, Vec<String>>,
}

impl SearchEngine {
    pub fn new() -> Self {
        SearchEngine {
            permissions: HashMap::new(),
            all_permission_names: Vec::new(),
            roles: HashMap::new(),
            all_role_names: Vec::new(),
            permission_to_roles: HashMap::new(),
            service_to_permissions: HashMap::new(),
        }
    }

    /// Add a role with its permissions
    pub fn index_role(&mut self, name: String, title: String, description: String, stage: String, permissions: Vec<String>) {
        // Index each permission and create reverse mapping
        for perm_name in &permissions {
            self.permission_to_roles
                .entry(perm_name.clone())
                .or_insert_with(Vec::new)
                .push(name.clone());

            // Auto-create permission if not exists
            if !self.permissions.contains_key(perm_name) {
                let parts: Vec<&str> = perm_name.split('.').collect();
                let service = parts.first().unwrap_or(&"").to_string();
                let resource = parts.get(1).unwrap_or(&"").to_string();
                let action = parts.get(2).unwrap_or(&"").to_string();

                self.permissions.insert(perm_name.clone(), Permission {
                    name: perm_name.clone(),
                    service: service.clone(),
                    resource,
                    action,
                    granted_by_roles: vec![],
                });
                self.all_permission_names.push(perm_name.clone());

                self.service_to_permissions
                    .entry(service)
                    .or_insert_with(Vec::new)
                    .push(perm_name.clone());
            }
        }

        let role = Role {
            name: name.clone(),
            title,
            description,
            stage,
            included_permissions: permissions,
        };

        self.roles.insert(name.clone(), role);
        self.all_role_names.push(name);
    }

    /// Add a standalone permission (not from a role)
    pub fn index_permission(&mut self, name: String, service: String) {
        if self.permissions.contains_key(&name) {
            return;
        }

        let parts: Vec<&str> = name.split('.').collect();
        let resource = parts.get(1).unwrap_or(&"").to_string();
        let action = parts.get(2).unwrap_or(&"").to_string();

        self.permissions.insert(name.clone(), Permission {
            name: name.clone(),
            service: service.clone(),
            resource,
            action,
            granted_by_roles: vec![],
        });
        self.all_permission_names.push(name.clone());

        self.service_to_permissions
            .entry(service)
            .or_insert_with(Vec::new)
            .push(name);
    }

    /// Finalize indexes after loading all data
    pub fn finalize(&mut self) {
        // Update permissions with their granting roles
        for (perm_name, perm) in self.permissions.iter_mut() {
            if let Some(roles) = self.permission_to_roles.get(perm_name) {
                perm.granted_by_roles = roles.clone();
            }
        }
    }

    /// Search permissions with associated roles
    pub fn search_permissions(&self, query: &str, mode: &str, threshold: f64) -> Vec<PermissionSearchResult> {
        let matches: Vec<(&String, f64)> = match mode {
            "exact" => {
                if let Some(perm) = self.permissions.get(query) {
                    vec![(&perm.name, 1.0)]
                } else {
                    vec![]
                }
            }
            "prefix" => {
                let query_lower = query.to_lowercase();
                self.all_permission_names
                    .iter()
                    .filter(|name| name.to_lowercase().starts_with(&query_lower))
                    .map(|name| (name, 0.9))
                    .collect()
            }
            _ => { // fuzzy
                let query_lower = query.to_lowercase();
                let query_ngrams = self.extract_ngrams(&query_lower, 3);

                self.all_permission_names
                    .iter()
                    .filter_map(|name| {
                        let name_lower = name.to_lowercase();
                        // Also check if query is contained in name (substring match)
                        if name_lower.contains(&query_lower) {
                            return Some((name, 0.85));
                        }
                        let name_ngrams = self.extract_ngrams(&name_lower, 3);
                        let score = self.calculate_similarity(&query_ngrams, &name_ngrams);
                        if score >= threshold {
                            Some((name, score))
                        } else {
                            None
                        }
                    })
                    .collect()
            }
        };

        matches
            .into_iter()
            .take(20)
            .filter_map(|(name, score)| {
                self.permissions.get(name).map(|perm| {
                    let granted_by_roles: Vec<RoleSummary> = self.permission_to_roles
                        .get(name)
                        .map(|role_names| {
                            role_names.iter()
                                .filter_map(|rn| self.roles.get(rn))
                                .map(|r| RoleSummary {
                                    name: r.name.clone(),
                                    title: r.title.clone(),
                                    stage: r.stage.clone(),
                                })
                                .take(5) // Limit to 5 roles per permission
                                .collect()
                        })
                        .unwrap_or_default();

                    PermissionSearchResult {
                        name: perm.name.clone(),
                        service: perm.service.clone(),
                        resource: perm.resource.clone(),
                        action: perm.action.clone(),
                        score,
                        granted_by_roles,
                    }
                })
            })
            .collect()
    }

    /// Search roles with their permissions
    pub fn search_roles(&self, query: &str, mode: &str, threshold: f64) -> Vec<RoleSearchResult> {
        let matches: Vec<(&String, f64)> = match mode {
            "exact" => {
                if let Some(role) = self.roles.get(query) {
                    vec![(&role.name, 1.0)]
                } else {
                    vec![]
                }
            }
            "prefix" => {
                let query_lower = query.to_lowercase();
                self.all_role_names
                    .iter()
                    .filter(|name| {
                        let role = self.roles.get(*name).unwrap();
                        name.to_lowercase().starts_with(&query_lower) ||
                        role.title.to_lowercase().starts_with(&query_lower)
                    })
                    .map(|name| (name, 0.9))
                    .collect()
            }
            _ => { // fuzzy
                let query_lower = query.to_lowercase();
                let query_ngrams = self.extract_ngrams(&query_lower, 3);

                self.all_role_names
                    .iter()
                    .filter_map(|name| {
                        let role = self.roles.get(name)?;
                        let name_lower = name.to_lowercase();
                        let title_lower = role.title.to_lowercase();

                        // Substring match
                        if name_lower.contains(&query_lower) || title_lower.contains(&query_lower) {
                            return Some((name, 0.85));
                        }

                        let name_ngrams = self.extract_ngrams(&name_lower, 3);
                        let title_ngrams = self.extract_ngrams(&title_lower, 3);
                        let name_score = self.calculate_similarity(&query_ngrams, &name_ngrams);
                        let title_score = self.calculate_similarity(&query_ngrams, &title_ngrams);
                        let score = name_score.max(title_score);

                        if score >= threshold {
                            Some((name, score))
                        } else {
                            None
                        }
                    })
                    .collect()
            }
        };

        matches
            .into_iter()
            .take(20)
            .filter_map(|(name, score)| {
                self.roles.get(name).map(|role| {
                    RoleSearchResult {
                        name: role.name.clone(),
                        title: role.title.clone(),
                        description: role.description.clone(),
                        stage: role.stage.clone(),
                        score,
                        permission_count: role.included_permissions.len(),
                        sample_permissions: role.included_permissions.iter().take(5).cloned().collect(),
                    }
                })
            })
            .collect()
    }

    /// Legacy exact search for backward compatibility
    pub fn search_exact(&self, query: &str) -> Option<SearchResult<String>> {
        self.permissions
            .get(query)
            .map(|perm| SearchResult {
                item: perm.name.clone(),
                score: 1.0,
            })
    }

    /// Legacy prefix search
    pub fn search_prefix(&self, query: &str) -> Vec<SearchResult<String>> {
        let query_lower = query.to_lowercase();
        self.all_permission_names
            .iter()
            .filter(|perm| perm.to_lowercase().starts_with(&query_lower))
            .map(|perm| SearchResult {
                item: perm.clone(),
                score: 0.8,
            })
            .take(20)
            .collect()
    }

    /// Legacy fuzzy search
    pub fn search_fuzzy(&self, query: &str, threshold: f64) -> Vec<SearchResult<String>> {
        let query_lower = query.to_lowercase();
        let query_ngrams = self.extract_ngrams(&query_lower, 3);

        self.all_permission_names
            .iter()
            .filter_map(|perm| {
                let perm_lower = perm.to_lowercase();
                // Substring match boost
                if perm_lower.contains(&query_lower) {
                    return Some(SearchResult {
                        item: perm.clone(),
                        score: 0.85,
                    });
                }
                let perm_ngrams = self.extract_ngrams(&perm_lower, 3);
                let score = self.calculate_similarity(&query_ngrams, &perm_ngrams);

                if score >= threshold {
                    Some(SearchResult {
                        item: perm.clone(),
                        score,
                    })
                } else {
                    None
                }
            })
            .take(20)
            .collect()
    }

    /// Get stats
    pub fn stats(&self) -> (usize, usize) {
        (self.permissions.len(), self.roles.len())
    }

    /// Extract n-grams from a string
    fn extract_ngrams(&self, text: &str, n: usize) -> Vec<String> {
        if text.len() < n {
            return vec![text.to_string()];
        }

        text.chars()
            .collect::<Vec<_>>()
            .windows(n)
            .map(|window| window.iter().collect::<String>())
            .collect()
    }

    /// Calculate Jaccard similarity between two n-gram sets
    fn calculate_similarity(&self, set1: &[String], set2: &[String]) -> f64 {
        if set1.is_empty() && set2.is_empty() {
            return 1.0;
        }

        let set1_unique: HashSet<_> = set1.iter().collect();
        let set2_unique: HashSet<_> = set2.iter().collect();

        let intersection = set1_unique.intersection(&set2_unique).count();
        let union = set1_unique.union(&set2_unique).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}
