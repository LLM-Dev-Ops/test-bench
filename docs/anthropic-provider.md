# Anthropic Claude Provider

Complete implementation of the Anthropic Claude API provider for LLM Test Bench, supporting all Claude 3 models with their 200,000 token context windows.

## Features

- **Non-streaming completions** - Standard request/response completions
- **Streaming completions** - Real-time token streaming via Server-Sent Events (SSE)
- **All Claude 3 models** - Opus, Sonnet, and Haiku variants
- **200K context windows** - Full support for large context processing
- **Automatic retry logic** - Exponential backoff for transient failures
- **Comprehensive error handling** - Detailed error messages with proper classification
- **Complete test coverage** - 18+ unit tests and 6+ integration tests

## Supported Models

| Model ID | Name | Context Length | Streaming | Notes |
|----------|------|----------------|-----------|-------|
| `claude-3-opus-20240229` | Claude 3 Opus | 200,000 tokens | ✅ | Most capable |
| `claude-3-sonnet-20240229` | Claude 3 Sonnet | 200,000 tokens | ✅ | Balanced |
| `claude-3-haiku-20240307` | Claude 3 Haiku | 200,000 tokens | ✅ | Fastest |

## Installation

The Anthropic provider is included in the `llm-test-bench-core` crate:

```toml
[dependencies]
llm-test-bench-core = "0.1.0"
```

## Basic Usage

### Non-Streaming Completion

```rust
use llm_test_bench_core::providers::{AnthropicProvider, CompletionRequest, Provider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create provider with API key
    let provider = AnthropicProvider::new(
        std::env::var("ANTHROPIC_API_KEY")?
    );

    // Create a completion request
    let request = CompletionRequest {
        model: "claude-3-sonnet-20240229".to_string(),
        prompt: "Explain Rust's ownership system in one paragraph.".to_string(),
        temperature: 0.7,
        max_tokens: Some(500),
        extra: serde_json::Value::Null,
    };

    // Get completion
    let response = provider.complete(&request).await?;

    println!("Response: {}", response.content);
    println!("Tokens used: {}", response.usage.total_tokens);

    Ok(())
}
```

### Streaming Completion

```rust
use llm_test_bench_core::providers::{AnthropicProvider, CompletionRequest};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = AnthropicProvider::new(
        std::env::var("ANTHROPIC_API_KEY")?
    );

    let request = CompletionRequest {
        model: "claude-3-haiku-20240307".to_string(),
        prompt: "Write a haiku about Rust programming.".to_string(),
        temperature: 0.8,
        max_tokens: Some(100),
        extra: serde_json::Value::Null,
    };

    // Stream the response
    let mut stream = provider.stream(&request).await?;

    while let Some(result) = stream.next().await {
        match result {
            Ok(text) => print!("{}", text),
            Err(e) => eprintln!("Stream error: {}", e),
        }
    }

    println!(); // New line after streaming
    Ok(())
}
```

### Using Different Models

```rust
// Use Claude 3 Opus for most capable responses
let opus_request = CompletionRequest {
    model: "claude-3-opus-20240229".to_string(),
    prompt: "Solve this complex problem...".to_string(),
    temperature: 0.7,
    max_tokens: Some(2000),
    extra: serde_json::Value::Null,
};

// Use Claude 3 Haiku for fast, cost-effective responses
let haiku_request = CompletionRequest {
    model: "claude-3-haiku-20240307".to_string(),
    prompt: "Quick answer: what is 2+2?".to_string(),
    temperature: 0.0,
    max_tokens: Some(10),
    extra: serde_json::Value::Null,
};
```

## Advanced Usage

### Custom Base URL (for testing)

```rust
let provider = AnthropicProvider::with_base_url(
    "test-key".to_string(),
    "http://localhost:8080".to_string(),
);
```

### Custom Retry Configuration

```rust
let provider = AnthropicProvider::with_config(
    api_key,
    "https://api.anthropic.com/v1".to_string(),
    5, // Maximum 5 retries
);
```

### Accessing Response Metadata

