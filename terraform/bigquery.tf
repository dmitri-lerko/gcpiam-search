# BigQuery Dataset
resource "google_bigquery_dataset" "fastly_logs" {
  dataset_id    = var.bigquery_dataset_id
  project       = var.gcp_project_id
  friendly_name = "Fastly Logs"
  description   = "BigQuery dataset for storing Fastly CDN access logs"
  location      = var.gcp_region

  access {
    role          = "OWNER"
    user_by_email = google_service_account.fastly_logging.email
  }

  access {
    role          = "READER"
    special_group = "projectReaders"
  }
}

# BigQuery Table for Fastly logs (with all available fields)
resource "google_bigquery_table" "fastly_access_logs" {
  dataset_id = google_bigquery_dataset.fastly_logs.dataset_id
  table_id   = var.bigquery_table_id
  project    = var.gcp_project_id

  description = "Fastly CDN access logs with all available fields"

  schema = jsonencode([
    # Timestamp and basic info
    { name = "timestamp", type = "TIMESTAMP", description = "Request timestamp" },
    { name = "time_elapsed", type = "INTEGER", description = "Time elapsed (ms)" },

    # Client information
    { name = "client_ip", type = "STRING", description = "Client IP address" },
    { name = "client_country", type = "STRING", description = "Client country code" },
    { name = "client_city", type = "STRING", description = "Client city" },
    { name = "client_asn", type = "INTEGER", description = "Client ASN" },
    { name = "user_agent", type = "STRING", description = "Client user agent" },

    # Request information
    { name = "request_method", type = "STRING", description = "HTTP method" },
    { name = "request_uri", type = "STRING", description = "Request URI" },
    { name = "request_protocol", type = "STRING", description = "HTTP protocol" },
    { name = "request_host", type = "STRING", description = "Request host header" },
    { name = "request_referrer", type = "STRING", description = "Referrer header" },
    { name = "request_headers", type = "STRING", description = "Request headers (JSON)" },

    # Response information
    { name = "response_status", type = "INTEGER", description = "HTTP status code" },
    { name = "response_size", type = "INTEGER", description = "Response size (bytes)" },
    { name = "response_body_size", type = "INTEGER", description = "Response body size (bytes)" },

    # Cache information
    { name = "cache_status", type = "STRING", description = "Cache status (HIT, MISS, etc.)" },
    { name = "cache_action", type = "STRING", description = "Cache action taken" },

    # Edge information
    { name = "edge_location", type = "STRING", description = "Edge location POP code" },
    { name = "edge_server", type = "STRING", description = "Edge server ID" },
    { name = "edge_response_time", type = "INTEGER", description = "Edge response time (ms)" },

    # Origin information
    { name = "origin_response_time", type = "INTEGER", description = "Origin response time (ms)" },
    { name = "origin_status", type = "INTEGER", description = "Origin status code" },

    # Connection information
    { name = "connection_ssl_protocol", type = "STRING", description = "SSL protocol version" },
    { name = "connection_ssl_cipher", type = "STRING", description = "SSL cipher" },

    # Service information
    { name = "service_id", type = "STRING", description = "Fastly service ID" }
  ])

  time_partitioning {
    type = "DAY"
    field = "timestamp"
  }

  clustering = ["service_id", "response_status", "cache_status"]
}
