# Dependency Implementation Guide
## Technical Specification for Upstream Integration

**Companion to:** DEPENDENCY_ARCHITECTURE_DESIGN.md  
**Version:** 1.0  
**Date:** December 3, 2025

---

## 1. Provider Implementation Template

### 1.1 Directory Structure

```
core/src/providers/
├── mod.rs                      # Provider trait and registry
├── openai/                     # Existing
├── anthropic/                  # Existing
├── google/                     # New - feature gated
│   ├── mod.rs
│   ├── client.rs
│   ├── types.rs
│   ├── stream.rs
│   └── tests.rs
├── ollama/                     # New - feature gated
│   ├── mod.rs
│   └── ...
└── registry.rs                 # Feature-aware provider registry
```

### 1.2 Feature-Gated Module Pattern

**File: `core/src/providers/mod.rs`**

```rust
// Existing providers (always available)
pub mod openai;
pub mod anthropic;

// Feature-gated providers
#[cfg(feature = "provider-google")]
pub mod google;

#[cfg(feature = "provider-ollama")]
pub mod ollama;

#[cfg(feature = "provider-cohere")]
pub mod cohere;

#[cfg(feature = "provider-mistral")]
pub mod mistral;

#[cfg(feature = "provider-together")]
pub mod together;

#[cfg(feature = "provider-replicate")]
pub mod replicate;

#[cfg(feature = "provider-huggingface")]
pub mod huggingface;

#[cfg(feature = "provider-vllm")]
pub mod vllm;

// Provider trait (required)
use async_trait::async_trait;

#[async_trait]
pub trait Provider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    async fn stream(&self, request: CompletionRequest) -> Result<CompletionStream>;
    fn name(&self) -> &str;
    fn supports_streaming(&self) -> bool;
}

// Provider registry with feature detection
pub struct ProviderRegistry;

impl ProviderRegistry {
    pub fn available_providers() -> Vec<&'static str> {
        let mut providers = vec!["openai", "anthropic"];
        
        #[cfg(feature = "provider-google")]
        providers.push("google");
        
        #[cfg(feature = "provider-ollama")]
        providers.push("ollama");
        
        #[cfg(feature = "provider-cohere")]
        providers.push("cohere");
        
        // ... etc
        
        providers
    }
    
    pub fn is_available(name: &str) -> bool {
        Self::available_providers().contains(&name)
    }
}
```

### 1.3 Provider Implementation Example

**File: `core/src/providers/google/mod.rs`**

```rust
//! Google Gemini provider integration
//!
//! Requires feature: `provider-google`

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::providers::{Provider, CompletionRequest, CompletionResponse, CompletionStream};
use crate::error::Result;

mod client;
mod types;
mod stream;

pub use client::GoogleProvider;
pub use types::{GeminiModel, GeminiConfig};

/// Google Gemini provider
pub struct GoogleProvider {
    client: google_generativeai::Client,
    model: GeminiModel,
}

impl GoogleProvider {
    /// Create new Google provider
    pub fn new(api_key: impl Into<String>, model: GeminiModel) -> Self {
        let client = google_generativeai::Client::new(api_key);
        Self { client, model }
    }
    
    /// Create with default model (Gemini 2.0 Pro)
    pub fn with_default_model(api_key: impl Into<String>) -> Self {
        Self::new(api_key, GeminiModel::Gemini20Pro)
    }
}

#[async_trait]
impl Provider for GoogleProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Implementation
        todo!("Convert request, call Google API, convert response")
    }
    
    async fn stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        // Implementation
        todo!("Create streaming response")
    }
    
    fn name(&self) -> &str {
        "google"
    }
    
    fn supports_streaming(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_google_provider_creation() {
        let provider = GoogleProvider::with_default_model("test-key");
        assert_eq!(provider.name(), "google");
        assert!(provider.supports_streaming());
    }
}
```

---

## 2. Observability Implementation

### 2.1 OpenTelemetry Integration

**File: `core/src/observability/mod.rs`**

```rust
#[cfg(feature = "observability-otel")]
pub mod otel;

#[cfg(feature = "observability-langsmith")]
pub mod langsmith;

#[cfg(feature = "observability-phoenix")]
pub mod phoenix;

pub mod prometheus; // Always available

// Observability trait
pub trait Tracer: Send + Sync {
    fn start_span(&self, name: &str) -> Span;
    fn record_metric(&self, name: &str, value: f64);
}

// Feature-aware tracer initialization
pub fn init_tracing() -> Result<()> {
    #[cfg(feature = "observability-otel")]
    {
        otel::init()?;
    }
    
    #[cfg(not(feature = "observability-otel"))]
    {
        // Fallback to basic tracing
        tracing_subscriber::fmt::init();
    }
    
    Ok(())
}
```

