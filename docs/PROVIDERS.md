# LLM Provider Support

LLM Test Bench provides comprehensive support for **13 LLM providers**, covering all major commercial APIs, open-source platforms, and local deployment options.

## Supported Providers

| Provider | Models | Streaming | Context Window | Use Case |
|----------|--------|-----------|----------------|----------|
| **OpenAI** | GPT-5, GPT-4.5, GPT-4.1, o3-mini, GPT-4o (29+ models) | ✅ | Up to 128K | Industry-leading quality, reasoning |
| **Anthropic** | Claude Opus 4, Sonnet 4.5, Sonnet 4, Claude 3.5/3 (17+ models) | ✅ | Up to 200K | Long context, safety-focused |
| **Google AI** | Gemini 2.5, Gemini 2.0, Gemini 1.5 Pro/Flash (16+ models) | ✅ | Up to 2M | Multimodal, massive context, computer use |
| **Mistral AI** | Mistral Code, Magistral, Voxtral (7+ models) | ✅ | Varies | Code generation, multilingual, audio |
| **Cohere** | Command, Command R/R+ | ✅ | Up to 128K | Enterprise RAG, search |
| **Azure OpenAI** | GPT-4, GPT-3.5 (Azure-hosted) | ✅ | Up to 128K | Enterprise compliance |
| **Groq** | Llama 3, Mixtral, Gemma | ✅ | Up to 32K | Ultra-fast inference |
| **Together AI** | Llama 2/3, Mixtral, CodeLlama | ✅ | Up to 32K | Open models at scale |
| **Hugging Face** | 100K+ models | ❌ | Varies | Research, experimentation |
| **Ollama** | Local models (Llama, Mistral, etc.) | ✅ | Varies | Privacy, offline use |
| **AWS Bedrock** | Claude, Titan, Llama 2, Command | ✅ | Varies | AWS ecosystem integration |
| **Replicate** | Llama 2, Mixtral, SDXL | ❌ | Varies | Easy deployment, GPUs |
| **Perplexity AI** | pplx-7b/70b, Online models | ✅ | Up to 16K | Search-augmented responses |

---

## Provider Details

### 1. OpenAI

**Most popular commercial LLM provider with industry-leading models.**

#### Supported Models (29+ models)

**GPT-5 Series (Latest Generation - August 2025)**
- `gpt-5` - Most advanced OpenAI model (Released Aug 7, 2025)

**GPT-4.5 Series (February 2025)**
- `gpt-4.5` - Enhanced GPT-4 variant (Released Feb 27, 2025)
- `gpt-4.5-2025-02-27` - February 2025 snapshot

**GPT-4.1 Series (April 2025)**
- `gpt-4.1` - Improved GPT-4 variant (Released Apr 2025)
- `gpt-4.1-2025-04` - April 2025 snapshot

**GPT-4o Series (Multimodal Flagship)**
- `gpt-4o` - Latest GPT-4o (128K context, vision, audio)
- `gpt-4o-2024-11-20` - November 2024 snapshot
- `gpt-4o-2024-08-06` - August 2024 snapshot
- `gpt-4o-2024-05-13` - May 2024 snapshot
- `gpt-4o-mini` - Faster, more affordable variant (128K context)
- `gpt-4o-mini-2024-07-18` - July 2024 snapshot

**o-Series (Advanced Reasoning Models)**
- `o3-mini` - Latest reasoning model (Released Jan 2025)
- `o1` - Advanced reasoning model
- `o1-preview` - Preview of o1 capabilities
- `o1-preview-2024-09-12` - September 2024 snapshot
- `o1-mini` - Smaller, faster reasoning model
- `o1-mini-2024-09-12` - September 2024 snapshot

**GPT-4 Turbo Series**
- `gpt-4-turbo` - Latest GPT-4 Turbo (128K context)
- `gpt-4-turbo-2024-04-09` - April 2024 snapshot
- `gpt-4-turbo-preview` - Preview features
- `gpt-4-0125-preview` - January 2025 preview
- `gpt-4-1106-preview` - November 2023 preview