```rust
let response = provider.complete(&request).await?;

println!("Message ID: {}", response.metadata["id"]);
println!("Stop reason: {}", response.metadata["stop_reason"]);
println!("Model used: {}", response.model);
println!("Prompt tokens: {}", response.usage.prompt_tokens);
println!("Completion tokens: {}", response.usage.completion_tokens);
```

## API Format Conversion

The Anthropic provider automatically converts between our standard `CompletionRequest` format and Claude's Messages API format:

### Our Format
```json
{
  "model": "claude-3-sonnet-20240229",
  "prompt": "Hello, Claude!",
  "temperature": 0.7,
  "max_tokens": 100
}
```

### Claude Messages API Format (Internal)
```json
{
  "model": "claude-3-sonnet-20240229",
  "messages": [
    {"role": "user", "content": "Hello, Claude!"}
  ],
  "max_tokens": 100,
  "temperature": 0.7
}
```

## Error Handling

The provider implements comprehensive error handling with automatic retries:

```rust
use llm_test_bench_core::providers::ProviderError;

match provider.complete(&request).await {
    Ok(response) => {
        println!("Success: {}", response.content);
    }
    Err(ProviderError::AuthenticationError(msg)) => {
        eprintln!("Authentication failed: {}", msg);
        eprintln!("Check your ANTHROPIC_API_KEY");
    }
    Err(ProviderError::RateLimitExceeded) => {
        eprintln!("Rate limit exceeded, please wait and retry");
    }
    Err(ProviderError::RequestError(msg)) => {
        eprintln!("Request error: {}", msg);
    }
    Err(ProviderError::InvalidResponse(msg)) => {
        eprintln!("Invalid response: {}", msg);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

### Retry Logic

The provider automatically retries on:
- Rate limit errors (429)
- Network errors (500, 502, 503, 504)
- Transient failures

Retry behavior:
- **Default retries**: 3 attempts
- **Backoff strategy**: Exponential (1s, 2s, 4s, 8s, ...)
- **Max backoff**: 60 seconds
- **Non-retryable errors**: Authentication errors, invalid requests

## Streaming Format

Claude's streaming events are parsed automatically:

### Event Types
- `message_start` - Stream begins (ignored)
- `content_block_start` - Content block begins (ignored)
- `content_block_delta` - Contains text delta (extracted)
- `content_block_stop` - Content block ends (ignored)
- `message_delta` - Message metadata update (ignored)
- `message_stop` - Stream ends (terminates stream)

### Example SSE Event
```json
{
  "type": "content_block_delta",
  "index": 0,
  "delta": {
    "type": "text_delta",
    "text": "Hello"
  }
}
```

The provider extracts just the `text` field from `text_delta` events.

## Large Context Support

All Claude 3 models support 200,000 token context windows:

```rust
// Create a large prompt (example with ~50K tokens)
let large_document = std::fs::read_to_string("large_document.txt")?;

let request = CompletionRequest {
    model: "claude-3-sonnet-20240229".to_string(),
    prompt: format!(
        "Summarize this document:\n\n{}",
        large_document
    ),
    temperature: 0.5,
    max_tokens: Some(1000),
    extra: serde_json::Value::Null,
};

// The provider handles large contexts automatically
let response = provider.complete(&request).await?;
```

**Note**: The provider uses a 5-minute timeout to accommodate processing of large contexts.

## Testing

### Running Unit Tests

```bash
# Run all unit tests
cargo test -p llm-test-bench-core providers::anthropic

# Run with output
cargo test -p llm-test-bench-core providers::anthropic -- --nocapture
```

The unit tests include:
- ✅ Provider creation and configuration
- ✅ Request body formatting
- ✅ Response parsing and conversion
- ✅ Error parsing (auth, rate limit, invalid request)
- ✅ Retry logic and backoff calculation
- ✅ Streaming event parsing
- ✅ Multiple content block handling
- ✅ Metadata preservation

### Running Integration Tests

```bash
# Run mocked API tests
cargo test -p llm-test-bench-core --test anthropic_integration

