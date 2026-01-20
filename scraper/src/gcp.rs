use crate::error::{Result, ScraperError};
use crate::models::{IamRole, IamPermission, RawGcpData, GcpRolesResponse};
use chrono::Utc;
use std::time::Duration;
use std::collections::HashSet;
use tracing::{info, warn, debug};

const GCP_IAM_API_BASE: &str = "https://iam.googleapis.com/v1/roles";
const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF_MS: u64 = 100;

/// GCP IAM API client
pub struct GcpClient {
    client: reqwest::Client,
    access_token: String,
}

impl GcpClient {
    /// Create a new GCP client with service account authentication
    pub async fn new() -> Result<Self> {
        // Get credentials from environment
        let credentials_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS")
            .map_err(|_| ScraperError::EnvError(
                "GOOGLE_APPLICATION_CREDENTIALS environment variable not set".to_string()
            ))?;

        info!("Loading GCP credentials from: {}", credentials_path);

        // For now, we use a placeholder. In real implementation, we'd parse the service account JSON
        // and use yup-oauth2 to get a real access token from GCP
        let access_token = std::env::var("GCP_ACCESS_TOKEN")
            .unwrap_or_else(|_| "mock-token".to_string());

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ScraperError::HttpError(e))?;

        Ok(GcpClient {
            client,
            access_token,
        })
    }

    /// Fetch all roles and permissions from GCP
    pub async fn fetch_all_data(&self) -> Result<RawGcpData> {
        info!("Starting to fetch all roles and permissions from GCP IAM API");

        let mut all_roles = Vec::new();
        let mut page_token: Option<String> = None;
        let mut role_count = 0;

        // Fetch all roles with pagination
        loop {
            match self.list_roles(page_token.clone()).await {
                Ok(response) => {
                    let count = response.roles.len();
                    role_count += count;
                    debug!("Fetched {} roles in this page (total: {})", count, role_count);

                    all_roles.extend(response.roles.into_iter().map(IamRole::from_gcp));

                    page_token = response.next_page_token;
                    if page_token.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    if e.is_rate_limit_error() {
                        warn!("Rate limited by GCP API, waiting before retry...");
                        tokio::time::sleep(Duration::from_secs(10)).await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }

        info!("Fetched {} total roles", all_roles.len());

        // Extract unique permissions from all roles
        let mut unique_permissions = HashSet::new();
        for role in &all_roles {
            for perm_name in &role.included_permissions {
                unique_permissions.insert(perm_name.clone());
            }
        }

        let permissions: Vec<IamPermission> = unique_permissions
            .into_iter()
            .map(IamPermission::from_name)
            .collect();

        info!("Extracted {} unique permissions", permissions.len());

        let fetched_at = Utc::now().to_rfc3339();

        Ok(RawGcpData {
            roles: all_roles,
            permissions,
            fetched_at,
        })
    }

    /// Fetch roles with pagination and retry logic
    async fn list_roles(&self, page_token: Option<String>) -> Result<GcpRolesResponse> {
        let mut retry_count = 0;

        loop {
            let url = if let Some(token) = &page_token {
                format!("{}?pageToken={}&pageSize=1000", GCP_IAM_API_BASE, token)
            } else {
                format!("{}?pageSize=1000", GCP_IAM_API_BASE)
            };

            match self.fetch_with_auth(&url).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if e.is_rate_limit_error() && retry_count < MAX_RETRIES {
                        let backoff_ms = INITIAL_BACKOFF_MS * 2u64.pow(retry_count);
                        warn!(
                            "Rate limit error, retrying after {}ms (attempt {}/{})",
                            backoff_ms, retry_count + 1, MAX_RETRIES
                        );
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                        retry_count += 1;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }

    /// Fetch data from GCP API with authentication
    async fn fetch_with_auth(&self, url: &str) -> Result<GcpRolesResponse> {
        let response = self
            .client
            .get(url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| ScraperError::HttpError(e))?;

        let status = response.status();

        if status == 401 || status == 403 {
            return Err(ScraperError::GcpAuthError(
                "Unauthorized to access GCP IAM API. Check service account permissions.".to_string(),
            ));
        }

        if status == 429 {
            return Err(ScraperError::GcpRateLimitError(
                "GCP API rate limit exceeded".to_string(),
            ));
        }

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ScraperError::GcpApiError(format!(
                "GCP API error ({}): {}",
                status, error_text
            )));
        }

        response
            .json::<GcpRolesResponse>()
            .await
            .map_err(|e| ScraperError::HttpError(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gcp_client_creation_without_credentials() {
        // Should fail when GOOGLE_APPLICATION_CREDENTIALS is not set
        // Clear the env var if it exists
        std::env::remove_var("GOOGLE_APPLICATION_CREDENTIALS");
        let client = GcpClient::new().await;
        assert!(client.is_err());
        if let Err(ScraperError::EnvError(msg)) = client {
            assert!(msg.contains("GOOGLE_APPLICATION_CREDENTIALS"));
        }
    }

    #[tokio::test]
    async fn test_gcp_client_creation_with_credentials() {
        // Mock credentials path
        std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", "/tmp/mock-creds.json");
        let client = GcpClient::new().await;
        assert!(client.is_ok());
    }

    #[test]
    fn test_auth_error_detection() {
        let auth_err = ScraperError::GcpAuthError("Unauthorized".to_string());
        assert!(auth_err.is_auth_error());
        assert!(!auth_err.is_rate_limit_error());
    }

    #[test]
    fn test_rate_limit_detection() {
        let rate_err = ScraperError::GcpRateLimitError("Rate limited".to_string());
        assert!(!rate_err.is_auth_error());
        assert!(rate_err.is_rate_limit_error());
    }
}