**GPT-4 Classic**
- `gpt-4` - Original GPT-4 (8K context)
- `gpt-4-0613` - June 2023 snapshot

**GPT-3.5 Turbo**
- `gpt-3.5-turbo` - Fast, cost-effective (16K context)
- `gpt-3.5-turbo-0125` - January 2025 snapshot
- `gpt-3.5-turbo-1106` - November 2023 snapshot

#### Configuration
```toml
[[providers]]
name = "openai"
api_key_env = "OPENAI_API_KEY"
base_url = "https://api.openai.com/v1"
default_model = "gpt-4-turbo"
timeout_seconds = 120
max_retries = 3
```

#### CLI Usage
```bash
# Run benchmark with OpenAI
llm-test-bench bench \
  --dataset coding-tasks.json \
  --providers openai \
  --models gpt-4,gpt-3.5-turbo

# Compare GPT-4 vs GPT-3.5
llm-test-bench compare \
  --prompt "Explain quantum computing" \
  --models openai:gpt-4,openai:gpt-3.5-turbo
```

---

### 2. Anthropic

**Safety-focused provider with industry-leading context windows.**

#### Supported Models (17+ models)

**Claude Opus 4 (Latest Generation - May 2025)**
- `claude-opus-4` - Most capable Claude model (Released May 2025)
- `claude-opus-4-20250501` - May 2025 snapshot

**Claude Sonnet 4.5 (Latest Flagship - September 2025)**
- `claude-sonnet-4.5` - Latest Claude Sonnet (Released Sept 2025)
- `claude-sonnet-4.5-20250901` - September 2025 snapshot

**Claude Sonnet 4 (May 2025)**
- `claude-sonnet-4` - Claude Sonnet 4 (Released May 2025)
- `claude-sonnet-4-20250514` - May 2025 snapshot

**Claude 3.5 Series**
- `claude-3-5-sonnet-latest` - Latest Claude 3.5 Sonnet (200K context)
- `claude-3-5-sonnet-20241022` - October 2024 snapshot
- `claude-3-5-sonnet-20240620` - June 2024 snapshot
- `claude-3-5-haiku-latest` - Latest Claude 3.5 Haiku (fast, affordable)
- `claude-3-5-haiku-20241022` - October 2024 snapshot

**Claude 3 Series**
- `claude-3-opus-latest` - Most capable Claude 3 (200K context)
- `claude-3-opus-20240229` - February 2024 snapshot
- `claude-3-sonnet-20240229` - Balanced performance (200K)
- `claude-3-haiku-20240307` - Fast, affordable (200K)

#### Configuration
```toml
[[providers]]
name = "anthropic"
api_key_env = "ANTHROPIC_API_KEY"
base_url = "https://api.anthropic.com"
default_model = "claude-3-5-sonnet-20240620"
timeout_seconds = 120
max_retries = 3
```

#### CLI Usage
```bash
# Benchmark with Anthropic
llm-test-bench bench \
  --dataset legal-docs.json \
  --providers anthropic \
  --models claude-3-opus-20240229,claude-3-sonnet-20240229

# Long-context testing
llm-test-bench compare \
  --prompt "$(cat long-document.txt)" \
  --models anthropic:claude-3-opus-20240229
```

---

### 3. Google AI (Gemini)

**Google's multimodal models with massive context windows.**

#### Supported Models (16+ models)

**Gemini 2.5 Series (Latest Generation - 2025)**
- `gemini-2.5-pro` - Most advanced Gemini model (Released 2025)
- `gemini-2.5-computer-use` - Autonomous agent with computer control (Released Oct 7, 2025)
- `gemini-2.5-computer-use-20251007` - October 2025 snapshot

**Gemini 2.0 Series**
- `gemini-2.0-flash-exp` - Experimental Gemini 2.0 Flash (1M+ context)
- `gemini-2.0-flash-thinking-exp-1219` - Extended thinking mode (experimental)

