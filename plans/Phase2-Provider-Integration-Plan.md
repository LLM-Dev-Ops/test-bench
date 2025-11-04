# LLM Test Bench - Phase 2: Provider Integration
## Detailed Implementation Plan

**Phase:** Phase 2 - Provider Integration (Weeks 5-8)
**Planning Date:** November 4, 2025
**Document Version:** 1.0
**Status:** Ready for Implementation
**Previous Phase:** Phase 1 Complete ✅

---

## Executive Summary

### Phase 2 Objectives

Phase 2 focuses on implementing the core LLM provider integrations that will power the test bench. This phase transforms the CLI from a skeleton to a functional tool capable of making actual API calls to OpenAI and Anthropic Claude, handling responses, and managing errors gracefully.

### Key Deliverables

1. **Provider Abstraction Layer** - Complete implementation of the Provider trait
2. **OpenAI Integration** - Full-featured OpenAI API client with streaming
3. **Anthropic Integration** - Claude API client with 200K context support
4. **CLI Test Command** - Functional `llm-test-bench test` command
5. **Error Handling** - Robust retry logic and error recovery
6. **Integration Tests** - Comprehensive test suite with mocked and real API calls

### Success Criteria

- ✅ Make successful API calls to OpenAI GPT-4
- ✅ Make successful API calls to Anthropic Claude 3
- ✅ `llm-test-bench test openai --prompt "Hello"` works end-to-end
- ✅ Streaming response support for both providers
- ✅ Retry logic with exponential backoff
- ✅ 80%+ code coverage on provider modules
- ✅ Integration tests with real and mocked API calls

---

## Table of Contents

