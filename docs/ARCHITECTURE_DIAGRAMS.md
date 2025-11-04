# LLM Test Bench - Architecture Diagrams

This document provides visual representations of the system architecture to complement the main ARCHITECTURE.md document.

---

## System Layer Diagram

```
┌───────────────────────────────────────────────────────────────────────┐
│                          PRESENTATION LAYER                            │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │  CLI Interface (User-Facing)                                    │  │
│  │  - Command Parser (yargs)                                       │  │
│  │  - Interactive Mode (inquirer)                                  │  │
│  │  - Output Formatter (chalk, table)                              │  │
│  │  - Progress Indicators (ora)                                    │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────┬───────────────────────────────────────┘
                                │
┌───────────────────────────────▼───────────────────────────────────────┐
│                          APPLICATION LAYER                             │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │  Orchestration Services                                         │  │
│  │  - Configuration Manager                                        │  │
│  │  - Test Orchestrator                                            │  │
│  │  - Plugin Manager                                               │  │
│  │  - Report Generator                                             │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────┬───────────────────────────────────────┘
                                │
┌───────────────────────────────▼───────────────────────────────────────┐
│                          BUSINESS LOGIC LAYER                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                │
│  │   Provider   │  │  Assertion   │  │   Metrics    │                │
│  │   Services   │  │   Engine     │  │  Collector   │                │
│  └──────────────┘  └──────────────┘  └──────────────┘                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                │
│  │     Cache    │  │    State     │  │    Hooks     │                │
│  │   Manager    │  │   Manager    │  │   System     │                │
│  └──────────────┘  └──────────────┘  └──────────────┘                │
└───────────────────────────────┬───────────────────────────────────────┘
                                │
┌───────────────────────────────▼───────────────────────────────────────┐
│                          INTEGRATION LAYER                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                │
│  │   Provider   │  │   Storage    │  │   External   │                │
│  │   Adapters   │  │   Adapters   │  │   Services   │                │
│  │  (OpenAI,    │  │  (SQLite,    │  │  (Vault,     │                │
│  │  Anthropic)  │  │   Redis)     │  │   Webhook)   │                │
│  └──────────────┘  └──────────────┘  └──────────────┘                │
└───────────────────────────────────────────────────────────────────────┘
```

---

## Module Dependency Graph

```
┌────────────┐
│     CLI    │
│   Entry    │
└─────┬──────┘
      │
      ├──────────────────┬──────────────────┬──────────────────┐
      │                  │                  │                  │
      ▼                  ▼                  ▼                  ▼
┌──────────┐      ┌──────────┐      ┌──────────┐      ┌──────────┐
│  Config  │      │   Test   │      │  Report  │      │  Cache   │
│  Loader  │      │Discovery │      │Generator │      │ Manager  │
└─────┬────┘      └─────┬────┘      └─────┬────┘      └─────┬────┘
      │                 │                 │                 │
      └────────┬────────┴────────┬────────┘                 │
               │                 │                          │
               ▼                 ▼                          │
        ┌─────────────────────────────┐                     │
        │    Test Orchestrator        │                     │
        └──────────┬──────────────────┘                     │
                   │                                        │
      ┬────────────┼────────────┬                          │
      │            │            │                          │
      ▼            ▼            ▼                          │
┌──────────┐ ┌──────────┐ ┌──────────┐                    │
│ Provider │ │Assertion │ │ Metrics  │                    │
│ Registry │ │ Registry │ │Collector │◄───────────────────┘
└─────┬────┘ └─────┬────┘ └──────────┘
      │            │
      ▼            ▼
┌──────────┐ ┌──────────┐
│ Provider │ │Assertion │
│ Plugins  │ │ Plugins  │
└──────────┘ └──────────┘
      │            │
      ▼            ▼
┌─────────────────────────┐
│   External LLM APIs     │
│  (OpenAI, Anthropic)    │
└─────────────────────────┘
```

---

## Test Execution Sequence Diagram