**Gemini 1.5 Pro (Extended Context)**
- `gemini-1.5-pro` - Latest Gemini 1.5 Pro (2M token context)
- `gemini-1.5-pro-latest` - Always-latest pointer
- `gemini-1.5-pro-002` - Version 002 (stable)
- `gemini-1.5-pro-001` - Version 001

**Gemini 1.5 Flash (Fast Variant)**
- `gemini-1.5-flash` - Latest Gemini 1.5 Flash (1M token context)
- `gemini-1.5-flash-latest` - Always-latest pointer
- `gemini-1.5-flash-002` - Version 002 (stable)
- `gemini-1.5-flash-001` - Version 001
- `gemini-1.5-flash-8b` - Lightweight 8B parameter model

**Gemini 1.0 Series**
- `gemini-pro` - Original Gemini Pro (30K context)
- `gemini-pro-vision` - Multimodal capabilities (30K context)

#### Configuration
```toml
[[providers]]
name = "google"
api_key_env = "GOOGLE_API_KEY"
base_url = "https://generativelanguage.googleapis.com/v1beta"
default_model = "gemini-1.5-pro"
timeout_seconds = 120
max_retries = 3
```

#### CLI Usage
```bash
# Test Gemini models
llm-test-bench bench \
  --dataset general-qa.json \
  --providers google \
  --models gemini-1.5-pro,gemini-1.5-flash

# Ultra-long context test
llm-test-bench compare \
  --prompt "$(cat book.txt)" \
  --models google:gemini-1.5-pro
```

---

### 4. Mistral AI

**French AI company providing open and commercial models with strong code and multilingual capabilities.**

#### Supported Models (7+ models)

**Mistral Code (June 2025)**
- `mistral-code` - Code-specialized model (Released Jun 4, 2025)
- `mistral-code-20250604` - June 2025 snapshot

**Magistral Family (Enterprise Models - June 2025)**
- `magistral-large` - Large enterprise model (Released June 2025)
- `magistral-medium` - Medium enterprise model (Released June 2025)
- `magistral-small` - Small enterprise model (Released June 2025)

**Voxtral (Audio Model - July 2025)**
- `voxtral-small` - Audio-specialized model (Released Jul 2025)
- `voxtral-small-20250701` - July 2025 snapshot

#### Configuration
```toml
[[providers]]
name = "mistral"
api_key_env = "MISTRAL_API_KEY"
base_url = "https://api.mistral.ai/v1"
default_model = "mistral-code"
timeout_seconds = 120
max_retries = 3
```

#### CLI Usage
```bash
# Benchmark with Mistral Code
llm-test-bench bench \
  --dataset coding-tasks.json \
  --providers mistral \
  --models mistral-code

# Test audio model
llm-test-bench bench \
  --dataset audio-transcripts.json \
  --providers mistral \
  --models voxtral-small
```

---

### 5. Cohere

**Enterprise-focused provider with strong RAG capabilities.**

#### Supported Models
- `command` - General purpose (4K context)
- `command-light` - Fast, efficient (4K)
- `command-nightly` - Latest features (4K)
- `command-r` - RAG-optimized (128K context)
- `command-r-plus` - Most capable (128K)

#### Configuration
```toml
[[providers]]
name = "cohere"
api_key_env = "COHERE_API_KEY"
base_url = "https://api.cohere.ai/v1"
default_model = "command-r-plus"
timeout_seconds = 120
max_retries = 3
```

#### CLI Usage
```bash
# Benchmark Cohere models
llm-test-bench bench \
  --dataset enterprise-qa.json \
  --providers cohere \
  --models command-r-plus,command-r
```

---

### 6. Azure OpenAI

**Enterprise-grade OpenAI models with Microsoft Azure compliance.**

#### Supported Models
- `gpt-35-turbo` - GPT-3.5 Turbo (4K)
- `gpt-35-turbo-16k` - Extended context (16K)
- `gpt-4` - GPT-4 (8K)
- `gpt-4-32k` - Extended context (32K)
- `gpt-4-turbo` - Latest GPT-4 (128K)

