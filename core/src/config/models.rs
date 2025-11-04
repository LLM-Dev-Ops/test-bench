//! Configuration data structures for LLM Test Bench
//!
//! This module defines the complete configuration schema including:
//! - Provider configurations (OpenAI, Anthropic, etc.)
//! - Benchmark settings
//! - Evaluation metrics configuration
//! - Output and reporting options

use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use std::collections::HashMap;
use std::path::PathBuf;

/// Root configuration structure for LLM Test Bench
///
/// This is the top-level configuration that combines all settings.
/// Configuration sources are merged in this precedence (highest to lowest):
/// 1. CLI Arguments
/// 2. Environment Variables (LLM_TEST_BENCH_ prefix)
/// 3. Config Files (~/.config/llm-test-bench/config.toml)
/// 4. Defaults
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
#[serde(default)]
pub struct Config {
    /// Provider configurations (OpenAI, Anthropic, etc.)
    pub providers: HashMap<String, ProviderConfig>,

    /// Benchmark execution settings
    pub benchmarks: BenchmarkConfig,

    /// Evaluation metrics configuration
    pub evaluation: EvaluationConfig,

    /// Orchestration configuration (Phase 4)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orchestration: Option<OrchestrationConfig>,

    /// Analytics configuration (Phase 4)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics: Option<AnalyticsConfig>,

    /// Dashboard configuration (Phase 4)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dashboard: Option<DashboardConfig>,

    /// Global timeout settings (optional override)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_timeout_seconds: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        let mut providers = HashMap::new();

        // Default OpenAI provider
        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                api_key_env: "OPENAI_API_KEY".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
                default_model: "gpt-4o".to_string(),  // Updated to latest GPT-4o
                timeout_seconds: 30,
                max_retries: 3,
                rate_limit_rpm: None,
            },
        );

        // Default Anthropic provider
        providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                api_key_env: "ANTHROPIC_API_KEY".to_string(),
                base_url: "https://api.anthropic.com/v1".to_string(),
                default_model: "claude-3-5-sonnet-latest".to_string(),  // Updated to Claude 3.5 Sonnet
                timeout_seconds: 30,
                max_retries: 3,
                rate_limit_rpm: None,
            },
        );

        // Default Google provider
        providers.insert(
            "google".to_string(),
            ProviderConfig {
                api_key_env: "GOOGLE_API_KEY".to_string(),
                base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
                default_model: "gemini-1.5-pro".to_string(),  // Added Google Gemini
                timeout_seconds: 30,
                max_retries: 3,
                rate_limit_rpm: None,
            },
        );

        Self {
            providers,
            benchmarks: BenchmarkConfig::default(),
            evaluation: EvaluationConfig::default(),
            orchestration: Some(OrchestrationConfig::default()),
            analytics: Some(AnalyticsConfig::default()),
            dashboard: Some(DashboardConfig::default()),
            global_timeout_seconds: None,
        }
    }
}

/// Provider-specific configuration
///
/// Each LLM provider (OpenAI, Anthropic, etc.) has its own configuration
/// including API credentials, endpoints, and retry settings.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct ProviderConfig {
    /// Environment variable name containing the API key
    ///
    /// Example: "OPENAI_API_KEY", "ANTHROPIC_API_KEY"
    #[validate(min_length = 1)]
    pub api_key_env: String,

    /// Base URL for the provider's API
    ///
    /// Example: "https://api.openai.com/v1"
    #[validate(min_length = 1)]
    pub base_url: String,

    /// Default model to use if not specified in request
    ///
    /// Example: "gpt-4-turbo", "claude-3-sonnet-20240229"
    #[validate(min_length = 1)]
    pub default_model: String,

    /// Request timeout in seconds
    ///
    /// Default: 30 seconds
    #[validate(minimum = 1)]
    #[validate(maximum = 300)]
    pub timeout_seconds: u64,

    /// Maximum number of retry attempts for failed requests
    ///
    /// Default: 3 retries
    #[validate(maximum = 10)]
    pub max_retries: u32,

    /// Optional rate limit in requests per minute
    ///
    /// If set, the client will throttle requests to stay under this limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_rpm: Option<u32>,
}

/// Benchmark execution configuration
///
/// Controls how benchmarks are executed, including parallelism,
/// output directories, and response storage options.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
#[serde(default)]
pub struct BenchmarkConfig {
    /// Directory where benchmark results will be saved
    ///
    /// Default: "./bench-results"
    pub output_dir: PathBuf,

    /// Whether to save full LLM responses to disk
    ///
    /// Default: true
    /// Warning: This can consume significant disk space for large benchmark suites
    pub save_responses: bool,

    /// Number of parallel requests to execute concurrently
    ///
    /// Default: 5
    /// Note: Higher values may hit provider rate limits
    #[validate(minimum = 1)]
    #[validate(maximum = 100)]
    pub parallel_requests: usize,

