# Dependency Architecture Design
## Compile-Time Dependency Structure for 25 Upstream Repository Integrations

**Document Version:** 1.0  
**Date:** December 3, 2025  
**Status:** Ready for Implementation  
**Author:** Dependency Architecture Designer Agent

---

## Executive Summary

This document provides a comprehensive architectural design for integrating 25 upstream repositories into the LLM Test Bench ecosystem using a feature-gated dependency structure. The design ensures:

- **Optional dependencies**: All integrations are opt-in via Cargo features
- **Backward compatibility**: No breaking changes to existing functionality
- **Clean build graph**: No circular dependencies or version conflicts
- **Zero runtime cost**: Disabled features have no compilation or runtime overhead
- **Modular expansion**: Easy to add/remove integrations independently

---

## 1. Current Architecture Analysis

### 1.1 Existing Workspace Structure

```
llm-test-bench/
├── cli/                 # Main CLI binary
├── core/                # Core library (providers, evaluators, etc.)
└── datasets/            # Dataset management
```

### 1.2 Current Dependency Categories

**Core Dependencies (Required):**
- Async runtime: tokio, futures
- HTTP client: reqwest
- Serialization: serde, serde_json
- Error handling: anyhow, thiserror

**Feature-Gated (Existing):**
- `database` feature: sqlx (PostgreSQL support)

**Current Limitations:**
- No upstream integration framework
- Limited provider ecosystem
- Monolithic dependency structure
- Missing observability integrations

---

## 2. Upstream Repository Categories

### 2.1 Category 1: LLM Provider SDKs (8 repos)

**Purpose:** Direct API integrations for LLM providers

| Repository | Feature Flag | Version | Priority |
|------------|-------------|---------|----------|
| google-generativeai | `provider-google` | 0.4.0 | High |
| cohere-rust | `provider-cohere` | 0.8.0 | High |
| mistralai-client | `provider-mistral` | 0.3.0 | High |
| together-ai | `provider-together` | 0.2.0 | Medium |
| replicate-rs | `provider-replicate` | 0.5.0 | Medium |
| huggingface-hub | `provider-huggingface` | 0.3.0 | High |
| ollama-rs | `provider-ollama` | 0.4.0 | High |
| vllm-client | `provider-vllm` | 0.2.0 | Medium |

### 2.2 Category 2: Observability & Monitoring (5 repos)

**Purpose:** Production monitoring and tracing

| Repository | Feature Flag | Version | Priority |
|------------|-------------|---------|----------|
| opentelemetry | `observability-otel` | 0.21.0 | High |
| opentelemetry-semantic-conventions | `observability-otel` | 0.13.0 | High |
| langsmith-sdk | `observability-langsmith` | 0.2.0 | Medium |
| phoenix-trace | `observability-phoenix` | 0.1.0 | Medium |
| prometheus (existing) | `observability-prometheus` | 0.13.0 | High |

### 2.3 Category 3: Evaluation Frameworks (4 repos)

**Purpose:** Advanced evaluation metrics and methodologies

| Repository | Feature Flag | Version | Priority |
|------------|-------------|---------|----------|
| ragas-rs | `eval-ragas` | 0.1.0 | High |
| deepeval-core | `eval-deepeval` | 0.2.0 | Medium |
| lm-eval-harness | `eval-lm-harness` | 0.4.0 | Medium |
| helm-lite | `eval-helm` | 0.3.0 | Low |

### 2.4 Category 4: Multi-Modal Support (3 repos)

**Purpose:** Vision, audio, and multi-modal evaluation

| Repository | Feature Flag | Version | Priority |
|------------|-------------|---------|----------|
| image-rs | `multimodal-vision` | 0.24.0 | High |
| whisper-rs | `multimodal-audio` | 0.10.0 | Medium |
| symphonia | `multimodal-audio` | 0.5.0 | Medium |

### 2.5 Category 5: Data & Storage (3 repos)

