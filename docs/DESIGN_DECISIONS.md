# LLM Test Bench - Design Decisions

**Version:** 1.0
**Purpose:** Document key architectural decisions and their rationale

---

## Table of Contents

1. [Technology Choices](#technology-choices)
2. [Architectural Patterns](#architectural-patterns)
3. [API Design](#api-design)
4. [Configuration Strategy](#configuration-strategy)
5. [Performance Decisions](#performance-decisions)
6. [Security Choices](#security-choices)
7. [Developer Experience](#developer-experience)
8. [Trade-offs](#trade-offs)

---

## Technology Choices

### Decision 1: Node.js + TypeScript

**Choice**: Use Node.js with TypeScript as the primary runtime and language.

**Rationale**:
- **Node.js**:
  - Excellent async/await support for I/O-heavy operations (API calls)
  - Rich ecosystem with existing LLM SDKs (OpenAI, Anthropic)
  - Fast startup time for CLI applications
  - Native streaming support
  - Single runtime for both CLI and potential web UI

- **TypeScript**:
  - Type safety catches errors at compile time
  - Better IDE support and autocompletion
  - Self-documenting code through types
  - Easier refactoring
  - Industry standard for modern Node.js projects

**Alternatives Considered**:
- **Python**: Great ML ecosystem but slower startup, packaging complexity
- **Go**: Fast but smaller LLM SDK ecosystem, verbose error handling
- **Rust**: Excellent performance but steep learning curve, smaller community

**Decision**: Node.js + TypeScript offers the best balance of performance, ecosystem, and developer experience.

---

### Decision 2: YAML for Configuration

**Choice**: Use YAML as the primary configuration format.

**Rationale**:
- Human-readable and writable
- Supports comments (critical for documentation)
- Anchors and aliases for DRY configuration
- Industry standard for configuration (Kubernetes, Docker Compose, GitHub Actions)
- No trailing commas or syntax noise

**Example**:
```yaml
# YAML - Clean and readable
defaults: &defaults
  temperature: 0.7
  max_tokens: 1000

providers:
  openai:
    <<: *defaults
    type: openai
    api_key: ${env:OPENAI_API_KEY}
```

vs JSON:
```json
{
  "defaults": {
    "temperature": 0.7,
    "max_tokens": 1000
  },
  "providers": {
    "openai": {
      "type": "openai",
      "temperature": 0.7,
      "max_tokens": 1000
    }
  }
}
```

**Trade-off**: YAML can be error-prone with indentation. We mitigate this with:
- Schema validation
- Clear error messages showing line/column
- IDE plugins for validation

---

### Decision 3: Zod for Schema Validation

**Choice**: Use Zod instead of JSON Schema or class-validator.

**Rationale**:
- **Type inference**: TypeScript types derived from schemas
- **Runtime validation**: Catches errors early
- **Composability**: Easy to build complex schemas
- **Great error messages**: User-friendly validation errors
- **Small bundle size**: ~8kb minified

**Example**:
```typescript
const ProviderSchema = z.object({
  type: z.enum(['openai', 'anthropic', 'ollama']),
  apiKey: z.string().optional(),
  baseUrl: z.string().url().optional(),
});

// TypeScript type automatically inferred!
type Provider = z.infer<typeof ProviderSchema>;
```

**Alternatives Considered**:
- **JSON Schema**: More verbose, requires separate type definitions
- **class-validator**: Requires decorators, ties validation to classes
- **ajv**: Fast but less ergonomic API

---

### Decision 4: yargs for CLI Parsing

**Choice**: Use yargs instead of commander.js or oclif.

**Rationale**:
- **Rich API**: Automatic help generation, validation, coercion
- **Middleware support**: Plugin architecture for commands
- **Subcommand support**: Natural nested command structure
- **TypeScript support**: First-class TypeScript definitions
- **Widely used**: Battle-tested, active maintenance

**Example**:
```typescript
yargs
  .command('run', 'Run tests', (yargs) => {
    return yargs
      .option('parallel', { type: 'number', default: 1 })
      .option('cache', { type: 'boolean', default: true });
  })
  .help()
  .parse();
```

**Alternatives Considered**:
- **commander.js**: Less powerful, more manual configuration
- **oclif**: Too heavy for our needs, opinionated framework
- **minimist**: Too low-level, no help generation

---

## Architectural Patterns

### Decision 5: Plugin Architecture

**Choice**: Core functionality with plugin-based extensions.

**Rationale**:
- **Extensibility**: Users can add custom providers, assertions, reporters
- **Maintainability**: Core stays small and focused
- **Community contributions**: Easy to contribute plugins
- **Testing**: Plugins tested independently
- **Lazy loading**: Only load what's needed

**Architecture**:
```
Core (always loaded)
  ├── CLI
  ├── Config
  └── Orchestrator

Plugins (loaded on demand)
  ├── Providers
  │   ├── OpenAI (built-in)
  │   ├── Anthropic (built-in)
  │   └── Custom (user-provided)
  ├── Assertions
  │   ├── Regex (built-in)
  │   ├── JSON Schema (built-in)
  │   └── Custom (user-provided)
  └── Reporters
      ├── JSON (built-in)
      ├── HTML (built-in)
      └── Custom (user-provided)
```

**Trade-off**: More complex initialization. We mitigate with:
- Clear plugin registration API
- Extensive documentation
- Example plugins

---

### Decision 6: Provider Abstraction Layer

**Choice**: Abstract all LLM providers behind a unified interface.

**Rationale**:
- **Provider independence**: Switch providers without changing tests
- **Consistent API**: Same interface for all providers
- **Future-proof**: Easy to add new providers
- **Testing**: Mock providers for unit tests
- **Multi-provider tests**: Compare responses across providers

**Interface**:
```typescript
interface LLMProvider {
  complete(request: CompletionRequest): Promise<CompletionResponse>;
  stream(request: CompletionRequest): AsyncIterableIterator<CompletionChunk>;
}
```

**Design Pattern**: Adapter Pattern
- Each provider (OpenAI, Anthropic, etc.) is an adapter
- Adapters translate between provider-specific APIs and our interface
- Core code only depends on the interface

**Trade-off**: Some provider-specific features may not be supported. We mitigate with:
- `extensions` field for provider-specific parameters
- Documentation of provider capabilities
- Feature detection API

---

### Decision 7: Async-First API

**Choice**: All operations are asynchronous by default.

**Rationale**:
- **LLM APIs are async**: Natural fit
- **Parallel execution**: Easy to run tests concurrently
- **Non-blocking**: CLI remains responsive
- **Streaming**: Support for streaming responses

**Example**:
```typescript
// Everything returns Promises
await config.load();
await provider.complete(request);
await evaluator.evaluate(response);
```

**Trade-off**: More complex error handling. We mitigate with:
- Comprehensive try-catch blocks
- Async error recovery strategies
- Promise.all for parallel operations

---

## API Design

### Decision 8: Fluent Configuration API

**Choice**: Support both declarative (YAML) and programmatic (JS/TS) configuration.

**Rationale**:
- **Flexibility**: Use YAML for simple cases, code for complex logic
- **Type safety**: TypeScript config gets full type checking
- **Dynamic tests**: Generate tests programmatically
- **Integration**: Embed in existing test frameworks

**Example**:
```typescript
// Programmatic configuration
export default {
  name: 'API tests',
  tests: await generateTests(), // Dynamic!
  assertions: [
    {
      type: 'custom',
      fn: (response) => validateComplex(response), // Complex logic!
    },
  ],
};
```

**Trade-off**: Two ways to do things can be confusing. We mitigate with:
- Clear documentation on when to use each
- Examples for both approaches
- Validation works the same for both

---

### Decision 9: Immutable Test Results

**Choice**: Test results are immutable once created.

**Rationale**:
- **Predictability**: Results never change after creation
- **Thread safety**: Safe for parallel execution
- **Debugging**: Results are consistent across inspections
- **Historical comparison**: Results can be cached and compared

**Implementation**:
```typescript
interface TestResult {
  readonly testName: string;
  readonly passed: boolean;
  readonly response: CompletionResponse;
  // All fields readonly
}
```

**Trade-off**: Memory usage for large result sets. We mitigate with:
- Streaming results to disk
- Configurable in-memory limits
- Result pruning options

---

## Configuration Strategy

### Decision 10: Hierarchical Configuration

**Choice**: Support multiple configuration layers with precedence.

**Rationale**:
- **Flexibility**: Override at any level
- **Sharing**: Team configs in repo, personal configs in home
- **Environments**: Different configs for dev/staging/prod
- **Secrets**: Keep secrets out of repo

**Precedence** (highest to lowest):
```
1. CLI arguments        (--config, --parallel)
2. Environment vars     (LTB_PARALLEL=4)
3. Project config       (./.ltb/config.yaml)
4. User config          (~/.ltb/config.yaml)
5. System config        (/etc/ltb/config.yaml)
6. Default values
```

**Merging Strategy**: Deep merge with arrays concatenated

**Trade-off**: Configuration debugging can be complex. We mitigate with:
- `ltb config show` command to see final merged config
- `ltb config validate` to check for issues
- Source attribution in debug mode

---

### Decision 11: Environment Variable Interpolation

**Choice**: Support `${env:VAR}` syntax in configuration files.

**Rationale**:
- **Security**: Keep secrets out of config files
- **Flexibility**: Same config, different environments
- **CI/CD friendly**: Use environment variables in pipelines

**Syntax**:
```yaml
providers:
  openai:
    api_key: ${env:OPENAI_API_KEY}
    organization: ${env:OPENAI_ORG_ID}
```

**Additional References**:
- `${file:path/to/file}` - Read from file
- `${vault:secret/path}` - Fetch from Vault

**Trade-off**: Adds complexity to config loading. We mitigate with:
- Clear error messages if variable missing
- Validation before execution
- Documentation with examples

---

## Performance Decisions

### Decision 12: Content-Hash Caching

**Choice**: Cache test results based on content hash, not test name.

**Rationale**:
- **Accurate invalidation**: Cache invalidated only when content changes
- **Rename safe**: Renaming test doesn't invalidate cache
- **Deduplication**: Identical requests cached once

**Hash includes**:
- Provider name
- Model name
- Messages (full content)
- Parameters (temperature, etc.)

**Example**:
```typescript
function generateCacheKey(test: Test): string {
  return hash({
    provider: test.provider,
    model: test.model,
    messages: test.messages,
    parameters: test.parameters,
  });
}
```

**Trade-off**: Small changes invalidate cache. This is intentional for correctness.

---

### Decision 13: Parallel Execution with Concurrency Control

**Choice**: Support parallel execution with configurable concurrency.

**Rationale**:
- **Speed**: Run multiple tests simultaneously
- **Control**: Respect rate limits with max concurrency
- **Resources**: Avoid overwhelming the system
- **Fair sharing**: Distribute load across providers

**Implementation**:
```typescript
// Use p-limit for concurrency control
const limit = pLimit(concurrency);

const promises = tests.map(test =>
  limit(() => executeTest(test))
);

await Promise.all(promises);
```

**Default**: `concurrency = 4` (reasonable balance)

**Trade-off**: Higher concurrency = faster but more resource usage. Configurable per use case.

---

### Decision 14: Two-Level Cache (Memory + Disk)

**Choice**: L1 cache in memory, L2 cache on disk (SQLite).

**Rationale**:
- **Speed**: Memory cache for hot data (microseconds)
- **Persistence**: Disk cache survives restarts
- **Size**: Memory cache limited, disk cache larger
- **Sharing**: Disk cache shareable across processes

**Cache Strategy**:
```
Request
  ↓
Check L1 (Map)
  ↓ (miss)
Check L2 (SQLite)
  ↓ (miss)
Execute request
  ↓
Store in L2
  ↓
Store in L1
  ↓
Return result
```

**Trade-off**: More complex cache management. Benefits outweigh complexity for large test suites.

---

## Security Choices

### Decision 15: No Secrets in Configuration Files

**Choice**: Never allow secrets directly in config files.

**Rationale**:
- **Security**: Prevent accidental commit of secrets
- **Compliance**: Meet security audit requirements
- **Best practice**: Industry standard approach

**Enforcement**:
```typescript
// Warn if API key looks hardcoded
if (config.apiKey && !config.apiKey.startsWith('${')) {
  console.warn('Warning: API key appears to be hardcoded. Use ${env:VAR} instead.');
}
```

**Recommended Approaches**:
1. Environment variables: `${env:OPENAI_API_KEY}`
2. `.env` files (gitignored)
3. Vault integration: `${vault:secret/key}`
4. System keychain

**Trade-off**: Slightly more setup for users. Clear documentation makes this easy.

---

### Decision 16: Sandbox Custom Code

**Choice**: Execute custom assertions in a sandboxed environment.

**Rationale**:
- **Security**: Prevent malicious code execution
- **Stability**: Isolate crashes to sandbox
- **Resource limits**: Prevent infinite loops

**Implementation**:
```typescript
import { VM } from 'vm2';

const vm = new VM({
  timeout: 5000,
  sandbox: {
    response,
    // Limited globals
  },
});

const result = vm.run(customCode);
```

**Limitations**:
- No file system access
- No network access
- 5 second timeout
- Limited memory

**Trade-off**: Reduced functionality. Security is worth it.

---

## Developer Experience

### Decision 17: Convention over Configuration

**Choice**: Sensible defaults, minimal required configuration.

**Rationale**:
- **Quick start**: Working in under 5 minutes
- **Less boilerplate**: Focus on tests, not setup
- **Consistent**: Projects look similar

**Defaults**:
```yaml
# Minimal config
providers:
  openai:
    type: openai
    # api_key from env automatically

# Everything else uses defaults:
# - timeout: 30s
# - retries: 3
# - parallel: 4
# - cache: enabled
```

**Trade-off**: Advanced users need to learn configuration options. Documentation addresses this.

---

### Decision 18: Excellent Error Messages

**Choice**: Invest heavily in clear, actionable error messages.

**Rationale**:
- **User experience**: Frustrated users abandon tools
- **Support burden**: Good errors reduce support tickets
- **Learning**: Errors teach best practices

**Examples**:

**Bad**:
```
Error: Invalid configuration
```

**Good**:
```
Configuration Error (E101):
  providers.openai.api_key is required

Possible fixes:
  1. Set environment variable: export OPENAI_API_KEY="sk-..."
  2. Add to config: api_key: ${env:OPENAI_API_KEY}
  3. Use .env file (see docs)

File: ltb.config.yaml
Line: 3, Column: 5

Learn more: https://docs.ltb.dev/providers/openai
```

**Implementation**:
- Error codes for all error types
- Context (file, line, column)
- Suggested fixes
- Documentation links

**Trade-off**: More code for error handling. Worth it for UX.

---

### Decision 19: Progressive Disclosure

**Choice**: Simple by default, advanced features discoverable.

**Rationale**:
- **Approachable**: Beginners not overwhelmed
- **Powerful**: Experts find advanced features
- **Gradual learning**: Users grow with tool

**Example**:

**Basic usage**:
```bash
ltb run
```

**Intermediate**:
```bash
ltb run --parallel 8 --cache false
```

**Advanced**:
```bash
ltb run \
  --config custom.yaml \
  --filter "tag:smoke" \
  --parallel 8 \
  --format html \
  --output reports/ \
  --compare baseline.json \
  --debug
```

**Documentation Structure**:
1. Quick start (5 minutes)
2. Common use cases
3. Advanced features
4. API reference

---

## Trade-offs

### Decision 20: CLI-First, API-Second

**Choice**: Optimize for CLI experience first.

**Rationale**:
- **Primary use case**: Most users will use CLI
- **Focus**: Do one thing well
- **Complexity**: API support adds complexity

**Implementation**:
```typescript
// Core logic is library-friendly
export class TestOrchestrator {
  async run(options: RunOptions): Promise<TestSummary> {
    // ...
  }
}

// CLI is thin wrapper
async function runCommand(args) {
  const orchestrator = new TestOrchestrator();
  const summary = await orchestrator.run(args);
  console.log(formatResults(summary));
}
```

**Future**: Library API can be exposed later without refactoring core.

**Trade-off**: Library users have to dig into internals. CLI users get great experience.

---

### Decision 21: Synchronous Test Execution Order

**Choice**: Tests execute in deterministic order by default.

**Rationale**:
- **Predictability**: Same order every run
- **Debugging**: Easier to reproduce issues
- **Stateful tests**: Tests can depend on previous results

**Parallel mode**: Optional `--parallel` flag

**Implementation**:
```typescript
// Sequential (default)
for (const test of tests) {
  await executeTest(test);
}

// Parallel (opt-in)
await Promise.all(
  tests.map(test => executeTest(test))
);
```

**Trade-off**: Slower by default. Correctness > speed. Users can opt into parallel for speed.

---

### Decision 22: JSON as Universal Output Format

**Choice**: All reporters can output to JSON.

**Rationale**:
- **Integration**: Easy to parse and process
- **Completeness**: JSON captures all data
- **Transformation**: Can convert JSON to any format
- **Tooling**: Standard format for CI/CD

**Example**:
```bash
# Generate report
ltb run --format json > report.json

# Transform to HTML
ltb report report.json --format html

# Compare historical
ltb report report.json --compare baseline.json
```

**Trade-off**: JSON files can be large. Compression and streaming solve this.

---

### Decision 23: Fail-Fast vs Complete Execution

**Choice**: Complete execution by default, fail-fast optional.

**Rationale**:
- **Information**: See all failures, not just first
- **CI/CD**: Full report for dashboards
- **Debugging**: Multiple failures may share root cause

**Fail-fast mode**: `--fail-fast` flag for quick feedback

**Implementation**:
```typescript
for (const test of tests) {
  const result = await executeTest(test);

  if (!result.passed && options.failFast) {
    throw new Error('Test failed, stopping execution');
  }
}
```

**Trade-off**: Slower feedback when early test fails. Users can opt into fail-fast.

---

## Summary of Key Principles

1. **Developer Experience First**: Great UX > technical purity
2. **Correctness > Performance**: Get it right, then fast
3. **Security by Default**: No secrets in configs, sandboxing
4. **Extensibility**: Plugin architecture for customization
5. **Convention > Configuration**: Smart defaults, minimal config
6. **Clear Communication**: Excellent error messages and docs
7. **Incremental Adoption**: Works great for simple cases, scales to complex

---

## Decision Log Template

For future decisions, use this template:

```markdown
### Decision N: [Title]

**Choice**: [What we decided]

**Rationale**:
- [Why we chose this]
- [Benefits]
- [Alignment with principles]

**Alternatives Considered**:
- **Option A**: [Why we didn't choose this]
- **Option B**: [Why we didn't choose this]

**Trade-offs**: [What we gave up, how we mitigate]

**Validation**: [How we'll know if this was right]
```

---

These design decisions form the foundation of the LLM Test Bench architecture. Each decision was made thoughtfully, considering trade-offs and alignment with our core principles of developer experience, extensibility, and correctness.
