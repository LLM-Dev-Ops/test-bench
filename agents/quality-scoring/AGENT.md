# Quality Scoring Agent

## Purpose Statement

The Quality Scoring Agent computes normalized quality scores for model outputs using deterministic scoring profiles. It evaluates model-generated content across multiple configurable dimensions (accuracy, format compliance, keyword presence, etc.) and produces consistent, reproducible quality assessments. This agent is designed for automated quality gates, regression testing, and comparative analysis of LLM outputs.

## Responsibilities

| Action | Responsibility |
|--------|---------------|
| Evaluate output quality | ✅ YES |
| Apply deterministic scoring profiles | ✅ YES |
| Normalize scores to 0-1 range | ✅ YES |
| Aggregate multiple quality dimensions | ✅ YES |
| Calculate confidence scores | ✅ YES |
| Execute benchmarks | ❌ NO |
| Compare/rank models | ❌ NO |
| Enforce policies | ❌ NO |
| Orchestrate workflows | ❌ NO |
| Call other agents | ❌ NO |
| Store API keys | ❌ NO |

## Contract Summary

### Input Schema

```typescript
QualityScoringInputSchema = {
  outputs: ModelOutput[],           // Model outputs to score (1-1000)
  scoring_profile: ScoringProfile,  // Scoring dimensions and weights
  evaluation_config?: EvaluationConfig,
  caller_id?: string,
  correlation_id?: string (UUID)
}
```

### Output Schema

```typescript
QualityScoringOutputSchema = {
  scoring_id: string (UUID),
  profile_id: string,
  profile_name: string,
  scores: OutputScore[],            // Per-output scores
  model_stats: ModelQualityStats[], // Per-model aggregates
  summary: ScoringSummary,          // Overall summary
  evaluation_config_used: EvaluationConfig,
  started_at: string (datetime),
  completed_at: string (datetime),
  duration_ms: number
}
```

### DecisionEvent Mapping

| Field | Value |
|-------|-------|
| agent_id | `quality-scoring` |
| agent_version | `1.0.0` |
| decision_type | `quality_scoring` |
| inputs_hash | SHA-256 of input |
| inputs_summary | `{ profile_id, output_count, dimension_count }` |
| outputs | Full QualityScoringOutput |
| confidence | 0-1 based on scoring factors |
| constraints_applied | Any constraints triggered |

## CLI Contract

```bash
# Basic usage
agentics quality-scoring --input-file outputs.json

# With separate profile file
agentics quality-scoring --input-file outputs.json --profile-file profile.json

# From stdin
cat outputs.json | agentics quality-scoring --input-stdin

# Different output formats
agentics quality-scoring --input-file outputs.json --output-format table
agentics quality-scoring --input-file outputs.json --output-format csv

# Dry run (validate only)
agentics quality-scoring --dry-run --input-file outputs.json

# Write to file
agentics quality-scoring --input-file outputs.json --output-file results.json
```

