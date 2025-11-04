use anyhow::{anyhow, Context, Result};
use clap::Args;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use llm_test_bench_core::config::ConfigLoader;
use llm_test_bench_core::providers::{
    AnthropicProvider, CompletionRequest, OpenAIProvider, Provider, ProviderError,
};
use std::path::PathBuf;
use std::time::Instant;

use crate::output::{display_error, display_response, OutputFormat, StreamingOutput};

#[derive(Args, Debug)]
pub struct TestArgs {
    /// LLM provider to use (openai, anthropic)
    pub provider: String,

    /// Prompt to send to the LLM
    #[arg(short, long)]
    pub prompt: String,

    /// Model to use for the test
    #[arg(short, long)]
    pub model: Option<String>,

    /// Temperature for generation (0.0-2.0)
    #[arg(short, long)]
    pub temperature: Option<f32>,

    /// Maximum tokens to generate
    #[arg(long)]
    pub max_tokens: Option<u32>,

    /// Top-p sampling parameter (0.0-1.0)
    #[arg(long)]
    pub top_p: Option<f32>,

    /// Stop sequences
    #[arg(long)]
    pub stop: Option<Vec<String>>,

    /// Enable streaming mode
    #[arg(short, long)]
    pub stream: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value = "pretty")]
    pub output_format: OutputFormat,

    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

/// Create a provider instance from configuration
fn create_provider(
    provider_name: &str,
    config_path: &Option<PathBuf>,
) -> Result<Box<dyn Provider>> {
    // Load configuration
    let mut loader = ConfigLoader::new();
    if let Some(path) = config_path {
        loader = loader.with_file(path);
    }

    let config = loader.load().context("Failed to load configuration")?;

    // Get provider config
    let provider_config = config
        .providers
        .get(provider_name)
        .ok_or_else(|| anyhow!("Provider '{}' not found in configuration", provider_name))?;

    // Get API key from environment
    let api_key = std::env::var(&provider_config.api_key_env)
        .with_context(|| format!("Environment variable '{}' not set", provider_config.api_key_env))?;

    // Create provider instance
    match provider_name {
        "openai" => {
            let provider = OpenAIProvider::with_base_url(api_key, provider_config.base_url.clone())
                .context("Failed to create OpenAI provider")?;
            Ok(Box::new(provider))
        }
        "anthropic" => {
            let provider = AnthropicProvider::with_base_url(api_key, provider_config.base_url.clone());
            Ok(Box::new(provider))
        }
        _ => Err(anyhow!("Unknown provider: {}", provider_name)),
    }
}

