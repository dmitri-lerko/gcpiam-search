# GCS bucket for storing Terraform state
resource "google_storage_bucket" "terraform_state" {
  name          = var.state_bucket_name
  project       = var.gcp_project_id
  location      = var.state_bucket_location
  force_destroy = false

  versioning {
    enabled = true
  }

  uniform_bucket_level_access = true

  lifecycle_rule {
    action {
      type = "Delete"
    }
    condition {
      num_newer_versions = 5
    }
  }
}

# Enable state locking with GCS
# Note: Backend is configured via terraform init -backend-config="bucket=gcpiam-terraform-state"