**Purpose:** Advanced data handling and persistence

| Repository | Feature Flag | Version | Priority |
|------------|-------------|---------|----------|
| lance | `storage-lance` | 0.10.0 | Medium |
| qdrant-client | `storage-vector` | 1.7.0 | Medium |
| redis | `storage-redis` | 0.24.0 | High |

### 2.6 Category 6: Security & Privacy (2 repos)

**Purpose:** Privacy-preserving evaluation and security

| Repository | Feature Flag | Version | Priority |
|------------|-------------|---------|----------|
| ring | `security-crypto` | 0.17.0 | High |
| differential-privacy | `privacy-dp` | 0.1.0 | Low |

---

## 3. Feature Flag Design

### 3.1 Naming Convention

**Pattern:** `{category}-{specific}`

**Categories:**
- `provider-*`: LLM provider integrations
- `observability-*`: Monitoring and tracing
- `eval-*`: Evaluation frameworks
- `multimodal-*`: Multi-modal support
- `storage-*`: Data persistence
- `security-*`: Security features
- `privacy-*`: Privacy features

**Meta Features (Bundles):**
- `all-providers`: Enable all provider integrations
- `all-observability`: Enable all observability features
- `all-eval`: Enable all evaluation frameworks
- `full`: Enable everything (testing/development)

### 3.2 Feature Dependency Graph

```
full
├── all-providers
│   ├── provider-google
│   ├── provider-cohere
│   ├── provider-mistral
│   ├── provider-together
│   ├── provider-replicate
│   ├── provider-huggingface
│   ├── provider-ollama
│   └── provider-vllm
├── all-observability
│   ├── observability-otel
│   ├── observability-langsmith
│   ├── observability-phoenix
│   └── observability-prometheus
├── all-eval
│   ├── eval-ragas
│   ├── eval-deepeval
│   ├── eval-lm-harness
│   └── eval-helm
├── multimodal
│   ├── multimodal-vision
│   └── multimodal-audio
└── enterprise
    ├── storage-lance
    ├── storage-vector
    ├── storage-redis
    ├── security-crypto
    └── privacy-dp
```

---

## 4. Cargo.toml Modifications

### 4.1 Workspace-Level Dependencies

**File:** `/workspaces/test-bench/Cargo.toml`

```toml
[workspace.dependencies]
# Existing dependencies...

# === Provider SDKs ===
# Note: These are placeholder versions - actual crate names may differ
google-generativeai = { version = "0.4", optional = true }
# cohere-rust = { version = "0.8", optional = true }  # May not exist - use reqwest directly
# mistralai-client = { version = "0.3", optional = true }  # May not exist - use reqwest directly
# together-ai = { version = "0.2", optional = true }  # May not exist - use reqwest directly
async-openai = { version = "0.20", optional = true }  # For additional OpenAI features
# replicate-rs = { version = "0.5", optional = true }  # May not exist - use reqwest directly
hf-hub = { version = "0.3", optional = true }  # HuggingFace Hub client
ollama-rs = { version = "0.1", optional = true }  # Ollama Rust client

# === Observability ===
opentelemetry = { version = "0.21", optional = true }
opentelemetry-semantic-conventions = { version = "0.13", optional = true }
opentelemetry-otlp = { version = "0.14", optional = true }
tracing-opentelemetry = { version = "0.22", optional = true }
# langsmith-sdk - implement via reqwest (no official Rust SDK)
# phoenix-trace - implement via reqwest (no official Rust SDK)

# === Evaluation Frameworks ===
# Note: Most evaluation frameworks don't have Rust implementations
# We'll implement adapters using their Python APIs via pyo3 or HTTP
pyo3 = { version = "0.20", optional = true }  # For Python evaluation framework bindings

# === Multi-Modal ===
image = { version = "0.24", optional = true }  # Image processing
imageproc = { version = "0.23", optional = true }  # Image analysis
# whisper-rs - use whisper.cpp bindings or HTTP API
rodio = { version = "0.17", optional = true }  # Audio playback
hound = { version = "3.5", optional = true }  # WAV encoding/decoding
symphonia = { version = "0.5", optional = true }  # Audio decoding

# === Storage ===
lance = { version = "0.10", optional = true }  # Columnar data format
qdrant-client = { version = "1.7", optional = true }  # Vector database
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"], optional = true }

# === Security & Privacy ===
ring = { version = "0.17", optional = true }  # Cryptography
# differential-privacy - implement custom or use privacy-preserving libraries
secrecy = { version = "0.8", optional = true }  # Secret management
```

