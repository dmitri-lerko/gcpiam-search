terraform {
  required_version = ">= 1.5"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
    fastly = {
      source  = "fastly/fastly"
      version = "~> 1.0"
    }
  }

  # Backend configured in backend.tf with dynamic bucket reference
}

provider "google" {
  project = var.gcp_project_id
  region  = var.gcp_region
}

provider "fastly" {
  api_key = var.fastly_api_token
}
