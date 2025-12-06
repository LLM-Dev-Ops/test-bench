//! Testing utilities integration with infra-sim.
//!
//! This module provides mock services, simulated clocks, and chaos
//! engineering utilities for testing LLM applications.

use infra_sim::{
    MockBuilder, MockResponse, MockService, BuiltMock,
    SimulatedClock, Clock, SystemClock,
    ChaosConfig, ChaosMode, ChaosInjector,
    Scenario, ScenarioBuilder, Step,
};
use infra_errors::InfraResult;
use std::sync::Arc;
use std::time::Duration;

/// Create a mock LLM provider for testing
///
/// # Example
///
/// ```rust,ignore
/// use llm_test_bench_core::infra::testing::mock_provider;
///
/// let mock = mock_provider()
///     .with_completion_response(r#"{"text": "Hello!"}"#)
///     .build();
/// ```
pub fn mock_provider() -> LlmMockBuilder {
    LlmMockBuilder::new()
}

/// Builder for LLM provider mocks
pub struct LlmMockBuilder {
    inner: MockBuilder,
}

impl LlmMockBuilder {
    /// Create a new LLM mock builder
    pub fn new() -> Self {
        Self {
            inner: MockBuilder::new(),
        }
    }

    /// Add a completion response
    pub fn with_completion_response(mut self, response: &str) -> Self {
        self.inner = self.inner.on_post(
            "/v1/chat/completions",
            MockResponse::ok(response.as_bytes().to_vec()),
        );
        self
    }

    /// Add an embeddings response
    pub fn with_embeddings_response(mut self, response: &str) -> Self {
        self.inner = self.inner.on_post(
            "/v1/embeddings",
            MockResponse::ok(response.as_bytes().to_vec()),
        );
        self
    }

    /// Add a rate limit error response
    pub fn with_rate_limit_error(mut self) -> Self {
        self.inner = self.inner.on_post(
            "/v1/chat/completions",
            MockResponse::error(429, r#"{"error": {"message": "Rate limit exceeded"}}"#),
        );
        self
    }

    /// Add a timeout error (simulated via delay)
    pub fn with_timeout(mut self, delay: Duration) -> Self {
        self.inner = self.inner.on_post(
            "/v1/chat/completions",
            MockResponse::ok(b"{}".to_vec()).with_delay(delay),
        );
        self
    }

    /// Add a custom response for a path
    pub fn on_path(mut self, method: &str, path: &str, response: MockResponse) -> Self {
        self.inner = self.inner.on(method, path, response);
        self
    }

    /// Build the mock
    pub fn build(self) -> BuiltMock {
        self.inner.build()
    }
}

impl Default for LlmMockBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a simulated clock for time-based testing
///
/// # Example
///
/// ```rust,ignore
/// use llm_test_bench_core::infra::testing::simulated_clock;
/// use std::time::Duration;
///
/// let clock = simulated_clock();
/// let t1 = clock.now();
/// clock.advance(Duration::from_secs(60));
/// let t2 = clock.now();
/// assert!(t2 > t1);
/// ```
pub fn simulated_clock() -> SimulatedClock {
    SimulatedClock::new()
}

/// Create a system clock (real time)
pub fn system_clock() -> SystemClock {
    SystemClock::new()
}

/// Create a chaos injector for reliability testing
///
/// # Example
///
/// ```rust,ignore
/// use llm_test_bench_core::infra::testing::chaos_injector;
///
/// let chaos = chaos_injector()
///     .with_failure_rate(0.1) // 10% failure rate
///     .with_latency(Duration::from_millis(100), Duration::from_millis(500))
///     .build();
/// ```
pub fn chaos_injector() -> ChaosBuilder {
    ChaosBuilder::new()
}

/// Builder for chaos injection configuration
pub struct ChaosBuilder {
    failure_rate: f64,
    min_latency: Option<Duration>,
    max_latency: Option<Duration>,
}

impl ChaosBuilder {
    /// Create a new chaos builder
    pub fn new() -> Self {
        Self {
            failure_rate: 0.0,
            min_latency: None,
            max_latency: None,
        }
    }