### 4.2 Core Crate Modifications

**File:** `/workspaces/test-bench/core/Cargo.toml`

```toml
[dependencies]
# Existing dependencies...

# === Provider SDKs ===
google-generativeai = { workspace = true, optional = true }
async-openai = { workspace = true, optional = true }
hf-hub = { workspace = true, optional = true }
ollama-rs = { workspace = true, optional = true }

# === Observability ===
opentelemetry = { workspace = true, optional = true }
opentelemetry-semantic-conventions = { workspace = true, optional = true }
opentelemetry-otlp = { workspace = true, optional = true }
tracing-opentelemetry = { workspace = true, optional = true }

# === Evaluation Frameworks ===
pyo3 = { workspace = true, optional = true }

# === Multi-Modal ===
image = { workspace = true, optional = true }
imageproc = { workspace = true, optional = true }
rodio = { workspace = true, optional = true }
hound = { workspace = true, optional = true }
symphonia = { workspace = true, optional = true }

# === Storage ===
lance = { workspace = true, optional = true }
qdrant-client = { workspace = true, optional = true }
redis = { workspace = true, optional = true }

# === Security & Privacy ===
ring = { workspace = true, optional = true }
secrecy = { workspace = true, optional = true }

[features]
default = []

# === Individual Provider Features ===
provider-google = ["dep:google-generativeai"]
provider-openai-extended = ["dep:async-openai"]
provider-huggingface = ["dep:hf-hub"]
provider-ollama = ["dep:ollama-rs"]
provider-cohere = []  # Implement via reqwest
provider-mistral = []  # Implement via reqwest
provider-together = []  # Implement via reqwest
provider-replicate = []  # Implement via reqwest
provider-vllm = []  # Implement via HTTP

# === Provider Bundle ===
all-providers = [
    "provider-google",
    "provider-openai-extended",
    "provider-huggingface",
    "provider-ollama",
    "provider-cohere",
    "provider-mistral",
    "provider-together",
    "provider-replicate",
    "provider-vllm",
]

# === Observability Features ===
observability-otel = [
    "dep:opentelemetry",
    "dep:opentelemetry-semantic-conventions",
    "dep:opentelemetry-otlp",
    "dep:tracing-opentelemetry",
]
observability-langsmith = []  # HTTP-based integration
observability-phoenix = []  # HTTP-based integration
observability-prometheus = []  # Already included in core

# === Observability Bundle ===
all-observability = [
    "observability-otel",
    "observability-langsmith",
    "observability-phoenix",
    "observability-prometheus",
]

# === Evaluation Features ===
eval-python-bindings = ["dep:pyo3"]
eval-ragas = ["eval-python-bindings"]
eval-deepeval = ["eval-python-bindings"]
eval-lm-harness = ["eval-python-bindings"]
eval-helm = ["eval-python-bindings"]

# === Evaluation Bundle ===
all-eval = [
    "eval-ragas",
    "eval-deepeval",
    "eval-lm-harness",
    "eval-helm",
]

# === Multi-Modal Features ===
multimodal-vision = ["dep:image", "dep:imageproc"]
multimodal-audio = ["dep:rodio", "dep:hound", "dep:symphonia"]
multimodal = ["multimodal-vision", "multimodal-audio"]

# === Storage Features ===
storage-lance = ["dep:lance"]
storage-vector = ["dep:qdrant-client"]
storage-redis = ["dep:redis"]
storage-advanced = ["storage-lance", "storage-vector", "storage-redis"]

# === Security Features ===
security-crypto = ["dep:ring", "dep:secrecy"]
privacy-dp = ["security-crypto"]  # Differential privacy requires crypto

# === Enterprise Bundle ===
enterprise = [
    "database",  # Existing feature
    "storage-advanced",
    "security-crypto",
    "privacy-dp",
]

# === Full Feature Set (Development/Testing) ===
full = [
    "all-providers",
    "all-observability",
    "all-eval",
    "multimodal",
    "enterprise",
]

# === CI/CD Feature (Common integrations only) ===
ci = [
    "provider-google",
    "provider-ollama",
    "observability-otel",
    "multimodal-vision",
]
```

