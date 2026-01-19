/**
 * Synthetic Data Generator Agent - Edge Function Handler
 *
 * AGENT PURPOSE:
 * Generate synthetic datasets for testing, benchmarking, and stress evaluation
 * of LLM systems using pure algorithmic generation (no LLM calls).
 *
 * This agent:
 * - Generates synthetic datasets (YES)
 * - Supports multiple data types (YES)
 * - Applies configurable generation strategies (YES)
 * - Does NOT call LLMs for generation (NO)
 * - Does NOT execute benchmarks (NO)
 * - Does NOT compare models (NO)
 *
 * Deployed as: Google Cloud Edge Function
 * Part of: LLM-Test-Bench unified GCP service
 */

import { randomUUID } from 'crypto';
import {
  // Contracts
  SyntheticDataGeneratorInputSchema,
  SyntheticDataGeneratorOutputSchema,
  DecisionEvent,
  AgentError,
  validateInput,
  hashInputs,
  // Constants
  SYNTHETIC_DATA_GENERATOR_AGENT,
  SYNTHETIC_DATA_VALID_CONSTRAINTS,
  calculateSyntheticDataConfidence,
  // Types
  SyntheticDataGeneratorInput,
  SyntheticDataGeneratorOutput,
  GeneratedItem,
  GenerationStats,
  QualityMetrics,
  DistributionAnalysis,
  SyntheticDataType,
  GenerationStrategy,
  Template,
  SeedExample,
  GenerationConstraints,
  CodingConfig,
  ConversationConfig,
} from '../contracts';

import {
  getRuVectorClient,
  createTelemetryEmitter,
  TelemetryEmitter,
} from '../services';

// =============================================================================
// TYPES
// =============================================================================

export interface EdgeFunctionRequest {
  body: unknown;
  headers: Record<string, string>;
  method: string;
  path: string;
}

export interface EdgeFunctionResponse {
  statusCode: number;
  headers: Record<string, string>;
  body: string;
}

interface ExecutionContext {
  executionId: string;
  startedAt: Date;
  telemetry: TelemetryEmitter;
  constraintsApplied: string[];
  generatedHashes: Set<string>;
  rng: () => number;
}

// =============================================================================
// SIMPLE SEEDED RANDOM (for determinism)
// =============================================================================

function createSeededRandom(seed: number): () => number {
  let state = seed;
  return () => {
    state = (state * 1103515245 + 12345) & 0x7fffffff;
    return state / 0x7fffffff;
  };
}

// =============================================================================
// MAIN HANDLER
// =============================================================================

/**
 * Edge Function Handler for Synthetic Data Generator Agent
 *
 * This is the main entry point for the agent.
 * Deployed as a Google Cloud Edge Function.
 */
export async function handler(
  request: EdgeFunctionRequest
): Promise<EdgeFunctionResponse> {
  const executionId = randomUUID();
  const startedAt = new Date();

  // Initialize telemetry
  const telemetry = createTelemetryEmitter(
    SYNTHETIC_DATA_GENERATOR_AGENT.agent_id,
    SYNTHETIC_DATA_GENERATOR_AGENT.agent_version,
    executionId
  );

  try {
    // Emit invocation telemetry
    telemetry.emitInvoked();

    // Handle only POST requests
    if (request.method !== 'POST') {
      return createErrorResponse(405, 'Method Not Allowed');
    }

    // Parse and validate input
    const inputValidation = validateInput(SyntheticDataGeneratorInputSchema, request.body);
    if (!inputValidation.success) {
      const validationError = (inputValidation as { success: false; error: AgentError }).error;
      telemetry.emitValidationFailed('input', validationError.message ?? 'Validation failed');
      await telemetry.flush();

      return createErrorResponse(400, 'Validation Error', validationError);
    }

    const input = inputValidation.data;

    // Initialize RNG with seed for determinism
    const rng = input.random_seed !== undefined
      ? createSeededRandom(input.random_seed)
      : Math.random;

    const context: ExecutionContext = {
      executionId,
      startedAt,
      telemetry,
      constraintsApplied: [],
      generatedHashes: new Set<string>(),
      rng,
    };

    // Execute generation
    const output = await generateSyntheticData(input, context);

    // Calculate confidence
    const confidence = calculateSyntheticDataConfidence(output, input.difficulty_distribution);

    // Create DecisionEvent
    const decisionEvent = await createDecisionEvent(
      input,
      output,
      confidence,
      context
    );

    // Persist DecisionEvent (async, non-blocking)
    const ruVectorClient = getRuVectorClient();
    await ruVectorClient.persistDecisionEvent(decisionEvent);

    // Emit completion telemetry
    telemetry.emitDecision(decisionEvent.decision_id, confidence);
    telemetry.emitCompleted({
      duration_ms: Date.now() - startedAt.getTime(),
      items_generated: output.generation_stats.generated_count,
      items_failed: output.generation_stats.failed_count,
    });

    // Flush telemetry
    await telemetry.flush();

    // Return success response
    return createSuccessResponse(output, decisionEvent.decision_id);

  } catch (err) {
    // Handle unexpected errors
    const error = err instanceof Error ? err : new Error(String(err));

    telemetry.emitError('EXECUTION_ERROR', error.message);
    await telemetry.flush();

    return createErrorResponse(500, 'Internal Server Error', {
      code: 'EXECUTION_ERROR',
      message: error.message,
      recoverable: false,
      timestamp: new Date().toISOString(),
    });
  }
}

