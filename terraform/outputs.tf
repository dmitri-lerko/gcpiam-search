output "bigquery_dataset_id" {
  value       = google_bigquery_dataset.fastly_logs.dataset_id
  description = "BigQuery dataset ID for Fastly logs"
}

output "bigquery_table_id" {
  value       = google_bigquery_table.fastly_access_logs.table_id
  description = "BigQuery table ID for Fastly logs"
}

output "fastly_sa_email" {
  value       = google_service_account.fastly_logging.email
  description = "Service account email for Fastly logging"
}

output "state_bucket_name" {
  value       = google_storage_bucket.terraform_state.name
  description = "GCS bucket name for Terraform state"
}

output "terraform_init_command" {
  value       = "terraform init -backend-config=\"bucket=${google_storage_bucket.terraform_state.name}\""
  description = "Command to initialize Terraform with remote backend"
}

output "fastly_sa_private_key_json" {
  value       = sensitive(google_service_account_key.fastly_logging.private_key)
  description = "Service account private key (base64 encoded) for Fastly logging"
  sensitive   = true
}
