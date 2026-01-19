# Output Consistency Agent

## Purpose Statement

The Output Consistency Agent measures consistency across repeated executions of identical prompts. It produces deterministic consistency metrics by analyzing output variations when the same prompt is executed multiple times against the same model. This agent is essential for understanding model reliability, identifying non-deterministic behavior, and quantifying output stability for production readiness assessments.

## Responsibility Matrix

| Capability | Responsible |
|-----------|-------------|
| Measure output consistency | YES |
| Calculate similarity/variance metrics | YES |
| Detect semantic drift across executions | YES |
| Aggregate consistency scores by model/prompt | YES |
| Identify representative and divergent outputs | YES |
| Compute token-level analysis | YES |
| Compute pairwise similarity matrices | YES |
| Execute prompts | NO |
| Compare different models | NO |
| Enforce policies | NO |
| Orchestrate workflows | NO |
| Call other agents directly | NO |
| Store API keys | NO |
| Modify input outputs | NO |
| Generate content | NO |
| Make recommendations | NO |
| Cache analysis results | NO |

## Contract Summary

### Input Schema

```typescript
{
  execution_groups: [{
    group_id: string,           // Unique group identifier
    prompt: string,             // The prompt that was executed
    provider_name: string,      // Provider name
    model_id: string,           // Model identifier
    outputs: [{
      output_id: UUID,          // Unique output identifier
      content: string,          // The actual output content
      execution_number: number, // Execution sequence number
      executed_at: datetime,    // Timestamp of execution
      latency_ms?: number,      // Execution latency
      temperature?: number,     // Temperature used
      token_count?: number,     // Token count
    }],                         // Minimum 2 outputs required
    expected_output?: string,   // Reference output (optional)
  }],
  config?: {
    similarity_method: enum,    // Similarity algorithm to use
    consistency_threshold: 0-1, // Minimum score for "consistent"
    normalize_whitespace: bool, // Normalize whitespace
    case_sensitive: bool,       // Case sensitivity
    include_token_analysis: bool,
    include_char_variance: bool,
    compute_pairwise_matrix: bool,
  },
  caller_id?: string,
  correlation_id?: UUID,
}
```

### Output Schema

```typescript
{
  analysis_id: UUID,
  results: [{
    group_id: string,
    provider_name: string,
    model_id: string,
    output_count: number,
    consistency_score: 0-1,     // Overall consistency
    is_consistent: boolean,     // Meets threshold
    similarity_scores: {
      primary_score: 0-1,
      primary_method: string,
      additional_scores?: Record<string, number>,
    },
    token_analysis?: {...},     // Token-level metrics
    char_variance?: {...},      // Character variance
    pairwise_similarities?: number[], // Upper triangle
    representative_output_index: number,
    most_divergent_output_index: number,
    max_divergence_score: 0-1,
  }],
  model_stats: [{
    provider_name: string,
    model_id: string,
    groups_analyzed: number,
    avg_consistency_score: 0-1,
    min_consistency_score: 0-1,
    max_consistency_score: 0-1,
    stddev_consistency_score: number,
    consistency_rate: 0-1,
  }],
  summary: {
    total_groups_analyzed: number,
    total_outputs_analyzed: number,
    overall_avg_consistency: 0-1,
    overall_consistency_rate: 0-1,
    most_consistent_model: {...},
    least_consistent_model: {...},
    consistency_distribution: {...},
  },
  config_used: {...},
  started_at: datetime,
  completed_at: datetime,
  duration_ms: number,
}
```

### DecisionEvent Mapping

| Field | Value |
|-------|-------|
| agent_id | `output-consistency` |
| agent_version | `1.0.0` |
| decision_type | `output_consistency_analysis` |
| inputs_hash | SHA-256 of input data |
| inputs_summary | `{ groups_count, total_outputs, similarity_method }` |
| outputs | Full OutputConsistencyOutput |
| confidence | Calculated from confidence factors |
| constraints_applied | Any constraints triggered |

## Similarity Methods

| Method | Description | Best For |
|--------|-------------|----------|
| `exact_match` | Binary: 1.0 if identical, 0.0 otherwise | Deterministic outputs |
| `normalized_levenshtein` | Edit distance normalized to 0-1 | Minor variations |
| `jaccard_tokens` | Token overlap (default) | General purpose |
| `cosine_tfidf` | TF-IDF vector similarity | Long-form content |
| `character_ngram` | Character n-gram overlap | Short outputs |
| `word_ngram` | Word n-gram overlap | Phrase consistency |
| `semantic_embedding` | Embedding cosine similarity | Semantic equivalence |

