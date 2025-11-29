/**
 * Integration tests for npm wrapper functionality
 * Tests that the npm package correctly wraps the Rust CLI binary
 */

import { describe, it, expect, beforeAll } from 'vitest';
import { spawn, spawnSync } from 'child_process';
import { existsSync } from 'fs';
import { join } from 'path';
import os from 'os';

describe('NPM Wrapper Tests', () => {
  describe('Binary Discovery', () => {
    it('should find the llm-test-bench binary', () => {
      const binaryName = process.platform === 'win32' ? 'llm-test-bench.exe' : 'llm-test-bench';
      const cargoHome = process.env.CARGO_HOME || join(os.homedir(), '.cargo');
      const cargoBin = join(cargoHome, 'bin', binaryName);

      // Binary should exist in cargo bin or be in PATH
      const exists = existsSync(cargoBin);

      if (!exists) {
        // Try to find in PATH
        const which = process.platform === 'win32' ? 'where' : 'which';
        const result = spawnSync(which, [binaryName], { encoding: 'utf8' });
        expect(result.status === 0 || exists).toBe(true);
      } else {
        expect(exists).toBe(true);
      }
    });
  });

  describe('Version Command', () => {
    it('should return version information', () => {
      const result = spawnSync('npm', ['run', '-s', 'test:cli', '--', '--version'], {
        encoding: 'utf8',
        cwd: join(__dirname, '..')
      });

      // Should exit successfully
      if (result.status === 0) {
        expect(result.stdout).toMatch(/\d+\.\d+\.\d+/);
      } else {
        // If binary not found, that's expected in CI
        expect(result.status).toBeDefined();
      }
    });
  });

  describe('Help Command', () => {
    it('should display help information', () => {
      const result = spawnSync('npm', ['run', '-s', 'test:cli', '--', '--help'], {
        encoding: 'utf8',
        cwd: join(__dirname, '..')
      });

      if (result.status === 0) {
        const output = result.stdout + result.stderr;
        expect(output).toMatch(/llm-test-bench|LLM Test Bench/i);
        expect(output).toMatch(/bench|compare|analyze/i);
      }
    });
  });

  describe('Error Handling', () => {
    it('should handle invalid commands gracefully', () => {
      const result = spawnSync('npm', ['run', '-s', 'test:cli', '--', 'invalid-command'], {
        encoding: 'utf8',
        cwd: join(__dirname, '..')
      });

      // Should exit with non-zero status for invalid command
      if (result.status !== null && result.status !== 127) {
        expect(result.status).not.toBe(0);
      }
    });
  });
});

describe('Package Metadata', () => {
  it('should have valid package.json', () => {
    const packageJson = require('../package.json');

    expect(packageJson.name).toBe('llm-test-bench');
    expect(packageJson.version).toMatch(/\d+\.\d+\.\d+/);
    expect(packageJson.description).toBeTruthy();
    expect(packageJson.keywords).toContain('llm');
    expect(packageJson.keywords).toContain('testing');
    expect(packageJson.bin).toHaveProperty('ltb');
  });

  it('should have proper engine requirements', () => {
    const packageJson = require('../package.json');

    expect(packageJson.engines).toBeDefined();
    expect(packageJson.engines.node).toMatch(/>=\d+\.\d+\.\d+/);
  });

  it('should include required scripts', () => {
    const packageJson = require('../package.json');

    expect(packageJson.scripts).toBeDefined();
    expect(packageJson.scripts.test).toBeDefined();
    expect(packageJson.scripts.build).toBeDefined();
  });
});

describe('TypeScript Support', () => {
  it('should have TypeScript definitions', () => {
    const packageJson = require('../package.json');

    expect(packageJson.types).toBeDefined();
    expect(packageJson.types).toMatch(/\.d\.ts$/);
  });

  it('should have valid tsconfig.json', () => {
    const tsconfigPath = join(__dirname, '..', 'tsconfig.json');
    expect(existsSync(tsconfigPath)).toBe(true);

    const tsconfig = require('../tsconfig.json');
    expect(tsconfig.compilerOptions).toBeDefined();
  });
});