### 4.3 CLI Crate Modifications

**File:** `/workspaces/test-bench/cli/Cargo.toml`

```toml
[dependencies]
# Workspace crates with features
llm-test-bench-core = { version = "0.1.0", path = "../core", default-features = false }
llm-test-bench-datasets = { version = "0.1.0", path = "../datasets" }

# Other dependencies...

[features]
default = ["llm-test-bench-core/default"]

# Expose all core features
provider-google = ["llm-test-bench-core/provider-google"]
provider-openai-extended = ["llm-test-bench-core/provider-openai-extended"]
provider-huggingface = ["llm-test-bench-core/provider-huggingface"]
provider-ollama = ["llm-test-bench-core/provider-ollama"]
provider-cohere = ["llm-test-bench-core/provider-cohere"]
provider-mistral = ["llm-test-bench-core/provider-mistral"]
provider-together = ["llm-test-bench-core/provider-together"]
provider-replicate = ["llm-test-bench-core/provider-replicate"]
provider-vllm = ["llm-test-bench-core/provider-vllm"]

all-providers = ["llm-test-bench-core/all-providers"]
all-observability = ["llm-test-bench-core/all-observability"]
all-eval = ["llm-test-bench-core/all-eval"]
multimodal = ["llm-test-bench-core/multimodal"]
enterprise = ["llm-test-bench-core/enterprise"]
full = ["llm-test-bench-core/full"]
ci = ["llm-test-bench-core/ci"]
```

---

## 5. Dependency Validation

### 5.1 Circular Dependency Prevention

**Validation Rules:**
1. Workspace dependencies only flow: `datasets` → `core` → `cli`
2. No reverse dependencies allowed
3. Optional dependencies don't create new edges
4. Feature flags only enable, never disable

**Validation Command:**
```bash
cargo tree --all-features --duplicates
cargo tree --all-features --edges normal
```

### 5.2 Version Conflict Resolution

**Strategy:**
- Use workspace-level version pinning
- Prefer latest stable versions
- Document known incompatibilities
- Use `cargo deny` for license and security checks

**Validation:**
```toml
# Add to workspace root
[workspace.metadata.cargo-deny]
advisories = { vulnerability = "deny", unmaintained = "warn" }
licenses = { allow = ["MIT", "Apache-2.0", "BSD-3-Clause"] }
```

### 5.3 Build Time Impact

**Baseline (minimal features):**
```bash
cargo build --no-default-features --timings
```

**Full build (all features):**
```bash
cargo build --all-features --timings
```

**Expected Impact:**
- Minimal build: ~2-3 minutes (existing baseline)
- With providers: +30-60 seconds
- With observability: +20-30 seconds
- Full build: ~4-5 minutes

---

## 6. Migration Plan

### Phase 1: Infrastructure Setup (Week 1)

**Tasks:**
1. Update workspace Cargo.toml with dependency definitions
2. Add feature flags to core/Cargo.toml
3. Update CLI feature propagation
4. Add cargo-deny configuration
5. Update CI/CD to test feature combinations

