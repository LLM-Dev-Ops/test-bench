// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Integration tests for Anthropic Claude provider
//!
//! These tests include both mocked API tests using wiremock and
//! optional real API tests that require an ANTHROPIC_API_KEY.

use llm_test_bench_core::providers::{
    AnthropicProvider, CompletionRequest, Provider, ProviderError,
};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Helper function to create a test request
fn create_test_request(model: &str, prompt: &str) -> CompletionRequest {
    CompletionRequest {
        model: model.to_string(),
        prompt: prompt.to_string(),
        temperature: Some(0.7),
        max_tokens: Some(100),
        top_p: None,
        stop: None,
        stream: false,
    }
}

// Helper function to create a mock Claude response
fn mock_claude_response() -> serde_json::Value {
    serde_json::json!({
        "id": "msg_01XFDUDYJgAACzvnptvVoYEL",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": "Hello! I'm Claude, an AI assistant created by Anthropic."
            }
        ],
        "model": "claude-3-sonnet-20240229",
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 12,
            "output_tokens": 15
        }
    })
}

// Helper function to create a mock error response
fn mock_error_response(error_type: &str, message: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "error",
        "error": {
            "type": error_type,
            "message": message
        }
    })
}

#[tokio::test]
async fn test_anthropic_successful_completion() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(header("x-api-key", "test-key"))
        .and(header("anthropic-version", "2023-06-01"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_claude_response()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = AnthropicProvider::with_base_url(
        "test-key".to_string(),
        mock_server.uri(),
    );

    let request = create_test_request("claude-3-sonnet-20240229", "Hello!");
    let response = provider.complete(&request).await.unwrap();

    assert_eq!(response.content, "Hello! I'm Claude, an AI assistant created by Anthropic.");
    assert_eq!(response.model, "claude-3-sonnet-20240229");
    assert_eq!(response.usage.prompt_tokens, 12);
    assert_eq!(response.usage.completion_tokens, 15);
    assert_eq!(response.usage.total_tokens, 27);
}

#[tokio::test]
async fn test_anthropic_authentication_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_json(mock_error_response("authentication_error", "Invalid API key"))
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = AnthropicProvider::with_base_url(
        "invalid-key".to_string(),
        mock_server.uri(),
    );

    let request = create_test_request("claude-3-sonnet-20240229", "Hello!");
    let result = provider.complete(&request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ProviderError::AuthenticationError(msg) => assert_eq!(msg, "Invalid API key"),
        e => panic!("Expected AuthenticationError, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_anthropic_rate_limit_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(mock_error_response("rate_limit_error", "Rate limit exceeded"))
        )
        .expect(4) // Initial + 3 retries
        .mount(&mock_server)
        .await;

    let provider = AnthropicProvider::with_base_url(
        "test-key".to_string(),
        mock_server.uri(),
    );

    let request = create_test_request("claude-3-sonnet-20240229", "Hello!");
    let result = provider.complete(&request).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ProviderError::RateLimitExceeded { .. }));
}

#[tokio::test]
async fn test_anthropic_retry_success_on_second_attempt() {
    let mock_server = MockServer::start().await;

    // First request fails
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Second request succeeds
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_claude_response()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = AnthropicProvider::with_base_url(
        "test-key".to_string(),
        mock_server.uri(),
    );

    let request = create_test_request("claude-3-sonnet-20240229", "Hello!");
    let response = provider.complete(&request).await.unwrap();

    assert_eq!(response.content, "Hello! I'm Claude, an AI assistant created by Anthropic.");
}

#[tokio::test]
async fn test_anthropic_invalid_request_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(mock_error_response("invalid_request_error", "Invalid model specified"))
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = AnthropicProvider::with_base_url(
        "test-key".to_string(),
        mock_server.uri(),
    );

    let request = create_test_request("invalid-model", "Hello!");
    let result = provider.complete(&request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ProviderError::RequestError(msg) => assert_eq!(msg, "Invalid model specified"),
        e => panic!("Expected RequestError, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_anthropic_request_format() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_claude_response()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = AnthropicProvider::with_base_url(
        "test-key".to_string(),
        mock_server.uri(),
    );

    let request = CompletionRequest {
        model: "claude-3-opus-20240229".to_string(),
        prompt: "What is Rust?".to_string(),
        temperature: Some(0.5),
        max_tokens: Some(500),
        top_p: None,
        stop: None,
        stream: false,
    };

    let _ = provider.complete(&request).await.unwrap();

    // Verify the request was made with correct headers
    // wiremock automatically validates this through the Mock matchers
}