// =============================================================================
// CORE GENERATION LOGIC
// =============================================================================

async function generateSyntheticData(
  input: SyntheticDataGeneratorInput,
  context: ExecutionContext
): Promise<SyntheticDataGeneratorOutput> {
  const generatedItems: GeneratedItem[] = [];
  let failedCount = 0;
  let duplicateCount = 0;
  const strategyDistribution: Record<string, number> = {};
  const templateDistribution: Record<string, number> = {};
  const difficultyDistribution: Record<string, number> = { easy: 0, medium: 0, hard: 0 };

  // Get generator based on strategy and data type
  const generator = getGenerator(input.generation_strategy, input.data_type);

  for (let i = 0; i < input.count; i++) {
    try {
      // Determine difficulty level for this item
      const distribution = {
        easy: input.difficulty_distribution?.easy ?? 0.33,
        medium: input.difficulty_distribution?.medium ?? 0.34,
        hard: input.difficulty_distribution?.hard ?? 0.33,
      };
      const difficultyLevel = selectDifficultyLevel(distribution, context.rng);

      // Generate item
      const item = generator.generate(input, i, context, difficultyLevel);

      // Check uniqueness
      const hash = hashContent(item.content);
      if (context.generatedHashes.has(hash)) {
        duplicateCount++;
        // Retry generation with different variation
        const retryItem = generator.generateUnique(input, i, context, difficultyLevel);
        if (retryItem) {
          const retryHash = hashContent(retryItem.content);
          if (!context.generatedHashes.has(retryHash)) {
            context.generatedHashes.add(retryHash);
            generatedItems.push(retryItem);
            updateDistributions(
              retryItem,
              strategyDistribution,
              templateDistribution,
              difficultyDistribution
            );
          } else {
            failedCount++;
            context.constraintsApplied.push('uniqueness_threshold_unmet');
          }
        } else {
          failedCount++;
        }
        continue;
      }

      context.generatedHashes.add(hash);

      // Validate constraints
      if (input.constraints && !satisfiesConstraints(item, input.constraints)) {
        failedCount++;
        if (!context.constraintsApplied.includes('constraint_satisfaction_below_threshold')) {
          context.constraintsApplied.push('constraint_satisfaction_below_threshold');
        }
        continue;
      }

      generatedItems.push(item);
      updateDistributions(
        item,
        strategyDistribution,
        templateDistribution,
        difficultyDistribution
      );

    } catch (err) {
      failedCount++;
    }
  }

  // Calculate statistics
  const stats = calculateGenerationStats(
    generatedItems,
    input,
    failedCount,
    duplicateCount,
    strategyDistribution,
    templateDistribution,
    difficultyDistribution
  );
  const qualityMetrics = calculateQualityMetrics(generatedItems, input.count);
  const distributionAnalysis = analyzeDistribution(generatedItems, input);

  const completedAt = new Date();

  return {
    execution_id: context.executionId,
    generated_items: generatedItems,
    generation_stats: stats,
    quality_metrics: qualityMetrics,
    distribution_analysis: distributionAnalysis,
    started_at: context.startedAt.toISOString(),
    completed_at: completedAt.toISOString(),
    duration_ms: completedAt.getTime() - context.startedAt.getTime(),
    input_config_summary: {
      data_type: input.data_type,
      generation_strategy: input.generation_strategy,
      requested_count: input.count,
      constraints_applied: context.constraintsApplied,
    },
  };
}

// =============================================================================
// GENERATOR FACTORY
// =============================================================================

interface DataGenerator {
  generate(
    input: SyntheticDataGeneratorInput,
    index: number,
    context: ExecutionContext,
    difficultyLevel: 'easy' | 'medium' | 'hard'
  ): GeneratedItem;

  generateUnique(
    input: SyntheticDataGeneratorInput,
    index: number,
    context: ExecutionContext,
    difficultyLevel: 'easy' | 'medium' | 'hard'
  ): GeneratedItem | null;
}

function getGenerator(
  strategy: GenerationStrategy,
  dataType: SyntheticDataType
): DataGenerator {
  return {
    generate: (input, index, context, difficultyLevel) => {
      switch (strategy) {
        case 'template_based':
          return generateFromTemplate(input, index, context, difficultyLevel);
        case 'variation':
          return generateVariation(input, index, context, difficultyLevel);
        case 'distribution_aware':
          return generateDistributionAware(input, index, context, difficultyLevel);
        case 'edge_case':
          return generateEdgeCase(input, index, context, difficultyLevel);
        case 'adversarial':
          return generateAdversarial(input, index, context, difficultyLevel);
        case 'combinatorial':
          return generateCombinatorial(input, index, context, difficultyLevel);
        case 'progressive_difficulty':
          return generateProgressiveDifficulty(input, index, context, difficultyLevel);
        case 'cross_domain':
          return generateCrossDomain(input, index, context, difficultyLevel);
        default:
          return generateFromTemplate(input, index, context, difficultyLevel);
      }
    },
    generateUnique: (input, index, context, difficultyLevel) => {
      // Try up to 3 times to generate a unique item
      for (let attempt = 0; attempt < 3; attempt++) {
        const item = getGenerator(strategy, dataType).generate(
          input,
          index * 1000 + attempt,
          context,
          difficultyLevel
        );
        const hash = hashContent(item.content);
        if (!context.generatedHashes.has(hash)) {
          return item;
        }
      }
      return null;
    },
  };
}

