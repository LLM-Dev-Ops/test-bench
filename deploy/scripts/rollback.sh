#!/bin/bash
# =============================================================================
# LLM-Test-Bench - Rollback Script
# =============================================================================
# Rollback to previous revision or specific revision
# Usage: ./rollback.sh [dev|staging|prod] [revision-name]
# =============================================================================

set -euo pipefail

ENV="${1:-dev}"
REVISION="${2:-}"
SERVICE_NAME="llm-test-bench"

# Environment-specific settings
case "$ENV" in
  dev)
    PROJECT_ID="agentics-dev"
    REGION="us-central1"
    ;;
  staging)
    PROJECT_ID="agentics-staging"
    REGION="us-central1"
    ;;
  prod)
    PROJECT_ID="agentics-prod"
    REGION="us-central1"
    ;;
  *)
    echo "❌ Invalid environment: $ENV"
    echo "Usage: $0 [dev|staging|prod] [revision-name]"
    exit 1
    ;;
esac

echo "============================================================"
echo "LLM-Test-Bench Rollback"
echo "============================================================"
echo "Environment: $ENV"
echo "Project:     $PROJECT_ID"
echo "Region:      $REGION"
echo "============================================================"

gcloud config set project "$PROJECT_ID" --quiet

# -----------------------------------------------------------------------------
# List Revisions
# -----------------------------------------------------------------------------

echo ""
echo "📋 Current revisions:"

gcloud run revisions list \
  --service="$SERVICE_NAME" \
  --region="$REGION" \
  --format="table(REVISION,ACTIVE,LAST DEPLOYED,TRAFFIC)"

# -----------------------------------------------------------------------------
# Determine Target Revision
# -----------------------------------------------------------------------------

if [ -z "$REVISION" ]; then
  echo ""
  echo "📋 Getting previous revision..."

  # Get the second most recent revision (current is first)
  REVISION=$(gcloud run revisions list \
    --service="$SERVICE_NAME" \
    --region="$REGION" \
    --format="value(REVISION)" \
    --limit=2 | tail -1)

  if [ -z "$REVISION" ]; then
    echo "❌ No previous revision found"
    exit 1
  fi
fi

echo ""
echo "🎯 Target revision: $REVISION"

# -----------------------------------------------------------------------------
# Confirm Rollback
# -----------------------------------------------------------------------------

read -p "⚠️  Proceed with rollback to $REVISION? [y/N] " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "Rollback cancelled"
  exit 0
fi

# -----------------------------------------------------------------------------
# Execute Rollback
# -----------------------------------------------------------------------------

echo ""
echo "🔄 Rolling back to $REVISION..."

gcloud run services update-traffic "$SERVICE_NAME" \
  --region="$REGION" \
  --to-revisions="$REVISION=100"

echo "  ✅ Traffic updated"

# -----------------------------------------------------------------------------
# Verify Rollback
# -----------------------------------------------------------------------------

echo ""
echo "🔍 Verifying rollback..."

SERVICE_URL=$(gcloud run services describe "$SERVICE_NAME" \
  --region="$REGION" \
  --format='value(status.url)')

# Wait for deployment to stabilize
sleep 5

# Health check
if curl -sf "$SERVICE_URL/health" | jq -e '.status == "healthy"' > /dev/null; then
  echo "  ✅ Health check passed"
else
  echo "  ❌ Health check failed!"
  echo "  ⚠️  Consider rolling forward or checking logs"
  exit 1
fi

# -----------------------------------------------------------------------------
# Done
# -----------------------------------------------------------------------------

echo ""
echo "============================================================"
echo "✅ Rollback Complete!"
echo "============================================================"
echo "Active Revision: $REVISION"
echo "Service URL:     $SERVICE_URL"
echo "============================================================"
echo ""
echo "View logs:"
echo "  gcloud run services logs read $SERVICE_NAME --region=$REGION"
echo ""
