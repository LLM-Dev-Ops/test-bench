/**
 * Agentics Dev Platform - Agents Module
 *
 * Central export for all agents implementing the Agentics Dev platform architecture.
 *
 * ARCHITECTURE RULES:
 * - Each agent executes inside its owning repo
 * - Agents are implemented as Google Cloud Edge Functions
 * - Agents are stateless at runtime
 * - Long-term memory persists ONLY via ruvector-service
 * - Agents NEVER call other agents directly
 * - Agents NEVER orchestrate workflows
 * - Agents NEVER enforce policy
 */

// Contracts (canonical schemas)
export * from './contracts';

// Services (ruvector-client, telemetry)
export * from './services';

// Registry (platform registration metadata)
export * from './registry';

// Benchmark Runner Agent
export * from './benchmark-runner';

// Regression Detection Agent
export * from './regression-detection';

// Quality Scoring Agent
export * from './quality-scoring';

// Hallucination Detector Agent
export * from './hallucination-detector';

// Faithfulness Verification Agent
export * from './faithfulness-verification';

// Bias Detection Agent
export * from './bias-detection';

// Golden Dataset Validator Agent
export * from './golden-dataset-validator';

// Synthetic Data Generator Agent
export * from './synthetic-data-generator';