// =============================================================================
// STRATEGY IMPLEMENTATIONS
// =============================================================================

function generateFromTemplate(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): GeneratedItem {
  const content = generateContentByType(input.data_type, input, index, context, difficultyLevel);

  return createGeneratedItem(
    input.data_type,
    content,
    {
      strategy_used: 'template_based',
      template_id: input.templates?.[0]?.template_id,
      difficulty_level: difficultyLevel,
      variation_index: index,
    },
    context
  );
}

function generateVariation(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): GeneratedItem {
  const seedExamples = input.seed_examples ?? [];
  const selectedSeed = seedExamples.length > 0
    ? seedExamples[Math.floor(context.rng() * seedExamples.length)]
    : null;

  const content = generateContentByType(input.data_type, input, index, context, difficultyLevel);

  return createGeneratedItem(
    input.data_type,
    content,
    {
      strategy_used: 'variation',
      seed_example_id: selectedSeed?.id,
      difficulty_level: difficultyLevel,
      variation_index: index,
    },
    context
  );
}

function generateDistributionAware(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): GeneratedItem {
  const content = generateContentByType(input.data_type, input, index, context, difficultyLevel);

  return createGeneratedItem(
    input.data_type,
    content,
    {
      strategy_used: 'distribution_aware',
      difficulty_level: difficultyLevel,
      variation_index: index,
    },
    context
  );
}

function generateEdgeCase(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): GeneratedItem {
  // Force harder difficulty for edge cases
  const edgeDifficulty = context.rng() < 0.7 ? 'hard' : 'medium';
  const content = generateContentByType(input.data_type, input, index, context, edgeDifficulty);

  return createGeneratedItem(
    input.data_type,
    content,
    {
      strategy_used: 'edge_case',
      difficulty_level: edgeDifficulty,
      variation_index: index,
    },
    context
  );
}

function generateAdversarial(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): GeneratedItem {
  const content = generateContentByType(input.data_type, input, index, context, 'hard');

  return createGeneratedItem(
    input.data_type,
    content,
    {
      strategy_used: 'adversarial',
      difficulty_level: 'hard',
      variation_index: index,
    },
    context
  );
}

function generateCombinatorial(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): GeneratedItem {
  const content = generateContentByType(input.data_type, input, index, context, difficultyLevel);

  return createGeneratedItem(
    input.data_type,
    content,
    {
      strategy_used: 'combinatorial',
      difficulty_level: difficultyLevel,
      variation_index: index,
    },
    context
  );
}

function generateProgressiveDifficulty(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): GeneratedItem {
  // Determine difficulty based on position in batch
  const progress = index / input.count;
  let progressiveDifficulty: 'easy' | 'medium' | 'hard';
  if (progress < 0.33) {
    progressiveDifficulty = 'easy';
  } else if (progress < 0.66) {
    progressiveDifficulty = 'medium';
  } else {
    progressiveDifficulty = 'hard';
  }

  const content = generateContentByType(input.data_type, input, index, context, progressiveDifficulty);

  return createGeneratedItem(
    input.data_type,
    content,
    {
      strategy_used: 'progressive_difficulty',
      difficulty_level: progressiveDifficulty,
      variation_index: index,
    },
    context
  );
}

function generateCrossDomain(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): GeneratedItem {
  const content = generateContentByType(input.data_type, input, index, context, difficultyLevel);

  return createGeneratedItem(
    input.data_type,
    content,
    {
      strategy_used: 'cross_domain',
      difficulty_level: difficultyLevel,
      variation_index: index,
    },
    context
  );
}

// =============================================================================
// DATA TYPE CONTENT GENERATORS
// =============================================================================

function generateContentByType(
  dataType: SyntheticDataType,
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): Record<string, unknown> {
  switch (dataType) {
    case 'text_prompt':
      return generateTextPromptContent(input, index, context, difficultyLevel);
    case 'qa_pair':
      return generateQAPairContent(input, index, context, difficultyLevel);
    case 'multi_turn_conversation':
      return generateConversationContent(input, index, context, difficultyLevel);
    case 'coding_task':
      return generateCodingTaskContent(input, index, context, difficultyLevel);
    case 'summarization':
      return generateSummarizationContent(input, index, context, difficultyLevel);
    case 'creative_writing':
      return generateCreativeWritingContent(input, index, context, difficultyLevel);
    case 'classification':
      return generateClassificationContent(input, index, context, difficultyLevel);
    case 'entity_extraction':
      return generateEntityExtractionContent(input, index, context, difficultyLevel);
    case 'translation':
      return generateTranslationContent(input, index, context, difficultyLevel);
    case 'reasoning_chain':
      return generateReasoningChainContent(input, index, context, difficultyLevel);
    default:
      return generateTextPromptContent(input, index, context, difficultyLevel);
  }
}

