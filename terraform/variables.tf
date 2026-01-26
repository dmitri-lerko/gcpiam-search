variable "gcp_project_id" {
  type        = string
  description = "GCP Project ID"
  default     = "gcpiam"
}

variable "gcp_region" {
  type        = string
  description = "GCP Region"
  default     = "us-central1"
}

variable "fastly_api_token" {
  type        = string
  description = "Fastly API token"
  sensitive   = true
}

variable "fastly_service_id" {
  type        = string
  description = "Fastly service ID for gcpiam-search"
  default     = "eUQUMxFI5qCyFrJ9pxGcy9"
}

variable "bigquery_dataset_id" {
  type        = string
  description = "BigQuery dataset ID for Fastly logs"
  default     = "fastly_logs"
}

variable "bigquery_table_id" {
  type        = string
  description = "BigQuery table ID for Fastly logs"
  default     = "access_logs"
}

variable "state_bucket_name" {
  type        = string
  description = "GCS bucket name for Terraform state"
  default     = "gcpiam-terraform-state"
}

variable "state_bucket_location" {
  type        = string
  description = "GCS bucket location for Terraform state"
  default     = "US"
}