    /// Whether to continue benchmarking after a test fails
    ///
    /// Default: true
    pub continue_on_failure: bool,

    /// Optional seed for reproducible randomization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub random_seed: Option<u64>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("./bench-results"),
            save_responses: true,
            parallel_requests: 5,
            continue_on_failure: true,
            random_seed: None,
        }
    }
}

/// Evaluation metrics configuration
///
/// Defines which metrics to compute and how to compute them,
/// including LLM-as-judge configurations.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
#[serde(default)]
pub struct EvaluationConfig {
    /// List of metrics to compute
    ///
    /// Available metrics:
    /// - "perplexity": Language model prediction quality
    /// - "faithfulness": Factual accuracy and hallucination detection
    /// - "relevance": Task/prompt alignment scoring
    /// - "coherence": Output fluency and logical consistency
    /// - "latency": Response time measurement
    /// - "token_efficiency": Token usage analysis
    #[validate(min_length = 1)]
    pub metrics: Vec<String>,

    /// Model to use for LLM-as-judge evaluations
    ///
    /// Default: "gpt-4"
    /// This model is used for qualitative assessments like faithfulness and relevance
    #[validate(min_length = 1)]
    pub llm_judge_model: String,

    /// Provider to use for LLM-as-judge
    ///
    /// Default: "openai"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_judge_provider: Option<String>,

    /// Temperature for judge model (0.0 = deterministic)
    ///
    /// Default: 0.0
    #[validate(minimum = 0.0)]
    #[validate(maximum = 2.0)]
    pub judge_temperature: f32,

    /// Maximum tokens for judge responses
    ///
    /// Default: 500
    #[validate(minimum = 50)]
    #[validate(maximum = 4000)]
    pub judge_max_tokens: usize,

    /// Enable evaluation result caching
    ///
    /// Default: true
    pub cache_enabled: bool,

    /// Cache directory for evaluation results
    ///
    /// Default: "~/.cache/llm-test-bench/evaluations"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_dir: Option<PathBuf>,

    /// Cache TTL in hours
    ///
    /// Default: 168 (7 days)
    #[validate(minimum = 1)]
    #[validate(maximum = 8760)]
    pub cache_ttl_hours: i64,

    /// Maximum evaluation cost per test (USD)
    ///
    /// Default: 0.10
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(minimum = 0.0)]
    pub max_evaluation_cost_per_test: Option<f64>,

    /// Confidence threshold for passing evaluations (0.0 - 1.0)
    ///
    /// Default: 0.7
    #[validate(minimum = 0.0)]
    #[validate(maximum = 1.0)]
    pub confidence_threshold: f64,

    /// Whether to include detailed evaluation explanations
    ///
    /// Default: true
    pub include_explanations: bool,
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            metrics: vec![
                "perplexity".to_string(),
                "faithfulness".to_string(),
                "relevance".to_string(),
                "latency".to_string(),
            ],
            llm_judge_model: "gpt-4".to_string(),
            llm_judge_provider: Some("openai".to_string()),
            judge_temperature: 0.0,
            judge_max_tokens: 500,
            cache_enabled: true,
            cache_dir: None, // Will use default if None
            cache_ttl_hours: 168,
            max_evaluation_cost_per_test: Some(0.10),
            confidence_threshold: 0.7,
            include_explanations: true,
        }
    }
}

/// Available evaluation metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metric {
    /// Language model prediction quality (lower is better)
    Perplexity,
    /// Factual accuracy and hallucination detection
    Faithfulness,
    /// Task/prompt alignment scoring
    Relevance,
    /// Output fluency and logical consistency
    Coherence,
    /// Response time measurement
    Latency,
    /// Token usage analysis
    TokenEfficiency,
}