```
User      CLI      Config    Discovery  Orchestrator  Provider  Assertion  Reporter
 │         │         │           │            │          │          │          │
 │ run     │         │           │            │          │          │          │
 ├────────>│         │           │            │          │          │          │
 │         │ load    │           │            │          │          │          │
 │         ├────────>│           │            │          │          │          │
 │         │<────────┤           │            │          │          │          │
 │         │         │           │            │          │          │          │
 │         │ discover│           │            │          │          │          │
 │         ├─────────┴──────────>│            │          │          │          │
 │         │<───────────────────┘            │          │          │          │
 │         │         │           │            │          │          │          │
 │         │ execute │           │            │          │          │          │
 │         ├─────────┴───────────┴───────────>│          │          │          │
 │         │         │           │            │          │          │          │
 │         │         │           │            │ complete │          │          │
 │         │         │           │            ├─────────>│          │          │
 │         │         │           │            │<─────────┤          │          │
 │         │         │           │            │          │          │          │
 │         │         │           │            │ evaluate │          │          │
 │         │         │           │            ├──────────┴─────────>│          │
 │         │         │           │            │<───────────────────┘          │
 │         │         │           │            │          │          │          │
 │         │<────────┴───────────┴────────────┤          │          │          │
 │         │         │           │            │          │          │          │
 │         │ generate│           │            │          │          │          │
 │         ├─────────┴───────────┴────────────┴──────────┴──────────┴─────────>│
 │         │<────────────────────────────────────────────────────────────────────┤
 │         │         │           │            │          │          │          │
 │<────────┤         │           │            │          │          │          │
 │ results │         │           │            │          │          │          │
```

---

## Provider Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Provider Interface                         │
│  + complete(request): Promise<Response>                         │
│  + stream(request): AsyncIterator<Chunk>                        │
│  + initialize(config): Promise<void>                            │
│  + getModels(): Promise<Model[]>                                │
└───────────────────────────┬─────────────────────────────────────┘
                            │
            ┌───────────────┴───────────────┬──────────────────────┐
            │                               │                      │
            ▼                               ▼                      ▼
┌──────────────────────┐      ┌──────────────────────┐  ┌──────────────────┐
│   OpenAI Provider    │      │ Anthropic Provider   │  │  Ollama Provider │
└──────────┬───────────┘      └──────────┬───────────┘  └────────┬─────────┘
           │                             │                       │
           └─────────────┬───────────────┴───────────────────────┘
                         │
           ┌─────────────▼──────────────┐
           │   Resilient Wrapper        │
           │  - Rate Limiting           │
           │  - Retry Logic             │
           │  - Circuit Breaker         │
           │  - Timeout Handling        │
           └────────────────────────────┘
```

---

## Assertion Pipeline

```
Test Response
     │
     ▼
┌─────────────────────────────────────┐
│   Assertion Registry                │
│   Loads configured assertions       │
└──────────────┬──────────────────────┘
               │
    ┌──────────┴──────────┬──────────────────┬──────────────────┐
    │                     │                  │                  │
    ▼                     ▼                  ▼                  ▼
┌─────────┐       ┌──────────────┐   ┌──────────────┐   ┌──────────┐
│ Exact   │       │   Regex      │   │   Semantic   │   │  Custom  │
│ Match   │       │   Pattern    │   │  Similarity  │   │ Function │
└────┬────┘       └──────┬───────┘   └──────┬───────┘   └────┬─────┘
     │                   │                  │                 │
     └───────────┬───────┴──────────────────┴─────────────────┘
                 │
                 ▼
         ┌───────────────┐
         │   Evaluator   │
         │  Aggregates   │
         │   Results     │
         └───────┬───────┘
                 │
                 ▼
         ┌───────────────┐
         │  Test Result  │
         │  - passed     │
         │  - score      │
         │  - details    │
         └───────────────┘
```

---

## Configuration Resolution Flow

```
CLI Arguments
     │
     ▼
Environment Variables
     │
     ▼
Project Config (.ltb/config.yaml)
     │
     ▼
User Config (~/.ltb/config.yaml)
     │
     ▼
System Config (/etc/ltb/config.yaml)
     │
     ▼
Default Values
     │
     └────────────────────────┐
                              │
                              ▼
                     ┌─────────────────┐
                     │  Config Merger  │
                     │  (Deep Merge)   │
                     └────────┬────────┘
                              │
                              ▼
                     ┌─────────────────┐
                     │ Schema Validator│
                     │   (Zod/JSON)    │
                     └────────┬────────┘
                              │
                              ▼
                     ┌─────────────────┐
                     │Reference Resolver│
                     │ ${env:...}      │
                     │ ${vault:...}    │
                     │ ${file:...}     │
                     └────────┬────────┘
                              │
                              ▼
                     ┌─────────────────┐
                     │  Final Config   │
                     └─────────────────┘
