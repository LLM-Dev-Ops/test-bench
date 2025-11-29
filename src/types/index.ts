/**
 * Type definitions for LLM Test Bench SDK
 *
 * This module re-exports all type definitions for convenient importing.
 */

export * from './providers.js';
export * from './benchmarks.js';
export * from './evaluators.js';

/**
 * SDK Configuration
 */
export interface SDKConfig {
  /** Path to the LLM Test Bench CLI binary (optional, auto-detected if not provided) */
  cliPath?: string;

  /** Working directory for operations */
  workingDir?: string;

  /** Whether to enable verbose logging */
  verbose?: boolean;

  /** Environment variables to pass to CLI commands */
  env?: Record<string, string>;

  /** Default timeout for CLI operations in milliseconds */
  timeout?: number;
}

/**
 * CLI execution result
 */
export interface CLIResult<T = unknown> {
  /** Whether the command succeeded */
  success: boolean;

  /** Parsed output data */
  data?: T;

  /** Error message (if failed) */
  error?: string;

  /** Exit code */
  exitCode: number;

  /** Raw stdout */
  stdout: string;

  /** Raw stderr */
  stderr: string;

  /** Execution duration in milliseconds */
  durationMs: number;
}
