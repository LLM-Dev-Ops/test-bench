# Synthetic Data Generator Agent

## Agent Identity

| Property | Value |
|----------|-------|
| **Agent ID** | `synthetic-data-generator` |
| **Version** | `1.0.0` |
| **Decision Type** | `synthetic_data_generation` |
| **Deployment** | Google Cloud Edge Function |
| **Service** | LLM-Test-Bench |

## Purpose

Generate synthetic datasets for testing, benchmarking, and stress evaluation of LLM systems. This agent produces high-quality synthetic data across multiple formats using **pure algorithmic generation** (no LLM calls required).

## Responsibility Matrix

### This Agent DOES:

- Generate synthetic datasets across 10 data types
- Support 8 different generation strategies
- Apply configurable constraints to generated data
- Ensure uniqueness of generated items
- Calculate quality metrics and confidence scores
- Emit DecisionEvents to ruvector-service
- Provide deterministic output with random seeds
- Support CLI and API invocation

### This Agent DOES NOT:

| Action | Reason |
|--------|--------|
| Call LLMs | Pure algorithmic generation |
| Compare models | Use `model-comparator` agent |
| Execute benchmarks | Use `benchmark-runner` agent |
| Score quality semantically | Requires LLM |
| Enforce policy | Policy is LLM-Policy-Engine's job |
| Orchestrate workflows | Orchestration is LLM-Orchestrator's job |
| Call other agents | Agents never call each other directly |
| Store API keys | Security violation |
| Generate PII | Privacy violation |
| Generate harmful content | Safety violation |
| Execute generated code | Security violation |

## Supported Data Types

| Type | Description |
|------|-------------|
| `text_prompt` | Single prompts/instructions |
| `qa_pair` | Question-answer pairs |
| `multi_turn_conversation` | Multi-turn dialogues |
| `coding_task` | Code problems with test cases |
| `summarization` | Document + summary pairs |
| `creative_writing` | Creative writing prompts |
| `classification` | Text + label pairs |
| `entity_extraction` | Text + entities pairs |
| `translation` | Source + target language pairs |
| `reasoning_chain` | Multi-step reasoning problems |

## Generation Strategies

| Strategy | Description |
|----------|-------------|
| `template_based` | Generate from templates with placeholders |
| `variation` | Generate variations from seed examples |
| `distribution_aware` | Match specified difficulty distributions |
| `edge_case` | Focus on boundary conditions |
| `adversarial` | Generate challenging/tricky examples |
| `combinatorial` | Combine elements systematically |
| `progressive_difficulty` | Increase complexity gradually |
| `cross_domain` | Mix elements across domains |

## API Contract

### Input Schema

```typescript
{
  // Required
  data_type: SyntheticDataType,
  generation_strategy: GenerationStrategy,
  count: number, // 1-10000

  // Optional
  seed_examples?: SeedExample[],
  templates?: Template[],
  constraints?: GenerationConstraints,
  difficulty_distribution?: { easy: number, medium: number, hard: number },
  coding_config?: CodingConfig,
  conversation_config?: ConversationConfig,
  output_format?: 'json' | 'jsonl' | 'csv',
  random_seed?: number,
  caller_id?: string,
  correlation_id?: string
}
```

### Output Schema

```typescript
{
  execution_id: string,
  generated_items: GeneratedItem[],
  generation_stats: {
    requested_count: number,
    generated_count: number,
    failed_count: number,
    duplicate_count: number,
    strategy_distribution: Record<string, number>,
    difficulty_distribution?: Record<string, number>
  },
  quality_metrics: {
    avg_length_chars: number,
    avg_token_count: number,
    avg_complexity_score: number,
    constraint_satisfaction_rate: number,
    unique_items_rate: number
  },
  distribution_analysis: {
    length_distribution: { min, max, mean, median, stddev },
    difficulty_actual?: Record<string, number>
  },
  started_at: string,
  completed_at: string,
  duration_ms: number,
  input_config_summary: InputConfigSummary
}
```

## CLI Usage

```bash
# Basic usage
agentics synthetic-data-generator --type qa_pair --count 100

# With strategy
agentics synthetic-data-generator --type coding_task --strategy progressive_difficulty --count 50

# With preset
agentics synthetic-data-generator --preset qa-benchmark --count 1000

# With seed for reproducibility
agentics synthetic-data-generator --type text_prompt --count 10 --seed 42

# From input file
agentics synthetic-data-generator --input-file config.json

# Output to file
agentics synthetic-data-generator --type qa_pair --count 100 --output-file data.json

# Different output formats
agentics synthetic-data-generator --type qa_pair --count 100 --output-format jsonl
```

## Presets

| Preset | Description |
|--------|-------------|
| `qa-benchmark` | QA pairs with difficulty distribution (30/50/20) |
| `coding-challenge` | Coding tasks with Python/JS and test cases |
| `stress-test-prompts` | Edge case prompts for stress testing |
| `conversation-dataset` | Multi-turn conversations (3-8 turns) |
| `adversarial-inputs` | Adversarial/tricky prompts |

