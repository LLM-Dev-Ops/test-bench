# LLM Test Bench - System Architecture

**Version:** 1.0
**Date:** 2025-11-04
**Status:** Design Blueprint

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [System Overview](#system-overview)
3. [Architectural Principles](#architectural-principles)
4. [Component Architecture](#component-architecture)
5. [Data Flow](#data-flow)
6. [Interface Specifications](#interface-specifications)
7. [Security Architecture](#security-architecture)
8. [Scalability & Extensibility](#scalability--extensibility)
9. [Error Handling Strategy](#error-handling-strategy)
10. [Technology Stack](#technology-stack)

---

## Executive Summary

The LLM Test Bench is a production-grade CLI framework designed to enable systematic testing, validation, and benchmarking of Large Language Model (LLM) applications. The architecture follows clean architecture principles with clear separation of concerns, dependency inversion, and plugin-based extensibility.

**Key Design Goals:**
- Provider-agnostic: Support any LLM provider through abstraction
- Extensible: Plugin architecture for assertions, reporters, and providers
- Developer-friendly: Intuitive CLI with excellent DX
- Production-ready: Robust error handling, retry logic, and observability
- Performance-focused: Parallel execution, caching, and streaming support

---

## System Overview

### High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI INTERFACE                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Commands   │  │  Interactive │  │   Output     │          │
│  │   Parser     │  │     Mode     │  │  Formatter   │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                    CONFIGURATION SYSTEM                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Loader     │  │  Validator   │  │   Secrets    │          │
│  │              │  │   (Schema)   │  │   Manager    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                       CORE ENGINE                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │     Test     │  │  Execution   │  │    State     │          │
│  │   Discovery  │  │ Orchestrator │  │   Manager    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  ┌──────────────┐  ┌──────────────┐                            │
│  │    Cache     │  │ Parallelizer │                            │
│  │   Manager    │  │              │                            │
│  └──────────────┘  └──────────────┘                            │
└───────┬─────────────────────┬─────────────────────┬────────────┘
        │                     │                     │
┌───────▼────────┐  ┌─────────▼────────┐  ┌────────▼───────────┐
│   PROVIDER     │  │   ASSERTION      │  │     REPORTING      │
│   ABSTRACTION  │  │     ENGINE       │  │      SYSTEM        │
│                │  │                  │  │                    │
│  ┌──────────┐  │  │  ┌────────────┐  │  │  ┌──────────────┐ │
│  │ Provider │  │  │  │ Built-in   │  │  │  │  Formatters  │ │
│  │ Registry │  │  │  │ Assertions │  │  │  │ (JSON/HTML)  │ │
│  └──────────┘  │  │  └────────────┘  │  │  └──────────────┘ │
│  ┌──────────┐  │  │  ┌────────────┐  │  │  ┌──────────────┐ │
│  │  OpenAI  │  │  │  │   Custom   │  │  │  │   Metrics    │ │
│  │ Adapter  │  │  │  │   Plugins  │  │  │  │  Aggregator  │ │
│  └──────────┘  │  │  └────────────┘  │  │  └──────────────┘ │
│  ┌──────────┐  │  │  ┌────────────┐  │  │  ┌──────────────┐ │
│  │Anthropic │  │  │  │  Scorer    │  │  │  │  Historical  │ │
│  │ Adapter  │  │  │  │            │  │  │  │  Comparison  │ │
│  └──────────┘  │  │  └────────────┘  │  │  └──────────────┘ │
│  ┌──────────┐  │  │                  │  │                    │
│  │  Local   │  │  │                  │  │                    │
│  │  Models  │  │  │                  │  │                    │
│  └──────────┘  │  │                  │  │                    │
└────────────────┘  └──────────────────┘  └────────────────────┘
```

---

## Architectural Principles

### 1. Clean Architecture
- **Dependency Rule**: Dependencies point inward (CLI → Core → Domain)
- **Domain Independence**: Core business logic independent of frameworks
- **Interface Segregation**: Small, focused interfaces

### 2. Plugin-Based Extensibility
- Providers, assertions, and reporters are plugins
- Hot-loadable custom plugins
- Well-defined plugin contracts

### 3. Composition over Inheritance
- Favor composition for flexibility
- Use interfaces/protocols for polymorphism

### 4. Fail-Fast with Graceful Degradation
- Validate early, fail fast during configuration
- Graceful handling of runtime errors with retries

### 5. Observability First
- Comprehensive logging at all levels
- Metrics collection for performance analysis
- Trace IDs for distributed debugging

---

## Component Architecture

### 1. CLI INTERFACE

#### 1.1 Command Structure

```
ltb (root command)
├── init              - Initialize new test suite
├── run               - Execute tests
│   ├── --suite       - Run specific suite
│   ├── --test        - Run specific test(s)
│   ├── --filter      - Filter by tags/patterns
│   ├── --parallel    - Parallel execution count
│   ├── --watch       - Watch mode
│   └── --debug       - Debug mode
├── validate          - Validate configuration
├── list              - List available tests/suites
├── report            - Generate/view reports
│   ├── --format      - Output format
│   ├── --output      - Output file
│   └── --compare     - Compare with historical
├── cache             - Cache management
│   ├── clear         - Clear cache
│   └── info          - Cache statistics
├── providers         - Provider management
│   ├── list          - List providers
│   ├── test          - Test provider connection
│   └── add           - Add provider configuration
└── config            - Configuration management
    ├── show          - Display configuration
    ├── set           - Set configuration value
    └── validate      - Validate configuration
```

#### 1.2 Argument Parsing Strategy

**Technology**: `yargs` for Node.js (rich parsing, validation, help generation)

**Design Patterns**:
- **Builder Pattern**: For complex command construction
- **Command Pattern**: Each command is encapsulated object

**Features**:
- Type validation at parse time
- Auto-generated help text
- Shell completion support
- Environment variable overrides
- Configuration file integration

#### 1.3 Interactive vs Batch Modes

**Interactive Mode** (when no args or with --interactive):
- Guided prompts using `inquirer.js`
- Progressive disclosure
- Validation feedback
- Test selection via fuzzy search

**Batch Mode** (with arguments):
- Scriptable, CI/CD friendly
- JSON/structured output
- Exit codes for pipeline integration
- Silent mode for automation

#### 1.4 Output Formatting

**Format Options**:
- `text` - Human-readable, colorized (default for TTY)
- `json` - Structured, machine-readable (default for non-TTY)
- `junit` - JUnit XML for CI integration
- `tap` - Test Anything Protocol
- `markdown` - Documentation-friendly

**Implementation**:
```typescript
interface OutputFormatter {
  formatTestStart(test: Test): string;
  formatTestResult(result: TestResult): string;
  formatSummary(summary: TestSummary): string;
  formatError(error: Error): string;
}

class FormatterRegistry {
  private formatters: Map<string, OutputFormatter>;

  register(name: string, formatter: OutputFormatter): void;
  get(name: string): OutputFormatter;
  supports(name: string): boolean;
}
```

#### 1.5 Error Handling & User Feedback

**Principles**:
- Clear, actionable error messages
- Suggest fixes when possible
- Stack traces in debug mode only
- Error codes for programmatic handling

**Error Categories**:
- Configuration errors (E1xx)
- Provider errors (E2xx)
- Assertion errors (E3xx)
- System errors (E4xx)

---

### 2. CONFIGURATION SYSTEM

#### 2.1 File Format

**Primary Format**: YAML (human-friendly, comments, anchors)
**Alternative Formats**: TOML, JSON (auto-detected)

**Configuration File Hierarchy**:
```
1. /etc/ltb/config.yaml           (system-wide)
2. ~/.ltb/config.yaml              (user)
3. ./.ltb/config.yaml              (project)
4. ./ltb.config.yaml               (project root)
5. Environment variables           (LTB_*)
6. CLI arguments                   (highest priority)
```

#### 2.2 Configuration Schema

```yaml
# ltb.config.yaml

# Metadata
version: "1.0"
name: "My Test Suite"
description: "E2E tests for chatbot"

# Global settings
defaults:
  provider: "openai"
  timeout: 30000  # ms
  retries: 3
  parallel: 4

# Provider configurations
providers:
  openai:
    type: "openai"
    apiKey: "${env:OPENAI_API_KEY}"  # Environment variable
    organization: "org-123"
    defaults:
      model: "gpt-4"
      temperature: 0.7
      maxTokens: 1000

  anthropic:
    type: "anthropic"
    apiKey: "${vault:anthropic-key}"  # Vault reference
    defaults:
      model: "claude-sonnet-4"
      maxTokens: 4096

  local:
    type: "ollama"
    baseUrl: "http://localhost:11434"
    defaults:
      model: "llama2"

# Test discovery
tests:
  include:
    - "tests/**/*.test.yaml"
    - "tests/**/*.test.json"
  exclude:
    - "tests/wip/**"
  tags:
    - "smoke"
    - "regression"

# Assertion configuration
assertions:
  plugins:
    - "./plugins/custom-assertions.js"
  semantic:
    provider: "openai"
    model: "text-embedding-ada-002"
    threshold: 0.85

# Reporting
reporting:
  formats:
    - "json"
    - "html"
  output: "./reports"
  historical:
    enabled: true
    storage: "./reports/history"
    retention: 30  # days

# Caching
cache:
  enabled: true
  directory: "./.ltb/cache"
  ttl: 3600  # seconds
  strategy: "content-hash"

# Advanced
advanced:
  logging:
    level: "info"  # debug, info, warn, error
    file: "./logs/ltb.log"
  telemetry:
    enabled: false
  hooks:
    before: "./scripts/setup.sh"
    after: "./scripts/teardown.sh"
```

#### 2.3 Configuration Loader

```typescript
interface ConfigLoader {
  load(paths?: string[]): Promise<Config>;
  merge(...configs: Config[]): Config;
  validate(config: Config): ValidationResult;
}

class HierarchicalConfigLoader implements ConfigLoader {
  private validators: SchemaValidator[];
  private parsers: Map<string, ConfigParser>;

  async load(paths?: string[]): Promise<Config> {
    const configs = await this.loadAll(paths || this.getDefaultPaths());
    const merged = this.merge(...configs);
    const validated = this.validate(merged);

    if (!validated.valid) {
      throw new ConfigurationError(validated.errors);
    }

    return this.resolveReferences(merged);
  }

  private async resolveReferences(config: Config): Promise<Config> {
    // Resolve ${env:VAR}, ${vault:key}, ${file:path}
  }
}
```

#### 2.4 Environment Variable Support

**Convention**: `LTB_<SECTION>_<KEY>`

Examples:
```bash
LTB_PROVIDERS_OPENAI_API_KEY=sk-...
LTB_DEFAULTS_PARALLEL=8
LTB_REPORTING_OUTPUT=./custom-reports
```

**Deep Merge Strategy**: Environment variables override file configuration

#### 2.5 Secrets Management

**Supported Sources**:
1. Environment variables (direct)
2. File-based secrets (`.env` files)
3. Vault integration (HashiCorp Vault, AWS Secrets Manager)
4. Keychain/credential store (OS-level)

**Security**:
- Secrets never logged
- In-memory only, never written to disk
- Masked in error messages
- Optional encryption at rest

```typescript
interface SecretProvider {
  get(key: string): Promise<string>;
  set(key: string, value: string): Promise<void>;
  delete(key: string): Promise<void>;
}

class VaultSecretProvider implements SecretProvider {
  constructor(private vaultUrl: string, private token: string) {}

  async get(key: string): Promise<string> {
    // Fetch from HashiCorp Vault
  }
}

class SecretsManager {
  private providers: Map<string, SecretProvider>;

  registerProvider(name: string, provider: SecretProvider): void;
  async resolve(reference: string): Promise<string>;
}
```

#### 2.6 Schema Validation

**Technology**: JSON Schema / Zod for runtime validation

**Features**:
- Type checking
- Required field validation
- Format validation (URLs, emails, etc.)
- Custom validators
- Helpful error messages with path to error

```typescript
import { z } from 'zod';

const ProviderConfigSchema = z.object({
  type: z.enum(['openai', 'anthropic', 'ollama', 'custom']),
  apiKey: z.string().optional(),
  baseUrl: z.string().url().optional(),
  defaults: z.object({
    model: z.string(),
    temperature: z.number().min(0).max(2).optional(),
    maxTokens: z.number().positive().optional(),
  }),
});

const ConfigSchema = z.object({
  version: z.string(),
  name: z.string(),
  providers: z.record(ProviderConfigSchema),
  tests: z.object({
    include: z.array(z.string()),
    exclude: z.array(z.string()).optional(),
  }),
  // ... more fields
});

type Config = z.infer<typeof ConfigSchema>;
```

---

### 3. PROVIDER ABSTRACTION

#### 3.1 Multi-LLM Provider Support

**Design Philosophy**:
- Abstract common operations
- Provider-specific features via extensions
- Zero-cost abstractions (no performance penalty)

#### 3.2 Provider Interface Design

```typescript
// Core provider interface
interface LLMProvider {
  readonly name: string;
  readonly version: string;

  // Lifecycle
  initialize(config: ProviderConfig): Promise<void>;
  validate(): Promise<ValidationResult>;
  shutdown(): Promise<void>;

  // Core operations
  complete(request: CompletionRequest): Promise<CompletionResponse>;
  stream(request: CompletionRequest): AsyncIterableIterator<CompletionChunk>;

  // Capabilities
  supports(feature: ProviderFeature): boolean;
  getModels(): Promise<ModelInfo[]>;

  // Observability
  getMetrics(): ProviderMetrics;
}

// Request/Response types
interface CompletionRequest {
  model: string;
  messages: Message[];
  temperature?: number;
  maxTokens?: number;
  stopSequences?: string[];
  metadata?: Record<string, unknown>;

  // Provider-specific extensions
  extensions?: Record<string, unknown>;
}

interface CompletionResponse {
  id: string;
  model: string;
  content: string;
  finishReason: 'stop' | 'length' | 'error';
  usage: {
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  };
  metadata?: Record<string, unknown>;
}

interface Message {
  role: 'system' | 'user' | 'assistant';
  content: string;
  name?: string;
}

// Provider features enumeration
enum ProviderFeature {
  STREAMING = 'streaming',
  FUNCTION_CALLING = 'function_calling',
  VISION = 'vision',
  JSON_MODE = 'json_mode',
  EMBEDDINGS = 'embeddings',
}
```

#### 3.3 Provider Registry

```typescript
class ProviderRegistry {
  private providers: Map<string, ProviderFactory>;
  private instances: Map<string, LLMProvider>;

  register(name: string, factory: ProviderFactory): void {
    this.providers.set(name, factory);
  }

  async create(name: string, config: ProviderConfig): Promise<LLMProvider> {
    const factory = this.providers.get(name);
    if (!factory) {
      throw new Error(`Provider '${name}' not found`);
    }

    const instance = await factory.create(config);
    await instance.initialize(config);

    this.instances.set(name, instance);
    return instance;
  }

  get(name: string): LLMProvider | undefined {
    return this.instances.get(name);
  }

  list(): string[] {
    return Array.from(this.providers.keys());
  }
}

// Built-in providers
registry.register('openai', new OpenAIProviderFactory());
registry.register('anthropic', new AnthropicProviderFactory());
registry.register('ollama', new OllamaProviderFactory());
```

#### 3.4 Provider Adapters

**OpenAI Adapter**:
```typescript
class OpenAIProvider implements LLMProvider {
  private client: OpenAI;
  private config: OpenAIConfig;

  async complete(request: CompletionRequest): Promise<CompletionResponse> {
    const response = await this.client.chat.completions.create({
      model: request.model,
      messages: request.messages,
      temperature: request.temperature,
      max_tokens: request.maxTokens,
    });

    return this.adaptResponse(response);
  }

  async *stream(request: CompletionRequest): AsyncIterableIterator<CompletionChunk> {
    const stream = await this.client.chat.completions.create({
      model: request.model,
      messages: request.messages,
      stream: true,
    });

    for await (const chunk of stream) {
      yield this.adaptChunk(chunk);
    }
  }

  private adaptResponse(response: OpenAI.ChatCompletion): CompletionResponse {
    // Transform OpenAI response to standard format
  }
}
```

**Anthropic Adapter**:
```typescript
class AnthropicProvider implements LLMProvider {
  private client: Anthropic;

  async complete(request: CompletionRequest): Promise<CompletionResponse> {
    // Convert messages format (OpenAI-style to Anthropic-style)
    const { system, messages } = this.convertMessages(request.messages);

    const response = await this.client.messages.create({
      model: request.model,
      max_tokens: request.maxTokens || 4096,
      system,
      messages,
    });

    return this.adaptResponse(response);
  }

  private convertMessages(messages: Message[]): { system?: string; messages: any[] } {
    // Handle system message extraction
  }
}
```

#### 3.5 Authentication Handling

```typescript
interface AuthStrategy {
  authenticate(request: Request): Promise<Request>;
  refresh(): Promise<void>;
  isValid(): boolean;
}

class APIKeyAuthStrategy implements AuthStrategy {
  constructor(private apiKey: string, private header: string = 'Authorization') {}

  async authenticate(request: Request): Promise<Request> {
    request.headers[this.header] = `Bearer ${this.apiKey}`;
    return request;
  }
}

class OAuth2AuthStrategy implements AuthStrategy {
  private accessToken?: string;
  private refreshToken?: string;
  private expiresAt?: Date;

  async authenticate(request: Request): Promise<Request> {
    if (!this.isValid()) {
      await this.refresh();
    }
    request.headers['Authorization'] = `Bearer ${this.accessToken}`;
    return request;
  }

  async refresh(): Promise<void> {
    // OAuth2 token refresh logic
  }

  isValid(): boolean {
    return this.accessToken != null &&
           this.expiresAt != null &&
           this.expiresAt > new Date();
  }
}
```

#### 3.6 Rate Limiting & Retry Logic

```typescript
interface RateLimiter {
  acquire(tokens?: number): Promise<void>;
  release(tokens?: number): void;
  getStatus(): RateLimitStatus;
}

class TokenBucketRateLimiter implements RateLimiter {
  private tokens: number;
  private lastRefill: Date;

  constructor(
    private capacity: number,
    private refillRate: number,  // tokens per second
  ) {
    this.tokens = capacity;
    this.lastRefill = new Date();
  }

  async acquire(tokens: number = 1): Promise<void> {
    this.refill();

    while (this.tokens < tokens) {
      await this.wait(this.calculateWaitTime(tokens));
      this.refill();
    }

    this.tokens -= tokens;
  }

  private refill(): void {
    const now = new Date();
    const elapsed = (now.getTime() - this.lastRefill.getTime()) / 1000;
    const newTokens = elapsed * this.refillRate;

    this.tokens = Math.min(this.capacity, this.tokens + newTokens);
    this.lastRefill = now;
  }
}

class RetryStrategy {
  constructor(
    private maxRetries: number = 3,
    private baseDelay: number = 1000,
    private maxDelay: number = 30000,
  ) {}

  async execute<T>(
    operation: () => Promise<T>,
    shouldRetry?: (error: Error) => boolean,
  ): Promise<T> {
    let lastError: Error;

    for (let attempt = 0; attempt <= this.maxRetries; attempt++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error as Error;

        if (attempt === this.maxRetries ||
            (shouldRetry && !shouldRetry(error))) {
          throw error;
        }

        const delay = this.calculateDelay(attempt);
        await this.wait(delay);
      }
    }

    throw lastError!;
  }

  private calculateDelay(attempt: number): number {
    // Exponential backoff with jitter
    const exponential = this.baseDelay * Math.pow(2, attempt);
    const jitter = Math.random() * 0.3 * exponential;
    return Math.min(this.maxDelay, exponential + jitter);
  }
}

// Provider with rate limiting and retries
class ResilientProvider implements LLMProvider {
  constructor(
    private provider: LLMProvider,
    private rateLimiter: RateLimiter,
    private retryStrategy: RetryStrategy,
  ) {}

  async complete(request: CompletionRequest): Promise<CompletionResponse> {
    await this.rateLimiter.acquire();

    return this.retryStrategy.execute(
      () => this.provider.complete(request),
      (error) => this.isRetryable(error),
    );
  }

  private isRetryable(error: Error): boolean {
    // Retry on rate limits, timeouts, 5xx errors
    return error.name === 'RateLimitError' ||
           error.name === 'TimeoutError' ||
           error.name === 'ServiceUnavailableError';
  }
}
```

#### 3.7 Streaming vs Non-Streaming Responses

```typescript
// Unified interface for both modes
interface ProviderClient {
  complete(request: CompletionRequest): Promise<CompletionResponse>;
  stream(request: CompletionRequest): AsyncIterableIterator<CompletionChunk>;
}

// Usage in tests
class TestExecutor {
  async executeTest(test: Test, provider: LLMProvider): Promise<TestResult> {
    if (test.streaming) {
      return this.executeStreamingTest(test, provider);
    } else {
      return this.executeStandardTest(test, provider);
    }
  }

  private async executeStreamingTest(
    test: Test,
    provider: LLMProvider,
  ): Promise<TestResult> {
    const chunks: string[] = [];

    for await (const chunk of provider.stream(test.request)) {
      chunks.push(chunk.content);

      // Optional: stream assertions
      if (test.streamAssertions) {
        await this.assertChunk(chunk, test.streamAssertions);
      }
    }

    const fullResponse = chunks.join('');
    return this.evaluate(fullResponse, test.assertions);
  }
}
```

---

### 4. ASSERTION ENGINE

#### 4.1 Assertion Types

```typescript
// Base assertion interface
interface Assertion {
  readonly type: string;
  readonly name?: string;

  evaluate(response: CompletionResponse, context: TestContext): Promise<AssertionResult>;
}

interface AssertionResult {
  passed: boolean;
  score?: number;  // 0-1 for partial matches
  message: string;
  details?: Record<string, unknown>;
}

// Built-in assertion types

// 1. Exact Match
class ExactMatchAssertion implements Assertion {
  readonly type = 'exact';

  constructor(private expected: string, private caseSensitive: boolean = true) {}

  async evaluate(response: CompletionResponse): Promise<AssertionResult> {
    const actual = response.content;
    const expected = this.expected;

    const matches = this.caseSensitive
      ? actual === expected
      : actual.toLowerCase() === expected.toLowerCase();

    return {
      passed: matches,
      message: matches
        ? 'Exact match succeeded'
        : `Expected: "${expected}", got: "${actual}"`,
    };
  }
}

// 2. Contains
class ContainsAssertion implements Assertion {
  readonly type = 'contains';

  constructor(private substring: string, private caseSensitive: boolean = false) {}

  async evaluate(response: CompletionResponse): Promise<AssertionResult> {
    const content = this.caseSensitive
      ? response.content
      : response.content.toLowerCase();
    const search = this.caseSensitive
      ? this.substring
      : this.substring.toLowerCase();

    const passed = content.includes(search);

    return {
      passed,
      message: passed
        ? `Contains "${this.substring}"`
        : `Does not contain "${this.substring}"`,
    };
  }
}

// 3. Regex
class RegexAssertion implements Assertion {
  readonly type = 'regex';

  constructor(private pattern: RegExp) {}

  async evaluate(response: CompletionResponse): Promise<AssertionResult> {
    const matches = this.pattern.test(response.content);

    return {
      passed: matches,
      message: matches
        ? `Matches pattern /${this.pattern.source}/`
        : `Does not match pattern /${this.pattern.source}/`,
      details: {
        pattern: this.pattern.source,
        flags: this.pattern.flags,
      },
    };
  }
}

// 4. JSON Schema Validation
class JSONSchemaAssertion implements Assertion {
  readonly type = 'json-schema';

  constructor(private schema: object) {}

  async evaluate(response: CompletionResponse): Promise<AssertionResult> {
    try {
      const data = JSON.parse(response.content);
      const validator = new JSONSchemaValidator(this.schema);
      const result = validator.validate(data);

      return {
        passed: result.valid,
        message: result.valid
          ? 'Valid JSON schema'
          : `Schema validation failed: ${result.errors.join(', ')}`,
        details: { errors: result.errors },
      };
    } catch (error) {
      return {
        passed: false,
        message: `Invalid JSON: ${error.message}`,
      };
    }
  }
}

// 5. Semantic Similarity
class SemanticSimilarityAssertion implements Assertion {
  readonly type = 'semantic';

  constructor(
    private expected: string,
    private threshold: number = 0.85,
    private embeddingProvider: EmbeddingProvider,
  ) {}

  async evaluate(response: CompletionResponse): Promise<AssertionResult> {
    const [expectedEmb, actualEmb] = await Promise.all([
      this.embeddingProvider.embed(this.expected),
      this.embeddingProvider.embed(response.content),
    ]);

    const similarity = this.cosineSimilarity(expectedEmb, actualEmb);
    const passed = similarity >= this.threshold;

    return {
      passed,
      score: similarity,
      message: `Semantic similarity: ${(similarity * 100).toFixed(2)}% ` +
               `(threshold: ${(this.threshold * 100).toFixed(2)}%)`,
      details: {
        similarity,
        threshold: this.threshold,
      },
    };
  }

  private cosineSimilarity(a: number[], b: number[]): number {
    // Vector similarity calculation
    const dotProduct = a.reduce((sum, val, i) => sum + val * b[i], 0);
    const magnitudeA = Math.sqrt(a.reduce((sum, val) => sum + val * val, 0));
    const magnitudeB = Math.sqrt(b.reduce((sum, val) => sum + val * val, 0));
    return dotProduct / (magnitudeA * magnitudeB);
  }
}

// 6. Length constraints
class LengthAssertion implements Assertion {
  readonly type = 'length';

  constructor(
    private min?: number,
    private max?: number,
  ) {}

  async evaluate(response: CompletionResponse): Promise<AssertionResult> {
    const length = response.content.length;

    if (this.min !== undefined && length < this.min) {
      return {
        passed: false,
        message: `Length ${length} is less than minimum ${this.min}`,
      };
    }

    if (this.max !== undefined && length > this.max) {
      return {
        passed: false,
        message: `Length ${length} exceeds maximum ${this.max}`,
      };
    }

    return {
      passed: true,
      message: `Length ${length} within bounds`,
    };
  }
}

// 7. Custom function assertion
class CustomAssertion implements Assertion {
  readonly type = 'custom';

  constructor(
    private fn: (response: CompletionResponse, context: TestContext) => boolean | Promise<boolean>,
    private name: string = 'custom',
  ) {}

  async evaluate(response: CompletionResponse, context: TestContext): Promise<AssertionResult> {
    try {
      const passed = await this.fn(response, context);
      return {
        passed,
        message: passed ? `Custom assertion '${this.name}' passed` : `Custom assertion '${this.name}' failed`,
      };
    } catch (error) {
      return {
        passed: false,
        message: `Custom assertion error: ${error.message}`,
      };
    }
  }
}
```

#### 4.2 Assertion Plugins

```typescript
interface AssertionPlugin {
  name: string;
  version: string;

  register(registry: AssertionRegistry): void;
}

class AssertionRegistry {
  private assertions: Map<string, AssertionFactory>;

  register(type: string, factory: AssertionFactory): void {
    this.assertions.set(type, factory);
  }

  create(config: AssertionConfig): Assertion {
    const factory = this.assertions.get(config.type);
    if (!factory) {
      throw new Error(`Unknown assertion type: ${config.type}`);
    }
    return factory.create(config);
  }
}

// Plugin example
class SentimentAssertionPlugin implements AssertionPlugin {
  name = 'sentiment';
  version = '1.0.0';

  register(registry: AssertionRegistry): void {
    registry.register('sentiment', {
      create: (config) => new SentimentAssertion(
        config.expected,
        config.threshold,
      ),
    });
  }
}

// Usage
const registry = new AssertionRegistry();
const plugin = new SentimentAssertionPlugin();
plugin.register(registry);

// In test file:
// assertions:
//   - type: sentiment
//     expected: positive
//     threshold: 0.7
```

#### 4.3 Scoring & Evaluation

```typescript
interface Scorer {
  score(result: AssertionResult): number;
}

class WeightedScorer implements Scorer {
  constructor(private weights: Map<string, number>) {}

  score(result: AssertionResult): number {
    if (!result.passed) return 0;
    return result.score ?? 1.0;
  }
}

class TestEvaluator {
  async evaluate(
    response: CompletionResponse,
    assertions: Assertion[],
    context: TestContext,
  ): Promise<TestResult> {
    const results = await Promise.all(
      assertions.map(a => a.evaluate(response, context))
    );

    const passed = results.every(r => r.passed);
    const score = this.calculateScore(results);

    return {
      passed,
      score,
      assertions: results,
      response,
      duration: context.duration,
    };
  }

  private calculateScore(results: AssertionResult[]): number {
    if (results.length === 0) return 1.0;

    const scores = results.map(r => r.score ?? (r.passed ? 1 : 0));
    return scores.reduce((sum, s) => sum + s, 0) / scores.length;
  }
}
```

#### 4.4 Pass/Fail Criteria

```typescript
interface PassFailCriteria {
  evaluate(results: TestResult[]): boolean;
}

// All tests must pass
class StrictCriteria implements PassFailCriteria {
  evaluate(results: TestResult[]): boolean {
    return results.every(r => r.passed);
  }
}

// Percentage threshold
class ThresholdCriteria implements PassFailCriteria {
  constructor(private threshold: number) {}

  evaluate(results: TestResult[]): boolean {
    const passCount = results.filter(r => r.passed).length;
    const passRate = passCount / results.length;
    return passRate >= this.threshold;
  }
}

// Score-based
class ScoreCriteria implements PassFailCriteria {
  constructor(private minimumScore: number) {}

  evaluate(results: TestResult[]): boolean {
    const avgScore = results.reduce((sum, r) => sum + r.score, 0) / results.length;
    return avgScore >= this.minimumScore;
  }
}

// Composite criteria
class CompositeCriteria implements PassFailCriteria {
  constructor(private criteria: PassFailCriteria[]) {}

  evaluate(results: TestResult[]): boolean {
    return this.criteria.every(c => c.evaluate(results));
  }
}
```

---

### 5. REPORTING SYSTEM

#### 5.1 Report Formats

```typescript
interface Reporter {
  generate(results: TestSummary): Promise<string>;
  write(results: TestSummary, output: string): Promise<void>;
}

// JSON Reporter
class JSONReporter implements Reporter {
  async generate(results: TestSummary): Promise<string> {
    return JSON.stringify(results, null, 2);
  }

  async write(results: TestSummary, output: string): Promise<void> {
    await fs.writeFile(output, await this.generate(results));
  }
}

// HTML Reporter
class HTMLReporter implements Reporter {
  async generate(results: TestSummary): Promise<string> {
    return `
<!DOCTYPE html>
<html>
<head>
  <title>LTB Test Report</title>
  <style>
    /* Beautiful CSS for test results */
  </style>
</head>
<body>
  <h1>Test Report: ${results.name}</h1>
  <div class="summary">
    <div class="stat">
      <span class="label">Total:</span>
      <span class="value">${results.total}</span>
    </div>
    <div class="stat passed">
      <span class="label">Passed:</span>
      <span class="value">${results.passed}</span>
    </div>
    <div class="stat failed">
      <span class="label">Failed:</span>
      <span class="value">${results.failed}</span>
    </div>
  </div>

  <div class="tests">
    ${this.renderTests(results.tests)}
  </div>

  <script>
    // Interactive features
  </script>
</body>
</html>
    `;
  }

  private renderTests(tests: TestResult[]): string {
    // Render individual test results
  }
}

// Markdown Reporter
class MarkdownReporter implements Reporter {
  async generate(results: TestSummary): Promise<string> {
    return `
# Test Report: ${results.name}

**Date:** ${results.timestamp}
**Duration:** ${results.duration}ms

## Summary

- **Total Tests:** ${results.total}
- **Passed:** ${results.passed} ✅
- **Failed:** ${results.failed} ❌
- **Success Rate:** ${(results.passed / results.total * 100).toFixed(2)}%

## Test Results

${this.renderTests(results.tests)}

## Metrics

${this.renderMetrics(results.metrics)}
    `;
  }
}

// JUnit XML Reporter (for CI/CD)
class JUnitReporter implements Reporter {
  async generate(results: TestSummary): Promise<string> {
    return `
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="${results.name}" tests="${results.total}" failures="${results.failed}" time="${results.duration / 1000}">
  <testsuite name="${results.name}" tests="${results.total}" failures="${results.failed}">
    ${results.tests.map(t => this.renderTest(t)).join('\n')}
  </testsuite>
</testsuites>
    `;
  }

  private renderTest(test: TestResult): string {
    if (test.passed) {
      return `<testcase name="${test.name}" time="${test.duration / 1000}" />`;
    } else {
      return `
<testcase name="${test.name}" time="${test.duration / 1000}">
  <failure message="${test.error?.message}">
    ${test.error?.stack}
  </failure>
</testcase>
      `;
    }
  }
}
```

#### 5.2 Metrics Collection

```typescript
interface MetricsCollector {
  collect(result: TestResult): void;
  aggregate(): Metrics;
  reset(): void;
}

interface Metrics {
  // Performance metrics
  totalDuration: number;
  avgDuration: number;
  minDuration: number;
  maxDuration: number;

  // Token usage
  totalTokens: number;
  promptTokens: number;
  completionTokens: number;

  // Cost estimation
  estimatedCost: number;

  // Success metrics
  passRate: number;
  avgScore: number;

  // Provider metrics
  providerMetrics: Map<string, ProviderMetrics>;
}

class DefaultMetricsCollector implements MetricsCollector {
  private results: TestResult[] = [];

  collect(result: TestResult): void {
    this.results.push(result);
  }

  aggregate(): Metrics {
    const durations = this.results.map(r => r.duration);
    const tokens = this.results.map(r => r.response.usage);

    return {
      totalDuration: durations.reduce((sum, d) => sum + d, 0),
      avgDuration: durations.reduce((sum, d) => sum + d, 0) / durations.length,
      minDuration: Math.min(...durations),
      maxDuration: Math.max(...durations),

      totalTokens: tokens.reduce((sum, t) => sum + t.totalTokens, 0),
      promptTokens: tokens.reduce((sum, t) => sum + t.promptTokens, 0),
      completionTokens: tokens.reduce((sum, t) => sum + t.completionTokens, 0),

      estimatedCost: this.calculateCost(tokens),

      passRate: this.results.filter(r => r.passed).length / this.results.length,
      avgScore: this.results.reduce((sum, r) => sum + r.score, 0) / this.results.length,

      providerMetrics: this.aggregateByProvider(),
    };
  }

  private calculateCost(tokens: TokenUsage[]): number {
    // Cost calculation based on provider pricing
  }

  private aggregateByProvider(): Map<string, ProviderMetrics> {
    // Group metrics by provider
  }
}
```

#### 5.3 Historical Comparison

```typescript
interface HistoricalStorage {
  save(results: TestSummary): Promise<void>;
  load(id: string): Promise<TestSummary>;
  query(filter: HistoricalFilter): Promise<TestSummary[]>;
  cleanup(retention: number): Promise<void>;
}

class FileBasedHistoricalStorage implements HistoricalStorage {
  constructor(private directory: string) {}

  async save(results: TestSummary): Promise<void> {
    const filename = `${results.timestamp.toISOString()}.json`;
    const path = join(this.directory, filename);
    await fs.writeFile(path, JSON.stringify(results, null, 2));
  }

  async query(filter: HistoricalFilter): Promise<TestSummary[]> {
    const files = await fs.readdir(this.directory);
    const summaries = await Promise.all(
      files
        .filter(f => f.endsWith('.json'))
        .map(f => this.loadFile(join(this.directory, f)))
    );

    return summaries.filter(s => this.matches(s, filter));
  }

  async cleanup(retention: number): Promise<void> {
    const cutoff = new Date(Date.now() - retention * 24 * 60 * 60 * 1000);
    const files = await fs.readdir(this.directory);

    for (const file of files) {
      const path = join(this.directory, file);
      const stats = await fs.stat(path);

      if (stats.mtime < cutoff) {
        await fs.unlink(path);
      }
    }
  }
}

class HistoricalComparator {
  compare(current: TestSummary, previous: TestSummary): Comparison {
    return {
      passRateChange: current.passRate - previous.passRate,
      avgDurationChange: current.metrics.avgDuration - previous.metrics.avgDuration,
      scoreChange: current.metrics.avgScore - previous.metrics.avgScore,
      regressions: this.findRegressions(current.tests, previous.tests),
      improvements: this.findImprovements(current.tests, previous.tests),
    };
  }

  private findRegressions(current: TestResult[], previous: TestResult[]): TestResult[] {
    // Find tests that passed before but fail now
    const previousMap = new Map(previous.map(t => [t.name, t]));

    return current.filter(t => {
      const prev = previousMap.get(t.name);
      return prev?.passed && !t.passed;
    });
  }
}
```

#### 5.4 Export Capabilities

```typescript
interface Exporter {
  export(results: TestSummary, options: ExportOptions): Promise<void>;
}

class MultiFormatExporter implements Exporter {
  private reporters: Map<string, Reporter>;

  async export(results: TestSummary, options: ExportOptions): Promise<void> {
    for (const format of options.formats) {
      const reporter = this.reporters.get(format);
      if (!reporter) continue;

      const filename = `${options.basename}.${format}`;
      const output = join(options.directory, filename);

      await reporter.write(results, output);
    }
  }
}

// Export to external systems
class WebhookExporter implements Exporter {
  constructor(private webhookUrl: string) {}

  async export(results: TestSummary): Promise<void> {
    await fetch(this.webhookUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(results),
    });
  }
}

class SlackExporter implements Exporter {
  constructor(private webhookUrl: string) {}

  async export(results: TestSummary): Promise<void> {
    const message = this.formatSlackMessage(results);
    await fetch(this.webhookUrl, {
      method: 'POST',
      body: JSON.stringify(message),
    });
  }

  private formatSlackMessage(results: TestSummary): any {
    return {
      blocks: [
        {
          type: 'header',
          text: {
            type: 'plain_text',
            text: `Test Results: ${results.name}`,
          },
        },
        {
          type: 'section',
          fields: [
            { type: 'mrkdwn', text: `*Passed:*\n${results.passed}` },
            { type: 'mrkdwn', text: `*Failed:*\n${results.failed}` },
            { type: 'mrkdwn', text: `*Duration:*\n${results.duration}ms` },
            { type: 'mrkdwn', text: `*Pass Rate:*\n${(results.passRate * 100).toFixed(2)}%` },
          ],
        },
      ],
    };
  }
}
```

---

### 6. CORE ENGINE

#### 6.1 Test Discovery & Loading

```typescript
interface TestDiscovery {
  discover(patterns: string[]): Promise<TestFile[]>;
  load(file: TestFile): Promise<Test[]>;
}

class GlobBasedDiscovery implements TestDiscovery {
  async discover(patterns: string[]): Promise<TestFile[]> {
    const files: TestFile[] = [];

    for (const pattern of patterns) {
      const matches = await glob(pattern);
      files.push(...matches.map(path => ({ path, type: this.detectType(path) })));
    }

    return files;
  }

  async load(file: TestFile): Promise<Test[]> {
    const loader = this.getLoader(file.type);
    return loader.load(file.path);
  }

  private detectType(path: string): 'yaml' | 'json' | 'js' {
    if (path.endsWith('.yaml') || path.endsWith('.yml')) return 'yaml';
    if (path.endsWith('.json')) return 'json';
    if (path.endsWith('.js') || path.endsWith('.ts')) return 'js';
    throw new Error(`Unknown file type: ${path}`);
  }

  private getLoader(type: string): TestLoader {
    // Return appropriate loader
  }
}

// Test file formats

// YAML format
/*
name: "Chatbot greeting test"
description: "Test friendly greeting response"
provider: "openai"
model: "gpt-4"
messages:
  - role: system
    content: "You are a friendly chatbot."
  - role: user
    content: "Hello!"
assertions:
  - type: contains
    value: "hello"
    case_sensitive: false
  - type: sentiment
    expected: positive
    threshold: 0.8
*/

// JavaScript/TypeScript format
/*
export default {
  name: 'Chatbot greeting test',
  provider: 'openai',
  model: 'gpt-4',
  messages: [
    { role: 'system', content: 'You are a friendly chatbot.' },
    { role: 'user', content: 'Hello!' },
  ],
  assertions: [
    { type: 'contains', value: 'hello', case_sensitive: false },
    {
      type: 'custom',
      fn: (response) => response.content.length > 10,
    },
  ],
};
*/

interface TestLoader {
  load(path: string): Promise<Test[]>;
}

class YAMLTestLoader implements TestLoader {
  async load(path: string): Promise<Test[]> {
    const content = await fs.readFile(path, 'utf-8');
    const data = yaml.parse(content);

    // Support both single test and test suite
    const tests = Array.isArray(data) ? data : [data];

    return tests.map(t => this.parseTest(t, path));
  }

  private parseTest(data: any, sourcePath: string): Test {
    return {
      name: data.name,
      description: data.description,
      provider: data.provider,
      model: data.model,
      messages: data.messages,
      assertions: data.assertions.map(a => this.parseAssertion(a)),
      metadata: {
        source: sourcePath,
        tags: data.tags || [],
      },
    };
  }
}
```

#### 6.2 Execution Orchestration

```typescript
interface TestOrchestrator {
  execute(tests: Test[], options: ExecutionOptions): Promise<TestSummary>;
}

class DefaultTestOrchestrator implements TestOrchestrator {
  constructor(
    private providerRegistry: ProviderRegistry,
    private assertionRegistry: AssertionRegistry,
    private metricsCollector: MetricsCollector,
    private cache: CacheManager,
  ) {}

  async execute(tests: Test[], options: ExecutionOptions): Promise<TestSummary> {
    const startTime = Date.now();

    // Filter tests
    const filtered = this.filterTests(tests, options.filter);

    // Execute tests
    const results = options.parallel > 1
      ? await this.executeParallel(filtered, options)
      : await this.executeSequential(filtered, options);

    // Collect metrics
    const metrics = this.metricsCollector.aggregate();

    return {
      name: options.suiteName || 'Test Suite',
      timestamp: new Date(),
      duration: Date.now() - startTime,
      total: results.length,
      passed: results.filter(r => r.passed).length,
      failed: results.filter(r => !r.passed).length,
      passRate: results.filter(r => r.passed).length / results.length,
      tests: results,
      metrics,
    };
  }

  private async executeSequential(
    tests: Test[],
    options: ExecutionOptions,
  ): Promise<TestResult[]> {
    const results: TestResult[] = [];

    for (const test of tests) {
      const result = await this.executeTest(test, options);
      results.push(result);

      // Report progress
      this.reportProgress(results.length, tests.length, result);

      // Early exit on failure if fail-fast enabled
      if (!result.passed && options.failFast) {
        break;
      }
    }

    return results;
  }

  private async executeParallel(
    tests: Test[],
    options: ExecutionOptions,
  ): Promise<TestResult[]> {
    const concurrency = options.parallel;
    const results: TestResult[] = [];

    // Use worker pool for parallel execution
    const pool = new WorkerPool(concurrency);

    const promises = tests.map(test =>
      pool.execute(() => this.executeTest(test, options))
    );

    return Promise.all(promises);
  }

  private async executeTest(
    test: Test,
    options: ExecutionOptions,
  ): Promise<TestResult> {
    const startTime = Date.now();

    try {
      // Check cache
      if (options.cache) {
        const cached = await this.cache.get(test);
        if (cached) return cached;
      }

      // Get provider
      const provider = await this.providerRegistry.get(test.provider);
      if (!provider) {
        throw new Error(`Provider '${test.provider}' not found`);
      }

      // Execute request
      const response = await provider.complete({
        model: test.model,
        messages: test.messages,
        ...test.parameters,
      });

      // Evaluate assertions
      const assertions = test.assertions.map(a =>
        this.assertionRegistry.create(a)
      );

      const evaluator = new TestEvaluator();
      const result = await evaluator.evaluate(response, assertions, {
        test,
        duration: Date.now() - startTime,
      });

      // Cache result
      if (options.cache) {
        await this.cache.set(test, result);
      }

      // Collect metrics
      this.metricsCollector.collect(result);

      return result;

    } catch (error) {
      return {
        passed: false,
        score: 0,
        assertions: [],
        response: null,
        duration: Date.now() - startTime,
        error: error as Error,
      };
    }
  }

  private filterTests(tests: Test[], filter?: TestFilter): Test[] {
    if (!filter) return tests;

    return tests.filter(test => {
      if (filter.tags && !filter.tags.every(tag => test.metadata.tags.includes(tag))) {
        return false;
      }

      if (filter.pattern && !new RegExp(filter.pattern).test(test.name)) {
        return false;
      }

      if (filter.provider && test.provider !== filter.provider) {
        return false;
      }

      return true;
    });
  }
}
```

#### 6.3 Parallelization Strategy

```typescript
class WorkerPool {
  private queue: (() => Promise<any>)[] = [];
  private active = 0;

  constructor(private concurrency: number) {}

  async execute<T>(task: () => Promise<T>): Promise<T> {
    while (this.active >= this.concurrency) {
      await this.waitForSlot();
    }

    this.active++;

    try {
      return await task();
    } finally {
      this.active--;
    }
  }

  private async waitForSlot(): Promise<void> {
    return new Promise(resolve => {
      const checkSlot = () => {
        if (this.active < this.concurrency) {
          resolve();
        } else {
          setTimeout(checkSlot, 10);
        }
      };
      checkSlot();
    });
  }
}

// Advanced: Worker thread pool for CPU-intensive tasks
class ThreadPool {
  private workers: Worker[] = [];
  private taskQueue: Task[] = [];

  constructor(private size: number) {
    for (let i = 0; i < size; i++) {
      this.workers.push(new Worker('./worker.js'));
    }
  }

  async execute(task: Task): Promise<any> {
    const worker = await this.getAvailableWorker();
    return this.runOnWorker(worker, task);
  }
}
```

#### 6.4 State Management

```typescript
interface StateManager {
  get(key: string): any;
  set(key: string, value: any): void;
  clear(): void;
  snapshot(): State;
  restore(snapshot: State): void;
}

class InMemoryStateManager implements StateManager {
  private state: Map<string, any> = new Map();

  get(key: string): any {
    return this.state.get(key);
  }

  set(key: string, value: any): void {
    this.state.set(key, value);
  }

  clear(): void {
    this.state.clear();
  }

  snapshot(): State {
    return new Map(this.state);
  }

  restore(snapshot: State): void {
    this.state = new Map(snapshot);
  }
}

// Context passed between tests
interface TestContext {
  test: Test;
  duration: number;
  state: StateManager;

  // Access previous test results
  getPreviousResult(testName: string): TestResult | undefined;

  // Share data between tests
  setShared(key: string, value: any): void;
  getShared(key: string): any;
}

// Stateful test example
/*
tests:
  - name: "Login"
    provider: "openai"
    messages:
      - role: user
        content: "Login with user@example.com"
    assertions:
      - type: contains
        value: "success"
    extract:
      token: "regex: token=([a-f0-9]+)"

  - name: "Get user info"
    provider: "openai"
    messages:
      - role: user
        content: "Get user info with token ${state.token}"
    assertions:
      - type: json-schema
        schema: {...}
*/
```

#### 6.5 Caching Mechanisms

```typescript
interface CacheManager {
  get(test: Test): Promise<TestResult | null>;
  set(test: Test, result: TestResult): Promise<void>;
  invalidate(test?: Test): Promise<void>;
  stats(): CacheStats;
}

class ContentHashCache implements CacheManager {
  private cache: Map<string, CacheEntry> = new Map();

  constructor(
    private ttl: number,
    private storage?: PersistentStorage,
  ) {}

  async get(test: Test): Promise<TestResult | null> {
    const key = this.generateKey(test);
    const entry = this.cache.get(key);

    if (!entry) {
      // Try persistent storage
      return this.storage?.get(key);
    }

    if (this.isExpired(entry)) {
      this.cache.delete(key);
      return null;
    }

    return entry.result;
  }

  async set(test: Test, result: TestResult): Promise<void> {
    const key = this.generateKey(test);
    const entry: CacheEntry = {
      result,
      timestamp: Date.now(),
      ttl: this.ttl,
    };

    this.cache.set(key, entry);

    // Persist to disk
    await this.storage?.set(key, entry);
  }

  async invalidate(test?: Test): Promise<void> {
    if (test) {
      const key = this.generateKey(test);
      this.cache.delete(key);
      await this.storage?.delete(key);
    } else {
      this.cache.clear();
      await this.storage?.clear();
    }
  }

  private generateKey(test: Test): string {
    // Hash test content (provider, model, messages, assertions)
    const content = JSON.stringify({
      provider: test.provider,
      model: test.model,
      messages: test.messages,
      assertions: test.assertions,
    });

    return this.hash(content);
  }

  private hash(content: string): string {
    // Use fast hash function (e.g., xxhash, murmur3)
    return crypto.createHash('sha256').update(content).digest('hex');
  }

  private isExpired(entry: CacheEntry): boolean {
    return Date.now() - entry.timestamp > entry.ttl;
  }

  stats(): CacheStats {
    return {
      size: this.cache.size,
      hitRate: this.calculateHitRate(),
      avgAge: this.calculateAvgAge(),
    };
  }
}

// Persistent storage
interface PersistentStorage {
  get(key: string): Promise<CacheEntry | null>;
  set(key: string, entry: CacheEntry): Promise<void>;
  delete(key: string): Promise<void>;
  clear(): Promise<void>;
}

class SQLiteCacheStorage implements PersistentStorage {
  private db: Database;

  constructor(dbPath: string) {
    this.db = new Database(dbPath);
    this.initialize();
  }

  private initialize(): void {
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS cache (
        key TEXT PRIMARY KEY,
        result TEXT,
        timestamp INTEGER,
        ttl INTEGER
      )
    `);
  }

  async get(key: string): Promise<CacheEntry | null> {
    const row = this.db.prepare('SELECT * FROM cache WHERE key = ?').get(key);
    if (!row) return null;

    return {
      result: JSON.parse(row.result),
      timestamp: row.timestamp,
      ttl: row.ttl,
    };
  }

  async set(key: string, entry: CacheEntry): Promise<void> {
    this.db.prepare(`
      INSERT OR REPLACE INTO cache (key, result, timestamp, ttl)
      VALUES (?, ?, ?, ?)
    `).run(key, JSON.stringify(entry.result), entry.timestamp, entry.ttl);
  }
}
```

---

## Data Flow

### End-to-End Test Execution Flow

```
1. CLI Input
   └─> Command parsing (yargs)
       └─> Argument validation
           └─> Configuration loading

2. Configuration Resolution
   └─> Load config hierarchy
       └─> Merge configurations
           └─> Resolve secrets
               └─> Validate schema

3. Test Discovery
   └─> Glob patterns
       └─> Load test files
           └─> Parse test definitions
               └─> Filter by tags/patterns

4. Initialization
   └─> Initialize providers
       └─> Load plugins
           └─> Setup cache
               └─> Initialize state

5. Test Execution
   └─> For each test:
       ├─> Check cache
       ├─> Acquire rate limit token
       ├─> Execute provider request
       │   ├─> Authentication
       │   ├─> API call (with retries)
       │   └─> Response parsing
       ├─> Evaluate assertions
       │   ├─> Load assertion plugins
       │   ├─> Execute each assertion
       │   └─> Calculate score
       ├─> Collect metrics
       └─> Cache result

6. Reporting
   └─> Aggregate results
       └─> Generate reports (JSON, HTML, etc.)
           └─> Save to disk
               └─> Export to external systems

7. Cleanup
   └─> Close provider connections
       └─> Flush cache
           └─> Save state
               └─> Exit with code
```

### Data Flow Diagram

```
┌─────────────┐
│   CLI Args  │
└──────┬──────┘
       │
       ▼
┌─────────────────────────┐
│  Configuration Loader   │
│  (File + Env + Args)    │
└──────────┬──────────────┘
           │
           ▼
┌──────────────────────────┐
│   Test Discovery         │
│   (Glob + Parse)         │
└──────────┬───────────────┘
           │
           ▼
┌──────────────────────────┐       ┌─────────────┐
│   Test Orchestrator      │◄──────┤   Cache     │
└──────────┬───────────────┘       └─────────────┘
           │
           ├─────────────────────────┐
           │                         │
           ▼                         ▼
┌─────────────────────┐   ┌────────────────────┐
│  Provider Registry  │   │ Assertion Registry │
└──────────┬──────────┘   └─────────┬──────────┘
           │                        │
           ▼                        ▼
┌─────────────────────┐   ┌────────────────────┐
│   LLM Provider      │   │    Evaluator       │
│   (OpenAI, etc.)    │   │   (Assertions)     │
└──────────┬──────────┘   └─────────┬──────────┘
           │                        │
           └───────────┬────────────┘
                       │
                       ▼
              ┌────────────────┐
              │  Test Result   │
              └────────┬───────┘
                       │
                       ▼
              ┌────────────────┐
              │  Metrics       │
              │  Collector     │
              └────────┬───────┘
                       │
                       ▼
              ┌────────────────┐
              │   Reporter     │
              │  (JSON/HTML)   │
              └────────────────┘
```

---

## Interface Specifications

### Core Interfaces

```typescript
// Test Definition
interface Test {
  name: string;
  description?: string;
  provider: string;
  model: string;
  messages: Message[];
  assertions: AssertionConfig[];
  parameters?: Record<string, unknown>;
  metadata: {
    source: string;
    tags: string[];
  };
}

// Test Result
interface TestResult {
  passed: boolean;
  score: number;
  assertions: AssertionResult[];
  response: CompletionResponse;
  duration: number;
  error?: Error;
}

// Test Summary
interface TestSummary {
  name: string;
  timestamp: Date;
  duration: number;
  total: number;
  passed: number;
  failed: number;
  passRate: number;
  tests: TestResult[];
  metrics: Metrics;
}

// Execution Options
interface ExecutionOptions {
  suiteName?: string;
  filter?: TestFilter;
  parallel: number;
  cache: boolean;
  failFast: boolean;
  timeout: number;
}

interface TestFilter {
  tags?: string[];
  pattern?: string;
  provider?: string;
}
```

---

## Security Architecture

### 1. Secrets Management

**Principles**:
- Never log secrets
- Never commit secrets to version control
- Use environment variables or vaults
- Encrypt secrets at rest

**Implementation**:
```typescript
class SecureSecretsManager {
  private secrets: Map<string, string> = new Map();

  // Secrets stored encrypted in memory
  set(key: string, value: string): void {
    const encrypted = this.encrypt(value);
    this.secrets.set(key, encrypted);
  }

  get(key: string): string {
    const encrypted = this.secrets.get(key);
    if (!encrypted) throw new Error(`Secret '${key}' not found`);
    return this.decrypt(encrypted);
  }

  // Mask secrets in logs
  maskSecret(text: string): string {
    for (const [key, value] of this.secrets) {
      const decrypted = this.decrypt(value);
      text = text.replace(new RegExp(decrypted, 'g'), '***REDACTED***');
    }
    return text;
  }

  private encrypt(value: string): string {
    // Use AES-256-GCM or similar
  }

  private decrypt(value: string): string {
    // Decrypt using key from environment
  }
}
```

### 2. API Key Rotation

```typescript
class RotatingAPIKeyProvider {
  private keys: string[];
  private currentIndex = 0;

  constructor(keys: string[]) {
    this.keys = keys;
  }

  getKey(): string {
    const key = this.keys[this.currentIndex];
    this.currentIndex = (this.currentIndex + 1) % this.keys.length;
    return key;
  }

  rotate(): void {
    // Remove oldest key, add new key
  }
}
```

### 3. Sandboxing Custom Code

```typescript
// Execute custom assertions in sandbox
class SandboxedAssertion implements Assertion {
  async evaluate(response: CompletionResponse): Promise<AssertionResult> {
    const vm = new VM({
      timeout: 5000,
      sandbox: {
        response,
        // Limited globals
      },
    });

    try {
      const result = await vm.run(this.code);
      return result;
    } catch (error) {
      return {
        passed: false,
        message: `Sandbox error: ${error.message}`,
      };
    }
  }
}
```

### 4. Input Validation

**Validate all inputs**:
- Configuration files (schema validation)
- CLI arguments (type checking)
- Test files (schema validation)
- Provider responses (sanitization)

### 5. Audit Logging

```typescript
class AuditLogger {
  log(event: AuditEvent): void {
    const entry = {
      timestamp: new Date(),
      event: event.type,
      user: event.user,
      details: this.sanitize(event.details),
    };

    // Write to secure audit log
    this.writeToAuditLog(entry);
  }

  private sanitize(details: any): any {
    // Remove sensitive information
    return redactSecrets(details);
  }
}
```

---

## Scalability & Extensibility

### 1. Plugin System

```typescript
interface Plugin {
  name: string;
  version: string;
  type: 'provider' | 'assertion' | 'reporter' | 'hook';

  register(registry: Registry): void;
  initialize?(config: PluginConfig): Promise<void>;
  destroy?(): Promise<void>;
}

class PluginManager {
  private plugins: Map<string, Plugin> = new Map();

  async load(path: string): Promise<void> {
    const module = await import(path);
    const plugin: Plugin = module.default;

    await plugin.initialize?.();
    plugin.register(this.getRegistry(plugin.type));

    this.plugins.set(plugin.name, plugin);
  }

  async unload(name: string): Promise<void> {
    const plugin = this.plugins.get(name);
    if (plugin) {
      await plugin.destroy?.();
      this.plugins.delete(name);
    }
  }
}
```

### 2. Horizontal Scaling

**Distributed execution**:
- Test sharding across multiple workers
- Message queue for work distribution
- Centralized result aggregation

```typescript
interface DistributedExecutor {
  shard(tests: Test[], shards: number): Test[][];
  distribute(shards: Test[][]): Promise<void>;
  aggregate(): Promise<TestSummary>;
}

// Example: Redis-based distribution
class RedisDistributedExecutor implements DistributedExecutor {
  constructor(private redis: Redis) {}

  async distribute(shards: Test[][]): Promise<void> {
    for (const [index, shard] of shards.entries()) {
      await this.redis.lpush('test-queue', JSON.stringify({ index, tests: shard }));
    }
  }

  async aggregate(): Promise<TestSummary> {
    // Collect results from all workers
    const results = await this.redis.lrange('results', 0, -1);
    return this.mergeResults(results.map(r => JSON.parse(r)));
  }
}
```

### 3. Extensibility Points

**Key extension points**:
1. **Custom Providers**: Implement `LLMProvider` interface
2. **Custom Assertions**: Implement `Assertion` interface
3. **Custom Reporters**: Implement `Reporter` interface
4. **Hooks**: Pre/post test execution
5. **Middleware**: Request/response transformation

```typescript
// Hook system
interface Hook {
  before?(context: HookContext): Promise<void>;
  after?(context: HookContext, result: TestResult): Promise<void>;
}

class HookManager {
  private hooks: Hook[] = [];

  register(hook: Hook): void {
    this.hooks.push(hook);
  }

  async executeBefore(context: HookContext): Promise<void> {
    for (const hook of this.hooks) {
      await hook.before?.(context);
    }
  }

  async executeAfter(context: HookContext, result: TestResult): Promise<void> {
    for (const hook of this.hooks) {
      await hook.after?.(context, result);
    }
  }
}
```

### 4. Performance Optimization

**Strategies**:
- **Connection pooling**: Reuse HTTP connections
- **Request batching**: Batch multiple tests in single request
- **Lazy loading**: Load plugins on-demand
- **Streaming**: Stream large responses
- **Caching**: Multi-level caching (memory + disk)

```typescript
// Connection pool
class ConnectionPool {
  private pool: Connection[] = [];
  private maxSize = 10;

  async acquire(): Promise<Connection> {
    if (this.pool.length > 0) {
      return this.pool.pop()!;
    }
    return this.createConnection();
  }

  release(conn: Connection): void {
    if (this.pool.length < this.maxSize) {
      this.pool.push(conn);
    } else {
      conn.close();
    }
  }
}
```

---

## Error Handling Strategy

### 1. Error Hierarchy

```typescript
// Base error class
class LTBError extends Error {
  constructor(
    message: string,
    public code: string,
    public details?: Record<string, unknown>,
  ) {
    super(message);
    this.name = this.constructor.name;
  }
}

// Specific error types
class ConfigurationError extends LTBError {
  constructor(message: string, details?: Record<string, unknown>) {
    super(message, 'E1xx', details);
  }
}

class ProviderError extends LTBError {
  constructor(message: string, public provider: string, details?: Record<string, unknown>) {
    super(message, 'E2xx', details);
  }
}

class AssertionError extends LTBError {
  constructor(message: string, public assertion: string, details?: Record<string, unknown>) {
    super(message, 'E3xx', details);
  }
}

class TimeoutError extends LTBError {
  constructor(message: string, public timeout: number) {
    super(message, 'E4xx', { timeout });
  }
}
```

### 2. Error Recovery

```typescript
class ErrorRecoveryManager {
  async recover<T>(
    operation: () => Promise<T>,
    recovery: RecoveryStrategy,
  ): Promise<T> {
    try {
      return await operation();
    } catch (error) {
      return recovery.recover(error);
    }
  }
}

interface RecoveryStrategy {
  recover(error: Error): any;
}

class FallbackRecovery implements RecoveryStrategy {
  constructor(private fallback: () => any) {}

  recover(error: Error): any {
    console.warn(`Error occurred, using fallback: ${error.message}`);
    return this.fallback();
  }
}

class RetryRecovery implements RecoveryStrategy {
  constructor(private retries: number, private operation: () => Promise<any>) {}

  async recover(error: Error): Promise<any> {
    for (let i = 0; i < this.retries; i++) {
      try {
        return await this.operation();
      } catch (e) {
        if (i === this.retries - 1) throw e;
        await this.delay(Math.pow(2, i) * 1000);
      }
    }
  }
}
```

### 3. User-Friendly Error Messages

```typescript
class ErrorFormatter {
  format(error: Error): string {
    if (error instanceof ConfigurationError) {
      return this.formatConfigError(error);
    }

    if (error instanceof ProviderError) {
      return this.formatProviderError(error);
    }

    return this.formatGenericError(error);
  }

  private formatConfigError(error: ConfigurationError): string {
    return `
Configuration Error (${error.code}):
  ${error.message}

Possible fixes:
  - Check your ltb.config.yaml file
  - Run 'ltb config validate' to see detailed errors
  - See https://docs.ltb.dev/config for examples

Details:
  ${JSON.stringify(error.details, null, 2)}
    `;
  }

  private formatProviderError(error: ProviderError): string {
    return `
Provider Error (${error.code}):
  Provider: ${error.provider}
  Message: ${error.message}

Possible fixes:
  - Check your API key is valid
  - Verify the provider is accessible
  - Run 'ltb providers test ${error.provider}'

Need help? https://docs.ltb.dev/providers/${error.provider}
    `;
  }
}
```

---

## Technology Stack

### Core Technologies

**Runtime**: Node.js (>=18.0.0)
- Mature ecosystem
- Excellent async/await support
- Native streaming support

**Language**: TypeScript
- Type safety
- Better IDE support
- Self-documenting code

### Key Dependencies

**CLI Framework**:
- `yargs` - Command-line parsing
- `inquirer` - Interactive prompts
- `chalk` - Terminal colors
- `ora` - Spinners

**Configuration**:
- `zod` - Schema validation
- `dotenv` - Environment variables
- `yaml` - YAML parsing

**Testing & Assertions**:
- Custom assertion engine
- `ajv` - JSON schema validation

**LLM Providers**:
- `openai` - Official OpenAI SDK
- `@anthropic-ai/sdk` - Anthropic SDK
- `axios` - HTTP client for custom providers

**Reporting**:
- `handlebars` - HTML templating
- `marked` - Markdown generation

**Caching**:
- `better-sqlite3` - SQLite for cache
- `lru-cache` - In-memory cache

**Utilities**:
- `glob` - File pattern matching
- `fast-glob` - Fast file globbing
- `p-limit` - Concurrency control
- `debug` - Debug logging

### Development Tools

**Testing**:
- `vitest` - Unit testing
- `playwright` - E2E testing (for CLI)

**Linting**:
- `eslint` - Code linting
- `prettier` - Code formatting

**Build**:
- `tsup` - TypeScript bundler
- `esbuild` - Fast bundler

**Documentation**:
- `typedoc` - API documentation
- `vitepress` - Documentation site

---

## Design Patterns Applied

1. **Strategy Pattern**: Provider abstraction, assertion types
2. **Factory Pattern**: Provider creation, assertion creation
3. **Builder Pattern**: Test configuration, command building
4. **Observer Pattern**: Progress reporting, event hooks
5. **Chain of Responsibility**: Error handling, middleware
6. **Singleton Pattern**: Configuration, registry
7. **Adapter Pattern**: Provider adapters
8. **Decorator Pattern**: Retry logic, rate limiting
9. **Template Method**: Test execution flow
10. **Plugin Pattern**: Extensibility system

---

## Migration Path & Versioning

**Semantic Versioning**: MAJOR.MINOR.PATCH

**Breaking Changes**:
- Major version bumps only
- Migration guides provided
- Deprecation warnings in advance

**Backward Compatibility**:
- Support previous config format for 1 major version
- Auto-migration tools provided

---

## Success Metrics

**Performance Targets**:
- Test execution: <100ms overhead per test
- Configuration loading: <50ms
- Report generation: <1s for 1000 tests

**Reliability Targets**:
- 99.9% success rate for valid tests
- Graceful degradation on provider failures

**Developer Experience**:
- Zero config for simple cases
- <5 minutes to first test
- Excellent error messages

---

This architecture provides a solid foundation for building a production-grade LLM testing framework that is:
- **Extensible**: Plugin system for custom providers, assertions, reporters
- **Performant**: Parallel execution, caching, connection pooling
- **Reliable**: Comprehensive error handling, retry logic, validation
- **Secure**: Secrets management, sandboxing, audit logging
- **Developer-friendly**: Intuitive CLI, excellent DX, clear documentation

The modular design allows components to evolve independently while maintaining clean interfaces and separation of concerns.