# Run real API tests (requires ANTHROPIC_API_KEY)
ANTHROPIC_API_KEY=your-key cargo test -p llm-test-bench-core --test anthropic_integration -- --ignored
```

Integration tests include:
- ✅ 15+ mocked API tests with wiremock
- ✅ 6+ real API tests (opt-in)
- ✅ All three model variants
- ✅ Streaming functionality
- ✅ Error scenarios
- ✅ Large context handling

## Implementation Details

### HTTP Client Configuration

```rust
const ANTHROPIC_API_VERSION: &str = "2023-06-01";
const DEFAULT_TIMEOUT_SECS: u64 = 300; // 5 minutes

let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(300))
    .default_headers({
        let mut headers = HeaderMap::new();
        headers.insert("anthropic-version", "2023-06-01");
        headers
    })
    .build()?;
```

### Required Headers

- `x-api-key`: Your Anthropic API key
- `anthropic-version`: API version (2023-06-01)
- `content-type`: application/json

### Response Structure

```rust
pub struct CompletionResponse {
    pub content: String,           // Generated text
    pub model: String,              // Model used
    pub usage: TokenUsage,          // Token counts
    pub metadata: serde_json::Value // Additional metadata
}

pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

## Differences from OpenAI Implementation

| Feature | Anthropic | OpenAI |
|---------|-----------|--------|
| **API Format** | Messages API | Chat Completions API |
| **Message Structure** | `messages: [{"role": "user", "content": "..."}]` | `messages: [{"role": "user", "content": "..."}]` |
| **System Messages** | Separate `system` field | Part of messages array |
| **Context Window** | 200K tokens (all models) | Varies (8K to 128K) |
| **Streaming Events** | Custom format | OpenAI format |
| **Required Headers** | `x-api-key`, `anthropic-version` | `Authorization: Bearer` |
| **Token Usage** | `input_tokens`, `output_tokens` | `prompt_tokens`, `completion_tokens` |

## Rate Limits

Anthropic implements rate limits per API key. The provider handles rate limit errors automatically with exponential backoff.

Typical limits (check Anthropic docs for current limits):
- Requests per minute
- Tokens per minute
- Tokens per day

When rate limited, the provider will:
1. Detect 429 status code
2. Wait with exponential backoff
3. Retry up to max_retries times
4. Return `ProviderError::RateLimitExceeded` if all retries fail

## Known Limitations

1. **Function calling**: Not yet supported (planned for future release)
2. **Vision**: Image input not yet supported
3. **System messages**: Currently not exposed (uses default empty system)
4. **Tool use**: Not implemented in this version
5. **Multi-turn conversations**: Each request is independent (no conversation state)

## Future Enhancements

- [ ] Function calling support
- [ ] Vision/image input support
- [ ] Multi-turn conversation support
- [ ] System message configuration
- [ ] Tool use implementation
- [ ] Prompt caching
- [ ] Token counting with Claude's tokenizer

## Troubleshooting

### "Authentication failed: Invalid API key"
- Check that `ANTHROPIC_API_KEY` is set correctly
- Verify the API key is valid and active
- Ensure no extra whitespace in the key

### "Rate limit exceeded"
- Wait before retrying (respect the backoff)
- Consider reducing request frequency
- Upgrade to higher rate limit tier if needed

### Streaming connection drops
- Check network stability
- Verify firewall allows SSE connections
- Increase timeout if processing large contexts

### Large context timeout
- Default timeout is 5 minutes
- For extremely large contexts, consider custom timeout
- Monitor token usage to stay within limits

## Resources

- [Anthropic API Documentation](https://docs.anthropic.com/claude/reference)
- [Claude 3 Model Comparison](https://www.anthropic.com/claude)
- [Messages API Guide](https://docs.anthropic.com/claude/reference/messages_post)
- [Streaming Guide](https://docs.anthropic.com/claude/reference/streaming)

## License

This implementation is licensed under MIT OR Apache-2.0, matching the parent project.

## Contributing

Contributions are welcome! Please ensure:
- All tests pass (`cargo test`)
- Code is formatted (`cargo fmt`)
- Clippy warnings are addressed (`cargo clippy`)
- Documentation is updated
- New features include tests

---

**Milestone**: Phase 2, Milestone 2.3
**Status**: Complete ✅
**Version**: 0.1.0
**Last Updated**: November 4, 2025
