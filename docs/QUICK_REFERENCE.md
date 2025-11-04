# LLM Test Bench - Quick Reference Guide

**Purpose**: Fast lookup for developers implementing the architecture

---

## Document Map

- **ARCHITECTURE.md** - Complete system architecture (READ FIRST)
- **ARCHITECTURE_DIAGRAMS.md** - Visual diagrams and flows
- **IMPLEMENTATION_ROADMAP.md** - Phased development plan
- **DESIGN_DECISIONS.md** - Why we made key choices
- **QUICK_REFERENCE.md** - This document (cheat sheet)

---

## File Structure at a Glance

```
src/
├── cli/              # User interface
├── config/           # Configuration loading
├── core/             # Orchestration & execution
├── providers/        # LLM provider adapters
├── assertions/       # Assertion engine
├── reporting/        # Reports & metrics
├── plugins/          # Plugin system
└── utils/            # Shared utilities
```

---

## Key Interfaces

### Test Definition
```typescript
interface Test {
  name: string;
  provider: string;  // "openai", "anthropic", etc.
  model: string;     // "gpt-4", "claude-3-opus", etc.
  messages: Message[];
  assertions: AssertionConfig[];
}
```

### Provider Interface
```typescript
interface LLMProvider {
  complete(request: CompletionRequest): Promise<CompletionResponse>;
  stream(request: CompletionRequest): AsyncIterableIterator<CompletionChunk>;
}
```

### Assertion Interface
```typescript
interface Assertion {
  readonly type: string;
  evaluate(response: CompletionResponse): Promise<AssertionResult>;
}
```

---

## Common Patterns

### 1. Creating a New Provider

```typescript
// src/providers/adapters/my-provider.ts
export class MyProvider implements LLMProvider {
  constructor(private config: ProviderConfig) {}

  async complete(request: CompletionRequest): Promise<CompletionResponse> {
    // 1. Call provider API
    const response = await this.callAPI(request);

    // 2. Transform to standard format
    return {
      id: response.id,
      content: response.text,
      usage: {
        promptTokens: response.usage.input,
        completionTokens: response.usage.output,
        totalTokens: response.usage.total,
      },
    };
  }
}

// Register in src/providers/registry.ts
registry.register('my-provider', new MyProviderFactory());
```

### 2. Creating a New Assertion

```typescript
// src/assertions/builtin/my-assertion.ts
export class MyAssertion implements Assertion {
  readonly type = 'my-assertion';

  constructor(private expected: any) {}

  async evaluate(response: CompletionResponse): Promise<AssertionResult> {
    const passed = this.check(response.content);

    return {
      passed,
      message: passed ? 'Success' : 'Failed because...',
      score: passed ? 1 : 0,
    };
  }
}

// Register in src/assertions/registry.ts
registry.register('my-assertion', MyAssertionFactory);
```

### 3. Adding a CLI Command

```typescript
// src/cli/commands/my-command.ts
export function registerCommand(yargs: Yargs) {
  return yargs.command(
    'my-command',
    'Description of command',
    (yargs) => {
      return yargs
        .option('my-option', {
          type: 'string',
          description: 'My option description',
        });
    },
    async (argv) => {
      // Command implementation
      console.log('Executing command...');
    }
  );
}

// Add to src/cli/index.ts
import { registerCommand } from './commands/my-command';
registerCommand(cli);
```

---

## Configuration Examples

### Minimal Config
```yaml
version: "1.0"

providers:
  openai:
    type: openai
```

### Complete Config
```yaml
version: "1.0"
name: "My Test Suite"

defaults:
  provider: "openai"
  timeout: 30000
  retries: 3

providers:
  openai:
    type: "openai"
    apiKey: "${env:OPENAI_API_KEY}"
    defaults:
      model: "gpt-4"
      temperature: 0.7

tests:
  include:
    - "tests/**/*.test.yaml"
  exclude:
    - "tests/wip/**"

cache:
  enabled: true
  ttl: 3600

reporting:
  formats: ["json", "html"]
  output: "./reports"
```

---

## Test File Examples

### Basic Test
```yaml
name: "Simple greeting"
provider: "openai"
model: "gpt-4"
messages:
  - role: "user"
    content: "Say hello"
assertions:
  - type: "contains"
    value: "hello"
```

### Advanced Test
```yaml
name: "Complex validation"
provider: "openai"
model: "gpt-4"
messages:
  - role: "system"
    content: "You are a helpful assistant."
  - role: "user"
    content: "Generate a JSON object with name and age"
assertions:
  - type: "json-schema"
    schema:
      type: "object"
      required: ["name", "age"]
      properties:
        name: { type: "string" }
        age: { type: "number" }
  - type: "length"
    min: 20
    max: 200
```

### Test Suite (Multiple Tests)
```yaml
- name: "Test 1"
  provider: "openai"
  model: "gpt-4"
  messages:
    - role: "user"
      content: "Hello"
  assertions:
    - type: "contains"
      value: "hi"

- name: "Test 2"
  provider: "anthropic"
  model: "claude-3-opus"
  messages:
    - role: "user"
      content: "Goodbye"
  assertions:
    - type: "contains"
      value: "bye"
```

---

## CLI Commands Reference

