#!/bin/bash
# =============================================================================
# LLM-Test-Bench - Cloud Run Deployment Script
#
# PHASE 1 - FOUNDATIONAL TOOLING (Layer 1)
#
# This script deploys to Cloud Run with:
# - Google Secret Manager for secrets (RUVECTOR_API_KEY, API keys)
# - Mandatory environment variables for agent identity
# - Health check validation before traffic routing
# =============================================================================

set -euo pipefail

# =============================================================================
# CONFIGURATION
# =============================================================================

PROJECT_ID="${PROJECT_ID:-your-gcp-project-id}"
REGION="${REGION:-us-central1}"
SERVICE_NAME="llm-test-bench"
IMAGE_TAG="${IMAGE_TAG:-latest}"
IMAGE_URL="gcr.io/${PROJECT_ID}/${SERVICE_NAME}:${IMAGE_TAG}"

# Environment (dev, staging, prod)
ENVIRONMENT="${ENVIRONMENT:-prod}"

# RuVector Service URL (must be set)
RUVECTOR_SERVICE_URL="${RUVECTOR_SERVICE_URL:-}"

if [ -z "$RUVECTOR_SERVICE_URL" ]; then
  echo "ERROR: RUVECTOR_SERVICE_URL must be set"
  echo "Example: export RUVECTOR_SERVICE_URL=https://ruvector-service-prod-xxxx-uc.a.run.app"
  exit 1
fi

# =============================================================================
# DISPLAY CONFIGURATION
# =============================================================================

echo "============================================"
echo "LLM-Test-Bench Cloud Run Deployment"
echo "PHASE 1 - FOUNDATIONAL TOOLING (Layer 1)"
echo "============================================"
echo "Project:     ${PROJECT_ID}"
echo "Region:      ${REGION}"
echo "Service:     ${SERVICE_NAME}"
echo "Image:       ${IMAGE_URL}"
echo "Environment: ${ENVIRONMENT}"
echo "RuVector:    ${RUVECTOR_SERVICE_URL}"
echo "============================================"

# =============================================================================
# CLOUD RUN DEPLOY COMMAND
# Uses --set-secrets for Google Secret Manager integration
# =============================================================================

echo "Deploying to Cloud Run..."

gcloud run deploy "${SERVICE_NAME}" \
  --project="${PROJECT_ID}" \
  --region="${REGION}" \
  --image="${IMAGE_URL}" \
  --platform=managed \
  --allow-unauthenticated \
  --memory=1Gi \
  --cpu=2 \
  --min-instances=0 \
  --max-instances=100 \
  --timeout=300 \
  --concurrency=80 \
  --cpu-throttling \
  --execution-environment=gen2 \
  --service-account="llm-test-bench-sa@${PROJECT_ID}.iam.gserviceaccount.com" \
  \
  --set-env-vars="SERVICE_NAME=${SERVICE_NAME}" \
  --set-env-vars="PLATFORM_ENV=${ENVIRONMENT}" \
  --set-env-vars="RUVECTOR_SERVICE_URL=${RUVECTOR_SERVICE_URL}" \
  \
  --set-env-vars="AGENT_NAME=llm-test-bench-service" \
  --set-env-vars="AGENT_DOMAIN=evaluation" \
  --set-env-vars="AGENT_PHASE=phase1" \
  --set-env-vars="AGENT_LAYER=layer1" \
  \
  --set-secrets="RUVECTOR_API_KEY=ruvector-api-key:latest" \
  --set-secrets="OPENAI_API_KEY=openai-api-key:latest" \
  --set-secrets="ANTHROPIC_API_KEY=anthropic-api-key:latest" \
  --set-secrets="GOOGLE_AI_API_KEY=google-ai-api-key:latest" \
  --set-secrets="MISTRAL_API_KEY=mistral-api-key:latest" \
  --set-secrets="GROQ_API_KEY=groq-api-key:latest"

echo "============================================"
echo "Deployment complete!"
echo "============================================"

# =============================================================================
# VERIFY DEPLOYMENT
# =============================================================================

echo "Verifying deployment..."

SERVICE_URL=$(gcloud run services describe "${SERVICE_NAME}" \
  --project="${PROJECT_ID}" \
  --region="${REGION}" \
  --format="value(status.url)")

echo "Service URL: ${SERVICE_URL}"

# Health check
echo "Running health check..."
HEALTH_RESPONSE=$(curl -s "${SERVICE_URL}/health")
echo "Health response: ${HEALTH_RESPONSE}"

# Check if healthy
if echo "${HEALTH_RESPONSE}" | grep -q '"status":"healthy"'; then
  echo "✅ Service is healthy"
else
  echo "❌ Service health check failed!"
  exit 1
fi

# Readiness check
echo "Running readiness check..."
READY_RESPONSE=$(curl -s "${SERVICE_URL}/ready")
echo "Ready response: ${READY_RESPONSE}"

if echo "${READY_RESPONSE}" | grep -q '"ready":true'; then
  echo "✅ Service is ready"
else
  echo "❌ Service readiness check failed!"
  exit 1
fi

echo "============================================"
echo "Deployment verification complete!"
echo "Service URL: ${SERVICE_URL}"
echo "============================================"
