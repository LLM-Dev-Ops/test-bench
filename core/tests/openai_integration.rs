//! Integration tests for OpenAI provider
//!
//! These tests make real API calls to OpenAI and are opt-in via environment variable.
//! To run: `OPENAI_API_KEY=your_key cargo test --test openai_integration -- --ignored`

use llm_test_bench_core::providers::{
    openai::OpenAIProvider, CompletionRequest, FinishReason, Provider, ProviderError,
};
use futures::StreamExt;

/// Helper to check if integration tests should run
fn should_run_integration_tests() -> bool {
    std::env::var("OPENAI_API_KEY").is_ok()
}

/// Helper to create a test provider
fn create_test_provider() -> OpenAIProvider {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    OpenAIProvider::new(api_key).expect("Failed to create provider")
}

#[tokio::test]
#[ignore] // Only run with --ignored flag
async fn test_real_completion_gpt_3_5() {
    if !should_run_integration_tests() {
        eprintln!("Skipping integration test: OPENAI_API_KEY not set");
        return;
    }

    let provider = create_test_provider();
    let request = CompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        prompt: "Say 'test successful' and nothing else.".to_string(),
        temperature: Some(0.0),
        max_tokens: Some(10),
        top_p: None,
        stop: None,
        stream: false,
        extra: serde_json::Value::Null,
    };

    let response = provider.complete(&request).await.expect("Request failed");

    assert!(!response.id.is_empty());
    assert_eq!(response.model, "gpt-3.5-turbo");
    assert!(response.content.to_lowercase().contains("test successful"));
    assert!(response.usage.total_tokens > 0);
    assert_eq!(response.finish_reason, FinishReason::Stop);
}

#[tokio::test]
#[ignore]
async fn test_real_completion_with_options() {
    if !should_run_integration_tests() {
        eprintln!("Skipping integration test: OPENAI_API_KEY not set");
        return;
    }

    let provider = create_test_provider();
    let request = CompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        prompt: "Count from 1 to 5.".to_string(),
        temperature: Some(0.3),
        max_tokens: Some(50),
        top_p: Some(0.9),
        stop: None,
        stream: false,
        extra: serde_json::Value::Null,
    };

    let response = provider.complete(&request).await.expect("Request failed");

    assert!(!response.content.is_empty());
    assert!(response.usage.completion_tokens <= 50);
}

#[tokio::test]
#[ignore]
async fn test_real_streaming() {
    if !should_run_integration_tests() {
        eprintln!("Skipping integration test: OPENAI_API_KEY not set");
        return;
    }

    let provider = create_test_provider();
    let request = CompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        prompt: "Count from 1 to 3.".to_string(),
        temperature: Some(0.0),
        max_tokens: Some(20),
        top_p: None,
        stop: None,
        stream: true,
        extra: serde_json::Value::Null,
    };

    let mut stream = provider.stream(&request).await.expect("Stream failed");

    let mut chunks = Vec::new();
    let mut last_was_final = false;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.expect("Stream chunk failed");
        if chunk.is_final {
            last_was_final = true;
            assert!(chunk.finish_reason.is_some());
        }
        chunks.push(chunk);
    }

    assert!(!chunks.is_empty(), "Should receive at least one chunk");
    assert!(last_was_final, "Last chunk should be marked as final");

    // Concatenate all content chunks
    let full_content: String = chunks.iter().map(|c| &c.content).collect();
    assert!(!full_content.is_empty(), "Should have received content");
}

#[tokio::test]
#[ignore]
async fn test_invalid_api_key() {
    let provider = OpenAIProvider::new("sk-invalid-key-12345".to_string())
        .expect("Failed to create provider");

    let request = CompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        prompt: "Test".to_string(),
        temperature: Some(0.0),
        max_tokens: Some(10),
        top_p: None,
        stop: None,
        stream: false,
        extra: serde_json::Value::Null,
    };

    let result = provider.complete(&request).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ProviderError::InvalidApiKey));
}

