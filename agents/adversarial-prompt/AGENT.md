# Adversarial Prompt Agent

## Purpose Statement

The Adversarial Prompt Agent generates categorized, severity-ranked adversarial prompts designed to probe LLM robustness, safety boundaries, and failure modes. It produces test inputs for Red Team and Stress Test agents without executing prompts or evaluating responses. This agent is a generation tool, not an execution or evaluation tool.

## Decision Type

`adversarial_prompt_generation`

## What This Agent Does

- Generates adversarial prompts across 25+ categories
- Classifies prompts by severity (low, medium, high, critical)
- Provides attack vector descriptions and expected model behavior
- Includes failure indicators for each generated prompt
- Generates benign variants for A/B testing
- Applies safety ceiling to limit maximum severity
- Produces quality metrics and diversity scores

## What This Agent Does NOT Do

| Non-Responsibility | Reason |
|-------------------|--------|
| Execute prompts against models | Use stress-test agent |
| Evaluate model responses | Use evaluation agents |
| Compare models | No ranking/comparison logic |
| Orchestrate workflows | Use LLM-Orchestrator |
| Call other agents directly | Platform architectural rule |
| Store API keys | Security requirement |
| Generate malware/exploits | Safety requirement |
| Execute actual attacks | Test generation only |
| Persist harmful content | Privacy protection |

## Input Schema

```typescript
interface AdversarialPromptInput {
  // Required: What categories to generate
  categories: AdversarialCategory[];
  severities: AdversarialSeverity[];

  // Counts
  count_per_category: number;
  total_max_count: number;

  // Generation strategy
  strategy: 'template_based' | 'mutation_based' | 'combinatorial' | 'gradient_inspired' | 'random_fuzzing';

  // Optional configurations
  template_config?: PromptTemplateConfig;
  mutation_config?: MutationConfig;
  filter_config?: PromptFilterConfig;

  // Context
  system_prompt_context?: string;
  domain_context?: string;
  language: string;

  // Safety
  include_benign_variants: boolean;
  safety_ceiling: AdversarialSeverity;

  // Purpose and authorization
  purpose: 'red_team_testing' | 'stress_testing' | 'security_audit' | 'research' | 'safety_evaluation';
  authorization_ref?: string;
}
```

## Output Schema

```typescript
interface AdversarialPromptOutput {
  execution_id: string;
  generation_run_id: string;

  // Generated prompts
  prompts: GeneratedPrompt[];

  // Summaries
  category_summaries: CategorySummary[];
  quality_metrics: GenerationQualityMetrics;

  // Request echo
  request_summary: {
    categories_requested: AdversarialCategory[];
    severities_requested: AdversarialSeverity[];
    strategy_used: GenerationStrategy;
    purpose: string;
  };

  // Timing
  started_at: string;
  completed_at: string;
  duration_ms: number;

  // Constraints and warnings
  constraints_applied: string[];
  warnings: string[];
}
```

## Adversarial Categories

| Category | Description | Risk Level |
|----------|-------------|------------|
| `prompt_injection` | Override system prompts | High |
| `jailbreak_attempt` | Bypass safety guidelines | High |
| `instruction_override` | Change model behavior | Medium |
| `role_play_escape` | Escape role-play constraints | Medium |
| `encoding_attacks` | Unicode tricks, homoglyphs | Medium |
| `format_confusion` | Mixed format parsing | Low |
| `delimiter_attacks` | Exploit delimiters | Medium |
| `whitespace_exploitation` | Zero-width chars | Low |
| `content_policy_probe` | Test policy boundaries | Medium |
| `sensitive_topic_probe` | Probe sensitive handling | Medium |
| `harmful_request_variants` | Obfuscated harmful requests | High |
| `dual_use_content` | Legitimate dual-use content | Medium |
| `context_overflow` | Exceed context limits | Low |
| `token_manipulation` | Token boundary exploits | Low |
| `repetition_attacks` | Repeated patterns | Low |
| `nested_structures` | Deeply nested structures | Low |
| `logical_contradictions` | Contradictory instructions | Low |
| `multi_turn_manipulation` | Build-up attacks | High |
| `authority_impersonation` | False authority claims | Medium |
| `urgency_manipulation` | False urgency | Medium |
| `output_format_attacks` | Force output formats | Low |
| `hallucination_triggers` | Cause hallucination | Low |
| `confidence_manipulation` | Manipulate confidence | Low |
| `api_confusion` | Confuse API boundaries | Medium |
| `system_prompt_extraction` | Extract system prompts | High |
| `training_data_extraction` | Extract training data | High |

## CLI Invocation

```bash
# Basic usage
agentics adversarial-prompt --preset basic

# Custom categories
agentics adversarial-prompt --categories prompt_injection,encoding_attacks --count 10

# Output to file
agentics adversarial-prompt --preset comprehensive -o prompts.jsonl --output-format jsonl

# Red team testing
agentics adversarial-prompt --preset red-team --max-severity high

# Dry run validation
agentics adversarial-prompt --input-file config.json --dry-run
```

## Allowed Consumers

- `stress-test` - Primary consumer for adversarial testing
- `red-team-agent` - Red team testing workflows
- `llm-orchestrator` - Workflow coordination
- `llm-observatory` - Telemetry/monitoring
- `security-audit-agent` - Security audits
- `llm-test-bench-ui` - Dashboard display

## Confidence Scoring

| Factor | Weight | Description |
|--------|--------|-------------|
| Category Coverage | 0.25 | Coverage of requested categories |
| Diversity Score | 0.20 | Diversity of generated prompts |
| Quality Ratio | 0.20 | Ratio passing quality filters |
| Severity Accuracy | 0.20 | Accuracy of severity classification |
| Metadata Completeness | 0.15 | Completeness of prompt metadata |

## Constraints

| Constraint | Description |
|-----------|-------------|
| `severity_ceiling_applied` | Prompts above ceiling were filtered |
| `count_limit_reached` | Maximum prompt count was reached |
| `category_filtered` | Category was excluded |
| `content_policy_filter` | Content was filtered for safety |
| `duplicate_removed` | Duplicate prompt was removed |
| `length_limit_applied` | Length constraints applied |
| `token_limit_applied` | Token constraints applied |
| `authorization_required` | Missing authorization reference |

## Data Persistence

### Persisted to ruvector-service

- DecisionEvent records (summary only)
- Generation metadata (counts, categories)
- Prompt hashes (for deduplication)
- Quality metrics
- Execution references

### NOT Persisted

- Full prompt text (privacy protection)
- Attack payloads
- API keys
- PII data
- Harmful content
- Jailbreak technique details

## Versioning

| Change Type | Version Bump |
|-------------|--------------|
| Breaking schema changes | Major |
| New optional fields | Minor |
| Bug fixes | Patch |

## Failure Modes

| Error Code | Description | Recoverable |
|-----------|-------------|-------------|
| `VALIDATION_ERROR` | Invalid input | Yes |
| `EXECUTION_ERROR` | Generation failed | No |
| `CONFIGURATION_ERROR` | Missing config | Yes |

## Deployment

- **Platform**: Google Cloud
- **Service**: llm-test-bench
- **Endpoint**: `/api/v1/agents/adversarial-prompt`
- **Timeout**: 60,000ms
- **Memory**: 256MB