**File: `core/src/observability/otel.rs`**

```rust
//! OpenTelemetry integration
//!
//! Requires feature: `observability-otel`

use opentelemetry::global;
use opentelemetry::sdk::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

use crate::error::Result;

/// Initialize OpenTelemetry tracing
pub fn init() -> Result<()> {
    // Create OTLP exporter
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://localhost:4317");
    
    // Create tracer provider
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .install_batch(opentelemetry::runtime::Tokio)?;
    
    global::set_tracer_provider(tracer);
    
    // Create tracing subscriber
    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(global::tracer("llm-test-bench"));
    
    let subscriber = Registry::default()
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry);
    
    tracing::subscriber::set_global_default(subscriber)?;
    
    Ok(())
}

/// Span creation with attributes
#[macro_export]
macro_rules! otel_span {
    ($name:expr, $($key:expr => $value:expr),*) => {
        {
            let span = tracing::info_span!($name);
            $(
                span.record($key, &$value);
            )*
            span
        }
    };
}
```

### 2.2 Provider Call Instrumentation

**File: `core/src/providers/base.rs`**

```rust
use crate::error::Result;

/// Instrument provider calls with tracing
#[cfg(feature = "observability-otel")]
pub async fn instrument_call<F, T>(
    provider: &str,
    model: &str,
    operation: &str,
    f: F,
) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    use tracing::Instrument;
    
    let span = tracing::info_span!(
        "provider_call",
        provider = provider,
        model = model,
        operation = operation,
    );
    
    f.instrument(span).await
}

#[cfg(not(feature = "observability-otel"))]
pub async fn instrument_call<F, T>(
    _provider: &str,
    _model: &str,
    _operation: &str,
    f: F,
) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    // No instrumentation without feature
    f.await
}
```

---

## 3. Multi-Modal Implementation

### 3.1 Vision Processing

**File: `core/src/multimodal/mod.rs`**

```rust
#[cfg(feature = "multimodal-vision")]
pub mod vision;

#[cfg(feature = "multimodal-audio")]
pub mod audio;

pub mod types;

pub use types::{MediaInput, MediaType};
```

**File: `core/src/multimodal/vision.rs`**

```rust
//! Vision processing utilities
//!
//! Requires feature: `multimodal-vision`

use image::{DynamicImage, ImageFormat};
use std::path::Path;

use crate::error::Result;

/// Load and validate image
pub fn load_image(path: impl AsRef<Path>) -> Result<DynamicImage> {
    let img = image::open(path)?;
    Ok(img)
}

/// Encode image to base64
pub fn encode_image(img: &DynamicImage, format: ImageFormat) -> Result<String> {
    let mut buffer = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buffer), format)?;
    Ok(base64::encode(&buffer))
}

/// Validate image dimensions
pub fn validate_dimensions(img: &DynamicImage, max_size: (u32, u32)) -> Result<()> {
    if img.width() > max_size.0 || img.height() > max_size.1 {
        return Err(crate::error::Error::InvalidImage(
            format!("Image too large: {}x{}", img.width(), img.height())
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_image_validation() {
        // Test implementation
    }
}
```

### 3.2 Audio Processing

**File: `core/src/multimodal/audio.rs`**

```rust
//! Audio processing utilities
//!
//! Requires feature: `multimodal-audio`

use hound::{WavReader, WavSpec};
use std::path::Path;

use crate::error::Result;

/// Load WAV audio file
pub fn load_wav(path: impl AsRef<Path>) -> Result<Vec<f32>> {
    let reader = WavReader::open(path)?;
    let samples: Vec<f32> = reader
        .into_samples::<i16>()
        .map(|s| s.map(|sample| sample as f32 / i16::MAX as f32))
        .collect::<std::result::Result<_, _>>()?;
    Ok(samples)
}

/// Get audio duration
pub fn get_duration(path: impl AsRef<Path>) -> Result<f64> {
    let reader = WavReader::open(path)?;
    let spec = reader.spec();
    let duration = reader.duration() as f64 / spec.sample_rate as f64;
    Ok(duration)
}
```

---

## 4. Evaluation Framework Integration

### 4.1 Python Binding Strategy

**File: `core/src/eval/python.rs`**