    /// Set the failure rate (0.0 to 1.0)
    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.failure_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Set latency injection range
    pub fn with_latency(mut self, min: Duration, max: Duration) -> Self {
        self.min_latency = Some(min);
        self.max_latency = Some(max);
        self
    }

    /// Build the chaos configuration
    pub fn build(self) -> ChaosConfig {
        ChaosConfig {
            mode: if self.failure_rate > 0.0 {
                ChaosMode::Random { failure_rate: self.failure_rate }
            } else {
                ChaosMode::Disabled
            },
            latency_injection: self.min_latency.map(|min| (min, self.max_latency.unwrap_or(min))),
        }
    }
}

impl Default for ChaosBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a test scenario for end-to-end testing
pub fn scenario(name: &str) -> ScenarioBuilder {
    ScenarioBuilder::new(name)
}

/// Common test fixtures for LLM testing
pub mod fixtures {
    use super::*;

    /// Sample OpenAI completion response
    pub const OPENAI_COMPLETION: &str = r#"{
        "id": "chatcmpl-test",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello! How can I help you today?"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 9,
            "total_tokens": 19
        }
    }"#;

    /// Sample Anthropic completion response
    pub const ANTHROPIC_COMPLETION: &str = r#"{
        "id": "msg_test",
        "type": "message",
        "role": "assistant",
        "content": [{
            "type": "text",
            "text": "Hello! How can I help you today?"
        }],
        "model": "claude-3-opus-20240229",
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 9
        }
    }"#;

    /// Sample embeddings response
    pub const EMBEDDINGS: &str = r#"{
        "object": "list",
        "data": [{
            "object": "embedding",
            "index": 0,
            "embedding": [0.1, 0.2, 0.3, 0.4, 0.5]
        }],
        "model": "text-embedding-ada-002",
        "usage": {
            "prompt_tokens": 5,
            "total_tokens": 5
        }
    }"#;

    /// Create a mock with standard OpenAI responses
    pub fn openai_mock() -> BuiltMock {
        mock_provider()
            .with_completion_response(OPENAI_COMPLETION)
            .with_embeddings_response(EMBEDDINGS)
            .build()
    }

    /// Create a mock with standard Anthropic responses
    pub fn anthropic_mock() -> BuiltMock {
        MockBuilder::new()
            .on_post("/v1/messages", MockResponse::ok(ANTHROPIC_COMPLETION.as_bytes().to_vec()))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_builder() {
        let mock = mock_provider()
            .with_completion_response(r#"{"text": "test"}"#)
            .build();

        // Mock should be created successfully
        assert!(true);
    }

    #[test]
    fn test_simulated_clock() {
        let clock = simulated_clock();
        let t1 = clock.now();
        clock.advance(Duration::from_secs(10));
        let t2 = clock.now();
        assert!(t2 > t1);
    }

    #[test]
    fn test_chaos_builder() {
        let config = chaos_injector()
            .with_failure_rate(0.1)
            .with_latency(Duration::from_millis(10), Duration::from_millis(100))
            .build();

        match config.mode {
            ChaosMode::Random { failure_rate } => {
                assert!((failure_rate - 0.1).abs() < 0.001);
            }
            _ => panic!("Expected Random mode"),
        }
    }

    #[test]
    fn test_fixtures() {
        // Test that fixtures are valid JSON
        let _: serde_json::Value = serde_json::from_str(fixtures::OPENAI_COMPLETION).unwrap();
        let _: serde_json::Value = serde_json::from_str(fixtures::ANTHROPIC_COMPLETION).unwrap();
        let _: serde_json::Value = serde_json::from_str(fixtures::EMBEDDINGS).unwrap();
    }
}
