# Dependency Architecture - Quick Reference

**For:** Developers implementing the architecture  
**Version:** 1.0  
**Date:** December 3, 2025

---

## Feature Flags Reference

### Providers
```toml
# Individual providers
provider-google              # Google Gemini
provider-ollama              # Local Ollama models
provider-huggingface        # HuggingFace Hub
provider-cohere             # Cohere API
provider-mistral            # Mistral AI
provider-together           # Together AI
provider-replicate          # Replicate
provider-vllm               # vLLM

# Bundle
all-providers               # All provider integrations
```

### Observability
```toml
# Individual observability features
observability-otel          # OpenTelemetry
observability-langsmith     # LangSmith
observability-phoenix       # Phoenix
observability-prometheus    # Prometheus

# Bundle
all-observability          # All observability features
```

### Evaluation
```toml
# Individual evaluation frameworks
eval-ragas                 # RAGAS framework
eval-deepeval              # DeepEval
eval-lm-harness           # LM Evaluation Harness
eval-helm                 # HELM benchmark

# Bundle
all-eval                  # All evaluation frameworks
```

### Multi-Modal & Enterprise
```toml
# Multi-modal
multimodal-vision         # Image processing
multimodal-audio          # Audio processing
multimodal                # Both vision and audio

# Storage
storage-lance             # Lance columnar format
storage-vector            # Qdrant vector DB
storage-redis             # Redis caching
storage-advanced          # All storage features

# Security
security-crypto           # Cryptographic primitives
privacy-dp                # Differential privacy

# Bundle
enterprise                # Storage + security + privacy
```

### Meta Features
```toml
ci                        # Common CI/CD features
full                      # Enable everything
```

---

## Common Commands

### Build Commands
```bash
# Minimal build (no optional features)
cargo build --no-default-features

# With specific provider
cargo build --features provider-google

# With multiple features
cargo build --features "provider-google,observability-otel"

# Full build
cargo build --all-features

# Release build with features
cargo build --release --features "all-providers,observability-otel"
```

### Test Commands
```bash
# Test specific feature
cargo test --features provider-google

# Test all features
cargo test --all-features

# Test minimal build
cargo test --no-default-features

# Run specific test with feature
cargo test --features provider-google google_provider_tests
```

### Validation Commands
```bash
# Check for circular dependencies
cargo tree --duplicates

# Check dependency graph
cargo tree --features provider-google

# Security audit
cargo deny check

# Check outdated dependencies
cargo outdated
```

---

## Adding a New Provider

### 1. Add Dependency
**Edit:** `Cargo.toml` (workspace level)
```toml
[workspace.dependencies]
new-provider-sdk = { version = "0.1", optional = true }
```

### 2. Add Feature Flag
**Edit:** `core/Cargo.toml`
```toml
[dependencies]
new-provider-sdk = { workspace = true, optional = true }

[features]
provider-newprovider = ["dep:new-provider-sdk"]
all-providers = [
    # ... existing providers
    "provider-newprovider",
]
```

### 3. Create Module
**Create:** `core/src/providers/newprovider/mod.rs`
```rust
//! New provider integration
//!
//! Requires feature: `provider-newprovider`

use async_trait::async_trait;
use crate::providers::{Provider, CompletionRequest, CompletionResponse};

pub struct NewProvider {
    client: new_provider_sdk::Client,
}

impl NewProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: new_provider_sdk::Client::new(api_key),
        }
    }
}

#[async_trait]
impl Provider for NewProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Implementation
        todo!()
    }
    
    async fn stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        todo!()
    }
    
    fn name(&self) -> &str {
        "newprovider"
    }
    
    fn supports_streaming(&self) -> bool {
        true
    }
}
```

### 4. Register in mod.rs
**Edit:** `core/src/providers/mod.rs`
```rust
#[cfg(feature = "provider-newprovider")]
pub mod newprovider;
```

### 5. Add Tests
**Create:** `core/src/providers/newprovider/tests.rs`
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_creation() {
        let provider = NewProvider::new("test-key");
        assert_eq!(provider.name(), "newprovider");
    }
}
```

### 6. Add Example
**Create:** `examples/newprovider.rs`
```rust
//! New provider example
//!
//! Run with: cargo run --example newprovider --features provider-newprovider