#### Configuration
```toml
[[providers]]
name = "azure-openai"
api_key_env = "AZURE_OPENAI_API_KEY"
base_url = "https://YOUR-RESOURCE.openai.azure.com"
default_model = "your-deployment-name"
timeout_seconds = 120
max_retries = 3
```

#### CLI Usage
```bash
# Set Azure credentials
export AZURE_OPENAI_API_KEY=your-key
export AZURE_OPENAI_ENDPOINT=https://your-resource.openai.azure.com

# Run benchmark
llm-test-bench bench \
  --dataset compliance-docs.json \
  --providers azure-openai
```

---

### 7. Groq

**Ultra-fast inference provider using custom LPU hardware.**

#### Supported Models
- `llama3-8b-8192` - Llama 3 8B (8K context)
- `llama3-70b-8192` - Llama 3 70B (8K)
- `mixtral-8x7b-32768` - Mixtral 8x7B (32K)
- `gemma-7b-it` - Gemma 7B (8K)

#### Configuration
```toml
[[providers]]
name = "groq"
api_key_env = "GROQ_API_KEY"
base_url = "https://api.groq.com/openai/v1"
default_model = "llama3-70b-8192"
timeout_seconds = 60
max_retries = 3
```

#### CLI Usage
```bash
# Test Groq's speed
llm-test-bench bench \
  --dataset speed-test.json \
  --providers groq \
  --models llama3-70b-8192,mixtral-8x7b-32768 \
  --metrics latency
```

---

### 8. Together AI

**Open-source model hosting with competitive pricing.**

#### Supported Models
- `meta-llama/Llama-2-70b-chat-hf` - Llama 2 70B (4K)
- `meta-llama/Llama-3-70b-chat-hf` - Llama 3 70B (8K)
- `mistralai/Mixtral-8x7B-Instruct-v0.1` - Mixtral (32K)
- `mistralai/Mistral-7B-Instruct-v0.2` - Mistral 7B (32K)
- `togethercomputer/CodeLlama-34b-Instruct` - Code Llama (16K)

#### Configuration
```toml
[[providers]]
name = "together"
api_key_env = "TOGETHER_API_KEY"
base_url = "https://api.together.xyz/v1"
default_model = "meta-llama/Llama-3-70b-chat-hf"
timeout_seconds = 120
max_retries = 3
```

---

### 9. Hugging Face

**Access to 100K+ open-source models via Inference API.**

#### Supported Models
- `meta-llama/Llama-2-7b-chat-hf` - Llama 2 7B
- `meta-llama/Llama-2-13b-chat-hf` - Llama 2 13B
- `mistralai/Mistral-7B-Instruct-v0.2` - Mistral 7B
- `mistralai/Mixtral-8x7B-Instruct-v0.1` - Mixtral
- `google/flan-t5-xxl` - FLAN-T5 XXL
- `bigcode/starcoder` - StarCoder

#### Configuration
```toml
[[providers]]
name = "huggingface"
api_key_env = "HUGGINGFACE_API_KEY"
base_url = "https://api-inference.huggingface.co/models"
default_model = "meta-llama/Llama-2-13b-chat-hf"
timeout_seconds = 120
max_retries = 3
```

---

### 10. Ollama

**Local model hosting for privacy and offline use.**

#### Supported Models
- `llama2` - Llama 2 7B (4K)
- `llama2:13b` - Llama 2 13B (4K)
- `llama2:70b` - Llama 2 70B (4K)
- `mistral` - Mistral 7B (32K)
- `mixtral` - Mixtral 8x7B (32K)
- `codellama` - Code Llama (16K)
- `phi` - Phi-2 (2K)
- `gemma:7b` - Gemma 7B (8K)

#### Configuration
```toml
[[providers]]
name = "ollama"
base_url = "http://localhost:11434"
default_model = "llama2"
timeout_seconds = 300
```

