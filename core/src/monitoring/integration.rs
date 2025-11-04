// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Integration helpers for monitoring LLM providers and benchmarks.

use std::sync::Arc;
use std::time::Instant;
use anyhow::Result;
use async_trait::async_trait;

use crate::providers::{Provider, ProviderError, CompletionRequest, CompletionResponse, ResponseStream, ModelInfo};
use crate::monitoring::{
    MonitoringSystem,
    events::{MonitoringEvent, RequestEvent, TokenUsage},
};

/// A monitored provider wrapper that automatically emits monitoring events
pub struct MonitoredProvider {
    inner: Arc<dyn Provider>,
    monitoring: Arc<MonitoringSystem>,
}

impl MonitoredProvider {
    /// Wrap a provider with monitoring
    pub fn new(provider: Arc<dyn Provider>, monitoring: Arc<MonitoringSystem>) -> Self {
        Self {
            inner: provider,
            monitoring,
        }
    }

    /// Get the inner provider
    pub fn inner(&self) -> &Arc<dyn Provider> {
        &self.inner
    }
}

#[async_trait]
impl Provider for MonitoredProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let provider_name = self.inner.name();
        let model = request.model.clone();
        let request_id = Self::generate_request_id();

        // Record request start
        self.monitoring.record_request(provider_name, &model);

        let start = Instant::now();

        // Execute request
        let result = self.inner.complete(request).await;

        let latency = start.elapsed().as_secs_f64();

        match result {
            Ok(ref response) => {
                // Record successful request
                self.monitoring.record_latency(provider_name, latency);

                // Record token usage
                self.monitoring.record_tokens(
                    provider_name,
                    response.usage.prompt_tokens,
                    response.usage.completion_tokens,
                );

                // Estimate cost (simplified - should use actual pricing)
                let cost = Self::estimate_cost(&model, response.usage.prompt_tokens, response.usage.completion_tokens);
                self.monitoring.record_cost(provider_name, cost);

                // Emit detailed event
                let event = MonitoringEvent::new(
                    crate::monitoring::events::EventType::RequestCompleted,
                    crate::monitoring::events::EventPayload::Request(RequestEvent {
                        provider: provider_name.to_string(),
                        model: model.clone(),
                        request_id: request_id.clone(),
                        latency: Some(latency),
                        tokens: Some(TokenUsage {
                            input_tokens: response.usage.prompt_tokens,
                            output_tokens: response.usage.completion_tokens,
                            total_tokens: response.usage.total_tokens,
                        }),
                        cost: Some(Self::estimate_cost(&model, response.usage.prompt_tokens, response.usage.completion_tokens)),
                        error: None,
                    }),
                );
                self.monitoring.event_bus().publish(event);
            }
            Err(ref e) => {
                // Record error
                let error_type = match e {
                    ProviderError::RateLimitExceeded(_) => "rate_limit",
                    ProviderError::AuthenticationError(_) => "auth_error",
                    ProviderError::InvalidRequest(_) => "invalid_request",
                    ProviderError::NetworkError(_) => "network_error",
                    _ => "unknown_error",
                };

                self.monitoring.record_error(provider_name, error_type);

                // Emit error event
                let event = MonitoringEvent::new(
                    crate::monitoring::events::EventType::RequestFailed,
                    crate::monitoring::events::EventPayload::Request(RequestEvent {
                        provider: provider_name.to_string(),
                        model: model.clone(),
                        request_id: request_id.clone(),
                        latency: Some(latency),
                        tokens: None,
                        cost: None,
                        error: Some(e.to_string()),
                    }),
                );
                self.monitoring.event_bus().publish(event);
            }
        }

        result
    }

    async fn stream(&self, request: CompletionRequest) -> Result<ResponseStream, ProviderError> {
        // For streaming, we record start but can't capture full metrics until stream completes
        self.monitoring.record_request(self.inner.name(), &request.model);
        self.inner.stream(request).await
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        self.inner.supported_models()
    }

    fn max_context_length(&self, model: &str) -> Option<usize> {
        self.inner.max_context_length(model)
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn validate_config(&self) -> Result<(), ProviderError> {
        self.inner.validate_config().await
    }

    fn estimate_tokens(&self, text: &str, model: &str) -> Result<usize, ProviderError> {
        self.inner.estimate_tokens(text, model)
    }
}

impl MonitoredProvider {
    /// Generate a unique request ID
    fn generate_request_id() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("req_{}", id)
    }

    /// Simplified cost estimation (should use actual pricing)
    fn estimate_cost(model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        // Rough estimates - replace with actual pricing
        let (input_rate, output_rate) = match model {
            m if m.contains("gpt-4") => (0.00003, 0.00006),
            m if m.contains("gpt-3.5") => (0.0000015, 0.000002),
            m if m.contains("claude-3-opus") => (0.000015, 0.000075),
            m if m.contains("claude-3-sonnet") => (0.000003, 0.000015),
            m if m.contains("claude-3-haiku") => (0.00000025, 0.00000125),
            _ => (0.000001, 0.000002), // Default fallback
        };

        (input_tokens as f64 * input_rate) + (output_tokens as f64 * output_rate)
    }
}

/// Helper to wrap multiple providers with monitoring
pub fn monitor_providers(
    providers: Vec<Arc<dyn Provider>>,
    monitoring: Arc<MonitoringSystem>,
) -> Vec<Arc<dyn Provider>> {
    providers
        .into_iter()
        .map(|p| Arc::new(MonitoredProvider::new(p, monitoring.clone())) as Arc<dyn Provider>)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Actual testing would require mock providers
    #[test]
    fn test_cost_estimation() {
        let cost = MonitoredProvider::estimate_cost("gpt-4", 1000, 500);
        assert!(cost > 0.0);
        assert!(cost < 1.0); // Sanity check
    }
}