```

---

## Caching Architecture

```
                    Test Request
                         │
                         ▼
                ┌─────────────────┐
                │   Cache Check   │
                │  (Content Hash) │
                └────────┬────────┘
                         │
              ┌──────────┴──────────┐
              │                     │
           HIT│                     │MISS
              │                     │
              ▼                     ▼
    ┌─────────────────┐   ┌──────────────────┐
    │ Return Cached   │   │  Execute Test    │
    │    Result       │   │                  │
    └─────────────────┘   └────────┬─────────┘
                                   │
                                   ▼
                          ┌──────────────────┐
                          │   Store Result   │
                          │                  │
                          │  Memory Cache    │
                          │      ↓           │
                          │  Disk Cache      │
                          │   (SQLite)       │
                          └──────────────────┘

Cache Key Generation:
┌────────────────────────────────────────┐
│  hash({                                │
│    provider,                           │
│    model,                              │
│    messages,                           │
│    parameters,                         │
│    assertions                          │
│  })                                    │
└────────────────────────────────────────┘
```

---

## Parallel Execution Model

```
Test Suite (100 tests)
         │
         ▼
    ┌─────────┐
    │ Sharding│
    │  (N=4)  │
    └────┬────┘
         │
    ┌────┴────┬────────┬────────┐
    │         │        │        │
    ▼         ▼        ▼        ▼
┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐
│Worker │ │Worker │ │Worker │ │Worker │
│  #1   │ │  #2   │ │  #3   │ │  #4   │
│(25    │ │(25    │ │(25    │ │(25    │
│tests) │ │tests) │ │tests) │ │tests) │
└───┬───┘ └───┬───┘ └───┬───┘ └───┬───┘
    │         │         │         │
    └────┬────┴────┬────┴────┬────┘
         │         │         │
         ▼         ▼         ▼
    ┌──────────────────────────┐
    │   Result Aggregator      │
    │   - Merge results        │
    │   - Calculate metrics    │
    │   - Generate report      │
    └──────────────────────────┘

Worker Pool Management:
┌────────────────────────────────┐
│  Semaphore (Max Concurrency)  │
│  ┌──┐ ┌──┐ ┌──┐ ┌──┐          │
│  │✓ │ │✓ │ │✓ │ │✓ │          │
│  └──┘ └──┘ └──┘ └──┘          │
│                                │
│  Queue: [T5, T6, T7, ...]      │
└────────────────────────────────┘
```

---

## Plugin System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Plugin Manager                           │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Provider   │  │  Assertion   │  │   Reporter   │     │
│  │   Registry   │  │   Registry   │  │   Registry   │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         │                 │                 │             │
└─────────┼─────────────────┼─────────────────┼─────────────┘
          │                 │                 │
          │                 │                 │
    ┌─────┴─────┐     ┌─────┴─────┐     ┌─────┴─────┐
    │           │     │           │     │           │
    ▼           ▼     ▼           ▼     ▼           ▼
┌────────┐  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
│Built-in│  │Custom  │ │Built-in│ │Custom  │ │Built-in│ │Custom  │
│OpenAI  │  │Plugin  │ │Regex   │ │Plugin  │ │  JSON  │ │Plugin  │
└────────┘  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘

Plugin Lifecycle:
   Load → Initialize → Register → Use → Destroy
     │         │          │        │       │
     ▼         ▼          ▼        ▼       ▼
  import()  .init()  .register() .exec() .destroy()
```

---

## Error Handling Flow

```
                      Operation
                          │
                          ▼
                    ┌──────────┐
                    │   Try    │
                    └────┬─────┘
                         │
              ┌──────────┴──────────┐
              │                     │
          Success                 Error
              │                     │
              ▼                     ▼
         ┌─────────┐         ┌──────────────┐
         │ Return  │         │ Error Handler│
         │ Result  │         └──────┬───────┘
         └─────────┘                │
                            ┌───────┴────────┬───────────┐
                            │                │           │
                      Retryable?       User Error   System Error
                            │                │           │
                      ┌─────┴─────┐          │           │
                      │           │          │           │
                   YES           NO          │           │
                      │           │          │           │
                      ▼           ▼          ▼           ▼
                ┌──────────┐  ┌────────┐ ┌────────┐ ┌────────┐
                │  Retry   │  │  Log & │ │Show    │ │  Log & │
                │  Logic   │  │  Fail  │ │Helpful │ │  Fail  │
                │          │  │        │ │Message │ │        │
                └────┬─────┘  └────────┘ └────────┘ └────────┘
                     │
                     └───► Back to Try (with backoff)

Error Categories:
┌──────────────────────────────────────────┐
│ E1xx: Configuration Errors               │
│   - Missing fields                       │
│   - Invalid types                        │
│   - Schema violations                    │
├──────────────────────────────────────────┤
│ E2xx: Provider Errors                    │
│   - Authentication failed                │
│   - Rate limited                         │
│   - Service unavailable                  │
├──────────────────────────────────────────┤
│ E3xx: Assertion Errors                   │
│   - Assertion failed                     │
│   - Invalid assertion type               │
│   - Plugin error                         │
├──────────────────────────────────────────┤
│ E4xx: System Errors                      │
│   - Timeout                              │
│   - File not found                       │
│   - Permission denied                    │
└──────────────────────────────────────────┘
```

