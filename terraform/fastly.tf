# Get the existing Fastly service
data "fastly_service" "gcpiam" {
  name = "gcpiam-search"
}

# Get the active version
locals {
  service_id     = data.fastly_service.gcpiam.id
  active_version = data.fastly_service.gcpiam.active_version
}

# BigQuery logging configuration for Fastly
resource "fastly_service_logging_bigquery" "gcpiam_logs" {
  service_id  = local.service_id
  version     = local.active_version
  name        = "gcpiam-bigquery-logging"
  project_id  = var.gcp_project_id
  dataset     = google_bigquery_dataset.fastly_logs.dataset_id
  table       = google_bigquery_table.fastly_access_logs.table_id
  secret_key  = base64decode(google_service_account_key.fastly_logging.private_key)
  gzip_level  = 9

  # Fields to log - ALL available fields for comprehensive logging
  format = jsonencode({
    timestamp                = "%{timestamp}V"
    time_elapsed             = "%{time_elapsed}V"
    client_ip                = "%{client_ip}V"
    client_country           = "%{client_country}V"
    client_city              = "%{client_city}V"
    client_asn               = "%{client_asn}V"
    user_agent               = "%{http_user_agent}V"
    request_method           = "%{request_method}V"
    request_uri              = "%{request_uri}V"
    request_protocol         = "%{request_protocol}V"
    request_host             = "%{http_host}V"
    request_referrer         = "%{http_referer}V"
    response_status          = "%{status}V"
    response_size            = "%{bytes_sent}V"
    response_body_size       = "%{http_content_length}V"
    cache_status             = "%{fastly_cachestatus}V"
    cache_action             = "%{fastly_cache_action}V"
    edge_location            = "%{pop}V"
    edge_server              = "%{server_identity}V"
    edge_response_time       = "%{time_firstbyte}V"
    origin_response_time     = "%{origin_time}V"
    origin_status            = "%{origin_status}V"
    connection_ssl_protocol  = "%{ssl_protocol}V"
    connection_ssl_cipher    = "%{ssl_cipher}V"
    service_id               = "%{fastly_service_id}V"
  })

  response_condition = ""
}
