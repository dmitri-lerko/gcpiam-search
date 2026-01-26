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
    { name = "client_country", type = "STRING", description = "Client country code (2-letter)" },
    { name = "client_city", type = "STRING", description = "Client city" },
    { name = "client_asn", type = "INTEGER", description = "Client ASN" },
    { name = "client_latitude", type = "FLOAT64", description = "Client latitude" },
    { name = "client_longitude", type = "FLOAT64", description = "Client longitude" },
    { name = "client_postal_code", type = "STRING", description = "Client postal/ZIP code" },
    { name = "client_region", type = "STRING", description = "Client region/state" },
    { name = "client_gmt_offset", type = "STRING", description = "Client timezone offset from GMT" },
    { name = "client_area_code", type = "STRING", description = "Client telephone area code" },
    { name = "client_dma_code", type = "STRING", description = "Client DMA code (US market)" },
    { name = "user_agent", type = "STRING", description = "Client user agent" },

    # Request information
    { name = "request_method", type = "STRING", description = "HTTP method" },
    { name = "request_uri", type = "STRING", description = "Request URI" },
    { name = "request_protocol", type = "STRING", description = "HTTP protocol" },
    { name = "request_host", type = "STRING", description = "Request host header" },
    { name = "request_referrer", type = "STRING", description = "Referrer header" },

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
    { name = "is_tls", type = "BOOLEAN", description = "Whether connection is over TLS" },

    # Origin information
    { name = "origin_response_time", type = "INTEGER", description = "Origin response time (ms)" },
    { name = "origin_status", type = "INTEGER", description = "Origin status code" },

    # TLS/SSL Protocol information
    { name = "tls_protocol", type = "STRING", description = "TLS protocol version (e.g., TLSv1.3)" },
    { name = "tls_cipher", type = "STRING", description = "TLS cipher suite" },
    { name = "tls_sni", type = "STRING", description = "Server Name Indication (SNI)" },

    # TLS Fingerprinting (Security Analysis)
    { name = "tls_ja4", type = "STRING", description = "JA4 TLS fingerprint" },
    { name = "tls_ja3_md5", type = "STRING", description = "JA3 MD5 TLS fingerprint" },
    { name = "tls_extensions_sha", type = "STRING", description = "SHA hash of TLS extensions (Base64)" },

    # Client Certificate (mTLS)
    { name = "cert_is_verified", type = "BOOLEAN", description = "Whether client certificate chain was verified" },
    { name = "cert_serial_number", type = "STRING", description = "Client certificate serial number" },
    { name = "cert_issuer_dn", type = "STRING", description = "Certificate issuer distinguished name" },
    { name = "cert_subject_dn", type = "STRING", description = "Certificate subject distinguished name" },
    { name = "cert_validity_start", type = "STRING", description = "Certificate validity start timestamp" },
    { name = "cert_validity_end", type = "STRING", description = "Certificate validity end timestamp" },
    { name = "cert_fingerprint", type = "STRING", description = "Certificate fingerprint" },

    # TCP Connection metrics
    { name = "tcp_rtt", type = "INTEGER", description = "TCP round-trip time (RTT in ms)" },
    { name = "tcp_rtt_variance", type = "INTEGER", description = "TCP RTT variance" },
    { name = "tcp_cwnd", type = "INTEGER", description = "TCP congestion window size" },
    { name = "tcp_mss", type = "INTEGER", description = "TCP maximum segment size" },
    { name = "tcp_ttl", type = "INTEGER", description = "TCP time to live" },

    # Service information
    { name = "service_id", type = "STRING", description = "Fastly service ID" }
  ])

  time_partitioning {
    type = "DAY"
    field = "timestamp"
  }

  clustering = ["service_id", "response_status", "cache_status"]
}
