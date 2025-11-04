<div align="center">

# 🧪 LLM Test Bench

**A comprehensive, production-ready framework for benchmarking, testing, and evaluating Large Language Models**

[![CI](https://img.shields.io/github/actions/workflow/status/globalbusinessadvisors/llm-test-bench/llm-benchmark.yml?branch=main&label=CI&logo=github)](https://github.com/globalbusinessadvisors/llm-test-bench/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange)](https://crates.io)

[Features](#-features) • [Quick Start](#-quick-start) • [Documentation](#-documentation) • [Architecture](#-architecture) • [Contributing](#-contributing)

</div>

---

## 📖 Overview

LLM Test Bench is a powerful, enterprise-grade framework built in Rust for comprehensive testing, benchmarking, and evaluation of Large Language Models. It provides a unified interface to test multiple LLM providers, evaluate responses with sophisticated metrics, and visualize results through an intuitive dashboard.

### Why LLM Test Bench?

- **🚀 Multi-Provider Support**: Test 14+ LLM providers through a single, unified interface
- **📊 Comprehensive Metrics**: Evaluate models with perplexity, coherence, relevance, faithfulness, and custom evaluators
- **⚡ High Performance**: Built in Rust for speed, safety, and scalability
- **🎨 Rich Visualization**: Interactive dashboards with real-time metrics and beautiful charts
- **🔌 Extensible**: Plugin system, custom evaluators, and distributed computing support
- **🐳 Production Ready**: Docker support, monitoring, REST/GraphQL APIs, and WebSocket streaming

---

## ✨ Features

### Core Capabilities

#### 🤖 Multi-Provider LLM Support

**OpenAI (22+ models)**
- GPT-4o, GPT-4o-mini (Latest multimodal flagships)
- o1, o1-preview, o1-mini (Advanced reasoning models)
- GPT-4 Turbo, GPT-4 (128K context)
- GPT-3.5 Turbo

**Anthropic (11+ models)**
- Claude Sonnet 4 (Latest generation)
- Claude 3.5 Sonnet, Claude 3.5 Haiku (Current flagships)
- Claude 3 Opus, Sonnet, Haiku (200K context)

**Google Gemini (13+ models)**
- Gemini 2.0 Flash (Latest with extended thinking)
- Gemini 1.5 Pro, Flash (2M+ token context)
- Gemini Pro Vision (Multimodal)

**Additional Providers**
- **Azure OpenAI**: Enterprise-grade OpenAI models
- **AWS Bedrock**: Claude, Llama, Titan, and more
- **Open Source**: Ollama, Hugging Face, Together AI, Replicate
- **Specialized**: Cohere, Mistral AI, Groq, Perplexity AI

#### 📈 Advanced Evaluation Metrics
- **Perplexity Analysis**: Statistical language model evaluation
- **Coherence Scoring**: Semantic consistency and logical flow
- **Relevance Evaluation**: Context-aware response quality
- **Faithfulness Testing**: Source attribution and hallucination detection
- **LLM-as-Judge**: Use LLMs to evaluate other LLMs
- **Text Analysis**: Readability, sentiment, toxicity, PII detection
- **Custom Evaluators**: Build your own evaluation metrics

#### 🎯 Benchmarking & Testing
- **Systematic Testing**: Automated test suites with rich assertions
- **Comparative Analysis**: Side-by-side model comparison
- **Performance Profiling**: Latency, throughput, and cost tracking
- **A/B Testing**: Statistical significance testing for model selection
- **Optimization Tools**: Automatic parameter tuning and model recommendation

#### 📊 Visualization & Reporting
- **Interactive Dashboard**: Real-time metrics with Chart.js
- **Rich Charts**: Performance graphs, cost analysis, trend visualization
- **Multiple Formats**: HTML reports, JSON exports, custom templates
- **Cost Analysis**: Track spending across providers and models
- **Historical Trends**: Long-term performance tracking

#### 🌐 API & Integration
- **REST API**: Complete HTTP API with authentication
- **GraphQL**: Flexible query interface for complex data needs
- **WebSocket**: Real-time streaming and live updates
- **Monitoring**: Prometheus metrics and health checks
- **Distributed Computing**: Scale benchmarks across multiple nodes

#### 🔌 Extensibility
- **Plugin System**: WASM-based sandboxed plugins
- **Custom Evaluators**: Implement domain-specific metrics
- **Multimodal Support**: Image, audio, and video evaluation
- **Database Backend**: PostgreSQL with repository pattern
- **Flexible Architecture**: Clean, modular design for easy extension

---

## 🚀 Quick Start

### Prerequisites

- **Rust**: 1.75.0 or later ([Install Rust](https://rustup.rs/))
- **API Keys**: At least one LLM provider API key

### Installation

```bash
# Clone the repository
git clone https://github.com/globalbusinessadvisors/llm-test-bench.git
cd llm-test-bench

# Build the project
cargo build --release

# Install CLI globally (optional)
cargo install --path cli
```

### Configuration

Set up your API keys as environment variables:

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."

# Google
export GOOGLE_API_KEY="..."

# AWS Bedrock
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."
export AWS_REGION="us-east-1"
```

Or create a `.env` file:

```bash
cp .env.example .env
# Edit .env with your API keys
```

### Basic Usage

```bash
# Run a simple benchmark
llm-test-bench bench --provider openai --model gpt-4 --prompt "Explain quantum computing"

# Compare multiple models
llm-test-bench compare \
  --models "openai:gpt-4,anthropic:claude-3-sonnet" \
  --prompt "Write a Python function to sort a list"

# Analyze results
llm-test-bench analyze --results benchmark_results.json

# Launch interactive dashboard
llm-test-bench dashboard --port 8080

# Optimize model selection
llm-test-bench optimize \
  --metric latency \
  --max-cost 0.01 \
  --dataset prompts.json
```

### Docker Deployment

```bash
# Using Docker Compose (includes PostgreSQL, Redis, Prometheus)
docker-compose up -d

# Access the dashboard
open http://localhost:8080

# View metrics
open http://localhost:9090  # Prometheus
```

---

## 📚 Documentation

### Getting Started
- [Quick Start Guide](docs/QUICKSTART_PHASE4.md) - Get up and running in 5 minutes
- [CLI Reference](docs/CLI_REFERENCE.md) - Complete command-line documentation
- [Configuration Guide](docs/CONFIGURATION.md) - Advanced configuration options

### Architecture & Design
- [Architecture Overview](docs/ARCHITECTURE_REPORT.md) - System design and components
- [Workspace Structure](docs/WORKSPACE_STRUCTURE.md) - Project organization
- [Technical Architecture](plans/PHASE5_TECHNICAL_ARCHITECTURE.md) - Deep dive into design

### Features
- [Provider Support](docs/PROVIDERS.md) - All supported LLM providers
- [API Documentation](docs/API.md) - REST & GraphQL API reference
- [Monitoring](docs/MONITORING.md) - Observability and metrics
- [Distributed Computing](docs/DISTRIBUTED.md) - Scaling across nodes
- [Multimodal](docs/MULTIMODAL.md) - Image, audio, and video support
- [Plugins](docs/PLUGINS.md) - Extensibility and custom plugins

### Deployment
- [Docker Deployment](docs/DOCKER_DEPLOYMENT.md) - Containerized deployment guide
- [Database Setup](docs/DATABASE.md) - PostgreSQL configuration

### Development
- [Phase Implementation Reports](docs/) - Detailed implementation history
- [Contributing Guide](CONTRIBUTING.md) - How to contribute
- [Development Setup](docs/DEVELOPMENT.md) - Set up your dev environment

---

## 🏗️ Architecture

LLM Test Bench follows a clean, modular architecture:

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI Layer                            │
│  bench │ compare │ analyze │ dashboard │ optimize          │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                     Core Library (core/)                    │
├─────────────────────────────────────────────────────────────┤
│  • Providers      • Evaluators     • Orchestration          │
│  • Analytics      • Visualization  • Monitoring             │
│  • Distributed    • Plugins        • Multimodal             │
│  • API Server     • Database       • Configuration          │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    External Services                        │
│  LLM APIs │ PostgreSQL │ Redis │ Prometheus │ S3            │
└─────────────────────────────────────────────────────────────┘
```

### Key Components

- **Providers**: Unified interface for 14+ LLM providers
- **Evaluators**: Pluggable metrics for response quality assessment
- **Orchestration**: Intelligent routing, ranking, and comparison
- **Visualization**: Interactive dashboards and rich reporting
- **API Server**: REST, GraphQL, and WebSocket endpoints
- **Distributed**: Cluster coordination for large-scale benchmarks
- **Monitoring**: Prometheus metrics and health checks
- **Plugins**: WASM-based extensibility system

---

## 🛠️ Technology Stack

- **Language**: Rust 🦀
- **CLI**: Clap (command-line parsing)
- **Async**: Tokio (async runtime)
- **HTTP**: Axum (web framework)
- **Database**: SQLx + PostgreSQL
- **Serialization**: Serde (JSON/YAML)
- **GraphQL**: Async-GraphQL
- **Monitoring**: Prometheus client
- **WebSocket**: Tokio-Tungstenite
- **Distributed**: Custom protocol over TCP
- **Plugins**: Wasmtime (WASM runtime)

---

## 📦 Project Structure

```
llm-test-bench/
├── cli/                    # Command-line interface
│   ├── src/
│   │   ├── commands/      # CLI commands (bench, compare, etc.)
│   │   └── main.rs
│   └── tests/             # Integration tests
├── core/                   # Core library
│   ├── src/
│   │   ├── providers/     # LLM provider implementations
│   │   ├── evaluators/    # Evaluation metrics
│   │   ├── orchestration/ # Model routing & comparison
│   │   ├── visualization/ # Dashboard & charts
│   │   ├── api/           # REST/GraphQL/WebSocket
│   │   ├── distributed/   # Cluster coordination
│   │   ├── monitoring/    # Metrics & health checks
│   │   ├── plugins/       # Plugin system
│   │   ├── multimodal/    # Image/audio/video
│   │   ├── analytics/     # Statistics & optimization
│   │   └── config/        # Configuration
│   └── tests/             # Unit & integration tests
├── docs/                   # Documentation
├── examples/               # Usage examples
├── plans/                  # Architecture & planning docs
└── docker-compose.yml      # Docker deployment
```

---

## 🎯 Use Cases

### 1. Model Selection
Compare multiple LLM providers to choose the best model for your use case based on quality, cost, and latency.

### 2. Quality Assurance
Systematic testing of LLM applications with rich assertions and automated evaluation metrics.

### 3. Performance Benchmarking
Measure and track latency, throughput, and cost across different models and configurations.

### 4. Regression Testing
Ensure model updates don't degrade quality with historical comparison and automated alerts.

### 5. Cost Optimization
Identify the most cost-effective model that meets your quality requirements.

### 6. Research & Experimentation
Rapid prototyping and comparison of different prompts, models, and parameters.

---

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone and build
git clone https://github.com/globalbusinessadvisors/llm-test-bench.git
cd llm-test-bench
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- bench --help

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings
```

### Areas for Contribution

- 🔌 New LLM provider integrations
- 📊 Additional evaluation metrics
- 🎨 Visualization improvements
- 📝 Documentation enhancements
- 🐛 Bug fixes and performance improvements
- ✨ New features and capabilities

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) 🦀
- Inspired by the need for comprehensive LLM testing tools
- Thanks to all contributors and the open-source community

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/globalbusinessadvisors/llm-test-bench/issues)
- **Discussions**: [GitHub Discussions](https://github.com/globalbusinessadvisors/llm-test-bench/discussions)
- **Documentation**: [docs/](docs/)

---

## 🗺️ Roadmap

### Completed ✅
- ✅ Multi-provider LLM support (14+ providers)
- ✅ Advanced evaluation metrics
- ✅ Visualization dashboard
- ✅ REST/GraphQL/WebSocket APIs
- ✅ Distributed computing
- ✅ Monitoring & observability
- ✅ Plugin system
- ✅ Docker deployment
- ✅ PostgreSQL backend

### In Progress 🚧
- 🚧 Enhanced multimodal support
- 🚧 Advanced cost optimization
- 🚧 Plugin marketplace
- 🚧 Cloud deployment templates

### Planned 📋
- 📋 Real-time collaboration features
- 📋 Advanced A/B testing framework
- 📋 Integration with MLOps platforms
- 📋 Enterprise SSO and RBAC

---

<div align="center">

**⭐ Star us on GitHub — it motivates us a lot!**

[Report Bug](https://github.com/globalbusinessadvisors/llm-test-bench/issues) • [Request Feature](https://github.com/globalbusinessadvisors/llm-test-bench/issues) • [Documentation](docs/)

Made with ❤️ by the LLM Test Bench Team

</div>
