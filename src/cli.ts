#!/usr/bin/env node

/**
 * CLI entry point for LLM Test Bench TypeScript wrapper
 *
 * This provides a thin TypeScript wrapper around the Rust CLI,
 * allowing the tool to be used via npm while maintaining all
 * the performance and functionality of the Rust implementation.
 */

import { spawn } from 'child_process';
import { findCLIPath } from './utils/cli-executor.js';

/**
 * Main CLI function
 */
async function main(): Promise<void> {
  // Find the CLI binary
  const cliPath = findCLIPath();

  if (!cliPath) {
    console.error('Error: LLM Test Bench CLI not found.');
    console.error('');
    console.error('Please install the CLI using one of these methods:');
    console.error('  1. cargo install llm-test-bench');
    console.error('  2. Download from https://github.com/globalbusinessadvisors/llm-test-bench/releases');
    console.error('');
    console.error('Or set CARGO_HOME environment variable to point to your Rust installation.');
    process.exit(1);
  }

  // Get arguments (remove 'node' and script name)
  const args = process.argv.slice(2);

  // Spawn the Rust CLI with all arguments
  const child = spawn(cliPath, args, {
    stdio: 'inherit', // Forward stdin, stdout, stderr
    env: process.env,
  });

  // Handle process exit
  child.on('exit', (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
    } else {
      process.exit(code ?? 0);
    }
  });

  // Handle errors
  child.on('error', (err) => {
    console.error(`Failed to start CLI: ${err.message}`);
    process.exit(1);
  });

  // Handle signals to properly forward them to child process
  const signals: NodeJS.Signals[] = ['SIGINT', 'SIGTERM', 'SIGHUP'];
  signals.forEach((signal) => {
    process.on(signal, () => {
      child.kill(signal);
    });
  });
}

// Run the CLI
main().catch((err) => {
  console.error('Unexpected error:', err);
  process.exit(1);
});