// Text Prompt Generator
function generateTextPromptContent(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): Record<string, unknown> {
  const prompts = {
    easy: [
      'Explain what a variable is in programming.',
      'What is the capital of France?',
      'Describe what a computer does.',
      'What is the difference between a cat and a dog?',
      'How do you make a sandwich?',
    ],
    medium: [
      'Explain the concept of recursion with an example.',
      'What are the key differences between SQL and NoSQL databases?',
      'Describe the process of photosynthesis.',
      'How does encryption protect data?',
      'Explain the theory of supply and demand.',
    ],
    hard: [
      'Analyze the implications of quantum computing on cryptography.',
      'Discuss the ethical considerations of AI in healthcare.',
      'Compare and contrast functional and object-oriented programming paradigms.',
      'Explain the P vs NP problem and its significance.',
      'Describe the challenges of achieving artificial general intelligence.',
    ],
  };

  const promptList = prompts[difficultyLevel];
  const selectedPrompt = promptList[Math.floor(context.rng() * promptList.length)];
  const variant = `[Variant ${index + 1}] ${selectedPrompt}`;

  return {
    prompt: variant,
    instruction_type: difficultyLevel === 'easy' ? 'simple' : difficultyLevel === 'medium' ? 'moderate' : 'complex',
    expected_response_length: difficultyLevel === 'easy' ? 'short' : difficultyLevel === 'medium' ? 'medium' : 'long',
  };
}

// QA Pair Generator
function generateQAPairContent(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): Record<string, unknown> {
  const qaPairs = {
    easy: [
      { q: 'What color is the sky on a clear day?', a: 'Blue' },
      { q: 'How many days are in a week?', a: 'Seven' },
      { q: 'What is 2 + 2?', a: '4' },
      { q: 'What animal says "meow"?', a: 'A cat' },
      { q: 'What is the opposite of hot?', a: 'Cold' },
    ],
    medium: [
      { q: 'What is the largest planet in our solar system?', a: 'Jupiter' },
      { q: 'Who wrote Romeo and Juliet?', a: 'William Shakespeare' },
      { q: 'What is the chemical symbol for water?', a: 'H2O' },
      { q: 'In what year did World War II end?', a: '1945' },
      { q: 'What is the speed of light in a vacuum?', a: 'Approximately 299,792,458 meters per second' },
    ],
    hard: [
      { q: 'Explain the difference between supervised and unsupervised learning.', a: 'Supervised learning uses labeled data where the model learns from input-output pairs, while unsupervised learning works with unlabeled data to find hidden patterns or structures.' },
      { q: 'What is the significance of the Turing test?', a: 'The Turing test measures a machine\'s ability to exhibit intelligent behavior indistinguishable from a human, serving as a benchmark for artificial intelligence.' },
      { q: 'Describe the halting problem and why it is undecidable.', a: 'The halting problem asks whether a given program will eventually halt or run forever. Alan Turing proved it undecidable because no algorithm can correctly determine this for all possible program-input pairs.' },
    ],
  };

  const pairs = qaPairs[difficultyLevel];
  const selected = pairs[Math.floor(context.rng() * pairs.length)];

  return {
    question: `[Q${index + 1}] ${selected.q}`,
    answer: selected.a,
    difficulty: difficultyLevel,
    category: 'general_knowledge',
  };
}

// Conversation Generator
function generateConversationContent(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): Record<string, unknown> {
  const config = input.conversation_config ?? { min_turns: 2, max_turns: 6 };
  const numTurns = Math.floor(
    context.rng() * (config.max_turns - config.min_turns + 1) + config.min_turns
  );

  const topics = config.topics ?? ['technology', 'science', 'daily life', 'business'];
  const selectedTopic = topics[Math.floor(context.rng() * topics.length)];

  const conversationStarters = {
    easy: [
      { user: 'Hello, how are you today?', assistant: 'I\'m doing well, thank you for asking! How can I help you today?' },
      { user: 'What\'s your favorite color?', assistant: 'As an AI, I don\'t have personal preferences, but I find all colors interesting in their own ways.' },
    ],
    medium: [
      { user: 'Can you explain how machine learning works?', assistant: 'Machine learning is a subset of AI where computers learn from data to make predictions or decisions without being explicitly programmed.' },
      { user: 'What are the benefits of cloud computing?', assistant: 'Cloud computing offers scalability, cost-efficiency, accessibility from anywhere, automatic updates, and reduced IT maintenance burden.' },
    ],
    hard: [
      { user: 'What are your thoughts on the alignment problem in AI?', assistant: 'The alignment problem concerns ensuring AI systems pursue goals aligned with human values. It\'s crucial because misaligned AI could cause unintended harm at scale.' },
      { user: 'How might quantum computing change cybersecurity?', assistant: 'Quantum computers could break current encryption methods like RSA, necessitating quantum-resistant cryptography. However, they also enable quantum key distribution for theoretically unbreakable encryption.' },
    ],
  };

  const starters = conversationStarters[difficultyLevel];
  const starter = starters[Math.floor(context.rng() * starters.length)];

  const turns: Array<{ role: string; content: string }> = [];
  turns.push({ role: 'user', content: starter.user });
  turns.push({ role: 'assistant', content: starter.assistant });

  // Add more turns
  for (let t = 2; t < numTurns; t++) {
    if (t % 2 === 0) {
      turns.push({ role: 'user', content: `Follow-up question ${t / 2} about ${selectedTopic}.` });
    } else {
      turns.push({ role: 'assistant', content: `Response ${Math.floor(t / 2) + 1} providing more details on ${selectedTopic}.` });
    }
  }

  return {
    conversation_id: `conv_${index + 1}`,
    topic: selectedTopic,
    turns,
    turn_count: turns.length,
    difficulty: difficultyLevel,
  };
}