/// Build a completion request from command arguments
fn build_completion_request(
    args: &TestArgs,
    provider: &Box<dyn Provider>,
) -> Result<CompletionRequest> {
    // Get default model if not specified
    let mut loader = ConfigLoader::new();
    if let Some(ref path) = args.config {
        loader = loader.with_file(path);
    }
    let config = loader.load()?;
    let provider_config = config.providers.get(&args.provider).unwrap();

    let model = args.model.clone().unwrap_or_else(|| provider_config.default_model.clone());

    // Validate model is supported
    let supported_models = provider.supported_models();
    if !supported_models.iter().any(|m| m.id == model) {
        return Err(anyhow!(
            "Model '{}' is not supported by provider '{}'\nSupported models: {}",
            model,
            args.provider,
            supported_models
                .iter()
                .map(|m| m.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    Ok(CompletionRequest {
        prompt: args.prompt.clone(),
        model,
        temperature: args.temperature,
        max_tokens: args.max_tokens.map(|t| t as usize),
        top_p: args.top_p,
        stop: args.stop.clone(),
        stream: args.stream,
    })
}

/// Execute non-streaming completion
async fn execute_non_streaming(
    provider: &Box<dyn Provider>,
    request: CompletionRequest,
    output_format: OutputFormat,
) -> Result<()> {
    // Show progress indicator for non-streaming
    let spinner = if output_format == OutputFormat::Pretty {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(format!("Requesting completion from {}...", provider.name()));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    let start = Instant::now();
    let response = provider.complete(request).await.map_err(map_provider_error)?;
    let elapsed = start.elapsed();

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    // Display response
    display_response(&response, output_format)?;

    // Show timing info for pretty format
    if output_format == OutputFormat::Pretty {
        println!("⏱️  Response time: {:.2}s", elapsed.as_secs_f64());
    }

    Ok(())
}

/// Execute streaming completion
async fn execute_streaming(
    provider: &Box<dyn Provider>,
    request: CompletionRequest,
    output_format: OutputFormat,
) -> Result<()> {
    let model = request.model.clone();
    let mut stream = provider.stream(request).await.map_err(map_provider_error)?;

    let mut output = StreamingOutput::new(output_format);
    output.display_header(&model);

    let mut is_first = true;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(map_provider_error)?;

        if !chunk.is_empty() {
            output.display_chunk(&chunk, is_first)?;
            is_first = false;
        }
    }

    output.display_footer(None);

    Ok(())
}

/// Map provider errors to user-friendly messages
fn map_provider_error(err: ProviderError) -> anyhow::Error {
    match err {
        ProviderError::InvalidApiKey => {
            anyhow::anyhow!("Invalid API key\n\nSet the appropriate environment variable (OPENAI_API_KEY or ANTHROPIC_API_KEY)")
        }
        ProviderError::RateLimitExceeded { retry_after } => {
            if let Some(duration) = retry_after {
                anyhow::anyhow!("Rate limit exceeded\n\nRetry after {} seconds", duration.as_secs())
            } else {
                anyhow::anyhow!("Rate limit exceeded\n\nWait a moment and try again")
            }
        }
        ProviderError::ContextLengthExceeded { tokens, max } => {
            anyhow::anyhow!(
                "Prompt too long: {} tokens (max: {})\n\nReduce prompt length or use a model with larger context",
                tokens, max
            )
        }
        ProviderError::NetworkError(e) => {
            anyhow::anyhow!("Network error: {}\n\nCheck your internet connection", e)
        }
        ProviderError::ModelNotFound { model } => {
            anyhow::anyhow!("Model not found: {}\n\nUse --help to see supported models", model)
        }
        ProviderError::Timeout(duration) => {
            anyhow::anyhow!("Request timeout after {:?}\n\nTry again or use a shorter prompt", duration)
        }
        other => anyhow::anyhow!("{}", other),
    }
}

pub async fn execute(args: TestArgs, verbose: bool) -> Result<()> {
    if verbose {
        tracing::info!("Test command starting with args: {:?}", args);
    }

    // Validate temperature
    if let Some(temp) = args.temperature {
        if !(0.0..=2.0).contains(&temp) {
            return Err(anyhow!("Temperature must be between 0.0 and 2.0"));
        }
    }

    // Validate top_p
    if let Some(top_p) = args.top_p {
        if !(0.0..=1.0).contains(&top_p) {
            return Err(anyhow!("Top-p must be between 0.0 and 1.0"));
        }
    }

    // Create provider
    let provider = create_provider(&args.provider, &args.config)
        .with_context(|| format!("Failed to initialize {} provider", args.provider))?;

    // Build request
    let request = build_completion_request(&args, &provider)?;

    if verbose {
        tracing::info!("Request: model={}, stream={}", request.model, args.stream);
    }

    // Execute request
    if args.stream {
        execute_streaming(&provider, request, args.output_format).await?;
    } else {
        execute_non_streaming(&provider, request, args.output_format).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_validation() {
        // This would be tested with actual command execution
        assert!((0.0..=2.0).contains(&0.7));
        assert!((0.0..=2.0).contains(&0.0));
        assert!((0.0..=2.0).contains(&2.0));
        assert!(!(0.0..=2.0).contains(&2.1));
        assert!(!(0.0..=2.0).contains(&-0.1));
    }

    #[test]
    fn test_top_p_validation() {
        assert!((0.0..=1.0).contains(&0.9));
        assert!((0.0..=1.0).contains(&0.0));
        assert!((0.0..=1.0).contains(&1.0));
        assert!(!(0.0..=1.0).contains(&1.1));
        assert!(!(0.0..=1.0).contains(&-0.1));
    }
}
