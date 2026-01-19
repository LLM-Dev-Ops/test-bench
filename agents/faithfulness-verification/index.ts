/**
 * Faithfulness Verification Agent - Module Exports
 *
 * Central export for the Faithfulness Verification Agent.
 */

// Handler
export { handler, FAITHFULNESS_VERIFICATION_AGENT } from './handler';
export type { EdgeFunctionRequest, EdgeFunctionResponse } from './handler';

// CLI
export { main as runCLI, parseArgs, formatOutput } from './cli';