---

## Reporting Pipeline

```
Test Results
     │
     ▼
┌─────────────────┐
│    Metrics      │
│   Collector     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Aggregator    │
│  - Group by tag │
│  - Calculate %  │
│  - Timing stats │
└────────┬────────┘
         │
    ┌────┴────┬────────┬────────┐
    │         │        │        │
    ▼         ▼        ▼        ▼
┌───────┐ ┌───────┐ ┌──────┐ ┌──────┐
│ JSON  │ │ HTML  │ │ MD   │ │JUnit │
│Report │ │Report │ │Report│ │ XML  │
└───┬───┘ └───┬───┘ └──┬───┘ └──┬───┘
    │         │        │        │
    └────┬────┴────┬───┴────┬───┘
         │         │        │
         ▼         ▼        ▼
    ┌────────────────────────┐
    │   Export Destinations  │
    │  - File System         │
    │  - Slack Webhook       │
    │  - S3 Bucket           │
    │  - CI/CD System        │
    └────────────────────────┘
```

---

## State Management

```
┌─────────────────────────────────────────────────────────┐
│                    Test Suite State                     │
│                                                         │
│  ┌────────────────┐        ┌────────────────┐          │
│  │  Global State  │        │   Test State   │          │
│  │  (Shared)      │        │   (Per Test)   │          │
│  │                │        │                │          │
│  │ - config       │        │ - request      │          │
│  │ - providers    │        │ - response     │          │
│  │ - cache        │        │ - assertions   │          │
│  │ - metrics      │        │ - result       │          │
│  └────────────────┘        └────────────────┘          │
│                                                         │
│  ┌────────────────────────────────────────────┐        │
│  │           Shared Variables                 │        │
│  │  (Cross-test data sharing)                 │        │
│  │                                            │        │
│  │  authToken = "..."                         │        │
│  │  userId = "..."                            │        │
│  │  previousResponse = {...}                  │        │
│  └────────────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────┘

State Access Pattern:
Test 1: Login
  ↓ Sets state.authToken
Test 2: Get Profile
  ↓ Uses ${state.authToken}
Test 3: Update Profile
  ↓ Uses ${state.authToken}
```

---

## Security Layers

```
┌─────────────────────────────────────────────────────────┐
│                    Input Layer                          │
│  - Schema Validation                                    │
│  - Type Checking                                        │
│  - Sanitization                                         │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│                 Secrets Management                      │
│  - Environment Variables (encrypted)                    │
│  - Vault Integration                                    │
│  - Secret Rotation                                      │
│  - Never log secrets                                    │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│                 Execution Sandbox                       │
│  - Custom code in VM                                    │
│  - Limited system access                                │
│  - Timeout constraints                                  │
│  - Resource limits                                      │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│                  Network Security                       │
│  - HTTPS only                                           │
│  - Certificate validation                               │
│  - Request signing                                      │
│  - Rate limiting                                        │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│                   Audit Layer                           │
│  - All actions logged                                   │
│  - Sensitive data masked                                │
│  - Immutable audit trail                                │
│  - Compliance reporting                                 │
└─────────────────────────────────────────────────────────┘
```

---

## File Structure