impl Metric {
    /// Parse metric from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "perplexity" => Some(Self::Perplexity),
            "faithfulness" => Some(Self::Faithfulness),
            "relevance" => Some(Self::Relevance),
            "coherence" => Some(Self::Coherence),
            "latency" => Some(Self::Latency),
            "token_efficiency" => Some(Self::TokenEfficiency),
            _ => None,
        }
    }

    /// Get metric name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Perplexity => "perplexity",
            Self::Faithfulness => "faithfulness",
            Self::Relevance => "relevance",
            Self::Coherence => "coherence",
            Self::Latency => "latency",
            Self::TokenEfficiency => "token_efficiency",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_config_has_providers() {
        let config = Config::default();
        assert!(config.providers.contains_key("openai"));
        assert!(config.providers.contains_key("anthropic"));
        assert!(config.providers.contains_key("google"));
    }

    #[test]
    fn test_provider_config_validation() {
        let provider = ProviderConfig {
            api_key_env: "TEST_KEY".to_string(),
            base_url: "https://api.example.com".to_string(),
            default_model: "test-model".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            rate_limit_rpm: None,
        };
        assert!(provider.validate().is_ok());
    }

    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.parallel_requests, 5);
        assert_eq!(config.save_responses, true);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_evaluation_config_default() {
        let config = EvaluationConfig::default();
        assert!(config.metrics.contains(&"faithfulness".to_string()));
        assert_eq!(config.llm_judge_model, "gpt-4");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_metric_from_str() {
        assert_eq!(Metric::from_str("perplexity"), Some(Metric::Perplexity));
        assert_eq!(Metric::from_str("FAITHFULNESS"), Some(Metric::Faithfulness));
        assert_eq!(Metric::from_str("unknown"), None);
    }

    #[test]
    fn test_metric_as_str() {
        assert_eq!(Metric::Latency.as_str(), "latency");
        assert_eq!(Metric::TokenEfficiency.as_str(), "token_efficiency");
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).expect("Failed to serialize");
        let deserialized: Config = toml::from_str(&toml_str).expect("Failed to deserialize");
        assert_eq!(config, deserialized);
    }
}

/// Orchestration configuration for Phase 4 features
///
/// Controls multi-model comparison and orchestration behavior
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
#[serde(default)]
pub struct OrchestrationConfig {
    /// Maximum number of models to compare in parallel
    #[validate(minimum = 1)]
    #[validate(maximum = 20)]
    pub max_parallel_models: usize,

    /// Timeout for comparison operations in seconds
    #[validate(minimum = 1)]
    #[validate(maximum = 3600)]
    pub comparison_timeout_seconds: u64,

    /// Routing strategy for model selection
    /// Options: "quality", "cost_optimized", "latency", "balanced"
    pub routing_strategy: String,

    /// Auto-update model profiles from benchmark results
    pub auto_update_profiles: bool,

    /// Path to model profiles storage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles_path: Option<PathBuf>,

    /// Enable statistical significance testing in comparisons
    pub enable_statistical_tests: bool,

    /// Weight for exponential moving average in profile updates (0.0-1.0)
    #[validate(minimum = 0.0)]
    #[validate(maximum = 1.0)]
    pub profile_update_weight: f64,
}

impl Default for OrchestrationConfig {
    fn default() -> Self {
        Self {
            max_parallel_models: 10,
            comparison_timeout_seconds: 300,
            routing_strategy: "balanced".to_string(),
            auto_update_profiles: true,
            profiles_path: None, // Will use default if None
            enable_statistical_tests: false,
            profile_update_weight: 0.1,
        }
    }
}

/// Analytics configuration for statistical analysis
///
/// Controls statistical testing and regression detection
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
#[serde(default)]
pub struct AnalyticsConfig {
    /// Confidence level for statistical tests (0.90, 0.95, 0.99)
    #[validate(minimum = 0.8)]
    #[validate(maximum = 0.999)]
    pub confidence_level: f64,

    /// Effect size threshold for practical significance
    #[validate(minimum = 0.0)]
    #[validate(maximum = 1.0)]
    pub effect_size_threshold: f64,

    /// Quality threshold for model acceptance (0.0-1.0)
    #[validate(minimum = 0.0)]
    #[validate(maximum = 1.0)]
    pub quality_threshold: f64,

    /// Sample size for statistical tests
    #[validate(minimum = 2)]
    pub min_sample_size: usize,

    /// Enable detailed statistical reports
    pub enable_detailed_reports: bool,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            confidence_level: 0.95,
            effect_size_threshold: 0.2,
            quality_threshold: 0.75,
            min_sample_size: 30,
            enable_detailed_reports: true,
        }
    }
}

/// Dashboard configuration for visualization
///
/// Controls dashboard generation and appearance
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
#[serde(default)]
pub struct DashboardConfig {
    /// Dashboard theme (light, dark, auto)
    pub theme: String,

    /// Chart color scheme
    pub chart_colors: Vec<String>,

    /// Maximum data points per chart
    #[validate(minimum = 10)]
    #[validate(maximum = 10000)]
    pub max_data_points: usize,

    /// Enable interactive charts
    pub enable_interactive: bool,

    /// Include raw data in dashboard
    pub include_raw_data: bool,

    /// Dashboard refresh interval in seconds (0 = no auto-refresh)
    pub refresh_interval_seconds: u64,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            theme: "auto".to_string(),
            chart_colors: vec![
                "#3B82F6".to_string(), // Blue
                "#10B981".to_string(), // Green
                "#F59E0B".to_string(), // Amber
                "#EF4444".to_string(), // Red
                "#8B5CF6".to_string(), // Purple
                "#EC4899".to_string(), // Pink
            ],
            max_data_points: 1000,
            enable_interactive: true,
            include_raw_data: false,
            refresh_interval_seconds: 0,
        }
    }
}
