/**
 * Evaluator types and interfaces
 *
 * This module defines TypeScript types for evaluating LLM responses.
 */

/**
 * Evaluation metric types
 */
export type EvaluatorType =
  | 'perplexity'
  | 'coherence'
  | 'relevance'
  | 'faithfulness'
  | 'llm-as-judge'
  | 'readability'
  | 'sentiment'
  | 'toxicity'
  | 'pii-detection'
  | 'custom';

/**
 * Base evaluation result
 */
export interface EvaluationResult {
  /** Type of evaluator */
  type: EvaluatorType;

  /** Evaluation score (typically 0.0 - 1.0) */
  score: number;

  /** Detailed explanation of the score */
  explanation?: string;

  /** Additional metadata */
  metadata?: Record<string, unknown>;

  /** Timestamp */
  timestamp: string;
}

/**
 * Perplexity evaluation result
 */
export interface PerplexityResult extends EvaluationResult {
  type: 'perplexity';

  /** Perplexity value (lower is better) */
  perplexity: number;

  /** Log probability */
  logProb: number;
}

/**
 * Coherence evaluation result
 */
export interface CoherenceResult extends EvaluationResult {
  type: 'coherence';

  /** Coherence score (0.0 - 1.0) */
  coherenceScore: number;

  /** Semantic similarity scores between sentences */
  sentenceSimilarities?: number[];

  /** Discourse marker analysis */
  discourseMarkers?: {
    count: number;
    types: string[];
  };
}

/**
 * Relevance evaluation result
 */
export interface RelevanceResult extends EvaluationResult {
  type: 'relevance';

  /** Relevance score (0.0 - 1.0) */
  relevanceScore: number;

  /** Similarity to reference/context */
  similarity?: number;

  /** Key topics covered */
  topicsCovered?: string[];
}

/**
 * Faithfulness evaluation result
 */
export interface FaithfulnessResult extends EvaluationResult {
  type: 'faithfulness';

  /** Faithfulness score (0.0 - 1.0) */
  faithfulnessScore: number;

  /** Claims that are supported by source */
  supportedClaims?: number;

  /** Total claims made */
  totalClaims?: number;

  /** Hallucination indicators */
  hallucinations?: string[];
}

/**
 * LLM-as-Judge evaluation result
 */
export interface LLMJudgeResult extends EvaluationResult {
  type: 'llm-as-judge';

  /** Judge's rating (typically 1-10) */
  rating: number;

  /** Judge's reasoning */
  reasoning: string;

  /** Criteria evaluated */
  criteria: string[];

  /** Judge model used */
  judgeModel: string;
}

/**
 * Readability evaluation result
 */
export interface ReadabilityResult extends EvaluationResult {
  type: 'readability';

  /** Flesch Reading Ease score */
  fleschScore?: number;

  /** Flesch-Kincaid Grade Level */
  gradeLevel?: number;

  /** Average sentence length */
  avgSentenceLength?: number;

  /** Average word length */
  avgWordLength?: number;
}

/**
 * Sentiment evaluation result
 */
export interface SentimentResult extends EvaluationResult {
  type: 'sentiment';

  /** Sentiment label */
  sentiment: 'positive' | 'negative' | 'neutral';

  /** Confidence score (0.0 - 1.0) */
  confidence: number;

  /** Detailed sentiment scores */
  scores?: {
    positive: number;
    negative: number;
    neutral: number;
  };
}

/**
 * Toxicity evaluation result
 */
export interface ToxicityResult extends EvaluationResult {
  type: 'toxicity';

  /** Toxicity score (0.0 - 1.0) */
  toxicityScore: number;

  /** Whether content is toxic */
  isToxic: boolean;

  /** Specific toxicity categories detected */
  categories?: {
    hate?: number;
    harassment?: number;
    violence?: number;
    sexual?: number;
    profanity?: number;
  };
}

/**
 * PII detection result
 */
export interface PIIDetectionResult extends EvaluationResult {
  type: 'pii-detection';

  /** Whether PII was detected */
  piiDetected: boolean;

  /** Types of PII found */
  piiTypes?: Array<{
    type: 'email' | 'phone' | 'ssn' | 'credit_card' | 'address' | 'name' | 'other';
    value: string;
    confidence: number;
  }>;

  /** Redacted text (if applicable) */
  redactedText?: string;
}

/**
 * Evaluation configuration
 */
export interface EvaluationConfig {
  /** Evaluators to use */
  evaluators: EvaluatorType[];

  /** Reference text for comparison (optional) */
  referenceText?: string;

  /** Context or source documents (optional) */
  context?: string[];

  /** Custom evaluation criteria */
  customCriteria?: string[];

  /** Judge model to use for LLM-as-Judge */
  judgeModel?: string;

  /** Whether to run evaluations in parallel */
  parallel?: boolean;

  /** Cache evaluation results */
  cacheResults?: boolean;
}

/**
 * Combined evaluation results
 */
export interface CombinedEvaluationResults {
  /** Text that was evaluated */
  text: string;

  /** Individual evaluation results */
  results: EvaluationResult[];

  /** Overall aggregate score */
  overallScore: number;

  /** Evaluation duration in milliseconds */
  durationMs: number;

  /** Timestamp */
  timestamp: string;
}
