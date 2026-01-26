# Fastly to BigQuery Logging via Terraform

This Terraform configuration sets up comprehensive logging from Fastly CDN to Google BigQuery, with all available fields captured for detailed analytics and monitoring.

## Architecture Overview

```
Fastly Service (eUQUMxFI5qCyFrJ9pxGcy9)
         ↓
   [Fastly Backend]
         ↓
  Service Account
         ↓
  BigQuery Dataset (fastly_logs)
         ↓
  BigQuery Table (access_logs)
         ↓
  Analysis & Queries
```

## Prerequisites

1. **GCP Project**: `gcpiam` (must exist before running Terraform)
2. **Fastly Account**: With service already created (`gcpiam-search`)
3. **Terraform**: >= 1.5
4. **gcloud CLI**: Authenticated to your GCP project
5. **Fastly API Token**: Available from Fastly dashboard

## Files

| File | Purpose |
|------|---------|
| `main.tf` | Terraform and provider configuration |
| `variables.tf` | Input variables definition |
| `backend.tf` | GCS bucket for remote state |
| `gcp.tf` | GCP service account and IAM setup |
| `bigquery.tf` | BigQuery dataset and table |
| `fastly.tf` | Fastly BigQuery logging backend |
| `outputs.tf` | Output values |
| `terraform.tfvars` | Default values (committed to repo) |
| `.gitignore` | Git ignore rules |

## Deployment Instructions

### Step 1: Set Environment Variables

```bash
# Set your Fastly API token (get from https://manage.fastly.com/account/tokens)
export FASTLY_API_TOKEN="your-fastly-api-token-here"

# Verify GCP authentication
gcloud auth application-default login
gcloud config set project gcpiam
```

### Step 2: Initialize Terraform (First Time Only)

```bash
cd terraform

# This will fail initially because backend bucket doesn't exist yet
terraform init
```

### Step 3: Create State Bucket

```bash
# Apply only the state bucket
terraform apply -target=google_storage_bucket.terraform_state

# Review and confirm the plan when prompted
```

### Step 4: Configure Remote Backend

```bash
# Now reconfigure Terraform to use the remote state bucket
terraform init -backend-config="bucket=gcpiam-terraform-state"

# Confirm migration of local state (answer "yes")
```

### Step 5: Review and Deploy

```bash
# Review all resources that will be created
terraform plan

# Create all infrastructure
terraform apply

# Confirm when prompted (answer "yes")
```

### Step 6: Verify Deployment

```bash
# Check outputs
terraform output

# Verify BigQuery dataset
bq ls -d --project_id=gcpiam

# Verify BigQuery table
bq show gcpiam.fastly_logs.access_logs

# Check Fastly service has logging backend
gcloud compute backend-buckets list  # Or use Fastly API
```

## Testing the Logging

### Make a Test Request

```bash
# Make a request to trigger logging
curl -I https://gcpiam.com/

# Wait 1-5 minutes for logs to appear (Fastly batches logs)
```

### Query the Logs

```bash
# Simple query to verify data
bq query --project_id=gcpiam << 'EOF'
SELECT
  timestamp,
  client_ip,
  request_method,
  request_uri,
  response_status,
  cache_status,
  edge_location
FROM `gcpiam.fastly_logs.access_logs`
ORDER BY timestamp DESC
LIMIT 10
EOF
```

### Useful Queries

**Cache Hit Ratio Analysis**:
```sql
SELECT
  cache_status,
  COUNT(*) as request_count,
  ROUND(100.0 * COUNT(*) / SUM(COUNT(*)) OVER (), 2) as percentage
FROM `gcpiam.fastly_logs.access_logs`
WHERE DATE(timestamp) = CURRENT_DATE()
GROUP BY cache_status
ORDER BY request_count DESC
```

**Geographic Traffic Analysis**:
```sql
SELECT
  client_country,
  client_city,
  COUNT(*) as request_count,
  AVG(CAST(response_status AS FLOAT64)) as avg_status
FROM `gcpiam.fastly_logs.access_logs`
WHERE DATE(timestamp) = CURRENT_DATE()
GROUP BY client_country, client_city
ORDER BY request_count DESC
LIMIT 20
```

