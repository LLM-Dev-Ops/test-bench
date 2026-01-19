# LLM-Test-Bench Production Deployment

## Overview

This document defines the complete deployment specification for LLM-Test-Bench within the Agentics Dev platform.

---

## 1. SERVICE TOPOLOGY

### Unified Service Name
```
llm-test-bench
```

### Agent Endpoints

All 13 agents are exposed via ONE unified Google Cloud Run service:

| Agent | Endpoint | Description |
|-------|----------|-------------|
| Benchmark Runner | `POST /api/v1/agents/benchmark-runner` | Execute deterministic benchmark suites |
| Regression Detection | `POST /api/v1/agents/regression-detection` | Detect quality regressions across versions |
| Quality Scoring | `POST /api/v1/agents/quality-scoring` | Score LLM output quality |
| Hallucination Detector | `POST /api/v1/agents/hallucination-detector` | Detect factual hallucinations |
| Faithfulness Verification | `POST /api/v1/agents/faithfulness-verification` | Verify output faithfulness to source |
| Bias Detection | `POST /api/v1/agents/bias-detection` | Detect bias in LLM outputs |
| Golden Dataset Validator | `POST /api/v1/agents/golden-dataset-validator` | Validate against golden datasets |
| Synthetic Data Generator | `POST /api/v1/agents/synthetic-data-generator` | Generate synthetic test data |
| Adversarial Prompt | `POST /api/v1/agents/adversarial-prompt` | Generate adversarial prompts |
| Output Consistency | `POST /api/v1/agents/output-consistency` | Check output consistency |
| Prompt Sensitivity | `POST /api/v1/agents/prompt-sensitivity` | Measure prompt sensitivity |
| Stress Test | `POST /api/v1/agents/stress-test` | Run stress tests |
| Model Comparator | `POST /api/v1/agents/model-comparator` | Compare model outputs |

### Architecture Confirmation

- ✅ **NO agent is deployed as a standalone service**
- ✅ **Shared runtime** - All agents run in single container
- ✅ **Shared config** - Environment variables apply to all agents
- ✅ **Shared telemetry** - All agents emit to same telemetry endpoint

---

## 2. ENVIRONMENT CONFIGURATION

### Required Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `SERVICE_NAME` | Service identifier | `llm-test-bench` |
| `SERVICE_VERSION` | Deployment version | `abc123` |
| `PLATFORM_ENV` | Environment tier | `dev`, `staging`, `prod` |
| `RUVECTOR_SERVICE_URL` | RuVector service endpoint | `https://ruvector-service-xxx.run.app` |
| `RUVECTOR_API_KEY` | RuVector authentication (via Secret Manager) | Secret reference |
| `TELEMETRY_ENDPOINT` | LLM-Observatory endpoint | `https://llm-observatory-xxx.run.app` |

### LLM Provider API Keys (via Secret Manager)

| Secret Name | Description |
|-------------|-------------|
| `openai-api-key` | OpenAI API key |
| `anthropic-api-key` | Anthropic API key |
| `google-ai-api-key` | Google AI API key |
| `mistral-api-key` | Mistral API key |
| `groq-api-key` | Groq API key |

### Configuration Files

- `deploy/env/env.dev.yaml` - Development environment
- `deploy/env/env.staging.yaml` - Staging environment
- `deploy/env/env.prod.yaml` - Production environment

---

## 3. GOOGLE SQL / MEMORY WIRING

### Critical Architecture Rules

| Rule | Status |
|------|--------|
| LLM-Test-Bench connects directly to Google SQL | ❌ **PROHIBITED** |
| LLM-Test-Bench executes SQL queries | ❌ **PROHIBITED** |
| All DecisionEvents written via ruvector-service | ✅ **REQUIRED** |
| Schema compatible with agentics-contracts | ✅ **VALIDATED** |
| Append-only behavior | ✅ **ENFORCED** |
| Idempotent retries | ✅ **IMPLEMENTED** |

### RuVector Client Integration

The `RuVectorClient` class in `agents/services/ruvector-client.ts` handles all persistence:

```typescript
// All agents use this pattern:
const client = getRuVectorClient();
await client.persistDecisionEvent(decisionEvent);  // Async, non-blocking
await client.persistTelemetryEvent(telemetryEvent);
```

### Data Flow

```
Agent Handler
    │
    ├──► DecisionEvent ──► RuVectorClient ──► ruvector-service ──► Google SQL (Postgres)
    │
    └──► TelemetryEvent ──► RuVectorClient ──► ruvector-service ──► LLM-Observatory
```