```rust
//! Python evaluation framework bindings
//!
//! Requires feature: `eval-python-bindings`

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::error::Result;

/// Python evaluation framework
pub struct PythonEvaluator {
    py: Python<'static>,
    module: PyObject,
}

impl PythonEvaluator {
    /// Initialize Python evaluator
    pub fn new(framework: &str) -> Result<Self> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        
        // Import framework
        let module = py.import(framework)?;
        
        Ok(Self {
            py,
            module: module.into(),
        })
    }
    
    /// Run evaluation
    pub fn evaluate(&self, data: &PyDict) -> Result<PyObject> {
        let result = self.module.call_method1(self.py, "evaluate", (data,))?;
        Ok(result)
    }
}

#[cfg(feature = "eval-ragas")]
pub mod ragas {
    use super::*;
    
    pub fn evaluate_rag(
        question: &str,
        answer: &str,
        context: &[String],
    ) -> Result<f64> {
        // Initialize RAGAS
        let evaluator = PythonEvaluator::new("ragas")?;
        
        // Prepare data
        let data = PyDict::new(evaluator.py);
        data.set_item("question", question)?;
        data.set_item("answer", answer)?;
        data.set_item("contexts", context)?;
        
        // Evaluate
        let result = evaluator.evaluate(data)?;
        let score: f64 = result.extract(evaluator.py)?;
        
        Ok(score)
    }
}
```

### 4.2 HTTP-Based Integration (Alternative)

**File: `core/src/eval/http.rs`**

```rust
//! HTTP-based evaluation framework integration

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Serialize)]
pub struct EvaluationRequest {
    pub framework: String,
    pub data: serde_json::Value,
}

#[derive(Deserialize)]
pub struct EvaluationResponse {
    pub score: f64,
    pub metrics: serde_json::Value,
}

/// HTTP evaluator client
pub struct HttpEvaluator {
    client: Client,
    endpoint: String,
}

impl HttpEvaluator {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            endpoint: endpoint.into(),
        }
    }
    
    pub async fn evaluate(&self, request: EvaluationRequest) -> Result<EvaluationResponse> {
        let response = self.client
            .post(&self.endpoint)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response)
    }
}
```

---

## 5. CLI Feature Detection

### 5.1 Runtime Feature Detection

**File: `cli/src/features.rs`**

```rust
//! Runtime feature detection

pub struct FeatureFlags;

impl FeatureFlags {
    /// Check if provider is available
    pub fn has_provider(name: &str) -> bool {
        match name {
            "openai" | "anthropic" => true,
            #[cfg(feature = "provider-google")]
            "google" => true,
            #[cfg(feature = "provider-ollama")]
            "ollama" => true,
            #[cfg(feature = "provider-cohere")]
            "cohere" => true,
            #[cfg(feature = "provider-mistral")]
            "mistral" => true,
            _ => false,
        }
    }
    
    /// List all available providers
    pub fn available_providers() -> Vec<&'static str> {
        let mut providers = vec!["openai", "anthropic"];
        
        #[cfg(feature = "provider-google")]
        providers.push("google");
        
        #[cfg(feature = "provider-ollama")]
        providers.push("ollama");
        
        #[cfg(feature = "provider-cohere")]
        providers.push("cohere");
        
        #[cfg(feature = "provider-mistral")]
        providers.push("mistral");
        
        providers
    }
    
    /// Check observability features
    pub fn has_observability(name: &str) -> bool {
        match name {
            #[cfg(feature = "observability-otel")]
            "opentelemetry" => true,
            #[cfg(feature = "observability-langsmith")]
            "langsmith" => true,
            _ => false,
        }
    }
}
```

### 5.2 CLI Error Messages

**File: `cli/src/error.rs`**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Provider '{0}' is not available. Enable with feature flag: {1}")]
    ProviderNotAvailable(String, String),
    
    #[error("Observability backend '{0}' is not available. Enable with feature flag: {1}")]
    ObservabilityNotAvailable(String, String),
    
    #[error("Multi-modal support not available. Enable with feature flag: multimodal")]
    MultiModalNotAvailable,
}

/// Helper to generate feature flag suggestions
pub fn suggest_feature(provider: &str) -> String {
    match provider {
        "google" => "provider-google".to_string(),
        "ollama" => "provider-ollama".to_string(),
        "cohere" => "provider-cohere".to_string(),
        _ => format!("provider-{}", provider),
    }
}
```

---

## 6. Testing Patterns

### 6.1 Feature-Gated Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Always runs
    #[test]
    fn test_basic_functionality() {
        // Test core functionality
    }
    
    // Only with feature
    #[cfg(feature = "provider-google")]
    #[test]
    fn test_google_provider() {
        // Test Google provider
    }
    
    // Integration test with multiple features
    #[cfg(all(feature = "provider-google", feature = "observability-otel"))]
    #[tokio::test]
    async fn test_google_with_tracing() {
        // Test provider with observability
    }
}
```

