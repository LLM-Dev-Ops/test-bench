#!/bin/bash
# =============================================================================
# LLM-Test-Bench - Deployment Script
# =============================================================================
# Usage: ./deploy.sh [dev|staging|prod]
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DEPLOY_DIR="$PROJECT_ROOT/deploy"

ENV="${1:-dev}"
SERVICE_NAME="llm-test-bench"

# Environment-specific settings
case "$ENV" in
  dev)
    PROJECT_ID="agentics-dev"
    REGION="us-central1"
    MIN_INSTANCES="0"
    MAX_INSTANCES="5"
    MEMORY="512Mi"
    CPU="1"
    ;;
  staging)
    PROJECT_ID="agentics-staging"
    REGION="us-central1"
    MIN_INSTANCES="1"
    MAX_INSTANCES="10"
    MEMORY="512Mi"
    CPU="1"
    ;;
  prod)
    PROJECT_ID="agentics-prod"
    REGION="us-central1"
    MIN_INSTANCES="2"
    MAX_INSTANCES="100"
    MEMORY="1Gi"
    CPU="2"
    ;;
  *)
    echo "❌ Invalid environment: $ENV"
    echo "Usage: $0 [dev|staging|prod]"
    exit 1
    ;;
esac

IMAGE="gcr.io/$PROJECT_ID/$SERVICE_NAME"
VERSION=$(git rev-parse --short HEAD 2>/dev/null || echo "local")

echo "============================================================"
echo "LLM-Test-Bench Deployment"
echo "============================================================"
echo "Environment:  $ENV"
echo "Project:      $PROJECT_ID"
echo "Region:       $REGION"
echo "Service:      $SERVICE_NAME"
echo "Version:      $VERSION"
echo "============================================================"

# -----------------------------------------------------------------------------
# Pre-flight Checks
# -----------------------------------------------------------------------------

echo ""
echo "📋 Running pre-flight checks..."

# Check gcloud auth
if ! gcloud auth print-identity-token &>/dev/null; then
  echo "❌ Not authenticated with gcloud. Run: gcloud auth login"
  exit 1
fi
echo "  ✅ gcloud authenticated"

# Set project
gcloud config set project "$PROJECT_ID" --quiet
echo "  ✅ Project set to $PROJECT_ID"

# Check required APIs
REQUIRED_APIS=(
  "run.googleapis.com"
  "cloudbuild.googleapis.com"
  "secretmanager.googleapis.com"
  "containerregistry.googleapis.com"
)

for api in "${REQUIRED_APIS[@]}"; do
  if ! gcloud services list --enabled --filter="name:$api" --format="value(name)" | grep -q "$api"; then
    echo "  ⚠️  Enabling $api..."
    gcloud services enable "$api" --quiet
  fi
done
echo "  ✅ Required APIs enabled"

# -----------------------------------------------------------------------------
# Build
# -----------------------------------------------------------------------------

echo ""
echo "🔨 Building container image..."

cd "$PROJECT_ROOT"

docker build \
  -t "$IMAGE:$VERSION" \
  -t "$IMAGE:latest" \
  -t "$IMAGE:$ENV" \
  -f deploy/Dockerfile \
  .

echo "  ✅ Image built: $IMAGE:$VERSION"

# -----------------------------------------------------------------------------
# Push
# -----------------------------------------------------------------------------

echo ""
echo "📤 Pushing image to GCR..."

docker push "$IMAGE:$VERSION"
docker push "$IMAGE:latest"
docker push "$IMAGE:$ENV"

echo "  ✅ Image pushed"

# -----------------------------------------------------------------------------
# Deploy
# -----------------------------------------------------------------------------

echo ""
echo "🚀 Deploying to Cloud Run..."

gcloud run deploy "$SERVICE_NAME" \
  --image="$IMAGE:$VERSION" \
  --region="$REGION" \
  --platform=managed \
  --allow-unauthenticated \
  --min-instances="$MIN_INSTANCES" \
  --max-instances="$MAX_INSTANCES" \
  --memory="$MEMORY" \
  --cpu="$CPU" \
  --timeout=300 \
  --concurrency=80 \
  --set-env-vars="SERVICE_NAME=$SERVICE_NAME" \
  --set-env-vars="SERVICE_VERSION=$VERSION" \
  --set-env-vars="PLATFORM_ENV=$ENV" \
  --set-secrets="RUVECTOR_API_KEY=ruvector-api-key:latest" \
  --set-secrets="OPENAI_API_KEY=openai-api-key:latest" \
  --set-secrets="ANTHROPIC_API_KEY=anthropic-api-key:latest" \
  --service-account="$SERVICE_NAME-sa@$PROJECT_ID.iam.gserviceaccount.com" \
  --labels="service=$SERVICE_NAME,env=$ENV,version=$VERSION" \
  --quiet

echo "  ✅ Deployed to Cloud Run"

# -----------------------------------------------------------------------------
# Verify
# -----------------------------------------------------------------------------

echo ""
echo "🔍 Verifying deployment..."

SERVICE_URL=$(gcloud run services describe "$SERVICE_NAME" \
  --region="$REGION" \
  --format='value(status.url)')

echo "  Service URL: $SERVICE_URL"

# Health check
echo "  Checking /health..."
if curl -sf "$SERVICE_URL/health" | jq -e '.status == "healthy"' > /dev/null; then
  echo "  ✅ Health check passed"
else
  echo "  ❌ Health check failed"
  exit 1
fi

# Readiness check
echo "  Checking /ready..."
if curl -sf "$SERVICE_URL/ready" | jq -e '.ready == true' > /dev/null; then
  echo "  ✅ Readiness check passed"
else
  echo "  ❌ Readiness check failed"
  exit 1
fi

# Agent list
echo "  Checking /api/v1/agents..."
AGENT_COUNT=$(curl -sf "$SERVICE_URL/api/v1/agents" | jq '.total')
echo "  ✅ Found $AGENT_COUNT agents"

# -----------------------------------------------------------------------------
# Done
# -----------------------------------------------------------------------------

echo ""
echo "============================================================"
echo "✅ Deployment Complete!"
echo "============================================================"
echo "Service URL: $SERVICE_URL"
echo "Agents:      $AGENT_COUNT"
echo "Version:     $VERSION"
echo "Environment: $ENV"
echo "============================================================"
echo ""
echo "Quick test:"
echo "  curl $SERVICE_URL/health"
echo "  curl $SERVICE_URL/api/v1/agents"
echo ""
