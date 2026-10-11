/**
 * CLI Executor utility
 *
 * Handles execution of the LLM Test Bench CLI and parsing of results.
 */

import { spawn } from 'child_process';
import { accessSync, constants } from 'fs';
import { delimiter, join } from 'path';

import { CLIResult } from '../types/index.js';

/**
 * Options for CLI execution
 */
export interface ExecuteOptions {
  /** Command arguments */
  args: string[];

  /** Working directory */
  cwd?: string;

  /** Environment variables */
  env?: Record<string, string>;

  /** Timeout in milliseconds */
  timeout?: number;

  /** Whether to parse JSON output */
  parseJson?: boolean;
}

/**
 * Execute a CLI command and return the result
 *
 * @param cliPath - Path to the CLI binary
 * @param options - Execution options
 * @returns Promise resolving to CLI result
 */
export async function executeCLI<T = unknown>(
  cliPath: string,
  options: ExecuteOptions,
): Promise<CLIResult<T>> {
  const startTime = Date.now();

  return new Promise((resolve) => {
    let stdout = '';
    let stderr = '';
    let timedOut = false;

    const child = spawn(cliPath, options.args, {
      cwd: options.cwd ?? process.cwd(),
      env: {
        ...process.env,
        ...options.env,
      },
    });

    // Set timeout if specified
    const timeoutId = options.timeout
      ? setTimeout(() => {
          timedOut = true;
          child.kill('SIGTERM');
        }, options.timeout)
      : undefined;

    // Collect stdout
    child.stdout?.on('data', (data: Buffer) => {
      stdout += data.toString();
    });

    // Collect stderr
    child.stderr?.on('data', (data: Buffer) => {
      stderr += data.toString();
    });

    // Handle process completion
    child.on('close', (exitCode) => {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }

      const durationMs = Date.now() - startTime;

      if (timedOut) {
        resolve({
          success: false,
          exitCode: -1,
          stdout,
          stderr,
          durationMs,
          error: `Command timed out after ${options.timeout}ms`,
        });
        return;
      }

      const success = exitCode === 0;
      let data: T | undefined;
      let error: string | undefined;

      // Parse JSON output if requested and successful
      if (success && options.parseJson && stdout.trim()) {
        try {
          data = JSON.parse(stdout) as T;
        } catch (e) {
          error = `Failed to parse JSON output: ${e instanceof Error ? e.message : String(e)}`;
        }
      }

      // Extract error from stderr if failed
      if (!success && stderr) {
        error = stderr.trim();
      }

      resolve({
        success,
        data,
        error,
        exitCode: exitCode ?? -1,
        stdout,
        stderr,
        durationMs,
      });
    });

    // Handle spawn errors
    child.on('error', (err) => {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }

      resolve({
        success: false,
        exitCode: -1,
        stdout,
        stderr,
        durationMs: Date.now() - startTime,
        error: `Failed to spawn process: ${err.message}`,
      });
    });
  });
}

/**
 * Find the CLI binary path
 *
 * @returns Path to the CLI binary or undefined if not found
 */
export function findCLIPath(cwd = process.cwd()): string | undefined {
  const binaryName = process.platform === 'win32' ? 'llm-test-bench.exe' : 'llm-test-bench';
  const cargoHome = process.env['CARGO_HOME'];
  const home = process.env['HOME'];

  const cargoPath = cargoHome
    ? join(cargoHome, 'bin', binaryName)
    : home
      ? join(home, '.cargo', 'bin', binaryName)
      : undefined;

  const npmGlobalPath = join('/usr/local/bin', binaryName);
  const npmLocalPath = join(cwd, 'node_modules', '.bin', binaryName);
  const pathCandidates = (process.env['PATH'] ?? '')
    .split(delimiter)
    .filter(Boolean)
    .map((directory) => join(directory, binaryName));

  const paths = [cargoPath, npmLocalPath, npmGlobalPath, ...pathCandidates].filter(
    (p): p is string => p !== undefined,
  );

  const accessMode = process.platform === 'win32' ? constants.F_OK : constants.X_OK;

  for (const candidate of new Set(paths)) {
    try {
      accessSync(candidate, accessMode);
      return candidate;
    } catch {
      // Keep searching: configured locations may legitimately be absent.
    }
  }

  return undefined;
}