```
llm-test-bench/
│
├── src/
│   ├── cli/                      # CLI interface
│   │   ├── commands/             # Command implementations
│   │   │   ├── run.ts
│   │   │   ├── init.ts
│   │   │   ├── report.ts
│   │   │   └── validate.ts
│   │   ├── formatters/           # Output formatters
│   │   │   ├── text.ts
│   │   │   ├── json.ts
│   │   │   └── junit.ts
│   │   └── index.ts
│   │
│   ├── config/                   # Configuration system
│   │   ├── loader.ts
│   │   ├── validator.ts
│   │   ├── schema.ts
│   │   └── secrets-manager.ts
│   │
│   ├── core/                     # Core engine
│   │   ├── orchestrator.ts
│   │   ├── discovery.ts
│   │   ├── executor.ts
│   │   ├── state-manager.ts
│   │   ├── cache/
│   │   │   ├── cache-manager.ts
│   │   │   ├── memory-cache.ts
│   │   │   └── sqlite-cache.ts
│   │   └── worker-pool.ts
│   │
│   ├── providers/                # Provider abstraction
│   │   ├── base.ts
│   │   ├── registry.ts
│   │   ├── adapters/
│   │   │   ├── openai.ts
│   │   │   ├── anthropic.ts
│   │   │   ├── ollama.ts
│   │   │   └── custom.ts
│   │   └── resilience/
│   │       ├── rate-limiter.ts
│   │       ├── retry.ts
│   │       └── circuit-breaker.ts
│   │
│   ├── assertions/               # Assertion engine
│   │   ├── base.ts
│   │   ├── registry.ts
│   │   ├── evaluator.ts
│   │   ├── builtin/
│   │   │   ├── exact-match.ts
│   │   │   ├── contains.ts
│   │   │   ├── regex.ts
│   │   │   ├── json-schema.ts
│   │   │   └── semantic.ts
│   │   └── plugins/
│   │
│   ├── reporting/                # Reporting system
│   │   ├── reporters/
│   │   │   ├── json.ts
│   │   │   ├── html.ts
│   │   │   ├── markdown.ts
│   │   │   └── junit.ts
│   │   ├── metrics-collector.ts
│   │   ├── historical-storage.ts
│   │   └── comparator.ts
│   │
│   ├── plugins/                  # Plugin system
│   │   ├── plugin-manager.ts
│   │   └── plugin-types.ts
│   │
│   ├── utils/                    # Utilities
│   │   ├── logger.ts
│   │   ├── errors.ts
│   │   └── helpers.ts
│   │
│   └── index.ts                  # Main entry point
│
├── tests/                        # Test suite
│   ├── unit/
│   ├── integration/
│   └── e2e/
│
├── examples/                     # Example test files
│   ├── basic/
│   ├── advanced/
│   └── plugins/
│
├── docs/                         # Documentation
│   ├── getting-started.md
│   ├── configuration.md
│   ├── providers.md
│   ├── assertions.md
│   └── plugins.md
│
├── templates/                    # Config templates
│   ├── ltb.config.yaml
│   └── test.template.yaml
│
├── package.json
├── tsconfig.json
├── ARCHITECTURE.md
└── README.md
```

---

## Data Models (TypeScript Interfaces)

```typescript
// Core domain models

interface Test {
  id: string;
  name: string;
  description?: string;
  provider: string;
  model: string;
  messages: Message[];
  assertions: AssertionConfig[];
  parameters?: Record<string, unknown>;
  metadata: TestMetadata;
}

interface Message {
  role: 'system' | 'user' | 'assistant';
  content: string;
  name?: string;
}

interface AssertionConfig {
  type: string;
  name?: string;
  [key: string]: unknown;
}

interface TestMetadata {
  source: string;
  tags: string[];
  timeout?: number;
  retries?: number;
}

interface TestResult {
  testId: string;
  testName: string;
  passed: boolean;
  score: number;
  assertions: AssertionResult[];
  response: CompletionResponse;
  duration: number;
  error?: Error;
  timestamp: Date;
}

interface AssertionResult {
  type: string;
  name?: string;
  passed: boolean;
  score?: number;
  message: string;
  details?: Record<string, unknown>;
}

interface TestSummary {
  name: string;
  timestamp: Date;
  duration: number;
  total: number;
  passed: number;
  failed: number;
  skipped: number;
  passRate: number;
  tests: TestResult[];
  metrics: Metrics;
}

interface Metrics {
  performance: {
    totalDuration: number;
    avgDuration: number;
    minDuration: number;
    maxDuration: number;
    p50: number;
    p95: number;
    p99: number;
  };
  tokens: {
    total: number;
    prompt: number;
    completion: number;
  };
  costs: {
    estimated: number;
    byProvider: Map<string, number>;
  };
  success: {
    passRate: number;
    avgScore: number;
    byTag: Map<string, number>;
  };
}
```

---

This visual architecture documentation complements the main ARCHITECTURE.md file by providing clear diagrams that show:

1. **System layers** and their responsibilities
2. **Component interactions** and dependencies
3. **Data flow** through the system
4. **Execution models** for tests
5. **Error handling** strategies
6. **Security** implementation
7. **File organization** structure

These diagrams help developers understand the system architecture at a glance and serve as a reference during implementation.