// Coding Task Generator
function generateCodingTaskContent(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): Record<string, unknown> {
  const config = input.coding_config ?? {
    languages: ['python'],
    include_test_cases: true,
    test_case_count: 3,
    include_edge_cases: true,
  };

  const language = config.languages[Math.floor(context.rng() * config.languages.length)];

  const problems = {
    easy: [
      {
        title: 'Sum of Two Numbers',
        description: 'Write a function that returns the sum of two numbers.',
        signature: 'def add(a: int, b: int) -> int:',
        test_cases: [
          { input: [1, 2], expected: 3 },
          { input: [0, 0], expected: 0 },
          { input: [-1, 1], expected: 0 },
        ],
      },
      {
        title: 'String Length',
        description: 'Write a function that returns the length of a string.',
        signature: 'def string_length(s: str) -> int:',
        test_cases: [
          { input: ['hello'], expected: 5 },
          { input: [''], expected: 0 },
          { input: ['a'], expected: 1 },
        ],
      },
    ],
    medium: [
      {
        title: 'Reverse String',
        description: 'Write a function that reverses a string without using built-in reverse methods.',
        signature: 'def reverse_string(s: str) -> str:',
        test_cases: [
          { input: ['hello'], expected: 'olleh' },
          { input: [''], expected: '' },
          { input: ['a'], expected: 'a' },
          { input: ['ab'], expected: 'ba' },
        ],
      },
      {
        title: 'Fibonacci Number',
        description: 'Write a function that returns the nth Fibonacci number.',
        signature: 'def fibonacci(n: int) -> int:',
        test_cases: [
          { input: [0], expected: 0 },
          { input: [1], expected: 1 },
          { input: [10], expected: 55 },
        ],
      },
    ],
    hard: [
      {
        title: 'Longest Palindromic Substring',
        description: 'Write a function that finds the longest palindromic substring in a given string.',
        signature: 'def longest_palindrome(s: str) -> str:',
        test_cases: [
          { input: ['babad'], expected: 'bab' },
          { input: ['cbbd'], expected: 'bb' },
          { input: ['a'], expected: 'a' },
          { input: [''], expected: '' },
        ],
      },
      {
        title: 'Merge Intervals',
        description: 'Given an array of intervals, merge all overlapping intervals.',
        signature: 'def merge_intervals(intervals: List[List[int]]) -> List[List[int]]:',
        test_cases: [
          { input: [[[1, 3], [2, 6], [8, 10], [15, 18]]], expected: [[1, 6], [8, 10], [15, 18]] },
          { input: [[[1, 4], [4, 5]]], expected: [[1, 5]] },
        ],
      },
    ],
  };

  const problemList = problems[difficultyLevel];
  const problem = problemList[Math.floor(context.rng() * problemList.length)];

  return {
    task_id: `code_${index + 1}`,
    title: problem.title,
    description: problem.description,
    language,
    function_signature: problem.signature,
    test_cases: config.include_test_cases ? problem.test_cases.slice(0, config.test_case_count) : [],
    difficulty: difficultyLevel,
    category: 'algorithm',
  };
}

// Summarization Generator
function generateSummarizationContent(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): Record<string, unknown> {
  const documents = {
    easy: {
      text: 'The sun rises in the east and sets in the west. This happens every day because the Earth rotates on its axis. The rotation takes about 24 hours to complete.',
      summary: 'The sun appears to move across the sky due to Earth\'s 24-hour rotation.',
    },
    medium: {
      text: 'Machine learning is a branch of artificial intelligence that enables computers to learn from data. Instead of being explicitly programmed, these systems improve through experience. Common applications include spam filtering, recommendation systems, and image recognition.',
      summary: 'Machine learning allows computers to learn from data and improve over time, with applications in spam filtering, recommendations, and image recognition.',
    },
    hard: {
      text: 'Quantum computing represents a paradigm shift in computational capability. By leveraging quantum mechanical phenomena such as superposition and entanglement, quantum computers can solve certain problems exponentially faster than classical computers. This has profound implications for cryptography, drug discovery, and optimization problems.',
      summary: 'Quantum computing uses quantum mechanics to solve complex problems faster than classical computers, impacting cryptography, drug discovery, and optimization.',
    },
  };

  const doc = documents[difficultyLevel];

  return {
    document_id: `doc_${index + 1}`,
    document: `[Document ${index + 1}] ${doc.text}`,
    reference_summary: doc.summary,
    word_count: doc.text.split(' ').length,
    difficulty: difficultyLevel,
  };
}

// Creative Writing Generator
function generateCreativeWritingContent(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): Record<string, unknown> {
  const prompts = {
    easy: [
      'Write a short story about a friendly robot.',
      'Describe your perfect day at the beach.',
      'Write about your favorite animal.',
    ],
    medium: [
      'Write a mystery story that takes place in a library.',
      'Create a dialogue between a time traveler and a historical figure.',
      'Write a poem about the changing seasons.',
    ],
    hard: [
      'Write a story that explores the nature of consciousness from multiple perspectives.',
      'Create a narrative that uses unreliable narration to reveal a surprising truth.',
      'Write a piece that blends magical realism with social commentary.',
    ],
  };

  const promptList = prompts[difficultyLevel];
  const selectedPrompt = promptList[Math.floor(context.rng() * promptList.length)];

  const genres = ['fiction', 'poetry', 'narrative', 'descriptive'];
  const genre = genres[Math.floor(context.rng() * genres.length)];

  return {
    prompt_id: `creative_${index + 1}`,
    prompt: selectedPrompt,
    genre,
    tone: input.constraints?.tone ?? 'creative',
    min_words: difficultyLevel === 'easy' ? 50 : difficultyLevel === 'medium' ? 150 : 300,
    difficulty: difficultyLevel,
  };
}