#[tokio::test]
async fn test_anthropic_multiple_content_blocks() {
    let mock_server = MockServer::start().await;

    let response_with_multiple_blocks = serde_json::json!({
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": "First part. "
            },
            {
                "type": "text",
                "text": "Second part."
            }
        ],
        "model": "claude-3-haiku-20240307",
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 8
        }
    });

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_with_multiple_blocks))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = AnthropicProvider::with_base_url(
        "test-key".to_string(),
        mock_server.uri(),
    );

    let request = create_test_request("claude-3-haiku-20240307", "Test");
    let response = provider.complete(&request).await.unwrap();

    assert_eq!(response.content, "First part. Second part.");
}

#[tokio::test]
async fn test_anthropic_all_models() {
    let provider = AnthropicProvider::new("test-key".to_string());
    let models = provider.supported_models();

    assert_eq!(models.len(), 3);

    let model_ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
    assert!(model_ids.contains(&"claude-3-opus-20240229"));
    assert!(model_ids.contains(&"claude-3-sonnet-20240229"));
    assert!(model_ids.contains(&"claude-3-haiku-20240307"));

    for model in &models {
        assert_eq!(model.max_tokens, 200_000);
        assert!(model.supports_streaming);
    }
}

#[tokio::test]
async fn test_anthropic_default_max_tokens() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_claude_response()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = AnthropicProvider::with_base_url(
        "test-key".to_string(),
        mock_server.uri(),
    );

    let request = CompletionRequest {
        model: "claude-3-sonnet-20240229".to_string(),
        prompt: "Hello!".to_string(),
        temperature: Some(0.7),
        max_tokens: None, // No max_tokens specified
        top_p: None,
        stop: None,
        stream: false,
    };

    let response = provider.complete(&request).await.unwrap();
    assert_eq!(response.content, "Hello! I'm Claude, an AI assistant created by Anthropic.");
}

#[tokio::test]
async fn test_anthropic_metadata_preserved() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_claude_response()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = AnthropicProvider::with_base_url(
        "test-key".to_string(),
        mock_server.uri(),
    );

    let request = create_test_request("claude-3-sonnet-20240229", "Hello!");
    let response = provider.complete(&request).await.unwrap();

    // Verify metadata is preserved
    assert!(response.metadata.is_object());
    assert_eq!(response.metadata["id"], "msg_01XFDUDYJgAACzvnptvVoYEL");
    assert_eq!(response.metadata["stop_reason"], "end_turn");
    assert_eq!(response.metadata["role"], "assistant");
}

#[tokio::test]
async fn test_anthropic_empty_response() {
    let mock_server = MockServer::start().await;

    let empty_response = serde_json::json!({
        "id": "msg_empty",
        "type": "message",
        "role": "assistant",
        "content": [],
        "model": "claude-3-sonnet-20240229",
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 5,
            "output_tokens": 0
        }
    });

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(empty_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = AnthropicProvider::with_base_url(
        "test-key".to_string(),
        mock_server.uri(),
    );

    let request = create_test_request("claude-3-sonnet-20240229", "Test");
    let response = provider.complete(&request).await.unwrap();

    assert_eq!(response.content, "");
    assert_eq!(response.usage.completion_tokens, 0);
}

#[tokio::test]
async fn test_anthropic_custom_retry_config() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
        .expect(6) // Initial + 5 retries
        .mount(&mock_server)
        .await;

    let provider = AnthropicProvider::with_config(
        "test-key".to_string(),
        mock_server.uri(),
        5, // 5 retries
    );

    let request = create_test_request("claude-3-sonnet-20240229", "Hello!");
    let result = provider.complete(&request).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_anthropic_network_error_handling() {
    // Use an invalid URL to trigger a network error
    let provider = AnthropicProvider::with_base_url(
        "test-key".to_string(),
        "http://invalid.localhost.invalid:9999".to_string(),
    );

    let request = create_test_request("claude-3-sonnet-20240229", "Hello!");
    let result = provider.complete(&request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ProviderError::RequestError(msg) => {
            assert!(msg.contains("HTTP request failed") || msg.contains("network"));
        }
        e => panic!("Expected RequestError, got: {:?}", e),
    }
}