#[cfg(not(feature = "provider-newprovider"))]
fn main() {
    eprintln!("Run with: cargo run --example newprovider --features provider-newprovider");
    std::process::exit(1);
}

#[cfg(feature = "provider-newprovider")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Example implementation
    Ok(())
}
```

---

## Feature Detection Pattern

### Runtime Check
```rust
use crate::providers::ProviderRegistry;

fn check_provider(name: &str) {
    if ProviderRegistry::is_available(name) {
        println!("Provider {} is available", name);
    } else {
        eprintln!("Provider {} not available. Enable with feature: provider-{}", name, name);
    }
}
```

### Conditional Compilation
```rust
#[cfg(feature = "provider-google")]
use crate::providers::google::GoogleProvider;

#[cfg(not(feature = "provider-google"))]
fn google_not_available() {
    eprintln!("Google provider not available. Enable with: --features provider-google");
}
```

---

## Testing Patterns

### Feature-Gated Test
```rust
#[cfg(test)]
mod tests {
    #[cfg(feature = "provider-google")]
    #[tokio::test]
    async fn test_google_integration() {
        // Only runs when feature is enabled
    }
    
    #[cfg(all(feature = "provider-google", feature = "observability-otel"))]
    #[tokio::test]
    async fn test_google_with_tracing() {
        // Runs when both features are enabled
    }
}
```

---

## Error Handling

### Feature Not Available Error
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Provider '{0}' not available. Enable with: --features {1}")]
    ProviderNotAvailable(String, String),
}

// Usage
#[cfg(not(feature = "provider-google"))]
pub fn create_google_provider() -> Result<Box<dyn Provider>> {
    Err(Error::ProviderNotAvailable(
        "google".to_string(),
        "provider-google".to_string(),
    ))
}
```

---

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Test Features

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        features:
          - ""
          - "provider-google"
          - "all-providers"
          - "observability-otel"
          - "full"
    
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      
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

---

## Troubleshooting

### Issue: Feature not found
```
error: Package `llm-test-bench-core` does not have feature `provider-google`
```

**Solution:** Check feature name spelling in Cargo.toml

### Issue: Dependency not found
```
error: no matching package named `google-generativeai` found
```

**Solution:** Add to workspace dependencies first

### Issue: Multiple versions of dependency
```
warning: multiple versions of package `tokio` found
```

**Solution:** Use workspace-level version pinning

### Issue: Circular dependency
```
error: cyclic package dependency
```

**Solution:** Review dependency graph with `cargo tree`

---

## Performance Tips

### Faster Incremental Builds
```toml
# .cargo/config.toml
[build]
incremental = true
pipelining = true
```

### Parallel Compilation
```bash
# Use all CPU cores
cargo build -j $(nproc)
```

### Caching
```bash
# Use sccache for CI
export RUSTC_WRAPPER=sccache
cargo build
```

---

## Documentation

### Module Documentation
```rust
//! Module description
//!
//! # Features
//!
//! This module requires the `provider-google` feature:
//!
//! ```toml
//! [dependencies]
//! llm-test-bench-core = { version = "0.1", features = ["provider-google"] }
//! ```
```

### Example in Documentation
```rust
/// # Examples
///
/// ```ignore
/// # #[cfg(feature = "provider-google")]
/// use llm_test_bench_core::providers::GoogleProvider;
///
/// let provider = GoogleProvider::new("api-key");
/// ```
```

---

## Useful Cargo Extensions

```bash
# Install helpful tools
cargo install cargo-tree      # Dependency visualization
cargo install cargo-deny      # License/security checks
cargo install cargo-outdated  # Check for updates
cargo install cargo-audit     # Security audit
cargo install cargo-hack      # Feature powerset testing

# Usage
cargo tree --features provider-google
cargo deny check
cargo outdated
cargo audit
cargo hack check --feature-powerset
```

---

## Quick Links

- **Full Architecture:** DEPENDENCY_ARCHITECTURE_DESIGN.md
- **Implementation Guide:** DEPENDENCY_IMPLEMENTATION_GUIDE.md
- **Validation Checklist:** DEPENDENCY_VALIDATION_CHECKLIST.md
- **Executive Summary:** DEPENDENCY_ARCHITECTURE_SUMMARY.md

---

**Last Updated:** December 3, 2025  
**Maintained by:** Dependency Architecture Team