1. [Phase 2 Overview](#phase-2-overview)
2. [Milestone Breakdown](#milestone-breakdown)
3. [Technical Architecture](#technical-architecture)
4. [Implementation Details](#implementation-details)
5. [Testing Strategy](#testing-strategy)
6. [Risk Assessment](#risk-assessment)
7. [Timeline and Resources](#timeline-and-resources)
8. [Success Metrics](#success-metrics)
9. [Appendices](#appendices)

---

## 1. Phase 2 Overview

### 1.1 Phase Scope

**In Scope:**
- OpenAI provider implementation (GPT-4, GPT-4 Turbo, GPT-3.5 Turbo)
- Anthropic provider implementation (Claude 3 Opus, Sonnet, Haiku)
- Provider trait finalization
- CLI test command integration
- Streaming response handling
- Retry logic and rate limiting
- Error handling and recovery
- Configuration integration
- Integration and unit testing

**Out of Scope (Deferred to Later Phases):**
- Benchmarking system (Phase 3)
- Evaluation metrics (Phase 4)
- Additional providers (Google Gemini, Cohere - Future)
- Local model support (Ollama, llama.cpp - Future)
- Performance optimization (Phase 5)
- Advanced features (caching, distributed testing - Future)

### 1.2 Dependencies

**From Phase 1:**
- ✅ Cargo workspace structure
- ✅ Configuration system (providers config)
- ✅ CLI command scaffolding
- ✅ Error handling infrastructure (anyhow, thiserror)
- ✅ Async runtime (Tokio)

**External Dependencies:**
- OpenAI API access (requires API key)
- Anthropic API access (requires API key)
- Internet connectivity for API calls
- Rate limit awareness (provider-specific)

### 1.3 Architecture Context

```
Phase 1 Foundation
        ↓
┌───────────────────────────────┐
│  Phase 2: Provider Integration│
├───────────────────────────────┤
│                               │
│  ┌─────────────────────────┐ │
│  │   Provider Trait        │ │
│  │   (Abstraction Layer)   │ │
│  └─────────────────────────┘ │
│           ↓         ↓         │
│    ┌─────────┐ ┌──────────┐  │
│    │ OpenAI  │ │ Anthropic│  │
│    │Provider │ │ Provider │  │
│    └─────────┘ └──────────┘  │
│           ↓         ↓         │
│  ┌─────────────────────────┐ │
│  │   CLI Test Command      │ │
│  └─────────────────────────┘ │
└───────────────────────────────┘
        ↓
Phase 3: Benchmarking
```

---

## 2. Milestone Breakdown

### Milestone 2.1: Provider Abstraction (Week 5, Days 1-2)

**Status:** Partially Complete (trait defined, needs finalization)
**Duration:** 2 days
**Priority:** CRITICAL (blocks all other work)

#### Objectives
- Finalize the `Provider` trait with all required methods
- Define comprehensive error types for provider operations
- Create shared types (Request, Response, ModelInfo, etc.)
- Implement provider registry pattern
- Document trait requirements and best practices

#### Tasks

**Task 2.1.1: Finalize Provider Trait** (6 hours)
```rust
// Location: core/src/providers/traits.rs

#[async_trait]
pub trait Provider: Send + Sync {
    /// Complete a prompt with the LLM
    async fn complete(
        &self,
        request: CompletionRequest
    ) -> Result<CompletionResponse, ProviderError>;

    /// Stream completion tokens as they're generated
    async fn stream(
        &self,
        request: CompletionRequest
    ) -> Result<ResponseStream, ProviderError>;

    /// Get list of supported models
    fn supported_models(&self) -> Vec<ModelInfo>;

    /// Get maximum context length for a model
    fn max_context_length(&self, model: &str) -> Option<usize>;

    /// Get provider name (e.g., "openai", "anthropic")
    fn name(&self) -> &str;

    /// Validate provider configuration
    async fn validate_config(&self) -> Result<(), ProviderError>;

    /// Get token count estimate for text (provider-specific tokenization)
    fn estimate_tokens(&self, text: &str, model: &str) -> Result<usize, ProviderError>;
}
```

**Task 2.1.2: Define Error Types** (4 hours)
```rust
// Location: core/src/providers/error.rs

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Rate limit exceeded. Retry after {retry_after:?}")]
    RateLimitExceeded {
        retry_after: Option<Duration>,
    },

    #[error("Model not found: {model}")]
    ModelNotFound {
        model: String,
    },

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Context length exceeded: {tokens} > {max}")]
    ContextLengthExceeded {
        tokens: usize,
        max: usize,
    },

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("API error: {status} - {message}")]
    ApiError {
        status: u16,
        message: String,
    },

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Provider internal error: {0}")]
    InternalError(String),
}
```

**Task 2.1.3: Shared Types Definition** (4 hours)
```rust
// Location: core/src/providers/types.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub prompt: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub model: String,
    pub content: String,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
    Error,
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub max_tokens: usize,
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
}

pub type ResponseStream = Pin<Box<dyn Stream<Item = Result<String, ProviderError>> + Send>>;
```

**Deliverables:**
- ✅ Complete Provider trait definition
- ✅ Comprehensive error types
- ✅ Shared type definitions
- ✅ Full rustdoc documentation
- ✅ Unit tests for error handling

---

### Milestone 2.2: OpenAI Integration (Week 5-6, Days 3-10)

**Duration:** 8 days
**Priority:** CRITICAL
**Lead:** Backend Agent

#### Objectives
- Implement complete OpenAI API client
- Support GPT-4, GPT-4 Turbo, GPT-3.5 Turbo models
- Implement streaming and non-streaming completions
- Add retry logic with exponential backoff
- Comprehensive error handling

#### Tasks

**Task 2.2.1: OpenAI HTTP Client Setup** (6 hours)
```rust
// Location: core/src/providers/openai.rs

pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    config: ProviderConfig,
}

impl OpenAIProvider {
    pub fn new(config: ProviderConfig) -> Result<Self, ProviderError> {
        let api_key = std::env::var(&config.api_key_env)
            .map_err(|_| ProviderError::InvalidApiKey)?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self {
            client,
            api_key,
            base_url: config.base_url,
            config,
        })
    }
}
```

**Task 2.2.2: Non-Streaming Completions** (8 hours)
- Implement `complete()` method
- Build proper request payload
- Parse OpenAI response format
- Handle API errors
- Extract token usage information

**Task 2.2.3: Streaming Completions** (10 hours)
- Implement `stream()` method
- Handle Server-Sent Events (SSE)
- Parse streaming chunks
- Accumulate partial responses
- Handle stream errors and disconnections

**Task 2.2.4: Retry Logic** (6 hours)
```rust
async fn complete_with_retry(
    &self,
    request: CompletionRequest,
) -> Result<CompletionResponse, ProviderError> {
    let mut attempts = 0;
    let max_retries = self.config.max_retries;

    loop {
        match self.complete_once(&request).await {
            Ok(response) => return Ok(response),
            Err(e) if Self::is_retryable(&e) && attempts < max_retries => {
                attempts += 1;
                let delay = Self::calculate_backoff(attempts);
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}

fn calculate_backoff(attempt: u32) -> Duration {
    let base_delay = 1000; // 1 second
    let max_delay = 60000; // 60 seconds
    let delay = base_delay * 2_u64.pow(attempt);
    Duration::from_millis(delay.min(max_delay))
}
```

**Task 2.2.5: Model Information** (4 hours)
```rust
fn supported_models(&self) -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            max_tokens: 8192,
            supports_streaming: true,
            supports_function_calling: true,
        },
        ModelInfo {
            id: "gpt-4-turbo".to_string(),
            name: "GPT-4 Turbo".to_string(),
            max_tokens: 128000,
            supports_streaming: true,
            supports_function_calling: true,
        },
        ModelInfo {
            id: "gpt-3.5-turbo".to_string(),
            name: "GPT-3.5 Turbo".to_string(),
            max_tokens: 16385,
            supports_streaming: true,
            supports_function_calling: true,
        },
    ]
}
```

**Task 2.2.6: Unit Tests** (8 hours)
- Test request building
- Test response parsing
- Test error handling
- Test retry logic
- Mock HTTP responses with wiremock

**Task 2.2.7: Integration Tests** (6 hours)
- Real API calls (opt-in with env var)
- Test all supported models
- Test streaming and non-streaming
- Test error scenarios

**Deliverables:**
- ✅ Complete OpenAI provider implementation
- ✅ Streaming and non-streaming support
- ✅ Retry logic with exponential backoff
- ✅ 80%+ unit test coverage
- ✅ Integration tests with real API
- ✅ Documentation and examples

---

### Milestone 2.3: Anthropic Integration (Week 7, Days 11-15)

**Duration:** 5 days
**Priority:** HIGH
**Lead:** Backend Agent

#### Objectives
- Implement Anthropic Claude API client
- Support Claude 3 Opus, Sonnet, and Haiku models
- Handle 200K context windows
- Implement Claude-specific features

#### Tasks

**Task 2.3.1: Anthropic HTTP Client Setup** (6 hours)
```rust
// Location: core/src/providers/anthropic.rs

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    config: ProviderConfig,
}

impl AnthropicProvider {
    pub fn new(config: ProviderConfig) -> Result<Self, ProviderError> {
        // Similar to OpenAI but with Anthropic-specific headers
        let api_key = std::env::var(&config.api_key_env)
            .map_err(|_| ProviderError::InvalidApiKey)?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    "anthropic-version",
                    "2024-01-01".parse().unwrap(),
                );
                headers
            })
            .build()?;

        Ok(Self {
            client,
            api_key,
            base_url: config.base_url,
            config,
        })
    }
}
```

**Task 2.3.2: Claude Messages API Implementation** (8 hours)
- Implement completions using Claude Messages API
- Handle Claude's message format (system, user, assistant)
- Convert between our standard format and Claude's format
- Parse Claude response structure

**Task 2.3.3: Streaming Implementation** (8 hours)
- Implement SSE streaming for Claude
- Handle Claude-specific streaming format
- Parse streaming events
- Handle stream completion

**Task 2.3.4: Model Information** (3 hours)
```rust
fn supported_models(&self) -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "claude-3-opus-20240229".to_string(),
            name: "Claude 3 Opus".to_string(),
            max_tokens: 200000,
            supports_streaming: true,
            supports_function_calling: false, // Future
        },
        ModelInfo {
            id: "claude-3-sonnet-20240229".to_string(),
            name: "Claude 3 Sonnet".to_string(),
            max_tokens: 200000,
            supports_streaming: true,
            supports_function_calling: false,
        },
        ModelInfo {
            id: "claude-3-haiku-20240307".to_string(),
            name: "Claude 3 Haiku".to_string(),
            max_tokens: 200000,
            supports_streaming: true,
            supports_function_calling: false,
        },
    ]
}
```

**Task 2.3.5: Unit Tests** (6 hours)
- Test request format conversion
- Test response parsing
- Test error handling
- Mock HTTP with wiremock

**Task 2.3.6: Integration Tests** (6 hours)
- Real API calls with Claude
- Test 200K context handling
- Test all model variants

**Deliverables:**
- ✅ Complete Anthropic provider implementation
- ✅ 200K context window support
- ✅ Streaming support
- ✅ 80%+ unit test coverage
- ✅ Integration tests
- ✅ Documentation

---

### Milestone 2.4: CLI Test Command (Week 8, Days 16-20)

**Duration:** 5 days
**Priority:** CRITICAL
**Lead:** CLI Agent

#### Objectives
- Implement functional `llm-test-bench test` command
- Integrate with provider implementations
- Add progress indicators and UI polish
- Support output formatting options

#### Tasks

**Task 2.4.1: Command Integration** (6 hours)
```rust
// Location: cli/src/commands/test.rs

pub async fn execute(args: TestArgs) -> Result<()> {
    // 1. Load configuration
    let config = load_config(&args.config)?;

    // 2. Create provider instance
    let provider = create_provider(&args.provider, &config)?;

    // 3. Build completion request
    let request = CompletionRequest {
        model: args.model.unwrap_or(provider_default_model),
        prompt: args.prompt,
        max_tokens: args.max_tokens,
        temperature: args.temperature,
        top_p: args.top_p,
        stop: args.stop,
        stream: args.stream,
    };

    // 4. Execute request
    if args.stream {
        stream_response(&provider, request).await?;
    } else {
        let response = provider.complete(request).await?;
        display_response(&response, &args.output_format)?;
    }

    Ok(())
}
```

**Task 2.4.2: Provider Factory** (4 hours)
```rust
fn create_provider(
    provider_name: &str,
    config: &Config,
) -> Result<Box<dyn Provider>> {
    let provider_config = config.providers.get(provider_name)
        .ok_or_else(|| anyhow!("Provider {} not configured", provider_name))?;

    match provider_name {
        "openai" => Ok(Box::new(OpenAIProvider::new(provider_config.clone())?)),
        "anthropic" => Ok(Box::new(AnthropicProvider::new(provider_config.clone())?)),
        _ => Err(anyhow!("Unknown provider: {}", provider_name)),
    }
}
```

**Task 2.4.3: Streaming UI** (8 hours)
- Implement real-time token streaming to terminal
- Add progress indicators (spinners, etc.)
- Handle Ctrl+C gracefully
- Show token counts during streaming

**Task 2.4.4: Output Formatting** (6 hours)
```rust
#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Pretty,    // Human-readable with colors
    Json,      // JSON output
    JsonPretty, // Pretty-printed JSON
    Plain,     // Plain text only
}

fn display_response(
    response: &CompletionResponse,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Pretty => display_pretty(response),
        OutputFormat::Json => println!("{}", serde_json::to_string(response)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(response)?),
        OutputFormat::Plain => println!("{}", response.content),
    }
    Ok(())
}
```

**Task 2.4.5: Error Display** (4 hours)
- User-friendly error messages
- Suggestions for common errors (invalid API key, etc.)
- Proper exit codes

**Task 2.4.6: Integration Tests** (8 hours)
- Test command with mocked providers
- Test all output formats
- Test streaming mode
- Test error scenarios
- Use assert_cmd

**Deliverables:**
- ✅ Functional `llm-test-bench test` command
- ✅ Streaming and non-streaming modes
- ✅ Multiple output formats
- ✅ Progress indicators
- ✅ Comprehensive error handling
- ✅ Integration tests
- ✅ User documentation

---

## 3. Technical Architecture

### 3.1 Provider Architecture

```
┌─────────────────────────────────────────┐
│           CLI Layer                      │
│  (cli/src/commands/test.rs)             │
└─────────────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────┐
│      Provider Factory                    │
│  Creates provider instances from config  │
└─────────────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────┐
│         Provider Trait                   │
│  (Box<dyn Provider>)                     │
└─────────────────────────────────────────┘
          ↓              ↓
┌──────────────┐  ┌──────────────┐
│   OpenAI     │  │  Anthropic   │
│   Provider   │  │   Provider   │
└──────────────┘  └──────────────┘
       ↓                 ↓
┌──────────────┐  ┌──────────────┐
│ OpenAI API   │  │ Claude API   │
│ (reqwest)    │  │ (reqwest)    │
└──────────────┘  └──────────────┘
```

### 3.2 Request Flow

```
User Command
     ↓
Parse Arguments (Clap)
     ↓
Load Configuration
     ↓
Create Provider Instance
     ↓
Build CompletionRequest
     ↓
┌────────────────────────┐
│  Streaming?            │
├─────────┬──────────────┤
│   Yes   │      No      │
↓         ↓
stream()  complete()
↓         ↓
Process   Display
chunks    response
↓         ↓
Display   Show usage
tokens    stats
```

### 3.3 Error Handling Strategy

```
Provider Error
     ↓
Is Retryable?
├─ Yes → Exponential Backoff → Retry
└─ No  → Map to CLI Error
           ↓
      Display User-Friendly Message
           ↓
      Exit with Appropriate Code
```

### 3.4 Key Design Patterns

**1. Trait Objects (Dynamic Dispatch)**
```rust
Box<dyn Provider>  // Allows runtime provider selection
```

**2. Builder Pattern**
```rust
CompletionRequest::builder()
    .model("gpt-4")
    .prompt("Hello")
    .temperature(0.7)
    .build()
```

**3. Factory Pattern**
```rust
ProviderFactory::create(provider_name, config)
```

**4. Strategy Pattern**
```rust
// Different retry strategies per provider
OpenAIRetryStrategy vs AnthropicRetryStrategy
```

**5. Adapter Pattern**
```rust
// Adapt provider-specific formats to our common format
OpenAIAdapter::to_completion_response(openai_response)
```

---

## 4. Implementation Details

### 4.1 HTTP Client Configuration

```rust
// Shared client configuration
pub fn build_http_client(config: &ProviderConfig) -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(config.timeout_seconds))
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        .http2_adaptive_window(true)
        .use_rustls_tls()
        .build()
        .context("Failed to build HTTP client")
}
```

### 4.2 Rate Limiting

```rust
use governor::{Quota, RateLimiter};

pub struct RateLimitedProvider<P: Provider> {
    inner: P,
    limiter: RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState>,
}

impl<P: Provider> RateLimitedProvider<P> {
    pub fn new(provider: P, requests_per_minute: u32) -> Self {
        let quota = Quota::per_minute(requests_per_minute.try_into().unwrap());
        let limiter = RateLimiter::direct(quota);

        Self { inner: provider, limiter }
    }
}

#[async_trait]
impl<P: Provider> Provider for RateLimitedProvider<P> {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        self.limiter.until_ready().await;
        self.inner.complete(request).await
    }
}
```

### 4.3 Streaming Implementation

```rust
use futures::Stream;
use reqwest_eventsource::EventSource;

async fn stream_completion(
    &self,
    request: CompletionRequest,
) -> Result<ResponseStream, ProviderError> {
    let url = format!("{}/chat/completions", self.base_url);
    let body = self.build_request_body(&request, true)?;

    let event_source = EventSource::new(
        self.client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
    )?;

    let stream = event_source
        .filter_map(|event| async move {
            match event {
                Ok(Event::Message(msg)) => {
                    if msg.data == "[DONE]" {
                        None
                    } else {
                        Some(parse_streaming_chunk(&msg.data))
                    }
                }
                Ok(Event::Open) => None,
                Err(e) => Some(Err(ProviderError::from(e))),
            }
        })
        .boxed();

    Ok(stream)
}
```

### 4.4 Token Counting

```rust
// Approximate token counting (actual implementation would use tiktoken)
pub fn estimate_tokens(&self, text: &str, _model: &str) -> Result<usize, ProviderError> {
    // Rough approximation: 4 characters per token
    Ok((text.len() as f64 / 4.0).ceil() as usize)
}

// For production, use tiktoken-rs or call tokenization API
```

### 4.5 Configuration Validation

```rust
async fn validate_config(&self) -> Result<(), ProviderError> {
    // 1. Check API key is set
    if self.api_key.is_empty() {
        return Err(ProviderError::InvalidApiKey);
    }

    // 2. Make a simple test request
    let test_request = CompletionRequest {
        model: self.config.default_model.clone(),
        prompt: "test".to_string(),
        max_tokens: Some(1),
        ..Default::default()
    };

    // 3. Verify we can authenticate
    match self.complete(test_request).await {
        Ok(_) => Ok(()),
        Err(ProviderError::AuthenticationError(_)) => Err(ProviderError::InvalidApiKey),
        Err(e) => Err(e),
    }
}
```

---

## 5. Testing Strategy

### 5.1 Unit Testing

**Coverage Target:** 80%+

**Test Categories:**

1. **Request Building Tests**
```rust
#[test]
fn test_build_completion_request() {
    let provider = OpenAIProvider::new(test_config()).unwrap();
    let request = CompletionRequest {
        model: "gpt-4".to_string(),
        prompt: "Hello".to_string(),
        ..Default::default()
    };

    let body = provider.build_request_body(&request, false).unwrap();
    assert_eq!(body["model"], "gpt-4");
    assert_eq!(body["messages"][0]["content"], "Hello");
}
```

2. **Response Parsing Tests**
```rust
#[test]
fn test_parse_openai_response() {
    let json = r#"{
        "id": "chatcmpl-123",
        "model": "gpt-4",
        "choices": [{
            "message": {"content": "Hello!"},
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    }"#;

    let response = parse_completion_response(json).unwrap();
    assert_eq!(response.content, "Hello!");
    assert_eq!(response.usage.total_tokens, 15);
}
```

3. **Error Handling Tests**
```rust
#[test]
fn test_rate_limit_error() {
    let error_json = r#"{
        "error": {
            "type": "rate_limit_exceeded",
            "message": "Rate limit exceeded"
        }
    }"#;

    let error = parse_error_response(429, error_json).unwrap();
    assert!(matches!(error, ProviderError::RateLimitExceeded { .. }));
}
```

4. **Retry Logic Tests**
```rust
#[tokio::test]
async fn test_exponential_backoff() {
    let mut attempts = vec![];

    for attempt in 0..5 {
        let delay = calculate_backoff(attempt);
        attempts.push(delay);
    }

    // Verify exponential growth
    assert_eq!(attempts[0], Duration::from_millis(1000));
    assert_eq!(attempts[1], Duration::from_millis(2000));
    assert_eq!(attempts[2], Duration::from_millis(4000));
}
```

### 5.2 Integration Testing

**Categories:**

1. **Mocked API Tests (wiremock)**
```rust
#[tokio::test]
async fn test_openai_completion_mocked() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "test-123",
            "choices": [{"message": {"content": "Hello!"}}],
            "usage": {"total_tokens": 15}
        })))
        .mount(&mock_server)
        .await;

    let provider = OpenAIProvider::with_base_url(
        test_config(),
        mock_server.uri(),
    ).unwrap();

    let response = provider.complete(test_request()).await.unwrap();
    assert_eq!(response.content, "Hello!");
}
```

2. **Real API Tests (opt-in)**
```rust
#[tokio::test]
#[ignore] // Only run with --ignored flag
async fn test_openai_real_api() {
    // Requires OPENAI_API_KEY environment variable
    if std::env::var("OPENAI_API_KEY").is_err() {
        return;
    }

    let provider = OpenAIProvider::from_env().unwrap();
    let request = CompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        prompt: "Say 'test successful'".to_string(),
        max_tokens: Some(10),
        ..Default::default()
    };

    let response = provider.complete(request).await.unwrap();
    assert!(response.content.contains("test successful"));
}
```

3. **CLI Integration Tests**
```rust
#[test]
fn test_cli_test_command() {
    Command::cargo_bin("llm-test-bench")
        .unwrap()
        .args(&[
            "test",
            "openai",
            "--prompt", "Hello",
            "--model", "gpt-3.5-turbo",
            "--output-format", "json"
        ])
        .env("OPENAI_API_KEY", "test-key")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"content\":"));
}
```

### 5.3 Testing Tools

```toml
[dev-dependencies]
tokio = { version = "1.48", features = ["test-util", "macros"] }
wiremock = "0.6"           # HTTP mocking
mockall = "0.12"           # Trait mocking
proptest = "1.4"           # Property-based testing
assert_cmd = "2.0"         # CLI testing
predicates = "3.1"         # Test assertions
tempfile = "3.10"          # Temporary files
```

---

## 6. Risk Assessment

### 6.1 Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **API breaking changes** | High | Medium | Version pinning, comprehensive tests, monitor changelogs |
| **Rate limiting during tests** | Medium | High | Mock API for unit tests, careful integration test design |
| **Streaming complexity** | High | Medium | Thorough SSE testing, fallback to non-streaming |
| **Token counting accuracy** | Medium | Medium | Use official tokenizers (tiktoken), document limitations |
| **Network reliability** | Medium | High | Robust retry logic, timeout handling, offline mode for tests |
| **Large context handling** | Medium | Low | Memory profiling, streaming for large responses |

### 6.2 Project Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **API key availability** | High | Low | Document setup clearly, provide test mode |
| **Testing costs** | Medium | Medium | Minimize real API calls, use cheap models for tests |
| **Provider API differences** | High | High | Strong abstraction layer, adapter pattern |
| **Scope creep** | Medium | Medium | Strict adherence to milestone plan |

### 6.3 Security Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **API key exposure** | Critical | Medium | Environment variables only, never log keys, audit code |
| **Prompt injection** | Medium | Medium | Input validation, document user responsibility |
| **Man-in-the-middle** | High | Low | HTTPS only (rustls), certificate validation |

---

## 7. Timeline and Resources

### 7.1 Detailed Timeline

```
Week 5: Provider Abstraction + OpenAI Foundation
├─ Day 1-2: Milestone 2.1 (Provider Abstraction)
│  ├─ Finalize trait definition
│  ├─ Error types
│  └─ Shared types
│
├─ Day 3-5: OpenAI HTTP Client + Basic Completion
│  ├─ Client setup
│  ├─ Non-streaming implementation
│  └─ Initial tests
│
└─ Day 6-7: OpenAI Streaming + Retry Logic
   ├─ Streaming implementation
   ├─ Retry logic
   └─ Unit tests

Week 6: OpenAI Completion + Documentation
├─ Day 8-10: OpenAI Testing + Polish
│  ├─ Integration tests
│  ├─ Bug fixes
│  └─ Documentation
│
└─ Buffer for unexpected issues

Week 7: Anthropic Integration
├─ Day 11-12: Anthropic Client Setup
│  ├─ HTTP client
│  └─ Messages API format
│
├─ Day 13-14: Anthropic Completions
│  ├─ Non-streaming
│  ├─ Streaming
│  └─ Error handling
│
└─ Day 15: Testing + Documentation
   ├─ Unit tests
   ├─ Integration tests
   └─ Documentation

Week 8: CLI Integration + Polish
├─ Day 16-17: CLI Test Command
│  ├─ Provider factory
│  ├─ Command implementation
│  └─ Basic functionality
│
├─ Day 18-19: UI Polish
│  ├─ Streaming UI
│  ├─ Output formats
│  ├─ Progress indicators
│  └─ Error messages
│
└─ Day 20: Final Testing + Documentation
   ├─ End-to-end tests
   ├─ Documentation
   └─ Phase 2 review
```

### 7.2 Resource Allocation

| Agent | Milestones | Time Allocation |
|-------|-----------|-----------------|
| **Backend Agent** | 2.1, 2.2, 2.3 | 15 days (75%) |
| **CLI Agent** | 2.4 | 5 days (25%) |
| **Testing Agent** | All milestones | Parallel (ongoing) |
| **Documentation Agent** | All milestones | Parallel (ongoing) |

### 7.3 Dependencies and Blockers

**Blockers:**
- Phase 1 must be complete ✅
- API keys must be available (user responsibility)

**Critical Path:**
```
2.1 (Provider Abstraction)
  ↓
2.2 (OpenAI) ──→ 2.4 (CLI)
  ↓
2.3 (Anthropic) → 2.4 (CLI)
```

**Parallelization Opportunities:**
- Testing can start as soon as implementations begin
- Documentation can be written alongside code
- CLI work can start after 2.2 is partially complete

---

## 8. Success Metrics

### 8.1 Functional Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **API Call Success Rate** | 99%+ | Integration tests |
| **Streaming Reliability** | 100% | No dropped tokens |
| **Error Detection** | 100% | All API errors handled |
| **Retry Success** | 90%+ | Transient failures recovered |

### 8.2 Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Unit Test Coverage** | 80%+ | cargo-tarpaulin |
| **Integration Tests** | 20+ tests | Test suite |
| **Build Time** | <2 min | CI pipeline |
| **Documentation** | 100% public API | cargo doc |

### 8.3 Performance Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **First Token Latency** | <2s | Streaming tests |
| **Non-Stream Latency** | <5s + API time | End-to-end tests |
| **Memory Usage** | <50MB | Profiling |
| **Retry Overhead** | <10% | Benchmarks |

### 8.4 User Experience Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Error Message Clarity** | 100% | User testing |
| **Command Success** | First try | Documentation quality |
| **Setup Time** | <5 min | User feedback |

---

## 9. Appendices

### Appendix A: Example API Requests

#### OpenAI Request
```json
POST https://api.openai.com/v1/chat/completions
Authorization: Bearer sk-...

{
  "model": "gpt-4",
  "messages": [
    {"role": "user", "content": "Hello!"}
  ],
  "temperature": 0.7,
  "max_tokens": 100,
  "stream": false
}
```

#### Anthropic Request
```json
POST https://api.anthropic.com/v1/messages
x-api-key: sk-ant-...
anthropic-version: 2024-01-01

{
  "model": "claude-3-sonnet-20240229",
  "max_tokens": 100,
  "messages": [
    {"role": "user", "content": "Hello!"}
  ]
}
```

### Appendix B: Example CLI Usage

```bash
# Basic completion
llm-test-bench test openai --prompt "Explain Rust ownership"

# With specific model
llm-test-bench test openai \
  --prompt "Hello" \
  --model gpt-4-turbo \
  --temperature 0.7

# Streaming mode
llm-test-bench test anthropic \
  --prompt "Write a poem" \
  --model claude-3-opus-20240229 \
  --stream

# JSON output
llm-test-bench test openai \
  --prompt "Hello" \
  --output-format json

# With configuration file
llm-test-bench test openai \
  --config ./custom-config.toml \
  --prompt "Test"
```

### Appendix C: Error Handling Examples

```rust
// User-friendly error messages
match provider.complete(request).await {
    Err(ProviderError::InvalidApiKey) => {
        eprintln!("Error: Invalid API key");
        eprintln!("Hint: Set the OPENAI_API_KEY environment variable");
        eprintln!("      or configure it in ~/.config/llm-test-bench/config.toml");
        std::process::exit(1);
    }
    Err(ProviderError::RateLimitExceeded { retry_after }) => {
        eprintln!("Error: Rate limit exceeded");
        if let Some(duration) = retry_after {
            eprintln!("Hint: Retry after {} seconds", duration.as_secs());
        }
        std::process::exit(2);
    }
    Err(ProviderError::ContextLengthExceeded { tokens, max }) => {
        eprintln!("Error: Prompt too long ({} tokens, max: {})", tokens, max);
        eprintln!("Hint: Reduce prompt length or use a model with larger context");
        std::process::exit(3);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    Ok(response) => {
        println!("{}", response.content);
    }
}
```

### Appendix D: Dependencies to Add

```toml
[dependencies]
# Phase 2 additions
reqwest-eventsource = "0.6"  # SSE streaming
governor = "0.6"              # Rate limiting
futures = "0.3"               # Stream utilities
chrono = "0.4"                # Timestamps
indicatif = "0.17"            # Progress bars

[dev-dependencies]
wiremock = "0.6"              # HTTP mocking
mockall = "0.12"              # Trait mocking
proptest = "1.4"              # Property testing
```

---

## Conclusion

Phase 2 represents the critical transformation of LLM Test Bench from a CLI skeleton to a functional LLM testing tool. By the end of this phase, users will be able to:

✅ Make real API calls to OpenAI and Anthropic
✅ Stream responses in real-time
✅ Handle errors gracefully with automatic retries
✅ Test prompts with multiple models
✅ Get detailed usage statistics

The implementation plan is detailed, realistic, and builds on the solid foundation of Phase 1. With proper execution, Phase 2 will deliver a production-ready tool for LLM testing.

---

**Next Review:** End of Week 6 (Milestone 2.2 complete)
**Phase 2 Completion Target:** End of Week 8
**Status:** Ready to Begin Implementation

**Prepared by:** Swarm Coordinator
**Date:** November 4, 2025
**Version:** 1.0