// ============================================================================
// Real API Tests (opt-in with ANTHROPIC_API_KEY environment variable)
// ============================================================================

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_anthropic_real_api_completion() {
    let api_key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping real API test - ANTHROPIC_API_KEY not set");
            return;
        }
    };

    let provider = AnthropicProvider::new(api_key);

    let request = CompletionRequest {
        model: "claude-3-haiku-20240307".to_string(), // Use fastest model for tests
        prompt: "Say 'test successful' and nothing else.".to_string(),
        temperature: Some(0.0), // Deterministic
        max_tokens: Some(20),
        top_p: None,
        stop: None,
        stream: false,
    };

    let response = provider.complete(&request).await.unwrap();

    assert!(!response.content.is_empty());
    assert!(response.content.to_lowercase().contains("test successful"));
    assert_eq!(response.model, "claude-3-haiku-20240307");
    assert!(response.usage.prompt_tokens > 0);
    assert!(response.usage.completion_tokens > 0);
    assert_eq!(
        response.usage.total_tokens,
        response.usage.prompt_tokens + response.usage.completion_tokens
    );
}

#[tokio::test]
#[ignore]
async fn test_anthropic_real_api_opus() {
    let api_key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping real API test - ANTHROPIC_API_KEY not set");
            return;
        }
    };

    let provider = AnthropicProvider::new(api_key);

    let request = CompletionRequest {
        model: "claude-3-opus-20240229".to_string(),
        prompt: "What is 2+2? Answer with just the number.".to_string(),
        temperature: Some(0.0),
        max_tokens: Some(10),
        top_p: None,
        stop: None,
        stream: false,
    };

    let response = provider.complete(&request).await.unwrap();

    assert!(!response.content.is_empty());
    assert!(response.content.contains('4'));
    assert_eq!(response.model, "claude-3-opus-20240229");
}

#[tokio::test]
#[ignore]
async fn test_anthropic_real_api_sonnet() {
    let api_key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping real API test - ANTHROPIC_API_KEY not set");
            return;
        }
    };

    let provider = AnthropicProvider::new(api_key);

    let request = CompletionRequest {
        model: "claude-3-sonnet-20240229".to_string(),
        prompt: "Hello, Claude!".to_string(),
        temperature: Some(0.7),
        max_tokens: Some(100),
        top_p: None,
        stop: None,
        stream: false,
    };

    let response = provider.complete(&request).await.unwrap();

    assert!(!response.content.is_empty());
    assert_eq!(response.model, "claude-3-sonnet-20240229");
}

#[tokio::test]
#[ignore]
async fn test_anthropic_real_api_streaming() {
    use futures::StreamExt;

    let api_key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping real API test - ANTHROPIC_API_KEY not set");
            return;
        }
    };

    let provider = AnthropicProvider::new(api_key);

    let request = CompletionRequest {
        model: "claude-3-haiku-20240307".to_string(),
        prompt: "Count from 1 to 5.".to_string(),
        temperature: Some(0.0),
        max_tokens: Some(50),
        top_p: None,
        stop: None,
        stream: false,
    };

    let mut stream = provider.stream(&request).await.unwrap();
    let mut chunks = Vec::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(text) => chunks.push(text),
            Err(e) => panic!("Streaming error: {:?}", e),
        }
    }

    assert!(!chunks.is_empty());
    let full_response = chunks.join("");
    assert!(!full_response.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_anthropic_real_api_large_context() {
    let api_key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping real API test - ANTHROPIC_API_KEY not set");
            return;
        }
    };

    let provider = AnthropicProvider::new(api_key);

    // Create a reasonably large prompt (not 200K but enough to test)
    let large_text = "word ".repeat(1000); // ~5000 characters
    let request = CompletionRequest {
        model: "claude-3-haiku-20240307".to_string(),
        prompt: format!("Here is some text: {}. How many times does the word 'word' appear?", large_text),
        temperature: Some(0.0),
        max_tokens: Some(50),
        top_p: None,
        stop: None,
        stream: false,
    };

    let response = provider.complete(&request).await.unwrap();

    assert!(!response.content.is_empty());
    assert!(response.usage.prompt_tokens > 1000); // Should be a large number
}