## CLI Contract

```bash
# Basic usage
agentics output-consistency --input-file executions.json

# With specific similarity method
agentics output-consistency -i executions.json -m normalized_levenshtein

# With custom threshold
agentics output-consistency -i executions.json -t 0.9

# Table output
agentics output-consistency -i executions.json -f table

# Dry run (validation only)
agentics output-consistency -i executions.json --dry-run

# From stdin
cat executions.json | agentics output-consistency -s
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Validation error |
| 2 | Execution error |
| 3 | Configuration error |
| 4 | Timeout error |

## Explicit Non-Responsibilities

This agent MUST NOT:

1. **Execute prompts** - Input must be pre-collected execution outputs
2. **Compare different models** - That's the model-comparator agent
3. **Enforce policies** - No policy decisions or thresholds enforced
4. **Orchestrate workflows** - Single-purpose analysis only
5. **Call other agents** - No agent-to-agent communication
6. **Store API keys** - Never persist sensitive credentials
7. **Modify outputs** - Input data is immutable
8. **Generate content** - Analysis only, no generation
9. **Make recommendations** - Reports metrics, doesn't prescribe actions
10. **Cache results** - Stateless execution

## Failure Modes

| Error Code | Condition | Recoverable |
|------------|-----------|-------------|
| VALIDATION_ERROR | Invalid input schema | Yes |
| EXECUTION_ERROR | Unexpected processing failure | No |
| TIMEOUT_ERROR | Analysis exceeded timeout | Yes (retry) |
| CONFIGURATION_ERROR | Invalid config options | Yes |
| PERSISTENCE_ERROR | Failed to persist DecisionEvent | Yes |

## Constraints

| Constraint | Trigger | Action |
|------------|---------|--------|
| max_groups_exceeded | >500 groups | Continue with warning |
| semantic_embedding_unavailable | Embedding service down | Fallback to Jaccard |
| pairwise_matrix_too_large | >50 outputs in group | Skip pairwise computation |
| outputs_too_short | Avg length <10 chars | Continue with warning |
| identical_outputs_detected | All outputs same | Continue with score 1.0 |
| encoding_normalization_applied | Unicode issues | Normalize and continue |
| truncation_applied | Very long outputs | Truncate and continue |

## Dependencies

| Dependency | Type | Purpose |
|------------|------|---------|
| ruvector-service | Required | DecisionEvent persistence |
| llm-observatory | Optional | Telemetry collection |
| embedding-service | Optional | Semantic similarity |

## Allowed Consumers

- `llm-orchestrator` - Consistency-aware routing
- `llm-observatory` - Consistency monitoring
- `llm-analytics` - Trend analysis
- `llm-test-bench-ui` - Dashboards
- `regression-detector` - Consistency regression
- `model-comparator` - Weighted comparisons
- `reliability-scorer` - Reliability metrics

## Versioning

| Change Type | Version Bump |
|-------------|--------------|
| Breaking schema changes | Major |
| New similarity methods | Minor |
| New config options | Minor |
| New metrics | Minor |
| Bug fixes | Patch |
| Performance improvements | Patch |
| Documentation | Patch |

## Example Usage

### Minimal Input

```json
{
  "execution_groups": [{
    "group_id": "test-1",
    "prompt": "What is 2+2?",
    "provider_name": "openai",
    "model_id": "gpt-4",
    "outputs": [
      {"output_id": "uuid1", "content": "4", "execution_number": 1, "executed_at": "2024-01-01T00:00:00Z"},
      {"output_id": "uuid2", "content": "4", "execution_number": 2, "executed_at": "2024-01-01T00:01:00Z"}
    ]
  }]
}
```

### Full Configuration

```json
{
  "execution_groups": [...],
  "config": {
    "similarity_method": "normalized_levenshtein",
    "additional_methods": ["jaccard_tokens", "cosine_tfidf"],
    "consistency_threshold": 0.90,
    "ngram_size": 3,
    "normalize_whitespace": true,
    "case_sensitive": false,
    "trim_content": true,
    "include_token_analysis": true,
    "include_char_variance": true,
    "compute_pairwise_matrix": true
  }
}
```
