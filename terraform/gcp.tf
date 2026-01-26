# Service account for Fastly to authenticate to BigQuery
resource "google_service_account" "fastly_logging" {
  account_id   = "fastly-logging"
  display_name = "Fastly BigQuery Logging Service Account"
  project      = var.gcp_project_id
}

# Get the service account email
locals {
  fastly_sa_email = google_service_account.fastly_logging.email
}

# Create and manage service account key
resource "google_service_account_key" "fastly_logging" {
  service_account_id = google_service_account.fastly_logging.name
  public_key_type    = "TYPE_X509_PEM"
}

# IAM role: BigQuery Data Editor (for inserting logs)
resource "google_project_iam_member" "fastly_bigquery_editor" {
  project = var.gcp_project_id
  role    = "roles/bigquery.dataEditor"
  member  = "serviceAccount:${local.fastly_sa_email}"
}

# IAM role: BigQuery Job User (for running insert jobs)
resource "google_project_iam_member" "fastly_bigquery_job_user" {
  project = var.gcp_project_id
  role    = "roles/bigquery.jobUser"
  member  = "serviceAccount:${local.fastly_sa_email}"
}