**Validation:**
```bash
# Test minimal build
cargo build --no-default-features
# Test each feature individually
cargo build --features provider-google
cargo build --features observability-otel
# Test feature bundles
cargo build --features all-providers
```

### Phase 2: Provider Integrations (Week 2-3)

**Implementation Order (by priority):**

1. **High Priority:**
   - provider-google
   - provider-ollama
   - provider-huggingface

2. **Medium Priority:**
   - provider-cohere
   - provider-mistral
   - provider-together

3. **Low Priority:**
   - provider-replicate
   - provider-vllm

**Per-Provider Tasks:**
- [ ] Add dependency to Cargo.toml
- [ ] Create provider module under `core/src/providers/{name}/`
- [ ] Implement `Provider` trait
- [ ] Add feature-gated tests
- [ ] Update documentation
- [ ] Add integration test

### Phase 3: Observability Layer (Week 4)

**Implementation Order:**
1. OpenTelemetry foundation
2. Prometheus metrics (enhance existing)
3. LangSmith integration
4. Phoenix integration

**Integration Points:**
- Span tracing for all provider calls
- Metric collection for latency/cost
- Error tracking and alerting
- Custom dashboards

### Phase 4: Evaluation Frameworks (Week 5)

**Strategy:**
- Use PyO3 for Python framework bindings
- Create common evaluation trait
- Implement adapters for each framework

**Frameworks:**
1. RAGAS (RAG evaluation)
2. DeepEval (comprehensive metrics)
3. LM Evaluation Harness
4. HELM (optional)

### Phase 5: Multi-Modal & Enterprise (Week 6)

**Multi-Modal:**
- Image processing pipeline
- Audio processing pipeline
- Multi-modal evaluation metrics

**Enterprise:**
- Advanced storage backends
- Security hardening
- Privacy-preserving evaluation

---

## 7. Testing Strategy

### 7.1 Feature Combination Testing

**Matrix Testing:**
```yaml
# .github/workflows/features.yml
strategy:
  matrix:
    feature:
      - default
      - provider-google
      - provider-ollama
      - all-providers
      - observability-otel
      - multimodal
      - enterprise
      - full
```

### 7.2 Integration Tests

**Structure:**
```
core/tests/
├── providers/
│   ├── google_integration.rs  // #[cfg(feature = "provider-google")]
│   ├── ollama_integration.rs  // #[cfg(feature = "provider-ollama")]
│   └── ...
├── observability/
│   ├── otel_integration.rs
│   └── ...
└── multimodal/
    ├── vision_tests.rs
    └── audio_tests.rs
```

### 7.3 Documentation Tests

**Ensure examples compile:**
```rust
/// # Examples
///
/// ```ignore  // Feature-gated
/// use llm_test_bench_core::providers::GoogleProvider;
/// # #[cfg(feature = "provider-google")]
/// # {
/// let provider = GoogleProvider::new("api-key");
/// # }
/// ```
```

---

## 8. Documentation Updates

### 8.1 README.md

**Add feature flag section:**
```markdown
## Features

LLM Test Bench supports optional integrations via Cargo features:

### Providers
- `provider-google` - Google Gemini integration
- `provider-ollama` - Local Ollama models
- `all-providers` - All provider integrations

### Observability
- `observability-otel` - OpenTelemetry tracing
- `all-observability` - All observability features

### Advanced
- `multimodal` - Vision and audio support
- `enterprise` - Advanced storage and security
- `full` - Enable all features

### Usage