**Response Time Analysis**:
```sql
SELECT
  DATETIME_TRUNC(timestamp, HOUR) as hour,
  ROUND(AVG(CAST(edge_response_time AS FLOAT64)), 2) as avg_edge_time_ms,
  ROUND(AVG(CAST(origin_response_time AS FLOAT64)), 2) as avg_origin_time_ms,
  COUNT(*) as request_count
FROM `gcpiam.fastly_logs.access_logs`
WHERE DATE(timestamp) = CURRENT_DATE()
GROUP BY hour
ORDER BY hour DESC
```

## Managing Terraform State

### Remote State Bucket

State is stored in: `gs://gcpiam-terraform-state/`

**Features**:
- Versioning enabled (keeps last 5 versions)
- Automatic cleanup of old versions
- Encrypted by default in GCS

**To view state history**:
```bash
gsutil ls -L gs://gcpiam-terraform-state/
```

**To restore from backup**:
```bash
# List versions
gsutil versioning get gs://gcpiam-terraform-state/

# Copy specific version if needed
gsutil cp gs://gcpiam-terraform-state/#<version-id> terraform.tfstate
```

## Scaling and Customization

### Add More Logging Fields

Edit `bigquery.tf` to add more fields to the schema:
```hcl
{ name = "new_field", type = "STRING", description = "Description" }
```

Then in `fastly.tf`, add the field to the format JSON:
```hcl
new_field = "%{new_field_variable}V"
```

### Adjust Partitioning and Clustering

In `bigquery.tf`, modify the `time_partitioning` and `clustering` blocks:
```hcl
time_partitioning {
  type = "DAY"
  field = "timestamp"
  expiration_ms = 7776000000  # 90 days
}

clustering = ["service_id", "response_status", "cache_status"]
```

### Enable Cost Optimization

Add table expiration:
```hcl
resource "google_bigquery_table" "fastly_access_logs" {
  ...
  expiration_time = 7776000000  # 90 days in milliseconds
}
```

## Troubleshooting

### "Failed to connect to Fastly API"

**Issue**: `Error: error reading Fastly service`

**Solution**:
```bash
# Verify token is set
echo $FASTLY_API_TOKEN

# Check token is valid
curl -i https://api.fastly.com/service \
  -H "Fastly-Key: $FASTLY_API_TOKEN"
```

### "BigQuery quota exceeded"

**Solution**: Increase limits in GCP Console or use dataset-level quotas.

### Logs not appearing in BigQuery

**Debugging**:
1. Check Fastly service has logging enabled:
   ```bash
   terraform state show fastly_service_logging_bigquery.gcpiam_logs
   ```

2. Check BigQuery table permissions:
   ```bash
   bq show --project_id=gcpiam gcpiam.fastly_logs.access_logs
   ```

3. Verify service account has permissions:
   ```bash
   gcloud projects get-iam-policy gcpiam \
     --flatten=bindings[].members \
     --filter="bindings.role:roles/bigquery.*"
   ```

## Maintenance

### Regular Backups

```bash
# Export state bucket to local backup
gsutil -m cp -r gs://gcpiam-terraform-state/ ./backup/

# Or create scheduled backup via GCS lifecycle
```

### Update Terraform Providers

```bash
terraform init -upgrade
terraform plan  # Review changes
terraform apply
```

### Monitor Costs

```bash
# BigQuery estimated bytes scanned
bq show --project_id=gcpiam gcpiam.fastly_logs.access_logs

# GCS storage usage
gsutil du -sh gs://gcpiam-terraform-state/
```

## Clean Up

To destroy all resources (except GCS state bucket):

```bash
# Remove logging backend first
terraform destroy -target=fastly_service_logging_bigquery.gcpiam_logs

# Then remove other resources
terraform destroy

# Remove state bucket separately if desired
terraform destroy -target=google_storage_bucket.terraform_state
```

## Support

For issues with:
- **Fastly**: https://support.fastly.com/
- **BigQuery**: https://cloud.google.com/bigquery/docs
- **Terraform**: https://www.terraform.io/docs