### CLI Options

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--input-file` | `-i` | string | - | Path to input JSON file |
| `--input-json` | `-j` | string | - | Input as JSON string |
| `--input-stdin` | `-s` | boolean | false | Read input from stdin |
| `--profile-file` | `-p` | string | - | Separate scoring profile file |
| `--output-format` | `-f` | string | json | Output format (json/csv/table) |
| `--output-file` | `-o` | string | - | Write output to file |
| `--verbose` | `-v` | boolean | false | Verbose output |
| `--quiet` | `-q` | boolean | false | Minimal output |
| `--dry-run` | `-d` | boolean | false | Validate without executing |

## Explicit Non-Responsibilities

1. **execute_benchmarks** - Does not run prompts against LLM providers
2. **compare_models** - Does not rank or compare models (use model-comparator)
3. **enforce_policy** - Does not make policy decisions
4. **orchestrate_workflows** - Does not coordinate multi-step workflows
5. **call_other_agents** - Does not invoke other agents directly
6. **store_api_keys** - Never persists API credentials
7. **modify_outputs** - Does not mutate input data
8. **generate_content** - Does not create new content
9. **make_recommendations** - Only scores, does not recommend
10. **cache_scoring_results** - Stateless, no internal caching

## Scoring Methods

| Method | Description | Use Case |
|--------|-------------|----------|
| `exact_match` | 1.0 if exact match, 0.0 otherwise | Factual answers |
| `contains` | 1.0 if contains expected substring | Flexible matching |
| `regex_match` | 1.0 if matches regex pattern | Pattern validation |
| `semantic_similarity` | Cosine similarity (requires embeddings) | Semantic matching |
| `length_ratio` | Ratio of actual/expected length | Conciseness check |
| `keyword_presence` | Proportion of keywords found | Coverage check |
| `format_compliance` | Valid JSON/XML/YAML/etc. | Format validation |
| `custom_evaluator` | External function reference | Custom logic |

## Failure Modes

| Error Code | Condition | Recoverable |
|------------|-----------|-------------|
| `VALIDATION_ERROR` | Invalid input schema | ✅ Yes |
| `EXECUTION_ERROR` | Scoring calculation failed | ❌ No |
| `TIMEOUT_ERROR` | Processing exceeded timeout | ✅ Yes (retry) |
| `CONFIGURATION_ERROR` | Invalid profile configuration | ✅ Yes |

## Constraints Applied

| Constraint | Condition |
|------------|-----------|
| `max_outputs_exceeded` | Batch exceeds 1000 outputs |
| `semantic_similarity_unavailable` | Embeddings service unavailable |
| `custom_evaluator_failed` | External evaluator error |
| `dimension_weight_adjusted` | Weights renormalized |
| `threshold_breach_detected` | Output failed threshold (fail-fast) |
| `parallel_evaluation_disabled` | Fallback to sequential |
| `normalization_edge_case` | All scores identical/extreme |

## Allowed Consumers

- `llm-orchestrator` - Workflow quality gates
- `llm-observatory` - Quality monitoring
- `llm-analytics` - Quality trend analysis
- `llm-test-bench-ui` - Quality dashboards
- `model-comparator` - Quality-weighted comparisons
- `regression-detector` - Quality regression detection

## Confidence Scoring

| Factor | Weight | Description |
|--------|--------|-------------|
| sample_size | 0.25 | Number of outputs scored |
| dimension_coverage | 0.20 | Proportion of dimensions evaluated |
| score_consistency | 0.25 | Low variance = high confidence |
| profile_maturity | 0.15 | Profile version maturity |
| method_reliability | 0.15 | Reliability of scoring methods |

## Versioning Rules

| Change Type | Version Bump | Examples |
|-------------|--------------|----------|
| Major | Breaking changes | Schema changes, algorithm changes |
| Minor | Backward compatible | New scoring methods, new config options |
| Patch | Bug fixes | Performance improvements, documentation |

## Preset Profiles

### accuracy-basic
Simple accuracy scoring with exact match and keyword presence.

### comprehensive
Multi-dimensional assessment with accuracy, format, keywords, and length.

## Smoke Test Commands

```bash
# Help
npx ts-node agents/quality-scoring/cli.ts --help

# Dry run with sample input
npx ts-node agents/quality-scoring/cli.ts --dry-run \
  --input-file agents/quality-scoring/examples/sample-input.json

# Full execution with table output
npx ts-node agents/quality-scoring/cli.ts \
  --input-file agents/quality-scoring/examples/sample-input.json \
  --output-format table

# Run tests
npm run test -- agents/quality-scoring/tests/smoke.test.ts
```

## Deployment Information

| Property | Value |
|----------|-------|
| Platform | Google Cloud Edge Function |
| Service | llm-test-bench |
| Endpoint | `/api/v1/agents/quality-scoring` |
| Timeout | 120 seconds |
| Memory | 256 MB |
| Concurrency | 200 requests |

## Data Persistence

### Persisted to ruvector-service:
- DecisionEvent (one per invocation)
- TelemetryEvents (for observability)

### NOT Persisted:
- Raw model outputs
- Scoring profile content
- API keys (only references)
- Intermediate calculation results

## Example Usage

```typescript
import { handler } from './agents/quality-scoring';

const response = await handler({
  body: {
    outputs: [
      {
        output_id: 'uuid-here',
        provider_name: 'openai',
        model_id: 'gpt-4o-mini',
        content: 'The capital of France is Paris.',
        expected_output: 'Paris',
      }
    ],
    scoring_profile: {
      profile_id: 'qa-basic',
      name: 'Basic QA',
      dimensions: [
        {
          dimension_id: 'accuracy',
          name: 'Accuracy',
          weight: 1.0,
          scoring_method: 'contains',
          pass_threshold: 0.8,
        }
      ],
      normalization: 'weighted_sum',
      version: '1.0.0',
    }
  },
  headers: {},
  method: 'POST',
  path: '/quality-scoring',
});

const result = JSON.parse(response.body);
console.log(`Score: ${result.data.scores[0].composite_score}`);
console.log(`Decision ID: ${result.decision_id}`);
```