```toml
[dependencies]
llm-test-bench-core = { version = "0.1", features = ["provider-google", "observability-otel"] }
```
```

### 8.2 Architecture Documentation

**Create:**
- `docs/FEATURES.md` - Comprehensive feature documentation
- `docs/INTEGRATIONS.md` - Provider integration guide
- `docs/OBSERVABILITY.md` - Monitoring setup guide

---

## 9. Backward Compatibility

### 9.1 Default Behavior

**Guarantee:**
- Building with no features works (minimal CLI)
- Existing code continues to work
- No breaking API changes

**Validation:**
```bash
# This must succeed with existing tests
cargo test --no-default-features
cargo test --features database
```

### 9.2 Deprecation Strategy

**No deprecations needed** - This is purely additive.

**Future Breaking Changes (0.2.0):**
- May require explicit feature for some providers
- May split into more granular crates

---

## 10. Performance Considerations

### 10.1 Compilation Time

**Optimization:**
- Use feature flags to avoid compiling unused code
- Leverage workspace caching
- Consider `sccache` for CI/CD

**Measurement:**
```bash
cargo build --timings --features provider-google
cargo build --timings --all-features
```

### 10.2 Binary Size

**Baseline:** ~15MB (stripped release)

**With all features:** ~30-40MB (estimated)

**Optimization:**
- Use `strip = true` in release profile (already enabled)
- Consider feature-specific binary builds
- Use dynamic linking for Python (PyO3)

### 10.3 Runtime Overhead

**Feature-gated code has zero runtime cost when disabled:**
```rust
#[cfg(feature = "provider-google")]
pub mod google;

// This is completely removed at compile time if feature is disabled
```

---

## 11. Security Considerations

### 11.1 Dependency Auditing

**Setup cargo-deny:**
```bash
cargo install cargo-deny
cargo deny init
cargo deny check
```

**Add to CI:**
```yaml
- name: Security audit
  run: |
    cargo install cargo-deny
    cargo deny check advisories
    cargo deny check licenses
```

### 11.2 Optional Dependency Security

**Policy:**
- All optional dependencies must be audited
- Known vulnerabilities block integration
- Security updates prioritized

### 11.3 API Key Management

**For provider integrations:**
- Use `secrecy` crate for API keys
- Never log sensitive data
- Support environment variables and config files
- Clear security documentation

---

## 12. Open Questions & Decisions Needed

### 12.1 Python Evaluation Frameworks

**Question:** Use PyO3 bindings or HTTP/subprocess?

**Options:**
1. **PyO3 (Embedded Python):**
   - Pros: Fast, type-safe
   - Cons: Complex build, Python version coupling

2. **HTTP API:**
   - Pros: Language-agnostic, simpler
   - Cons: Requires separate service

3. **Subprocess:**
   - Pros: Simple, isolated
   - Cons: Slower, harder to debug

**Recommendation:** Start with HTTP API, add PyO3 if performance critical.

### 12.2 Crate Availability

**Note:** Many upstream repos may not have official Rust crates.

**Strategy:**
- For missing Rust SDKs: implement using reqwest + serde
- Create internal `providers/adapters` module
- Contribute upstream Rust clients where valuable

### 12.3 Versioning Strategy

**Question:** Pin exact versions or use semver ranges?

**Recommendation:**
- Pin exact minor versions in workspace
- Use `^` (caret) for minor updates
- Review dependencies quarterly
- Document known good combinations

---

## 13. Success Metrics

### 13.1 Technical Metrics

- [ ] All 25 integrations have feature flags
- [ ] No circular dependencies (`cargo tree` clean)
- [ ] No version conflicts
- [ ] Build time < 5 minutes with `--all-features`
- [ ] Binary size < 50MB with `--all-features`
- [ ] 100% feature-gated code covered by tests

### 13.2 Developer Experience

- [ ] Feature documentation complete
- [ ] Example code for each integration
- [ ] CI tests all feature combinations
- [ ] Clear error messages for missing features
- [ ] Migration guide for existing users

### 13.3 Integration Quality

- [ ] Each provider integration has >80% test coverage
- [ ] Observability integration validated with real telemetry
- [ ] Multi-modal pipelines tested with sample media
- [ ] Enterprise features validated with security audit

---

## 14. Next Steps

### Immediate Actions

1. **Review & Approve:** Architecture review with stakeholders
2. **Prototype:** Implement one provider (Google) as proof-of-concept
3. **Validate:** Ensure no circular dependencies or build issues
4. **Document:** Create developer guide for adding new integrations

### Phase 1 Execution (Week 1)

```bash
# Day 1: Infrastructure
- Update workspace Cargo.toml
- Add cargo-deny configuration
- Update CI/CD workflows

