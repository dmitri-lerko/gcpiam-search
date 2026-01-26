# Note: To enable Fastly to BigQuery logging, you must configure it via Fastly API
# The Terraform Fastly provider does not currently support BigQuery logging backend
#
# LOGGING FORMAT (JSON) for comprehensive BigQuery logging:
# Include the following format string in your Fastly BigQuery logging backend configuration
locals {
  bigquery_logging_format = jsonencode({
    # Timestamp and basic info
    timestamp      = "%%{timestamp}V"
    time_elapsed   = "%%{time_elapsed}V"

    # Client information
    client_ip              = "%%{client_ip}V"
    client_country         = "%%{client_country}V"
    client_city            = "%%{client_city}V"
    client_asn             = "%%{client_asn}V"
    client_latitude        = "%%{client_geo_latitude}V"
    client_longitude       = "%%{client_geo_longitude}V"
    client_postal_code     = "%%{client_geo_postal_code}V"
    client_region          = "%%{client_geo_region}V"
    client_gmt_offset      = "%%{client_geo_gmt_offset}V"
    client_area_code       = "%%{client_geo_area_code}V"
    client_dma_code        = "%%{client_geo_dma_code}V"
    user_agent             = "%%{http_user_agent}V"

    # Request information
    request_method   = "%%{request_method}V"
    request_uri      = "%%{request_uri}V"
    request_protocol = "%%{request_protocol}V"
    request_host     = "%%{http_host}V"
    request_referrer = "%%{http_referer}V"

    # Response information
    response_status    = "%%{status}V"
    response_size      = "%%{bytes_sent}V"
    response_body_size = "%%{http_content_length}V"

    # Cache information
    cache_status = "%%{fastly_cachestatus}V"
    cache_action = "%%{fastly_cache_action}V"

    # Edge information
    edge_location       = "%%{pop}V"
    edge_server         = "%%{server_identity}V"
    edge_response_time  = "%%{time_firstbyte}V"
    is_tls              = "%%{fastly_info.edge.is_tls}V"

    # Origin information
    origin_response_time = "%%{origin_time}V"
    origin_status        = "%%{origin_status}V"

    # TLS/SSL Protocol information
    tls_protocol = "%%{tls.client.protocol}V"
    tls_cipher   = "%%{ssl_cipher}V"
    tls_sni      = "%%{tls.client.servername}V"

    # TLS Fingerprinting (Security Analysis)
    tls_ja4            = "%%{tls.client.ja4}V"
    tls_ja3_md5        = "%%{tls.client.ja3_md5}V"
    tls_extensions_sha = "%%{tls.client.tlsexts_sha}V"

    # Client Certificate (mTLS)
    cert_is_verified      = "%%{tls.client.certificate.is_verified}V"
    cert_serial_number    = "%%{tls.client.certificate.serial_number}V"
    cert_issuer_dn        = "%%{tls.client.certificate.issuer_dn}V"
    cert_subject_dn       = "%%{tls.client.certificate.subject_dn}V"
    cert_validity_start   = "%%{tls.client.certificate.validity_start}V"
    cert_validity_end     = "%%{tls.client.certificate.validity_end}V"
    cert_fingerprint      = "%%{tls.client.certificate.fingerprint}V"

    # TCP Connection metrics
    tcp_rtt          = "%%{client.socket.tcp_info.rtt}V"
    tcp_rtt_variance = "%%{client.socket.tcp_info.rtt_variance}V"
    tcp_cwnd         = "%%{client.socket.tcp_info.cwnd}V"
    tcp_mss          = "%%{client.socket.tcp_info.mss}V"
    tcp_ttl          = "%%{client.socket.ttl}V"

    # Service information
    service_id = "%%{fastly_service_id}V"
  })
}

# Manual Configuration Steps:
# 1. Log in to Fastly dashboard (https://manage.fastly.com/)
# 2. Select the gcpiam-search service
# 3. Go to Logging > BigQuery
# 4. Create new logging endpoint with:
#    - Service: gcpiam-search
#    - Name: gcpiam-bigquery-logging
#    - Project ID: gcpiam
#    - Dataset: fastly_logs
#    - Table: access_logs
#    - User: fastly-logging@gcpiam.iam.gserviceaccount.com
#    - Private Key: (copy from terraform output or GCP Console)
#    - Format: (use the format from 'local.bigquery_logging_format' above, or from README)
#
# To get the logging format JSON, run:
#   terraform output -raw bigquery_logging_format
#
# Or use via curl with the Fastly API:
# curl -X POST https://api.fastly.com/service/{service_id}/version/{version}/logging/bigquery \
#   -H "Fastly-Key: $FASTLY_API_TOKEN" \
#   -H "Content-Type: application/x-www-form-urlencoded" \
#   --data-urlencode "name=gcpiam-bigquery-logging" \
#   --data-urlencode "project_id=gcpiam" \
#   --data-urlencode "dataset=fastly_logs" \
#   --data-urlencode "table=access_logs" \
#   --data-urlencode "secret_key=$(cat key.json | base64)" \
#   --data-urlencode "format=$(terraform output -raw bigquery_logging_format)"
