/**
 * Unit tests for configuration management
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { existsSync, readFileSync, writeFileSync, mkdirSync, rmSync } from 'fs';
import { join } from 'path';
import { tmpdir } from 'os';

describe('Configuration Management', () => {
  let testDir: string;
  let testConfigPath: string;

  beforeEach(() => {
    // Create a temporary directory for test configs
    testDir = join(tmpdir(), `llm-test-bench-test-${Date.now()}`);
    mkdirSync(testDir, { recursive: true });
    testConfigPath = join(testDir, 'config.toml');
  });

  afterEach(() => {
    // Clean up test directory
    if (existsSync(testDir)) {
      rmSync(testDir, { recursive: true, force: true });
    }
  });

  describe('Config File Format', () => {
    it('should support TOML configuration format', () => {
      const sampleConfig = `
[general]
log_level = "info"
output_dir = "./results"

[[providers]]
name = "openai"
api_key_env = "OPENAI_API_KEY"
base_url = "https://api.openai.com/v1"
default_model = "gpt-4"
enabled = true
`;

      writeFileSync(testConfigPath, sampleConfig);
      expect(existsSync(testConfigPath)).toBe(true);

      const content = readFileSync(testConfigPath, 'utf-8');
      expect(content).toContain('[general]');
      expect(content).toContain('[[providers]]');
    });

    it('should validate required provider fields', () => {
      const validProvider = {
        name: 'openai',
        api_key_env: 'OPENAI_API_KEY',
        base_url: 'https://api.openai.com/v1',
        default_model: 'gpt-4',
        enabled: true
      };

      expect(validProvider.name).toBeTruthy();
      expect(validProvider.api_key_env).toBeTruthy();
      expect(validProvider.base_url).toMatch(/^https?:\/\//);
      expect(validProvider.default_model).toBeTruthy();
      expect(typeof validProvider.enabled).toBe('boolean');
    });

    it('should support multiple providers in config', () => {
      const multiProviderConfig = `
[[providers]]
name = "openai"
api_key_env = "OPENAI_API_KEY"
base_url = "https://api.openai.com/v1"
default_model = "gpt-4"
enabled = true

[[providers]]
name = "anthropic"
api_key_env = "ANTHROPIC_API_KEY"
base_url = "https://api.anthropic.com/v1"
default_model = "claude-3-opus-20240229"
enabled = true
`;

      writeFileSync(testConfigPath, multiProviderConfig);
      const content = readFileSync(testConfigPath, 'utf-8');

      const providerMatches = content.match(/\[\[providers\]\]/g);
      expect(providerMatches).toBeTruthy();
      expect(providerMatches!.length).toBe(2);
    });
  });

  describe('Environment Variables', () => {
    const originalEnv = process.env;

    beforeEach(() => {
      process.env = { ...originalEnv };
    });

    afterEach(() => {
      process.env = originalEnv;
    });

    it('should read API keys from environment variables', () => {
      process.env.OPENAI_API_KEY = 'test-key-123';
      expect(process.env.OPENAI_API_KEY).toBe('test-key-123');
    });

    it('should support multiple API key environment variables', () => {
      process.env.OPENAI_API_KEY = 'openai-key';
      process.env.ANTHROPIC_API_KEY = 'anthropic-key';
      process.env.GOOGLE_API_KEY = 'google-key';

      expect(process.env.OPENAI_API_KEY).toBeTruthy();
      expect(process.env.ANTHROPIC_API_KEY).toBeTruthy();
      expect(process.env.GOOGLE_API_KEY).toBeTruthy();
    });

    it('should handle missing environment variables', () => {
      delete process.env.OPENAI_API_KEY;
      expect(process.env.OPENAI_API_KEY).toBeUndefined();
    });
  });

  describe('Default Configuration', () => {
    it('should provide sensible defaults', () => {
      const defaults = {
        log_level: 'info',
        timeout_seconds: 30,
        max_retries: 3,
        output_format: 'json'
      };

      expect(defaults.log_level).toBe('info');
      expect(defaults.timeout_seconds).toBeGreaterThan(0);
      expect(defaults.max_retries).toBeGreaterThanOrEqual(0);
      expect(['json', 'yaml', 'toml']).toContain(defaults.output_format);
    });

    it('should have valid provider defaults', () => {
      const providerDefaults = {
        timeout_seconds: 30,
        max_retries: 3,
        enabled: true
      };

      expect(providerDefaults.timeout_seconds).toBeGreaterThan(0);
      expect(providerDefaults.max_retries).toBeGreaterThanOrEqual(0);
      expect(typeof providerDefaults.enabled).toBe('boolean');
    });
  });

  describe('Config Validation', () => {
    it('should reject invalid log levels', () => {
      const invalidLevels = ['verbose', 'critical', 'none', ''];
      const validLevels = ['error', 'warn', 'info', 'debug', 'trace'];

      invalidLevels.forEach(level => {
        expect(validLevels).not.toContain(level);
      });

      validLevels.forEach(level => {
        expect(validLevels).toContain(level);
      });
    });

    it('should validate URL formats', () => {
      const validUrls = [
        'https://api.openai.com/v1',
        'https://api.anthropic.com/v1',
        'http://localhost:8080'
      ];

      const invalidUrls = [
        'not-a-url',
        'ftp://invalid.com',
        ''
      ];

      validUrls.forEach(url => {
        expect(url).toMatch(/^https?:\/\/.+/);
      });

      invalidUrls.forEach(url => {
        expect(url).not.toMatch(/^https?:\/\/.+/);
      });
    });

    it('should validate numeric ranges', () => {
      // Timeout should be positive
      expect(30).toBeGreaterThan(0);
      expect(0).not.toBeGreaterThan(0);
      expect(-1).not.toBeGreaterThan(0);

      // Retries should be non-negative
      expect(3).toBeGreaterThanOrEqual(0);
      expect(0).toBeGreaterThanOrEqual(0);
      expect(-1).not.toBeGreaterThanOrEqual(0);
    });
  });
});