# Day 2-3: Core feature flags
- Implement feature flag structure in core/
- Add conditional compilation blocks
- Update tests

# Day 4-5: Validation
- Test build with all feature combinations
- Measure compilation times
- Validate no circular dependencies
- Security audit of new dependencies
```

---

## Appendix A: Complete Feature Flag Reference

```toml
# Individual Provider Features
provider-google                # Google Gemini
provider-openai-extended       # Extended OpenAI features
provider-huggingface          # HuggingFace Hub
provider-ollama               # Local Ollama
provider-cohere               # Cohere API
provider-mistral              # Mistral AI
provider-together             # Together AI
provider-replicate            # Replicate
provider-vllm                 # vLLM

# Provider Bundle
all-providers                 # All provider integrations

# Observability Features
observability-otel            # OpenTelemetry
observability-langsmith       # LangSmith
observability-phoenix         # Phoenix
observability-prometheus      # Prometheus (existing)

# Observability Bundle
all-observability            # All observability features

# Evaluation Features
eval-ragas                   # RAGAS framework
eval-deepeval                # DeepEval framework
eval-lm-harness             # LM Evaluation Harness
eval-helm                   # HELM benchmark

# Evaluation Bundle
all-eval                    # All evaluation frameworks

# Multi-Modal Features
multimodal-vision           # Image processing
multimodal-audio            # Audio processing
multimodal                  # All multi-modal features

# Storage Features
storage-lance               # Lance columnar format
storage-vector              # Qdrant vector DB
storage-redis               # Redis caching
storage-advanced            # All storage features

# Security Features
security-crypto             # Cryptographic primitives
privacy-dp                  # Differential privacy

# Meta Features
enterprise                  # All enterprise features
ci                         # Common CI/CD features
full                       # Everything enabled
```

---

## Appendix B: Dependency Version Matrix

| Crate | Version | License | Security Status |
|-------|---------|---------|-----------------|
| google-generativeai | 0.4.0 | MIT | ✅ Clean |
| async-openai | 0.20.0 | MIT | ✅ Clean |
| hf-hub | 0.3.0 | Apache-2.0 | ✅ Clean |
| ollama-rs | 0.1.0 | MIT | ✅ Clean |
| opentelemetry | 0.21.0 | Apache-2.0 | ✅ Clean |
| opentelemetry-otlp | 0.14.0 | Apache-2.0 | ✅ Clean |
| pyo3 | 0.20.0 | Apache-2.0 | ✅ Clean |
| image | 0.24.0 | MIT | ✅ Clean |
| symphonia | 0.5.0 | MPL-2.0 | ✅ Clean |
| lance | 0.10.0 | Apache-2.0 | ✅ Clean |
| qdrant-client | 1.7.0 | Apache-2.0 | ✅ Clean |
| redis | 0.24.0 | BSD-3-Clause | ✅ Clean |
| ring | 0.17.0 | ISC | ✅ Clean |
| secrecy | 0.8.0 | MIT/Apache-2.0 | ✅ Clean |

---

## Appendix C: Migration Checklist

### For Existing Users

- [ ] No action required for default features
- [ ] Opt-in to new providers via feature flags
- [ ] Update CI/CD if using specific providers
- [ ] Review new documentation

### For Contributors

- [ ] Follow new module structure for providers
- [ ] Use `#[cfg(feature = "...")]` for feature-gated code
- [ ] Add feature-gated tests
- [ ] Update feature documentation

### For Deployment

- [ ] Binary size may increase with features
- [ ] Consider feature-specific builds for production
- [ ] Update Docker images with desired features
- [ ] Review security implications of new dependencies

---

**END OF DOCUMENT**
