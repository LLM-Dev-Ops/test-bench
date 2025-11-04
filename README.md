# LLM Test Bench

A production-grade CLI framework for systematic testing, validation, and benchmarking of Large Language Model (LLM) applications.

## Status

[![CI](https://github.com/USERNAME/llm-test-bench/actions/workflows/ci.yml/badge.svg)](https://github.com/USERNAME/llm-test-bench/actions/workflows/ci.yml)
[![Security](https://github.com/USERNAME/llm-test-bench/actions/workflows/security.yml/badge.svg)](https://github.com/USERNAME/llm-test-bench/actions/workflows/security.yml)
[![codecov](https://codecov.io/gh/USERNAME/llm-test-bench/branch/main/graph/badge.svg)](https://codecov.io/gh/USERNAME/llm-test-bench)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Current Phase**: Phase 2 - Provider Integration ✅ (Milestone 2.4 Complete)

A production-ready CLI for testing and benchmarking LLM applications with support for OpenAI and Anthropic providers. The test command is now fully functional with streaming support, multiple output formats, and comprehensive error handling.

## Architecture Documentation

### Core Documents (Start Here)

1. **[ARCHITECTURE.md](./ARCHITECTURE.md)** (73KB)
   - Complete system architecture overview
   - All major components detailed
   - Interface specifications
   - Security and scalability considerations
   - **READ THIS FIRST**

2. **[ARCHITECTURE_DIAGRAMS.md](./ARCHITECTURE_DIAGRAMS.md)** (41KB)
   - Visual component diagrams
   - Data flow diagrams
   - Sequence diagrams
   - System layer architecture
   - **Great for visual learners**

3. **[IMPLEMENTATION_ROADMAP.md](./IMPLEMENTATION_ROADMAP.md)** (27KB)
   - 15-week phased implementation plan
   - Week-by-week breakdown
   - Code examples for each phase
   - Deliverables and milestones
   - **For developers ready to build**

4. **[DESIGN_DECISIONS.md](./DESIGN_DECISIONS.md)** (20KB)
   - Why we made key architectural choices
   - Technology selection rationale
   - Trade-offs and alternatives considered
   - Design patterns applied
   - **Understand the "why"**

5. **[QUICK_REFERENCE.md](./QUICK_REFERENCE.md)** (11KB)
   - Cheat sheet for developers
   - Common patterns and examples
   - CLI commands reference
   - Troubleshooting guide
   - **Keep this handy while coding**

### Market Research

6. **[MARKET_RESEARCH_REPORT.md](./MARKET_RESEARCH_REPORT.md)** (44KB)
   - Competitive analysis
   - Market landscape
   - User needs and pain points
   - Feature comparison

7. **[MARKET_INSIGHTS_SUPPLEMENT.md](./MARKET_INSIGHTS_SUPPLEMENT.md)** (26KB)
   - Additional market insights
   - Pricing strategies
   - Go-to-market analysis

8. **[EXECUTIVE_SUMMARY.md](./EXECUTIVE_SUMMARY.md)** (9.6KB)
   - High-level overview
   - Key value propositions
   - Target audience

## Quick Start (For Reviewers)

### 1. Understand the Vision (5 minutes)
```bash
# Read the executive summary
cat EXECUTIVE_SUMMARY.md
```

### 2. Learn the Architecture (30 minutes)
```bash
# Read the main architecture document
cat ARCHITECTURE.md

# View the visual diagrams
cat ARCHITECTURE_DIAGRAMS.md
```

### 3. See How to Build It (20 minutes)
```bash
# Review the implementation roadmap
cat IMPLEMENTATION_ROADMAP.md
```

### 4. Understand the Decisions (15 minutes)
```bash
# Read design decisions
cat DESIGN_DECISIONS.md
```

## Document Map

```
LLM Test Bench Documentation
│
├── README.md (You are here)
│
├── Architecture (Core)
│   ├── ARCHITECTURE.md ...................... Complete system design
│   ├── ARCHITECTURE_DIAGRAMS.md ............. Visual diagrams
│   └── QUICK_REFERENCE.md ................... Developer cheat sheet
│
├── Implementation
│   ├── IMPLEMENTATION_ROADMAP.md ............ 15-week build plan
│   └── DESIGN_DECISIONS.md .................. Why we chose this
│
└── Market Research
    ├── MARKET_RESEARCH_REPORT.md ............ Competitive analysis
    ├── MARKET_INSIGHTS_SUPPLEMENT.md ........ Additional insights
    └── EXECUTIVE_SUMMARY.md ................. High-level overview
```

## Key Features (Designed)

### 1. Multi-Provider Support
- OpenAI (GPT-4, GPT-3.5)
- Anthropic (Claude)
- Local models (Ollama, LM Studio)
- Custom providers via plugins

### 2. Rich Assertion Library
- Exact match
- Contains substring
- Regex patterns
- JSON schema validation
- Semantic similarity
- Custom assertions

### 3. Flexible Configuration
- YAML/JSON/TOML support
- Hierarchical config merging
- Environment variable interpolation
- Vault integration for secrets

### 4. Advanced Execution
- Parallel test execution
- Content-based caching
- Rate limiting
- Automatic retries
- Streaming support

### 5. Comprehensive Reporting
- JSON, HTML, Markdown formats
- JUnit XML for CI/CD
- Historical comparison
- Metrics dashboard
- Cost tracking

### 6. Developer Experience
- Intuitive CLI
- Excellent error messages
- Interactive mode
- Watch mode
- Hot reload

## Technology Stack

- **Runtime**: Node.js 18+
- **Language**: TypeScript
- **CLI**: yargs
- **Validation**: Zod
- **Testing**: Vitest
- **Build**: tsup

## Architecture Highlights

### Clean Architecture
- Dependency inversion
- Domain-driven design
- Plugin-based extensibility

### Security First
- No secrets in configs
- Sandboxed custom code
- Comprehensive audit logging

### Performance Optimized
- Two-level caching (memory + disk)
- Connection pooling
- Parallel execution
- Streaming responses

## Implementation Phases

1. **Phase 0**: Project foundation (Week 1)
2. **Phase 1**: Core foundation (Weeks 2-3)
3. **Phase 2**: Assertion engine (Weeks 4-5)
4. **Phase 3**: Test discovery & orchestration (Week 6)
5. **Phase 4**: Advanced features (Weeks 7-8)
6. **Phase 5**: Additional providers (Week 9)
7. **Phase 6**: Advanced assertions (Week 10)
8. **Phase 7**: Enhanced reporting (Week 11)
9. **Phase 8**: Plugin system (Week 12)
10. **Phase 9**: Polish & production ready (Weeks 13-14)
11. **Phase 10**: Launch (Week 15)

## Quick Start

### Prerequisites

1. Install Rust (1.75.0 or later)
2. Set up API keys:
   ```bash
   export OPENAI_API_KEY="your-key-here"
   export ANTHROPIC_API_KEY="your-key-here"
   ```

### Installation

```bash
# Clone the repository
git clone https://github.com/USERNAME/llm-test-bench
cd llm-test-bench

# Build the project
cargo build --release

# Install (optional)
cargo install --path cli
```

### Basic Usage

```bash
# Test with OpenAI
llm-test-bench test openai --prompt "Explain Rust ownership"

# Test with Anthropic Claude
llm-test-bench test anthropic --prompt "Write a haiku about programming"

# Stream responses in real-time
llm-test-bench test openai --prompt "Write a story" --stream

# Get JSON output for scripting
llm-test-bench test openai --prompt "What is 2+2?" --output-format json

# Use specific models and parameters
llm-test-bench test openai \
  --prompt "Generate a random number" \
  --model gpt-4 \
  --temperature 1.5 \
  --max-tokens 50

# Initialize configuration
llm-test-bench config init

# View current configuration
llm-test-bench config show
```

### Example Usage Scenarios

#### 1. Quick Testing
```bash
llm-test-bench test openai --prompt "Hello, world!" --output-format plain
```

#### 2. Streaming Mode
```bash
llm-test-bench test anthropic \
  --prompt "Write a short story" \
  --model claude-3-sonnet-20240229 \
  --stream
```

#### 3. JSON Output for Automation
```bash
llm-test-bench test openai \
  --prompt "List 5 programming languages" \
  --output-format json | jq '.content'
```

#### 4. Advanced Parameters
```bash
llm-test-bench test openai \
  --prompt "Generate creative text" \
  --model gpt-4-turbo \
  --temperature 1.2 \
  --top-p 0.95 \
  --max-tokens 200 \
  --stop "END"
```

### Available Commands

| Command | Description |
|---|---|
| `test` | Run a single test against an LLM provider |
| `bench` | Run benchmarks (coming in Phase 3) |
| `eval` | Evaluate test results (coming in Phase 4) |
| `config init` | Initialize configuration file |
| `config show` | Display current configuration |
| `completions <shell>` | Generate shell completions |

### Output Formats

- **pretty** (default): Human-readable with colors and formatting
- **json**: Compact JSON for piping
- **json-pretty**: Pretty-printed JSON
- **plain**: Just the content text

### Supported Providers

#### OpenAI
- ✅ GPT-4 (8K context)
- ✅ GPT-4 Turbo (128K context)
- ✅ GPT-3.5 Turbo (16K context)

#### Anthropic
- ✅ Claude 3 Opus (200K context)
- ✅ Claude 3 Sonnet (200K context)
- ✅ Claude 3 Haiku (200K context)
- ✅ Claude 3.5 Sonnet (200K context)

## Configuration Example

```yaml
# ltb.config.yaml
version: "1.0"
name: "My Test Suite"

providers:
  openai:
    type: openai
    apiKey: ${env:OPENAI_API_KEY}
    defaults:
      model: gpt-4
      temperature: 0.7

  anthropic:
    type: anthropic
    apiKey: ${env:ANTHROPIC_API_KEY}
    defaults:
      model: claude-sonnet-4

tests:
  include:
    - "tests/**/*.test.yaml"

cache:
  enabled: true
  ttl: 3600

reporting:
  formats: ["json", "html"]
  output: "./reports"
```

## Design Principles

1. **Developer Experience First** - Intuitive, helpful, delightful
2. **Correctness over Performance** - Get it right, then make it fast
3. **Security by Default** - Safe out of the box
4. **Extensibility** - Plugin architecture for everything
5. **Convention over Configuration** - Smart defaults, minimal setup
6. **Fail-Fast with Graceful Degradation** - Catch errors early, recover gracefully

## Project Status

### Phase 2: Provider Integration ✅

- [x] **Phase 1: Project Foundation** ✅
  - [x] Cargo workspace structure
  - [x] Configuration system with validation
  - [x] CLI scaffolding with clap
  - [x] Error handling infrastructure
  - [x] Basic documentation

- [x] **Phase 2: Provider Integration** ✅
  - [x] **Milestone 2.1**: Provider trait abstraction
  - [x] **Milestone 2.2**: OpenAI integration (complete + streaming)
  - [x] **Milestone 2.3**: Anthropic integration (complete + streaming)
  - [x] **Milestone 2.4**: CLI test command ✅ **LATEST**
    - [x] Provider factory implementation
    - [x] Streaming UI with progress indicators
    - [x] 4 output formats (Pretty, JSON, JsonPretty, Plain)
    - [x] User-friendly error messages
    - [x] 25+ integration tests
    - [x] Complete documentation

- [ ] **Phase 3: Benchmarking System** (Next)
- [ ] **Phase 4: Evaluation Metrics**
- [ ] **Phase 5: Advanced Features**

📄 **See [docs/milestone-2.4-implementation-report.md](./docs/milestone-2.4-implementation-report.md) for complete Phase 2.4 details**

## CI/CD Pipeline

The project includes a production-ready CI/CD pipeline with:

- **Continuous Integration**: Automated testing, linting, and type checking on every PR
- **Security Scanning**: Daily vulnerability scans and secret detection
- **Code Coverage**: 80% minimum coverage enforced (target: 90%)
- **Automated Releases**: Tag-based releases with automatic npm publishing
- **Quality Gates**: Zero warnings policy, strict type checking, format verification

📚 **See [docs/CI_CD.md](./docs/CI_CD.md) for complete documentation**

### Quick Commands

```bash
# Run all CI checks locally
npm run ci

# Individual checks
npm run lint          # ESLint with zero warnings
npm run typecheck     # TypeScript validation
npm test              # Run test suite
npm run test:coverage # Generate coverage report
npm run format:check  # Verify formatting
npm run build         # Build package

# Or use Makefile shortcuts
make ci               # Run all checks
make test             # Run tests
make coverage         # Generate and open coverage report
```

## Contributing

We welcome contributions! Please read our [Contributing Guide](./CONTRIBUTING.md) for:

- Development setup and workflow
- Code quality standards
- Testing requirements
- PR submission process
- Commit message conventions

### For Contributors

1. Read [CONTRIBUTING.md](./CONTRIBUTING.md)
2. Review [docs/CI_CD.md](./docs/CI_CD.md) for CI/CD pipeline details
3. Follow [docs/GITHUB_ACTIONS_SETUP.md](./docs/GITHUB_ACTIONS_SETUP.md) for setup
4. Write tests first (TDD approach)
5. Ensure all CI checks pass: `npm run ci`
6. Submit PR with clear description

## License

MIT License - See LICENSE file

## Author

Systems Architect: Claude (Anthropic)

---

**Note**: This is an architectural design. Implementation follows the roadmap in IMPLEMENTATION_ROADMAP.md.

## Next Steps

### For Architects/Reviewers
1. Review ARCHITECTURE.md for completeness
2. Validate design decisions in DESIGN_DECISIONS.md
3. Assess feasibility of IMPLEMENTATION_ROADMAP.md

### For Developers
1. Read all architecture docs
2. Set up development environment (Phase 0)
3. Start with Phase 1: Core foundation
4. Follow TDD approach
5. Submit PRs incrementally

### For Product Managers
1. Review EXECUTIVE_SUMMARY.md
2. Read MARKET_RESEARCH_REPORT.md
3. Validate feature priorities
4. Define success metrics

---

**Total Documentation**: ~250KB of comprehensive architectural design

**Time to Implementation**: ~15 weeks for v1.0

**Designed for**: Production-grade LLM testing at scale