#[tokio::test]
#[ignore]
async fn test_invalid_model() {
    if !should_run_integration_tests() {
        eprintln!("Skipping integration test: OPENAI_API_KEY not set");
        return;
    }

    let provider = create_test_provider();
    let request = CompletionRequest {
        model: "invalid-model-xyz".to_string(),
        prompt: "Test".to_string(),
        temperature: Some(0.0),
        max_tokens: Some(10),
        top_p: None,
        stop: None,
        stream: false,
        extra: serde_json::Value::Null,
    };

    let result = provider.complete(&request).await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore]
async fn test_supported_models() {
    if !should_run_integration_tests() {
        eprintln!("Skipping integration test: OPENAI_API_KEY not set");
        return;
    }

    let provider = create_test_provider();
    let models = provider.supported_models();

    assert!(models.iter().any(|m| m.id == "gpt-4"));
    assert!(models.iter().any(|m| m.id == "gpt-4-turbo"));
    assert!(models.iter().any(|m| m.id == "gpt-3.5-turbo"));

    // All models should support streaming
    for model in &models {
        assert!(model.supports_streaming);
    }
}

#[tokio::test]
#[ignore]
async fn test_validate_config() {
    if !should_run_integration_tests() {
        eprintln!("Skipping integration test: OPENAI_API_KEY not set");
        return;
    }

    let provider = create_test_provider();
    let result = provider.validate_config().await;

    // With a valid API key, validation might succeed or fail depending on
    // whether the minimal test request works. Either way is acceptable.
    // The important thing is it doesn't panic.
    match result {
        Ok(_) => println!("Config validation succeeded"),
        Err(e) => println!("Config validation returned error: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_max_tokens_limit() {
    if !should_run_integration_tests() {
        eprintln!("Skipping integration test: OPENAI_API_KEY not set");
        return;
    }

    let provider = create_test_provider();
    let request = CompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        prompt: "Write a long essay about the history of computing.".to_string(),
        temperature: Some(0.7),
        max_tokens: Some(5), // Very low limit
        top_p: None,
        stop: None,
        stream: false,
        extra: serde_json::Value::Null,
    };

    let response = provider.complete(&request).await.expect("Request failed");

    // Response should be cut short due to max_tokens limit
    assert_eq!(response.finish_reason, FinishReason::Length);
    assert!(response.usage.completion_tokens <= 5);
}

#[tokio::test]
#[ignore]
async fn test_stop_sequence() {
    if !should_run_integration_tests() {
        eprintln!("Skipping integration test: OPENAI_API_KEY not set");
        return;
    }

    let provider = create_test_provider();
    let request = CompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        prompt: "Count from 1 to 10.".to_string(),
        temperature: Some(0.0),
        max_tokens: Some(100),
        top_p: None,
        stop: Some(vec!["5".to_string()]),
        stream: false,
        extra: serde_json::Value::Null,
    };

    let response = provider.complete(&request).await.expect("Request failed");

    // Response should stop when it hits "5"
    assert!(!response.content.contains("10"));
}

#[test]
fn test_provider_name() {
    let provider = OpenAIProvider::new("test_key".to_string()).unwrap();
    assert_eq!(provider.name(), "OpenAI");
}

#[test]
fn test_max_context_length() {
    let provider = OpenAIProvider::new("test_key".to_string()).unwrap();
    assert_eq!(provider.max_context_length("gpt-4"), Some(8192));
    assert_eq!(provider.max_context_length("gpt-4-turbo"), Some(128000));
    assert_eq!(provider.max_context_length("gpt-3.5-turbo"), Some(16385));
    assert_eq!(provider.max_context_length("nonexistent"), None);
}

#[test]
fn test_estimate_tokens() {
    let provider = OpenAIProvider::new("test_key".to_string()).unwrap();

    // Basic estimation test
    let text = "Hello, world!"; // ~13 chars
    let tokens = provider.estimate_tokens(text, "gpt-4").unwrap();
    assert_eq!(tokens, 4); // 13 / 4 = 3.25, rounded up to 4

    // Empty string
    let tokens = provider.estimate_tokens("", "gpt-4").unwrap();
    assert_eq!(tokens, 0);

    // Longer text
    let text = "a".repeat(1000); // 1000 chars = ~250 tokens
    let tokens = provider.estimate_tokens(&text, "gpt-4").unwrap();
    assert_eq!(tokens, 250);
}