## DecisionEvent Contract

Every invocation emits exactly ONE DecisionEvent to ruvector-service:

```typescript
{
  agent_id: 'synthetic-data-generator',
  agent_version: '1.0.0',
  decision_type: 'synthetic_data_generation',
  decision_id: '<uuid>',
  inputs_hash: '<sha256>',
  inputs_summary: {
    data_type: string,
    generation_strategy: string,
    requested_count: number
  },
  outputs: SyntheticDataGeneratorOutput,
  confidence: number, // 0-1
  confidence_factors: [
    { factor: 'coverage_score', weight: 0.25, value: number },
    { factor: 'constraint_satisfaction', weight: 0.30, value: number },
    { factor: 'uniqueness_score', weight: 0.25, value: number }
  ],
  constraints_applied: string[],
  execution_ref: { execution_id: '<uuid>' },
  timestamp: '<iso8601>',
  duration_ms: number
}
```

## Confidence Scoring

Confidence is calculated as a weighted sum:

| Factor | Weight | Description |
|--------|--------|-------------|
| Coverage Score | 0.25 | generated_count / requested_count |
| Constraint Satisfaction | 0.30 | % of items satisfying constraints |
| Distribution Match | 0.20 | Match between requested and actual distributions |
| Uniqueness Score | 0.25 | % of unique items |

## Constraints

### Valid Constraints (may be applied)

- `max_generation_time_exceeded`
- `max_item_count_reached`
- `memory_limit_approached`
- `uniqueness_threshold_unmet`
- `constraint_satisfaction_below_threshold`
- `template_exhausted`
- `seed_examples_depleted`
- `complexity_target_unreachable`

### Generation Constraints (configurable)

```typescript
{
  min_length_chars?: number,
  max_length_chars?: number,
  min_tokens_approx?: number,
  max_tokens_approx?: number,
  required_keywords?: string[],
  forbidden_keywords?: string[],
  language?: string, // default: 'en'
  tone?: 'formal' | 'casual' | 'technical' | 'creative',
  domain?: string,
  complexity_level?: 'simple' | 'moderate' | 'complex' | 'expert'
}
```

## Error Codes

| Code | HTTP Status | Recoverable | Description |
|------|-------------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Yes | Input failed schema validation |
| `GENERATION_ERROR` | 500 | Yes | Data generation failed |
| `CONSTRAINT_ERROR` | 400 | Yes | Could not satisfy constraints |
| `TEMPLATE_ERROR` | 400 | Yes | Template parsing/expansion failed |
| `TIMEOUT_ERROR` | 504 | Yes | Generation exceeded time limit |
| `MEMORY_ERROR` | 500 | No | Memory limit exceeded |
| `CONFIGURATION_ERROR` | 400 | Yes | Invalid configuration |
| `PERSISTENCE_ERROR` | 500 | No | Failed to persist decision |

## Allowed Consumers

- `llm-orchestrator` - Workflow coordination
- `llm-observatory` - Telemetry/monitoring
- `llm-analytics` - Aggregation/analysis
- `llm-test-bench-ui` - Dashboard display
- `benchmark-runner` - Consume generated data for benchmarks
- `stress-test` - Consume generated data for stress testing
- `quality-scoring` - Validate generated data quality

## Runtime Configuration

| Setting | Value |
|---------|-------|
| Timeout | 5 minutes |
| Memory | 1024 MB |
| CPU | 2 vCPU |
| Max Concurrent | 50 requests |
| Max Items/Request | 10,000 |
| Rate Limit | 30/min, 500/hour |

## Examples

### Generate QA Pairs

```bash
curl -X POST https://api.llm-test-bench.com/api/v1/agents/synthetic-data-generator \
  -H "Content-Type: application/json" \
  -d '{
    "data_type": "qa_pair",
    "generation_strategy": "distribution_aware",
    "count": 100,
    "difficulty_distribution": {
      "easy": 0.3,
      "medium": 0.5,
      "hard": 0.2
    }
  }'
```

### Generate Coding Tasks

```bash
curl -X POST https://api.llm-test-bench.com/api/v1/agents/synthetic-data-generator \
  -H "Content-Type: application/json" \
  -d '{
    "data_type": "coding_task",
    "generation_strategy": "progressive_difficulty",
    "count": 50,
    "coding_config": {
      "languages": ["python", "javascript"],
      "include_test_cases": true,
      "test_case_count": 5,
      "include_edge_cases": true
    }
  }'
```

### Deterministic Generation

```bash
curl -X POST https://api.llm-test-bench.com/api/v1/agents/synthetic-data-generator \
  -H "Content-Type: application/json" \
  -d '{
    "data_type": "text_prompt",
    "generation_strategy": "edge_case",
    "count": 20,
    "random_seed": 42
  }'
```

## Versioning

| Version Change | When |
|----------------|------|
| Major (X.0.0) | Breaking changes to input/output schemas or algorithms |
| Minor (0.X.0) | New data types, strategies, or optional fields |
| Patch (0.0.X) | Bug fixes, performance improvements, documentation |