```bash
# Initialize project
ltb init

# Run tests
ltb run                              # All tests
ltb run tests/smoke.yaml             # Specific file
ltb run "tests/**/*.yaml"            # Glob pattern
ltb run --parallel 8                 # Parallel execution
ltb run --cache false                # Disable cache
ltb run --fail-fast                  # Stop on first failure
ltb run --filter "tag:smoke"         # Filter by tag
ltb run --debug                      # Debug mode

# Validate configuration
ltb validate
ltb config validate
ltb config show                      # Show merged config

# Manage providers
ltb providers list                   # List providers
ltb providers test openai            # Test provider connection

# Generate reports
ltb report                           # Latest run
ltb report --format html             # HTML format
ltb report --output ./reports        # Custom output
ltb report --compare baseline.json   # Historical comparison

# Cache management
ltb cache clear                      # Clear cache
ltb cache info                       # Cache statistics
```

---

## Environment Variables

```bash
# Configuration
LTB_CONFIG=./custom-config.yaml
LTB_DEBUG=true
LTB_LOG_LEVEL=debug

# Execution
LTB_PARALLEL=8
LTB_CACHE=false
LTB_TIMEOUT=60000

# Provider API Keys
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

# Reporting
LTB_REPORT_FORMAT=json
LTB_REPORT_OUTPUT=./reports
```

---

## Common Tasks

### Add Support for New LLM Provider

1. Create adapter in `src/providers/adapters/`
2. Implement `LLMProvider` interface
3. Register in `src/providers/registry.ts`
4. Add tests in `tests/providers/`
5. Update documentation

### Add New Built-in Assertion

1. Create assertion in `src/assertions/builtin/`
2. Implement `Assertion` interface
3. Create factory function
4. Register in `src/assertions/registry.ts`
5. Add tests
6. Update docs

### Add New Report Format

1. Create reporter in `src/reporting/reporters/`
2. Implement `Reporter` interface
3. Register in `src/reporting/registry.ts`
4. Add template if needed
5. Test with real data
6. Update docs

---

## Testing Checklist

When implementing a feature:

- [ ] Unit tests for core logic
- [ ] Integration tests for components
- [ ] E2E test with real providers
- [ ] Error handling tested
- [ ] Edge cases covered
- [ ] Documentation updated
- [ ] Examples added
- [ ] TypeScript types correct

---

## Code Style Guidelines

### Naming Conventions
- **Classes**: PascalCase (`TestOrchestrator`)
- **Interfaces**: PascalCase (`LLMProvider`)
- **Functions**: camelCase (`executeTest`)
- **Constants**: UPPER_SNAKE_CASE (`DEFAULT_TIMEOUT`)
- **Files**: kebab-case (`test-orchestrator.ts`)

### File Organization
```typescript
// 1. Imports
import { something } from 'external';
import { local } from './local';

// 2. Types & Interfaces
interface MyInterface {}
type MyType = {};

// 3. Constants
const DEFAULT_VALUE = 42;

// 4. Main class/function
export class MyClass {}

// 5. Helper functions
function helper() {}
```

### Error Handling
```typescript
try {
  const result = await operation();
  return result;
} catch (error) {
  throw new LTBError(
    'Operation failed',
    'E123',
    { details: error.message }
  );
}
```

### Async/Await
```typescript
// Good
async function doWork() {
  const result = await operation();
  return result;
}

// Avoid
function doWork() {
  return operation().then(result => result);
}
```

---

## Performance Tips

1. **Use content hash caching** - Avoid duplicate API calls
2. **Enable parallel execution** - For large test suites
3. **Implement rate limiting** - Respect provider limits
4. **Stream large responses** - Don't buffer everything
5. **Lazy load plugins** - Only load what's needed
6. **Use connection pooling** - Reuse HTTP connections

---

## Debugging Tips

### Enable Debug Mode
```bash
ltb run --debug
# or
LTB_DEBUG=true ltb run
```

### Check Configuration
```bash
ltb config show
ltb config validate
```

### Test Provider Connection
```bash
ltb providers test openai
```

### Clear Cache
```bash
ltb cache clear
```

### Verbose Logging
```bash
LTB_LOG_LEVEL=debug ltb run
```

---

## Release Checklist

Before releasing:

- [ ] All tests passing
- [ ] No TypeScript errors
- [ ] Linting passes
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped
- [ ] Git tag created
- [ ] Build succeeds
- [ ] Package published

---

## Common Errors & Solutions

### Error: Provider not found
```bash
# Check provider is registered
ltb providers list

# Verify config
ltb config show
```

### Error: Invalid configuration
```bash
# Validate config
ltb config validate

# Check schema
cat ltb.config.yaml
```

### Error: Rate limit exceeded
```bash
# Reduce parallelism
ltb run --parallel 1

# Add rate limiting to config
providers:
  openai:
    rateLimit:
      requests: 10
      period: 60000  # 10 requests per minute
```

### Error: Test timeout
```bash
# Increase timeout
ltb run --timeout 60000

# Or in config
defaults:
  timeout: 60000
```

---

## Useful Resources

### Documentation
- Main docs: `docs/`
- Examples: `examples/`
- API reference: `docs/api/`

### External Links
- TypeScript Handbook: https://www.typescriptlang.org/docs/
- Node.js Docs: https://nodejs.org/docs/
- yargs: https://yargs.js.org/
- Zod: https://zod.dev/

### Community
- GitHub Issues: (link to repo)
- Discord: (link to Discord)
- Twitter: (link to Twitter)

---

## Quick Start for New Developers

1. **Read**: ARCHITECTURE.md (understand the system)
2. **Setup**: Follow IMPLEMENTATION_ROADMAP.md Phase 0
3. **Build**: Choose a task from the roadmap
4. **Test**: Write tests first (TDD)
5. **Submit**: Create PR with tests and docs
6. **Review**: Address feedback
7. **Merge**: Celebrate! 🎉

---

This quick reference should help you navigate the codebase and implement features efficiently. For detailed information, always refer to the main ARCHITECTURE.md document.
