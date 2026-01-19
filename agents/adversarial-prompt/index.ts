/**
 * Adversarial Prompt Agent - Module Exports
 */

export { handler, ADVERSARIAL_PROMPT_AGENT } from './handler';
export { executeCLI, CLI_COMMAND_SPEC } from './cli';

// Re-export contracts for convenience
export {
  // Schemas
  AdversarialPromptInputSchema,
  AdversarialPromptOutputSchema,
  AdversarialPromptDecisionEventSchema,
  AdversarialPromptCLIArgsSchema,
  AdversarialCategorySchema,
  AdversarialSeveritySchema,
  GeneratedPromptSchema,
  // Types
  type AdversarialPromptInput,
  type AdversarialPromptOutput,
  type AdversarialPromptCLIArgs,
  type AdversarialCategory,
  type AdversarialSeverity,
  type GeneratedPrompt,
  // Constants
  ADVERSARIAL_PROMPT_VALID_CONSTRAINTS,
  ADVERSARIAL_PROMPT_NON_RESPONSIBILITIES,
  ADVERSARIAL_PROMPT_ALLOWED_CONSUMERS,
  ADVERSARIAL_CATEGORY_METADATA,
  // Functions
  calculateAdversarialPromptConfidence,
} from '../contracts';