// Classification Generator
function generateClassificationContent(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): Record<string, unknown> {
  const samples = {
    easy: [
      { text: 'I love this product! It works great!', label: 'positive' },
      { text: 'Terrible experience, would not recommend.', label: 'negative' },
      { text: 'It was okay, nothing special.', label: 'neutral' },
    ],
    medium: [
      { text: 'The product has some good features but the price is too high for what you get.', label: 'mixed' },
      { text: 'Customer service was helpful but the wait time was excessive.', label: 'mixed' },
      { text: 'Great value for money, exceeded my expectations!', label: 'positive' },
    ],
    hard: [
      { text: 'While the concept is innovative, the execution leaves much to be desired given the premium positioning.', label: 'critical_positive' },
      { text: 'I suppose it serves its purpose, though one might argue there are better alternatives at this price point.', label: 'reluctant_acceptance' },
    ],
  };

  const sampleList = samples[difficultyLevel];
  const sample = sampleList[Math.floor(context.rng() * sampleList.length)];

  return {
    sample_id: `class_${index + 1}`,
    text: `[Sample ${index + 1}] ${sample.text}`,
    label: sample.label,
    confidence: 0.9 - (difficultyLevel === 'hard' ? 0.2 : difficultyLevel === 'medium' ? 0.1 : 0),
    difficulty: difficultyLevel,
  };
}

// Entity Extraction Generator
function generateEntityExtractionContent(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): Record<string, unknown> {
  const samples = {
    easy: [
      {
        text: 'John Smith works at Microsoft in Seattle.',
        entities: [
          { text: 'John Smith', type: 'PERSON' },
          { text: 'Microsoft', type: 'ORG' },
          { text: 'Seattle', type: 'LOCATION' },
        ],
      },
    ],
    medium: [
      {
        text: 'Dr. Jane Doe, CEO of TechCorp, announced the acquisition of StartupXYZ for $500 million on January 15, 2024.',
        entities: [
          { text: 'Dr. Jane Doe', type: 'PERSON' },
          { text: 'TechCorp', type: 'ORG' },
          { text: 'StartupXYZ', type: 'ORG' },
          { text: '$500 million', type: 'MONEY' },
          { text: 'January 15, 2024', type: 'DATE' },
        ],
      },
    ],
    hard: [
      {
        text: 'The Federal Reserve, led by Chair Jerome Powell, raised interest rates by 0.25% following the FOMC meeting, impacting global markets from Tokyo to London.',
        entities: [
          { text: 'Federal Reserve', type: 'ORG' },
          { text: 'Jerome Powell', type: 'PERSON' },
          { text: '0.25%', type: 'PERCENT' },
          { text: 'FOMC', type: 'ORG' },
          { text: 'Tokyo', type: 'LOCATION' },
          { text: 'London', type: 'LOCATION' },
        ],
      },
    ],
  };

  const sampleList = samples[difficultyLevel];
  const sample = sampleList[Math.floor(context.rng() * sampleList.length)];

  return {
    sample_id: `entity_${index + 1}`,
    text: sample.text,
    entities: sample.entities,
    entity_count: sample.entities.length,
    difficulty: difficultyLevel,
  };
}

// Translation Generator
function generateTranslationContent(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): Record<string, unknown> {
  const pairs = {
    easy: [
      { source: 'Hello, how are you?', target: 'Hola, como estas?', source_lang: 'en', target_lang: 'es' },
      { source: 'Thank you very much.', target: 'Muchas gracias.', source_lang: 'en', target_lang: 'es' },
    ],
    medium: [
      { source: 'The weather forecast predicts rain tomorrow.', target: 'El pronostico del tiempo predice lluvia manana.', source_lang: 'en', target_lang: 'es' },
      { source: 'I would like to make a reservation for two people.', target: 'Me gustaria hacer una reservacion para dos personas.', source_lang: 'en', target_lang: 'es' },
    ],
    hard: [
      { source: 'The committee deliberated extensively before reaching a unanimous decision on the matter.', target: 'El comite delibero extensamente antes de llegar a una decision unanime sobre el asunto.', source_lang: 'en', target_lang: 'es' },
    ],
  };

  const pairList = pairs[difficultyLevel];
  const pair = pairList[Math.floor(context.rng() * pairList.length)];

  return {
    pair_id: `trans_${index + 1}`,
    source_text: pair.source,
    target_text: pair.target,
    source_language: pair.source_lang,
    target_language: pair.target_lang,
    difficulty: difficultyLevel,
  };
}