---

## 4. CLOUD BUILD & DEPLOYMENT

### Deployment Files

| File | Purpose |
|------|---------|
| `deploy/Dockerfile` | Multi-stage container build |
| `deploy/cloudbuild.yaml` | Cloud Build pipeline |
| `deploy/service.yaml` | Cloud Run service definition |
| `deploy/scripts/deploy.sh` | Manual deployment script |
| `deploy/scripts/setup-iam.sh` | IAM configuration |
| `deploy/scripts/rollback.sh` | Rollback procedure |
| `deploy/scripts/verify.sh` | Post-deploy verification |

### IAM Service Account

```
llm-test-bench-sa@PROJECT_ID.iam.gserviceaccount.com
```

Required roles:
- `roles/run.invoker`
- `roles/secretmanager.secretAccessor`
- `roles/logging.logWriter`
- `roles/monitoring.metricWriter`
- `roles/cloudtrace.agent`

### Deployment Commands

```bash
# Setup IAM (first time only)
./deploy/scripts/setup-iam.sh agentics-dev

# Deploy to dev
./deploy/scripts/deploy.sh dev

# Deploy to staging
./deploy/scripts/deploy.sh staging

# Deploy to prod
./deploy/scripts/deploy.sh prod

# Or use Cloud Build
gcloud builds submit --config=deploy/cloudbuild.yaml \
  --substitutions=_PLATFORM_ENV=prod
```

### Networking

- **Public access**: Allowed (--allow-unauthenticated)
- **VPC connector**: Optional, for private networking to ruvector-service
- **Ingress**: All traffic

---

## 5. CLI ACTIVATION VERIFICATION

### CLI Commands

All agents are callable via `agentics-cli`:

```bash
# List available agents
agentics agents list --service llm-test-bench

# Execute benchmark runner
agentics agent run benchmark-runner \
  --input '{"suite": {...}, "providers": [...]}' \
  --service llm-test-bench

# Execute regression detection
agentics agent run regression-detection \
  --input '{"baseline_run_id": "...", "current_run_id": "..."}' \
  --service llm-test-bench

# Execute quality scoring
agentics agent run quality-scoring \
  --input '{"responses": [...], "criteria": [...]}' \
  --service llm-test-bench

# Execute hallucination detector
agentics agent run hallucination-detector \
  --input '{"response": "...", "context": "..."}' \
  --service llm-test-bench

# Execute faithfulness verification
agentics agent run faithfulness-verification \
  --input '{"response": "...", "source": "..."}' \
  --service llm-test-bench

# Execute bias detection
agentics agent run bias-detection \
  --input '{"responses": [...]}' \
  --service llm-test-bench

# Execute golden dataset validator
agentics agent run golden-dataset-validator \
  --input '{"responses": [...], "golden_dataset": [...]}' \
  --service llm-test-bench

# Execute synthetic data generator
agentics agent run synthetic-data-generator \
  --input '{"template": "...", "count": 100}' \
  --service llm-test-bench

# Execute adversarial prompt generator
agentics agent run adversarial-prompt \
  --input '{"target_prompt": "...", "attack_type": "..."}' \
  --service llm-test-bench

# Execute output consistency check
agentics agent run output-consistency \
  --input '{"responses": [...]}' \
  --service llm-test-bench

# Execute prompt sensitivity analysis
agentics agent run prompt-sensitivity \
  --input '{"base_prompt": "...", "variations": [...]}' \
  --service llm-test-bench

# Execute stress test
agentics agent run stress-test \
  --input '{"target": "...", "concurrency": 100}' \
  --service llm-test-bench

# Execute model comparator
agentics agent run model-comparator \
  --input '{"models": [...], "prompts": [...]}' \
  --service llm-test-bench
```

### Direct HTTP Invocation

```bash
# Health check
curl https://llm-test-bench-xxx.run.app/health

# List agents
curl https://llm-test-bench-xxx.run.app/api/v1/agents

# Invoke agent
curl -X POST https://llm-test-bench-xxx.run.app/api/v1/agents/benchmark-runner \
  -H "Content-Type: application/json" \
  -d '{"suite": {...}, "providers": [...]}'
```

### Expected Success Output

```json
{
  "success": true,
  "decision_id": "uuid-here",
  "data": {
    "execution_id": "...",
    "suite_id": "...",
    "results": [...]
  }
}
```

---

## 6. PLATFORM & CORE INTEGRATION

