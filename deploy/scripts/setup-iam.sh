#!/bin/bash
# =============================================================================
# LLM-Test-Bench - IAM Setup Script
# =============================================================================
# Creates service account and configures permissions
# Usage: ./setup-iam.sh [project-id]
# =============================================================================

set -euo pipefail

PROJECT_ID="${1:-$(gcloud config get-value project)}"
SERVICE_NAME="llm-test-bench"
SA_NAME="$SERVICE_NAME-sa"
SA_EMAIL="$SA_NAME@$PROJECT_ID.iam.gserviceaccount.com"

echo "============================================================"
echo "LLM-Test-Bench IAM Setup"
echo "============================================================"
echo "Project: $PROJECT_ID"
echo "Service Account: $SA_EMAIL"
echo "============================================================"

# -----------------------------------------------------------------------------
# Create Service Account
# -----------------------------------------------------------------------------

echo ""
echo "📋 Creating service account..."

if gcloud iam service-accounts describe "$SA_EMAIL" &>/dev/null; then
  echo "  ⚠️  Service account already exists"
else
  gcloud iam service-accounts create "$SA_NAME" \
    --display-name="LLM Test Bench Service Account" \
    --description="Service account for LLM-Test-Bench Cloud Run service"
  echo "  ✅ Service account created"
fi

# -----------------------------------------------------------------------------
# Assign Roles
# -----------------------------------------------------------------------------

echo ""
echo "📋 Assigning IAM roles..."

ROLES=(
  # Cloud Run
  "roles/run.invoker"

  # Secret Manager (for API keys)
  "roles/secretmanager.secretAccessor"

  # Logging
  "roles/logging.logWriter"

  # Monitoring
  "roles/monitoring.metricWriter"

  # Tracing
  "roles/cloudtrace.agent"
)

for role in "${ROLES[@]}"; do
  echo "  Adding $role..."
  gcloud projects add-iam-policy-binding "$PROJECT_ID" \
    --member="serviceAccount:$SA_EMAIL" \
    --role="$role" \
    --quiet
done

echo "  ✅ Roles assigned"

# -----------------------------------------------------------------------------
# Create Secrets (if needed)
# -----------------------------------------------------------------------------

echo ""
echo "📋 Setting up secrets..."

SECRETS=(
  "ruvector-api-key"
  "openai-api-key"
  "anthropic-api-key"
  "google-ai-api-key"
  "mistral-api-key"
  "groq-api-key"
)

for secret in "${SECRETS[@]}"; do
  if gcloud secrets describe "$secret" &>/dev/null; then
    echo "  ⚠️  Secret '$secret' already exists"
  else
    echo "  Creating secret '$secret'..."
    echo -n "placeholder" | gcloud secrets create "$secret" \
      --data-file=- \
      --replication-policy="automatic"
    echo "  ⚠️  Created '$secret' with placeholder value - UPDATE THIS!"
  fi

  # Grant access to service account
  gcloud secrets add-iam-policy-binding "$secret" \
    --member="serviceAccount:$SA_EMAIL" \
    --role="roles/secretmanager.secretAccessor" \
    --quiet
done

echo "  ✅ Secrets configured"

# -----------------------------------------------------------------------------
# Done
# -----------------------------------------------------------------------------

echo ""
echo "============================================================"
echo "✅ IAM Setup Complete!"
echo "============================================================"
echo ""
echo "Service Account: $SA_EMAIL"
echo ""
echo "⚠️  IMPORTANT: Update the following secrets with real values:"
for secret in "${SECRETS[@]}"; do
  echo "    gcloud secrets versions add $secret --data-file=-"
done
echo ""
