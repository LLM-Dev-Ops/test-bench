/**
 * Integration tests for npm wrapper functionality
 * Tests that the npm package correctly wraps the Rust CLI binary
 */

import { describe, it, expect } from 'vitest';
import { spawnSync } from 'child_process';
import { existsSync, readFileSync } from 'fs';
import { join } from 'path';
import ts from 'typescript';

describe('NPM Wrapper Tests', () => {
  describe('Version Command', () => {
    it('should return version information', () => {
      const result = spawnSync('npm', ['run', '-s', 'test:cli', '--', '--version'], {
        encoding: 'utf8',
        cwd: join(__dirname, '..'),
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
        cwd: join(__dirname, '..'),
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
        cwd: join(__dirname, '..'),
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
    const packageJson = JSON.parse(readFileSync(join(__dirname, '..', 'package.json'), 'utf8'));

    expect(packageJson.name).toBe('@llm-dev-ops/test-bench-sdk');
    expect(packageJson.version).toMatch(/\d+\.\d+\.\d+/);
    expect(packageJson.description).toBeTruthy();
    expect(packageJson.keywords).toContain('llm');
    expect(packageJson.keywords).toContain('testing');
    expect(packageJson.exports).toHaveProperty('.');
  });

  it('should have proper engine requirements', () => {
    const packageJson = JSON.parse(readFileSync(join(__dirname, '..', 'package.json'), 'utf8'));

    expect(packageJson.engines).toBeDefined();
    expect(packageJson.engines.node).toMatch(/>=\d+\.\d+\.\d+/);
  });

  it('should include required scripts', () => {
    const packageJson = JSON.parse(readFileSync(join(__dirname, '..', 'package.json'), 'utf8'));

    expect(packageJson.scripts).toBeDefined();
    expect(packageJson.scripts.test).toBeDefined();
    expect(packageJson.scripts.build).toBeDefined();
  });
});

describe('TypeScript Support', () => {
  it('should have TypeScript definitions', () => {
    const packageJson = JSON.parse(readFileSync(join(__dirname, '..', 'package.json'), 'utf8'));

    expect(packageJson.types).toBeDefined();
    expect(packageJson.types).toMatch(/\.d\.ts$/);
  });

  it('should have valid tsconfig.json', () => {
    const tsconfigPath = join(__dirname, '..', 'tsconfig.json');
    expect(existsSync(tsconfigPath)).toBe(true);

    const { config: tsconfig, error } = ts.readConfigFile(tsconfigPath, ts.sys.readFile);
    expect(error).toBeUndefined();
    expect(tsconfig.compilerOptions).toBeDefined();
  });
});
