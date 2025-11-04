use colored::Colorize;
use std::fmt;
use thiserror::Error;

/// CLI-specific errors with contextual messages and suggestions
#[derive(Debug, Error)]
pub enum CliError {
    /// Model not supported
    #[error("Model '{model}' not supported by provider '{provider}'")]
    UnsupportedModel {
        model: String,
        provider: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    /// Evaluation failed
    #[error("Evaluation failed: {reason}")]
    EvaluationFailed {
        reason: String,
        suggestion: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    /// Cost limit exceeded
    #[error("Cost limit exceeded: ${actual:.2} > ${limit:.2}")]
    CostLimitExceeded { actual: f64, limit: f64 },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError {
        message: String,
        suggestion: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    /// File not found
    #[error("File not found: {path}")]
    FileNotFound { path: String, suggestion: String },

    /// Invalid input
    #[error("Invalid input: {message}")]
    InvalidInput { message: String, suggestion: String },

    /// Provider error
    #[error("Provider error: {provider} - {message}")]
    ProviderError {
        provider: String,
        message: String,
        suggestion: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    /// Regression detected
    #[error("Performance regression detected")]
    RegressionDetected {
        metric: String,
        baseline: f64,
        current: f64,
        threshold: f64,
    },

    /// Dataset error
    #[error("Dataset error: {message}")]
    DatasetError {
        message: String,
        suggestion: String,
        #[source]
        source: Option<anyhow::Error>,
    },
}

impl CliError {
    /// Create an unsupported model error with suggestions
    pub fn unsupported_model(model: &str, provider: &str, available_models: &[String]) -> Self {
        let mut err = Self::UnsupportedModel {
            model: model.to_string(),
            provider: provider.to_string(),
            source: None,
        };

        if !available_models.is_empty() {
            eprintln!(
                "\n{} Available models for {}:",
                "Suggestion:".yellow().bold(),
                provider.bold()
            );
            for m in available_models {
                eprintln!("  • {}", m.cyan());
            }
        }

        err
    }

    /// Create an evaluation failed error with suggestions
    pub fn evaluation_failed(reason: &str, suggestion: &str) -> Self {
        Self::EvaluationFailed {
            reason: reason.to_string(),
            suggestion: suggestion.to_string(),
            source: None,
        }
    }

    /// Create a cost limit exceeded error with suggestions
    pub fn cost_limit_exceeded(actual: f64, limit: f64) -> Self {
        let err = Self::CostLimitExceeded { actual, limit };

        eprintln!(
            "\n{} To proceed:",
            "Suggestion:".yellow().bold()
        );
        eprintln!("  • Increase your cost limit: --max-cost-increase {:.0}", (actual - limit) / limit * 100.0 + 10.0);
        eprintln!("  • Reduce monthly request volume: --monthly-requests");
        eprintln!("  • Use a more cost-effective model");

        err
    }

    /// Create a configuration error with suggestions
    pub fn configuration_error(message: &str, suggestion: &str) -> Self {
        Self::ConfigurationError {
            message: message.to_string(),
            suggestion: suggestion.to_string(),
            source: None,
        }
    }

    /// Create a file not found error with suggestions
    pub fn file_not_found(path: &str) -> Self {
        let suggestion = format!(
            "Check that the file exists and the path is correct: {}",
            path
        );

        let err = Self::FileNotFound {
            path: path.to_string(),
            suggestion,
        };

        eprintln!("\n{} The file might be:", "Suggestion:".yellow().bold());
        eprintln!("  • In a different directory");
        eprintln!("  • Named differently");
        eprintln!("  • Not yet created");

        err
    }

    /// Create an invalid input error with suggestions
    pub fn invalid_input(message: &str, suggestion: &str) -> Self {
        Self::InvalidInput {
            message: message.to_string(),
            suggestion: suggestion.to_string(),
        }
    }

    /// Create a provider error with suggestions
    pub fn provider_error(provider: &str, message: &str, suggestion: &str) -> Self {
        Self::ProviderError {
            provider: provider.to_string(),
            message: message.to_string(),
            suggestion: suggestion.to_string(),
            source: None,
        }
    }

    /// Create a regression detected error
    pub fn regression_detected(metric: &str, baseline: f64, current: f64, threshold: f64) -> Self {
        let err = Self::RegressionDetected {
            metric: metric.to_string(),
            baseline,
            current,
            threshold,
        };

        let change = ((current - baseline) / baseline * 100.0).abs();

        eprintln!("\n{}", "Regression Details:".red().bold());
        eprintln!("  Metric: {}", metric.yellow());
        eprintln!("  Baseline: {:.2}", baseline);
        eprintln!("  Current: {:.2}", current);
        eprintln!("  Change: {:.1}%", change);
        eprintln!("  Threshold: {:.2}", threshold);

        eprintln!("\n{} Consider:", "Recommendation:".yellow().bold());
        eprintln!("  • Review recent changes");
        eprintln!("  • Check system resources");
        eprintln!("  • Run more tests to confirm");
        eprintln!("  • Adjust threshold if this is expected");

        err
    }

    /// Create a dataset error with suggestions
    pub fn dataset_error(message: &str, suggestion: &str) -> Self {
        Self::DatasetError {
            message: message.to_string(),
            suggestion: suggestion.to_string(),
            source: None,
        }
    }

    /// Print the error with formatting and suggestions
    pub fn print_error(&self) {
        eprintln!("\n{} {}", "Error:".red().bold(), self);

        match self {
            Self::EvaluationFailed { suggestion, .. }
            | Self::ConfigurationError { suggestion, .. }
            | Self::FileNotFound { suggestion, .. }
            | Self::InvalidInput { suggestion, .. }
            | Self::ProviderError { suggestion, .. }
            | Self::DatasetError { suggestion, .. } => {
                if !suggestion.is_empty() {
                    eprintln!("\n{} {}", "Suggestion:".yellow().bold(), suggestion);
                }
            }
            _ => {}
        }
    }
}

/// Helper function to format available options
pub fn format_available_options(options: &[String]) -> String {
    if options.is_empty() {
        return "None available".to_string();
    }

    options
        .iter()
        .map(|o| format!("  • {}", o))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Exit codes for different error scenarios
pub mod exit_codes {
    /// Successful execution
    pub const SUCCESS: i32 = 0;

    /// General error
    pub const ERROR: i32 = 1;

    /// Regression detected (when --fail-on-regression is used)
    pub const REGRESSION: i32 = 2;

    /// Configuration error
    pub const CONFIG_ERROR: i32 = 3;

    /// Invalid input
    pub const INVALID_INPUT: i32 = 4;

    /// Provider error (API key missing, rate limit, etc.)
    pub const PROVIDER_ERROR: i32 = 5;

    /// Cost limit exceeded
    pub const COST_LIMIT: i32 = 6;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_model_error() {
        let available = vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()];
        let err = CliError::unsupported_model("gpt-5", "openai", &available);

        match err {
            CliError::UnsupportedModel { model, provider, .. } => {
                assert_eq!(model, "gpt-5");
                assert_eq!(provider, "openai");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_cost_limit_exceeded() {
        let err = CliError::cost_limit_exceeded(150.0, 100.0);

        match err {
            CliError::CostLimitExceeded { actual, limit } => {
                assert_eq!(actual, 150.0);
                assert_eq!(limit, 100.0);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_regression_detected() {
        let err = CliError::regression_detected("latency", 100.0, 150.0, 0.2);

        match err {
            CliError::RegressionDetected {
                metric,
                baseline,
                current,
                threshold,
            } => {
                assert_eq!(metric, "latency");
                assert_eq!(baseline, 100.0);
                assert_eq!(current, 150.0);
                assert_eq!(threshold, 0.2);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_format_available_options() {
        let options = vec!["option1".to_string(), "option2".to_string()];
        let formatted = format_available_options(&options);

        assert!(formatted.contains("option1"));
        assert!(formatted.contains("option2"));
        assert!(formatted.contains("•"));
    }

    #[test]
    fn test_format_available_options_empty() {
        let options: Vec<String> = vec![];
        let formatted = format_available_options(&options);

        assert_eq!(formatted, "None available");
    }
}
