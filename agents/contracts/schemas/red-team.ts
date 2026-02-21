/**
 * Red Team Agent - Contract Schema
 *
 * Defines input/output schemas and constants for the Red Team evaluation agent.
 * This agent executes adversarial attacks against target LLMs and measures
 * model resistance and safety boundaries.
 */

import { z } from 'zod';

// =============================================================================
// AGENT CONSTANTS
// =============================================================================

export const RED_TEAM_AGENT = {
  agent_id: 'red-team',
  agent_version: '1.0.0',
  decision_type: 'red_team_evaluation',
} as const;

export const RED_TEAM_VALID_CONSTRAINTS = [
  'max_duration_ms',
  'max_total_cost_usd',
  'max_severity',
  'stop_on_critical',
] as const;

export const RED_TEAM_ALLOWED_CONSUMERS = [
  'llm-orchestrator',
  'agentics-cli',
  'llm-test-bench-dashboard',
] as const;

export const RED_TEAM_NON_RESPONSIBILITIES = [
  'MUST NOT generate adversarial prompts without executing them (use adversarial-prompt agent)',
  'MUST NOT benchmark normal performance (use benchmark-runner agent)',
  'MUST NOT compare models (use model-comparator agent)',
  'MUST NOT orchestrate workflows',
  'MUST NOT enforce policy',
] as const;

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

export const RedTeamTargetModelSchema = z.object({
  provider_name: z.string(),
  model_id: z.string(),
  api_key_ref: z.string().optional(),
  base_url: z.string().url().optional(),
  timeout_ms: z.number().int().positive().default(30000),
});

export const RedTeamAttackCategory = z.enum([
  'prompt_injection',
  'jailbreak',
  'data_extraction',
  'harmful_content',
  'bias_elicitation',
  'privacy_violation',
  'misinformation',
  'system_prompt_leak',
]);

export const RedTeamSeverity = z.enum(['low', 'medium', 'high', 'critical']);

export const RedTeamExecutionConfig = z.object({
  max_duration_ms: z.number().int().positive().default(300000),
  max_total_cost_usd: z.number().positive().optional(),
  stop_on_critical: z.boolean().default(false),
  collect_responses: z.boolean().default(true),
});

export const RedTeamInputSchema = z.object({
  target_model: RedTeamTargetModelSchema,
  attack_categories: z.array(RedTeamAttackCategory).min(1),
  severity_levels: z.array(RedTeamSeverity).default(['low', 'medium', 'high']),
  tests_per_category: z.number().int().min(1).max(100).default(5),
  system_prompt: z.string().optional(),
  execution_config: RedTeamExecutionConfig.optional(),
});

export type RedTeamInput = z.infer<typeof RedTeamInputSchema>;

// =============================================================================
// OUTPUT TYPES
// =============================================================================

export interface RedTeamAttackResult {
  attack_id: string;
  category: string;
  severity: string;
  prompt: string;
  response?: string;
  success: boolean;
  model_resisted: boolean;
  resistance_score: number;
  latency_ms: number;
  timestamp: string;
  error_message?: string;
}

export interface RedTeamCategorySummary {
  category: string;
  total_attacks: number;
  model_resisted_count: number;
  model_failed_count: number;
  resistance_rate: number;
  avg_resistance_score: number;
  critical_failures: number;
}

export interface RedTeamOutput {
  execution_id: string;
  target_model: { provider_name: string; model_id: string };
  started_at: string;
  completed_at: string;
  total_duration_ms: number;
  total_attacks: number;
  total_resisted: number;
  total_failed: number;
  overall_resistance_rate: number;
  overall_safety_score: number;
  attack_results: RedTeamAttackResult[];
  category_summaries: RedTeamCategorySummary[];
  critical_vulnerabilities: string[];
  recommendations: string[];
}

// =============================================================================
// OUTPUT SCHEMA
// =============================================================================

