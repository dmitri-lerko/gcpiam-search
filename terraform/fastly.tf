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

# MANUAL SETUP REQUIRED:
# Configure BigQuery logging in Fastly dashboard at https://manage.fastly.com/
#
# Steps:
# 1. Select service: gcpiam-search
# 2. Go to Logging > BigQuery
# 3. Create new endpoint with:
#    - Name: gcpiam-bigquery-logging
#    - Project ID: mygha-461120
#    - Dataset: fastly_logs
#    - Table: access_logs
#    - Service Account JSON Key: (from terraform output fastly_sa_private_key_json)
#    - Log Line Format: (from terraform output bigquery_logging_format)
#    - Placement: none (required for WASM services)
# 4. Save and activate
#
# To get the credentials:
#   terraform output -raw fastly_sa_private_key_json > key.json
#   terraform output -raw bigquery_logging_format
#
# Service Account Email: fastly-logging@mygha-461120.iam.gserviceaccount.com