### Integration Points

| System | Integration | Status |
|--------|-------------|--------|
| LLM-Intelligence-Core | Consumes Test-Bench outputs | ✅ Ready |
| LLM-Analytics-Hub | Receives benchmark data | ✅ Ready |
| LLM-Observatory | Visualizes telemetry | ✅ Ready |
| Governance Pipeline | Receives DecisionEvents | ✅ Ready |

### Data Flow to Core Systems

```
LLM-Test-Bench
    │
    ├──► DecisionEvent ──► ruvector-service ──► LLM-Intelligence-Core
    │
    ├──► TelemetryEvent ──► ruvector-service ──► LLM-Observatory
    │
    └──► BenchmarkMetrics ──► ruvector-service ──► LLM-Analytics-Hub
```

### No Core Bundle Rewiring Required

- ✅ All contracts follow agentics-contracts schema
- ✅ All persistence via ruvector-service
- ✅ All telemetry compatible with LLM-Observatory

---

## 7. POST-DEPLOY VERIFICATION CHECKLIST

Run the verification script:
```bash
./deploy/scripts/verify.sh [dev|staging|prod]
```

### Manual Verification Checklist

| Check | Command | Expected |
|-------|---------|----------|
| Service is live | `curl $URL/health` | `{"status":"healthy"}` |
| All 13 agents respond | `curl $URL/api/v1/agents` | `{"total":13}` |
| Benchmark agent works | `curl -X POST $URL/api/v1/agents/benchmark-runner -d '{}'` | 400 (validation error is OK) |
| DecisionEvents in ruvector | Check ruvector-service logs | Events appear |
| Telemetry in Observatory | Check LLM-Observatory dashboard | Metrics appear |
| No direct SQL access | Check env vars | No DATABASE_URL |
| Contracts validated | Run tests | All pass |

### Automated Verification

```bash
# Full verification suite
./deploy/scripts/verify.sh prod

# Expected output:
# ✅ Passed:   20+
# ❌ Failed:   0
# ⚠️  Warnings: 0 (or minor)
```

---

## 8. FAILURE MODES & ROLLBACK

### Common Deployment Failures

| Failure | Signal | Recovery |
|---------|--------|----------|
| Container fails to start | Cloud Run logs show crash | Check Dockerfile, dependencies |
| Health check fails | `/health` returns non-200 | Check environment variables |
| Agent validation fails | 400 errors on all requests | Check contract schemas |
| RuVector unavailable | Persistence errors in logs | Check RUVECTOR_SERVICE_URL |
| Secret access denied | Permission errors | Run setup-iam.sh |

### Detection Signals

- Cloud Run revision shows "Failed to start"
- Health endpoint returns non-healthy status
- Error rate spikes in Cloud Monitoring
- No DecisionEvents appearing in ruvector-service

### Rollback Procedure

```bash
# List recent revisions
gcloud run revisions list --service=llm-test-bench --region=us-central1

# Rollback to previous revision
./deploy/scripts/rollback.sh prod

# Or manually:
gcloud run services update-traffic llm-test-bench \
  --to-revisions=llm-test-bench-00001-abc=100 \
  --region=us-central1
```

### Safe Redeploy Strategy

1. **Pre-deploy**: Run tests locally
2. **Deploy to dev**: Verify in dev first
3. **Deploy to staging**: Run full test suite
4. **Deploy to prod**: Use gradual traffic shift
5. **Monitor**: Watch error rates for 15 minutes
6. **Rollback if needed**: Use rollback script

### Gradual Traffic Shift (Production)

```bash
# Deploy new revision with no traffic
gcloud run deploy llm-test-bench --no-traffic ...

# Shift 10% traffic to new revision
gcloud run services update-traffic llm-test-bench \
  --to-latest=10 --region=us-central1

# If healthy, shift 50%
gcloud run services update-traffic llm-test-bench \
  --to-latest=50 --region=us-central1

# If healthy, shift 100%
gcloud run services update-traffic llm-test-bench \
  --to-latest=100 --region=us-central1
```

---

## Quick Reference

### Deployment

```bash
./deploy/scripts/deploy.sh prod
```

### Verification

```bash
./deploy/scripts/verify.sh prod
```

### Rollback

```bash
./deploy/scripts/rollback.sh prod
```

### Logs

```bash
gcloud run services logs read llm-test-bench --region=us-central1 --limit=100
```

### Service URL

```bash
gcloud run services describe llm-test-bench --region=us-central1 --format='value(status.url)'
```