#### Setup
```bash
# Install Ollama
curl https://ollama.ai/install.sh | sh

# Pull models
ollama pull llama2
ollama pull mistral
ollama pull codellama

# Run benchmark
llm-test-bench bench \
  --dataset local-test.json \
  --providers ollama \
  --models llama2,mistral
```

---

### 11. AWS Bedrock

**AWS-managed service with multiple model providers.**

#### Supported Models
- `anthropic.claude-3-sonnet-20240229-v1:0` - Claude 3 Sonnet
- `anthropic.claude-3-opus-20240229-v1:0` - Claude 3 Opus
- `anthropic.claude-v2:1` - Claude 2.1
- `amazon.titan-text-express-v1` - Titan Text Express
- `meta.llama2-70b-chat-v1` - Llama 2 70B
- `cohere.command-text-v14` - Cohere Command
- `ai21.j2-ultra-v1` - Jurassic-2 Ultra

#### Configuration
```toml
[[providers]]
name = "bedrock"
# Bedrock uses AWS credentials from environment
```

#### Setup
```bash
# Set AWS credentials
export AWS_ACCESS_KEY_ID=your-access-key
export AWS_SECRET_ACCESS_KEY=your-secret-key
export AWS_REGION=us-east-1

# Run benchmark
llm-test-bench bench \
  --dataset aws-test.json \
  --providers bedrock \
  --models anthropic.claude-3-sonnet-20240229-v1:0
```

**Note:** Full AWS Bedrock support requires AWS SDK integration. Current implementation is a placeholder.

---

### 12. Replicate

**Easy deployment of ML models with GPU access.**

#### Supported Models
- `meta/llama-2-70b-chat` - Llama 2 70B Chat
- `meta/llama-2-13b-chat` - Llama 2 13B Chat
- `mistralai/mixtral-8x7b-instruct-v0.1` - Mixtral 8x7B
- `stability-ai/sdxl` - Stable Diffusion XL

#### Configuration
```toml
[[providers]]
name = "replicate"
api_key_env = "REPLICATE_API_TOKEN"
base_url = "https://api.replicate.com/v1"
default_model = "meta/llama-2-70b-chat"
timeout_seconds = 300
max_retries = 3
```

---

### 13. Perplexity AI

**Search-augmented LLMs with real-time web access.**

#### Supported Models
- `pplx-7b-chat` - Perplexity 7B Chat (4K)
- `pplx-70b-chat` - Perplexity 70B Chat (4K)
- `pplx-7b-online` - 7B with web search (4K)
- `pplx-70b-online` - 70B with web search (4K)
- `codellama-34b-instruct` - Code Llama 34B (16K)
- `mistral-7b-instruct` - Mistral 7B (4K)
- `mixtral-8x7b-instruct` - Mixtral 8x7B (4K)

#### Configuration
```toml
[[providers]]
name = "perplexity"
api_key_env = "PERPLEXITY_API_KEY"
base_url = "https://api.perplexity.ai"
default_model = "pplx-70b-online"
timeout_seconds = 120
max_retries = 3
```

---

## Provider Comparison

### Cost Comparison (as of 2025)

| Provider | Budget Option | Mid-Tier | Premium |
|----------|---------------|----------|---------|
| OpenAI | GPT-3.5 Turbo ($0.50/1M tokens) | GPT-4 ($10/1M) | GPT-4 Turbo ($30/1M) |
| Anthropic | Claude 3 Haiku ($0.25/1M) | Claude 3 Sonnet ($3/1M) | Claude 3 Opus ($15/1M) |
| Google | Gemini 1.5 Flash ($0.35/1M) | Gemini Pro ($7/1M) | Gemini Ultra ($TBD) |
| Cohere | Command Light ($0.15/1M) | Command ($1/1M) | Command R+ ($3/1M) |
| Mistral | Open Mistral 7B ($0.25/1M) | Mistral Small ($1/1M) | Mistral Large ($4/1M) |

### Speed Comparison

