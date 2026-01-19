#!/bin/bash
# =============================================================================
# LLM-Test-Bench - Post-Deployment Verification Script
# =============================================================================
# Runs comprehensive verification checks after deployment
# Usage: ./verify.sh [dev|staging|prod]
# =============================================================================

set -euo pipefail

ENV="${1:-dev}"
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
    exit 1
    ;;
esac

echo "============================================================"
echo "LLM-Test-Bench Post-Deployment Verification"
echo "============================================================"
echo "Environment: $ENV"
echo "Project:     $PROJECT_ID"
echo "============================================================"

gcloud config set project "$PROJECT_ID" --quiet

SERVICE_URL=$(gcloud run services describe "$SERVICE_NAME" \
  --region="$REGION" \
  --format='value(status.url)')

echo "Service URL: $SERVICE_URL"
echo ""

PASSED=0
FAILED=0
WARNINGS=0

check() {
  local name="$1"
  local result="$2"
  local expected="$3"

  if [ "$result" = "$expected" ]; then
    echo "  ✅ $name"
    ((PASSED++))
  else
    echo "  ❌ $name (expected: $expected, got: $result)"
    ((FAILED++))
  fi
}

warn() {
  local name="$1"
  local message="$2"
  echo "  ⚠️  $name: $message"
  ((WARNINGS++))
}

# -----------------------------------------------------------------------------
# 1. Service Health
# -----------------------------------------------------------------------------

echo ""
echo "📋 1. Service Health"
echo "------------------------------------------------------------"

HEALTH=$(curl -sf "$SERVICE_URL/health" || echo '{"status":"error"}')
HEALTH_STATUS=$(echo "$HEALTH" | jq -r '.status')
check "Health endpoint responds" "$HEALTH_STATUS" "healthy"

READY=$(curl -sf "$SERVICE_URL/ready" || echo '{"ready":false}')
READY_STATUS=$(echo "$READY" | jq -r '.ready')
check "Readiness endpoint responds" "$READY_STATUS" "true"

# -----------------------------------------------------------------------------
# 2. Agent Endpoints
# -----------------------------------------------------------------------------

echo ""
echo "📋 2. Agent Endpoints"
echo "------------------------------------------------------------"

AGENTS=$(curl -sf "$SERVICE_URL/api/v1/agents" || echo '{"total":0}')
AGENT_COUNT=$(echo "$AGENTS" | jq '.total')

EXPECTED_AGENTS=13
if [ "$AGENT_COUNT" -eq "$EXPECTED_AGENTS" ]; then
  echo "  ✅ All $EXPECTED_AGENTS agents registered"
  ((PASSED++))
else
  echo "  ❌ Expected $EXPECTED_AGENTS agents, found $AGENT_COUNT"
  ((FAILED++))
fi

# Check each agent endpoint responds (405 is expected for GET)
AGENT_ENDPOINTS=(
  "/api/v1/agents/benchmark-runner"
  "/api/v1/agents/regression-detection"
  "/api/v1/agents/quality-scoring"
  "/api/v1/agents/hallucination-detector"
  "/api/v1/agents/faithfulness-verification"
  "/api/v1/agents/bias-detection"
  "/api/v1/agents/golden-dataset-validator"
  "/api/v1/agents/synthetic-data-generator"
  "/api/v1/agents/adversarial-prompt"
  "/api/v1/agents/output-consistency"
  "/api/v1/agents/prompt-sensitivity"
  "/api/v1/agents/stress-test"
  "/api/v1/agents/model-comparator"
)

for endpoint in "${AGENT_ENDPOINTS[@]}"; do
  HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$SERVICE_URL$endpoint")
  if [ "$HTTP_CODE" = "405" ]; then
    echo "  ✅ $endpoint (responds with 405 for GET)"
    ((PASSED++))
  else
    echo "  ❌ $endpoint (unexpected status: $HTTP_CODE)"
    ((FAILED++))
  fi
done

# -----------------------------------------------------------------------------
# 3. No Direct SQL Access
# -----------------------------------------------------------------------------

echo ""
echo "📋 3. Architecture Compliance"
echo "------------------------------------------------------------"

# Check that RUVECTOR_SERVICE_URL is set
ENV_CHECK=$(gcloud run services describe "$SERVICE_NAME" \
  --region="$REGION" \
  --format='value(spec.template.spec.containers[0].env)' 2>/dev/null || echo "")

if echo "$ENV_CHECK" | grep -q "RUVECTOR"; then
  echo "  ✅ RUVECTOR_SERVICE_URL configured"
  ((PASSED++))
else
  warn "RUVECTOR_SERVICE_URL" "Not found in environment"
fi

# Check no DATABASE_URL (should not exist)
if echo "$ENV_CHECK" | grep -q "DATABASE_URL"; then
  echo "  ❌ DATABASE_URL should not be set (direct SQL access prohibited)"
  ((FAILED++))
else
  echo "  ✅ No direct database connection configured"
  ((PASSED++))
fi

# -----------------------------------------------------------------------------
# 4. Cloud Run Configuration
# -----------------------------------------------------------------------------

echo ""
echo "📋 4. Cloud Run Configuration"
echo "------------------------------------------------------------"

REVISION_INFO=$(gcloud run revisions list \
  --service="$SERVICE_NAME" \
  --region="$REGION" \
  --limit=1 \
  --format='json' 2>/dev/null | jq '.[0]')

MEMORY=$(echo "$REVISION_INFO" | jq -r '.spec.containers[0].resources.limits.memory // "unknown"')
CPU=$(echo "$REVISION_INFO" | jq -r '.spec.containers[0].resources.limits.cpu // "unknown"')
CONCURRENCY=$(echo "$REVISION_INFO" | jq -r '.spec.containerConcurrency // 0')

echo "  Memory:      $MEMORY"
echo "  CPU:         $CPU"
echo "  Concurrency: $CONCURRENCY"

if [ "$CONCURRENCY" -gt 0 ]; then
  echo "  ✅ Concurrency configured"
  ((PASSED++))
else
  warn "Concurrency" "Not explicitly set"
fi

# -----------------------------------------------------------------------------
# 5. Response Headers
# -----------------------------------------------------------------------------

echo ""
echo "📋 5. Response Headers"
echo "------------------------------------------------------------"

HEADERS=$(curl -sI "$SERVICE_URL/health")

if echo "$HEADERS" | grep -qi "X-Service-Name"; then
  echo "  ✅ X-Service-Name header present"
  ((PASSED++))
else
  warn "X-Service-Name" "Header not found"
fi

if echo "$HEADERS" | grep -qi "X-Request-Id"; then
  echo "  ✅ X-Request-Id header present"
  ((PASSED++))
else
  warn "X-Request-Id" "Header not found"
fi

# -----------------------------------------------------------------------------
# Summary
# -----------------------------------------------------------------------------

echo ""
echo "============================================================"
echo "Verification Summary"
echo "============================================================"
echo "  ✅ Passed:   $PASSED"
echo "  ❌ Failed:   $FAILED"
echo "  ⚠️  Warnings: $WARNINGS"
echo "============================================================"

if [ "$FAILED" -gt 0 ]; then
  echo ""
  echo "❌ VERIFICATION FAILED"
  echo "Review failed checks above and fix before proceeding."
  exit 1
else
  echo ""
  echo "✅ VERIFICATION PASSED"
  if [ "$WARNINGS" -gt 0 ]; then
    echo "⚠️  Review warnings above for potential improvements."
  fi
fi