export const RedTeamOutputSchema = z.object({
  execution_id: z.string().uuid(),
  target_model: z.object({
    provider_name: z.string(),
    model_id: z.string(),
  }),
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  total_duration_ms: z.number().nonnegative(),
  total_attacks: z.number().int().nonnegative(),
  total_resisted: z.number().int().nonnegative(),
  total_failed: z.number().int().nonnegative(),
  overall_resistance_rate: z.number().min(0).max(1),
  overall_safety_score: z.number().min(0).max(1),
  attack_results: z.array(z.object({
    attack_id: z.string().uuid(),
    category: z.string(),
    severity: z.string(),
    prompt: z.string(),
    response: z.string().optional(),
    success: z.boolean(),
    model_resisted: z.boolean(),
    resistance_score: z.number().min(0).max(1),
    latency_ms: z.number().nonnegative(),
    timestamp: z.string().datetime(),
    error_message: z.string().optional(),
  })),
  category_summaries: z.array(z.object({
    category: z.string(),
    total_attacks: z.number().int().nonnegative(),
    model_resisted_count: z.number().int().nonnegative(),
    model_failed_count: z.number().int().nonnegative(),
    resistance_rate: z.number().min(0).max(1),
    avg_resistance_score: z.number().min(0).max(1),
    critical_failures: z.number().int().nonnegative(),
  })),
  critical_vulnerabilities: z.array(z.string()),
  recommendations: z.array(z.string()),
});

export const RedTeamDecisionEventSchema = z.object({
  agent_id: z.literal(RED_TEAM_AGENT.agent_id),
  agent_version: z.literal(RED_TEAM_AGENT.agent_version),
  decision_type: z.literal(RED_TEAM_AGENT.decision_type),
});

// =============================================================================
// CONFIDENCE CALCULATION
// =============================================================================

export function calculateRedTeamConfidence(output: RedTeamOutput): number {
  const sampleFactor = Math.min(1, output.total_attacks / 50);
  const categoryFactor = Math.min(1, output.category_summaries.length / 5);
  const consistencyFactor = output.total_attacks > 0
    ? 1 - Math.abs(output.overall_resistance_rate - 0.5) * 0.2
    : 0;
  return sampleFactor * 0.4 + categoryFactor * 0.3 + consistencyFactor * 0.3;
}

// =============================================================================
// CATEGORY METADATA
// =============================================================================

export const RED_TEAM_CATEGORY_METADATA: Record<string, {
  description: string;
  risk_level: string;
  typical_techniques: string[];
}> = {
  prompt_injection: {
    description: 'Attempts to override model instructions via injected prompts',
    risk_level: 'high',
    typical_techniques: ['instruction override', 'role manipulation', 'delimiter injection'],
  },
  jailbreak: {
    description: 'Attempts to bypass safety guardrails and content filters',
    risk_level: 'critical',
    typical_techniques: ['DAN prompts', 'roleplay exploitation', 'hypothetical framing'],
  },
  data_extraction: {
    description: 'Attempts to extract training data or system information',
    risk_level: 'high',
    typical_techniques: ['completion attacks', 'memorization probing', 'data reconstruction'],
  },
  harmful_content: {
    description: 'Attempts to generate harmful, illegal, or dangerous content',
    risk_level: 'critical',
    typical_techniques: ['indirect requests', 'euphemism chains', 'context manipulation'],
  },
  bias_elicitation: {
    description: 'Attempts to elicit biased or discriminatory outputs',
    risk_level: 'medium',
    typical_techniques: ['stereotype prompting', 'demographic targeting', 'implicit bias probing'],
  },
  privacy_violation: {
    description: 'Attempts to extract personal or sensitive information',
    risk_level: 'high',
    typical_techniques: ['social engineering', 'PII extraction', 'identity inference'],
  },
  misinformation: {
    description: 'Attempts to generate false or misleading information',
    risk_level: 'medium',
    typical_techniques: ['factual distortion', 'citation fabrication', 'authority impersonation'],
  },
  system_prompt_leak: {
    description: 'Attempts to reveal system prompt or hidden instructions',
    risk_level: 'high',
    typical_techniques: ['direct request', 'format manipulation', 'encoding tricks'],
  },
};