| Provider | Typical Latency | Tokens/Second |
|----------|----------------|---------------|
| Groq | 50-100ms | 500-1000 |
| OpenAI GPT-3.5 | 200-500ms | 100-200 |
| Anthropic Claude | 300-800ms | 80-150 |
| Google Gemini | 400-1000ms | 60-120 |
| Ollama (local) | 1-5s | 10-50 |

### Use Case Recommendations

| Use Case | Recommended Provider | Model |
|----------|---------------------|-------|
| **Code Generation** | OpenAI | GPT-4 Turbo |
| **Long Documents** | Anthropic | Claude 3 Opus |
| **Ultra-Long Context** | Google | Gemini 1.5 Pro |
| **Real-time Chat** | Groq | Llama 3 70B |
| **Cost-Sensitive** | Mistral | Open Mistral 7B |
| **Enterprise Compliance** | Azure OpenAI | GPT-4 |
| **Privacy/Offline** | Ollama | Llama 2 |
| **Research** | Hugging Face | Various |
| **Search-Augmented** | Perplexity | pplx-70b-online |

---

## Advanced Configuration

### Environment Variables

```bash
# Commercial providers
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
export GOOGLE_API_KEY=AI...
export COHERE_API_KEY=...
export MISTRAL_API_KEY=...
export GROQ_API_KEY=gsk_...
export TOGETHER_API_KEY=...
export HUGGINGFACE_API_KEY=hf_...
export PERPLEXITY_API_KEY=pplx-...
export REPLICATE_API_TOKEN=r8_...

# Azure OpenAI
export AZURE_OPENAI_API_KEY=...
export AZURE_OPENAI_ENDPOINT=https://...

# AWS Bedrock
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
export AWS_REGION=us-east-1

# Ollama (local)
export OLLAMA_HOST=http://localhost:11434
```

### Multi-Provider Benchmarking

```bash
# Compare all major providers
llm-test-bench bench \
  --dataset comprehensive-test.json \
  --providers openai,anthropic,google,cohere,mistral \
  --metrics faithfulness,relevance,coherence \
  --output results/multi-provider.json \
  --dashboard

# Generate comparison report
llm-test-bench compare \
  --prompt "Explain the theory of relativity" \
  --models \
    openai:gpt-4-turbo,\
    anthropic:claude-3-opus-20240229,\
    google:gemini-1.5-pro,\
    cohere:command-r-plus,\
    mistral:mistral-large-latest \
  --statistical-tests \
  --output comparison.html
```

---

## Troubleshooting

### Common Issues

#### 1. API Key Not Found
```bash
Error: OPENAI_API_KEY environment variable not set
```
**Solution:** Set the required environment variable:
```bash
export OPENAI_API_KEY=your-key-here
```

#### 2. Rate Limiting
```bash
Error: Rate limit exceeded (429)
```
**Solution:** Reduce parallelism or wait and retry:
```toml
[orchestration]
max_parallel_models = 2  # Reduce from default 5
```

#### 3. Ollama Connection Failed
```bash
Error: Cannot connect to Ollama: Connection refused
```
**Solution:** Ensure Ollama is running:
```bash
ollama serve
```

#### 4. Azure OpenAI Deployment Not Found
```bash
Error: The API deployment for this resource does not exist
```
**Solution:** Verify deployment name matches your Azure configuration:
```toml
default_model = "your-actual-deployment-name"  # Not "gpt-4"
```

---

## Provider Roadmap

### Planned Additions
- **OpenRouter** - Unified API for 50+ models
- **Anyscale** - Hosted open-source models
- **Fireworks AI** - Fast inference platform
- **DeepInfra** - Serverless model hosting
- **Modal** - Serverless GPU inference
- **RunPod** - GPU cloud computing
- **LlamaCpp** - Direct local inference

---

## Contributing

To add a new provider:

1. Implement the `Provider` trait in `core/src/providers/your_provider.rs`
2. Add module declaration in `core/src/providers/mod.rs`
3. Register factory function in `core/src/providers/factory.rs`
4. Add tests in provider implementation file
5. Update this documentation
6. Submit a pull request

See [CONTRIBUTING.md](../CONTRIBUTING.md) for details.
