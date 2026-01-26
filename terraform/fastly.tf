# Note: To enable Fastly to BigQuery logging, you must configure it via Fastly API
# The Terraform Fastly provider does not currently support BigQuery logging backend
#
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
#
# Alternatively, use Fastly API:
# curl -X POST https://api.fastly.com/service/{service_id}/version/{version}/logging/bigquery \
#   -H "Fastly-Key: $FASTLY_API_TOKEN" \
#   -H "Content-Type: application/x-www-form-urlencoded" \
#   -d "name=gcpiam-bigquery-logging&project_id=gcpiam&dataset=fastly_logs&table=access_logs&secret_key=$(cat key.json | base64)"