### 6.2 Mock Providers for Testing

```rust
#[cfg(test)]
pub mod mock {
    use super::*;
    
    pub struct MockProvider {
        name: String,
    }
    
    impl MockProvider {
        pub fn new(name: impl Into<String>) -> Self {
            Self { name: name.into() }
        }
    }
    
    #[async_trait]
    impl Provider for MockProvider {
        async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse> {
            Ok(CompletionResponse::mock())
        }
        
        async fn stream(&self, _request: CompletionRequest) -> Result<CompletionStream> {
            Ok(CompletionStream::mock())
        }
        
        fn name(&self) -> &str {
            &self.name
        }
        
        fn supports_streaming(&self) -> bool {
            true
        }
    }
}
```

---

## 7. Build Configuration

### 7.1 CI/CD Feature Matrix

**File: `.github/workflows/ci.yml`**

```yaml
name: CI

on: [push, pull_request]

jobs:
  test-features:
    name: Test Features
    runs-on: ubuntu-latest
    strategy:
      matrix:
        features:
          - ""  # Default
          - "provider-google"
          - "provider-ollama"
          - "all-providers"
          - "observability-otel"
          - "multimodal"
          - "full"
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          
      - name: Build
        run: |
          if [ -z "${{ matrix.features }}" ]; then
            cargo build --no-default-features
          else
            cargo build --features "${{ matrix.features }}"
          fi
          
      - name: Test
        run: |
          if [ -z "${{ matrix.features }}" ]; then
            cargo test --no-default-features
          else
            cargo test --features "${{ matrix.features }}"
          fi
```

### 7.2 Local Development

**File: `.cargo/config.toml`**

```toml
[build]
# Use all features by default for development
# Override with: cargo build --no-default-features
rustflags = []

[alias]
# Convenient aliases
build-minimal = "build --no-default-features"
build-full = "build --all-features"
test-minimal = "test --no-default-features"
test-full = "test --all-features"
check-features = "hack check --feature-powerset"
```

---

## 8. Documentation Examples

### 8.1 Feature-Aware Examples

**File: `examples/google_provider.rs`**

```rust
//! Google provider example
//!
//! Run with: cargo run --example google_provider --features provider-google

#[cfg(feature = "provider-google")]
use llm_test_bench_core::providers::GoogleProvider;

#[cfg(not(feature = "provider-google"))]
fn main() {
    eprintln!("This example requires the 'provider-google' feature");
    eprintln!("Run with: cargo run --example google_provider --features provider-google");
    std::process::exit(1);
}

#[cfg(feature = "provider-google")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Example code
    let provider = GoogleProvider::with_default_model(
        std::env::var("GOOGLE_API_KEY")?
    );
    
    println!("Google provider initialized: {}", provider.name());
    
    Ok(())
}
```

---

## 9. Security Best Practices

### 9.1 API Key Management

```rust
use secrecy::{Secret, ExposeSecret};

#[cfg(feature = "security-crypto")]
pub struct SecureConfig {
    api_key: Secret<String>,
}

#[cfg(feature = "security-crypto")]
impl SecureConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key: Secret::new(api_key),
        }
    }
    
    pub fn get_api_key(&self) -> &str {
        self.api_key.expose_secret()
    }
}

// Fallback without security feature
#[cfg(not(feature = "security-crypto"))]
pub struct SecureConfig {
    api_key: String,
}

#[cfg(not(feature = "security-crypto"))]
impl SecureConfig {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
    
    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }
}
```

---

## 10. Troubleshooting Guide

### Common Issues

**Issue 1: Feature not available at runtime**
```
Error: Provider 'google' is not available. Enable with feature flag: provider-google
```

**Solution:**
```bash
# Add to Cargo.toml
llm-test-bench = { version = "0.1", features = ["provider-google"] }

# Or build with feature
cargo build --features provider-google
```

**Issue 2: Conflicting dependencies**
```
error: failed to select a version for...
```

**Solution:**
```bash
# Check dependency tree
cargo tree --duplicates

# Use workspace versioning
# See Cargo.toml [workspace.dependencies]
```

**Issue 3: Missing Python dependencies**
```
Error: ModuleNotFoundError: No module named 'ragas'
```

**Solution:**
```bash
# Install Python dependencies
pip install ragas deepeval

# Or use HTTP-based integration
# See eval/http.rs
```

---

**END OF IMPLEMENTATION GUIDE**