// Reasoning Chain Generator
function generateReasoningChainContent(
  input: SyntheticDataGeneratorInput,
  index: number,
  context: ExecutionContext,
  difficultyLevel: 'easy' | 'medium' | 'hard'
): Record<string, unknown> {
  const problems = {
    easy: {
      question: 'If John has 5 apples and gives 2 to Mary, how many apples does John have left?',
      steps: [
        'John starts with 5 apples.',
        'John gives 2 apples to Mary.',
        '5 - 2 = 3',
      ],
      answer: '3 apples',
    },
    medium: {
      question: 'A train travels at 60 mph. How far will it travel in 2.5 hours?',
      steps: [
        'Speed = 60 mph',
        'Time = 2.5 hours',
        'Distance = Speed x Time',
        'Distance = 60 x 2.5 = 150 miles',
      ],
      answer: '150 miles',
    },
    hard: {
      question: 'In a room of 23 people, what is the probability that at least two people share a birthday?',
      steps: [
        'Calculate the probability that no two people share a birthday.',
        'P(no shared) = (365/365) x (364/365) x (363/365) x ... x (343/365)',
        'P(no shared) = 365!/((365-23)! x 365^23)',
        'P(no shared) approximately equals 0.493',
        'P(at least one shared) = 1 - P(no shared) = 1 - 0.493 = 0.507',
      ],
      answer: 'Approximately 50.7% or about 1 in 2',
    },
  };

  const problem = problems[difficultyLevel];

  return {
    problem_id: `reason_${index + 1}`,
    question: problem.question,
    reasoning_steps: problem.steps,
    final_answer: problem.answer,
    step_count: problem.steps.length,
    difficulty: difficultyLevel,
  };
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

function createGeneratedItem(
  dataType: SyntheticDataType,
  content: Record<string, unknown>,
  metadata: {
    strategy_used: GenerationStrategy;
    template_id?: string;
    seed_example_id?: string;
    difficulty_level?: 'easy' | 'medium' | 'hard';
    variation_index?: number;
  },
  context: ExecutionContext
): GeneratedItem {
  const contentStr = JSON.stringify(content);
  const lengthChars = contentStr.length;
  const tokenCountApprox = Math.ceil(contentStr.split(/\s+/).length * 1.3);

  return {
    item_id: randomUUID(),
    data_type: dataType,
    content,
    generation_metadata: metadata,
    quality_indicators: {
      length_chars: lengthChars,
      token_count_approx: tokenCountApprox,
      complexity_score: calculateComplexityScore(content, metadata.difficulty_level),
      uniqueness_hash: hashContent(content),
      constraint_satisfaction: 1.0, // Will be adjusted if constraints fail
    },
  };
}

function selectDifficultyLevel(
  distribution: { easy: number; medium: number; hard: number },
  rng: () => number
): 'easy' | 'medium' | 'hard' {
  const roll = rng();
  if (roll < distribution.easy) return 'easy';
  if (roll < distribution.easy + distribution.medium) return 'medium';
  return 'hard';
}

function hashContent(content: Record<string, unknown>): string {
  const normalized = JSON.stringify(content, Object.keys(content).sort());
  // Simple hash for demonstration - in production use crypto.subtle
  let hash = 0;
  for (let i = 0; i < normalized.length; i++) {
    const char = normalized.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash;
  }
  return Math.abs(hash).toString(16).padStart(16, '0');
}

function satisfiesConstraints(
  item: GeneratedItem,
  constraints: GenerationConstraints
): boolean {
  const content = JSON.stringify(item.content);

  if (constraints.min_length_chars && content.length < constraints.min_length_chars) {
    return false;
  }
  if (constraints.max_length_chars && content.length > constraints.max_length_chars) {
    return false;
  }

  if (constraints.forbidden_keywords) {
    for (const keyword of constraints.forbidden_keywords) {
      if (content.toLowerCase().includes(keyword.toLowerCase())) {
        return false;
      }
    }
  }

  return true;
}

function calculateComplexityScore(
  content: Record<string, unknown>,
  difficultyLevel?: 'easy' | 'medium' | 'hard'
): number {
  const baseScore = difficultyLevel === 'hard' ? 0.8 : difficultyLevel === 'medium' ? 0.5 : 0.3;
  const contentLength = JSON.stringify(content).length;
  const lengthFactor = Math.min(1, contentLength / 1000) * 0.2;
  return Math.min(1, baseScore + lengthFactor);
}

function updateDistributions(
  item: GeneratedItem,
  strategyDistribution: Record<string, number>,
  templateDistribution: Record<string, number>,
  difficultyDistribution: Record<string, number>
): void {
  const strategy = item.generation_metadata.strategy_used;
  strategyDistribution[strategy] = (strategyDistribution[strategy] ?? 0) + 1;

  if (item.generation_metadata.template_id) {
    const templateId = item.generation_metadata.template_id;
    templateDistribution[templateId] = (templateDistribution[templateId] ?? 0) + 1;
  }

  if (item.generation_metadata.difficulty_level) {
    const difficulty = item.generation_metadata.difficulty_level;
    difficultyDistribution[difficulty] = (difficultyDistribution[difficulty] ?? 0) + 1;
  }
}

function calculateGenerationStats(
  items: GeneratedItem[],
  input: SyntheticDataGeneratorInput,
  failedCount: number,
  duplicateCount: number,
  strategyDistribution: Record<string, number>,
  templateDistribution: Record<string, number>,
  difficultyDistribution: Record<string, number>
): GenerationStats {
  // Normalize difficulty distribution to ratios
  const total = items.length || 1;
  const normalizedDifficulty: Record<string, number> = {};
  for (const [key, value] of Object.entries(difficultyDistribution)) {
    normalizedDifficulty[key] = value / total;
  }

  return {
    requested_count: input.count,
    generated_count: items.length,
    failed_count: failedCount,
    duplicate_count: duplicateCount,
    strategy_distribution: strategyDistribution,
    template_distribution: Object.keys(templateDistribution).length > 0 ? templateDistribution : undefined,
    difficulty_distribution: normalizedDifficulty,
  };
}

function calculateQualityMetrics(
  items: GeneratedItem[],
  requestedCount: number
): QualityMetrics {
  if (items.length === 0) {
    return {
      avg_length_chars: 0,
      avg_token_count: 0,
      avg_complexity_score: 0,
      constraint_satisfaction_rate: 0,
      unique_items_rate: 0,
    };
  }

  const totalLength = items.reduce((sum, item) => sum + item.quality_indicators.length_chars, 0);
  const totalTokens = items.reduce((sum, item) => sum + item.quality_indicators.token_count_approx, 0);
  const totalComplexity = items.reduce((sum, item) => sum + item.quality_indicators.complexity_score, 0);
  const totalSatisfaction = items.reduce((sum, item) => sum + item.quality_indicators.constraint_satisfaction, 0);

  // Get unique hashes
  const uniqueHashes = new Set(items.map(item => item.quality_indicators.uniqueness_hash));

  return {
    avg_length_chars: totalLength / items.length,
    avg_token_count: totalTokens / items.length,
    avg_complexity_score: totalComplexity / items.length,
    constraint_satisfaction_rate: totalSatisfaction / items.length,
    unique_items_rate: uniqueHashes.size / items.length,
  };
}

function analyzeDistribution(
  items: GeneratedItem[],
  input: SyntheticDataGeneratorInput
): DistributionAnalysis {
  if (items.length === 0) {
    return {
      length_distribution: { min: 0, max: 0, mean: 0, median: 0, stddev: 0 },
    };
  }

  const lengths = items.map(item => item.quality_indicators.length_chars).sort((a, b) => a - b);

  const min = lengths[0];
  const max = lengths[lengths.length - 1];
  const mean = lengths.reduce((a, b) => a + b, 0) / lengths.length;
  const median = lengths[Math.floor(lengths.length / 2)];
  const variance = lengths.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / lengths.length;
  const stddev = Math.sqrt(variance);

  // Calculate actual difficulty distribution
  const difficultyActual: Record<string, number> = { easy: 0, medium: 0, hard: 0 };
  for (const item of items) {
    const diff = item.generation_metadata.difficulty_level ?? 'medium';
    difficultyActual[diff] = (difficultyActual[diff] ?? 0) + 1;
  }
  // Normalize
  for (const key of Object.keys(difficultyActual)) {
    difficultyActual[key] = difficultyActual[key] / items.length;
  }

  return {
    length_distribution: { min, max, mean, median, stddev },
    difficulty_actual: difficultyActual,
  };
}

// =============================================================================
// DECISION EVENT CREATION
// =============================================================================

async function createDecisionEvent(
  input: SyntheticDataGeneratorInput,
  output: SyntheticDataGeneratorOutput,
  confidence: number,
  context: ExecutionContext
): Promise<DecisionEvent> {
  const inputsHash = await hashInputs(input);

  return {
    agent_id: SYNTHETIC_DATA_GENERATOR_AGENT.agent_id,
    agent_version: SYNTHETIC_DATA_GENERATOR_AGENT.agent_version,
    decision_type: SYNTHETIC_DATA_GENERATOR_AGENT.decision_type,
    decision_id: randomUUID(),
    inputs_hash: inputsHash,
    inputs_summary: {
      data_type: input.data_type,
      generation_strategy: input.generation_strategy,
      requested_count: input.count,
    },
    outputs: output,
    confidence,
    confidence_factors: [
      {
        factor: 'coverage_score',
        weight: 0.25,
        value: output.generation_stats.generated_count / output.generation_stats.requested_count,
      },
      {
        factor: 'constraint_satisfaction',
        weight: 0.30,
        value: output.quality_metrics.constraint_satisfaction_rate,
      },
      {
        factor: 'uniqueness_score',
        weight: 0.25,
        value: output.quality_metrics.unique_items_rate,
      },
    ],
    constraints_applied: context.constraintsApplied,
    execution_ref: {
      execution_id: context.executionId,
    },
    timestamp: new Date().toISOString(),
    duration_ms: Date.now() - context.startedAt.getTime(),
  };
}

// =============================================================================
// RESPONSE HELPERS
// =============================================================================

function createSuccessResponse(
  output: SyntheticDataGeneratorOutput,
  decisionId: string
): EdgeFunctionResponse {
  return {
    statusCode: 200,
    headers: {
      'Content-Type': 'application/json',
      'X-Decision-Id': decisionId,
      'X-Agent-Id': SYNTHETIC_DATA_GENERATOR_AGENT.agent_id,
      'X-Agent-Version': SYNTHETIC_DATA_GENERATOR_AGENT.agent_version,
    },
    body: JSON.stringify({
      success: true,
      decision_id: decisionId,
      data: output,
    }),
  };
}

function createErrorResponse(
  statusCode: number,
  message: string,
  error?: AgentError
): EdgeFunctionResponse {
  return {
    statusCode,
    headers: {
      'Content-Type': 'application/json',
      'X-Agent-Id': SYNTHETIC_DATA_GENERATOR_AGENT.agent_id,
      'X-Agent-Version': SYNTHETIC_DATA_GENERATOR_AGENT.agent_version,
    },
    body: JSON.stringify({
      success: false,
      error: error ?? {
        code: statusCode === 400 ? 'VALIDATION_ERROR' : 'EXECUTION_ERROR',
        message,
        recoverable: statusCode < 500,
        timestamp: new Date().toISOString(),
      },
    }),
  };
}

// =============================================================================
// EXPORTS
// =============================================================================

export { SYNTHETIC_DATA_GENERATOR_AGENT };
