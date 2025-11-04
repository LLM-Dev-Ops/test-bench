//! Output formatting for CLI responses
//!
//! This module provides different output formats for LLM completions:
//! - Pretty: Human-readable with colors and formatting
//! - JSON: Compact JSON output
//! - JsonPretty: Pretty-printed JSON
//! - Plain: Just the content text

use anyhow::Result;
use colored::*;
use llm_test_bench_core::providers::CompletionResponse;
use serde::Serialize;

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable pretty output with colors
    Pretty,
    /// Compact JSON output
    Json,
    /// Pretty-printed JSON output
    JsonPretty,
    /// Plain text (only content)
    Plain,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Pretty
    }
}

/// Display a completion response in the specified format
pub fn display_response(response: &CompletionResponse, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Pretty => display_pretty(response),
        OutputFormat::Json => display_json(response),
        OutputFormat::JsonPretty => display_json_pretty(response),
        OutputFormat::Plain => display_plain(response),
    }
}

/// Display in pretty format with colors and structure
fn display_pretty(response: &CompletionResponse) -> Result<()> {
    println!();
    println!("{}", "━".repeat(80).bright_blue());
    println!("{} {}", "Response".bright_blue().bold(), response.id.dimmed());
    println!("{}", "━".repeat(80).bright_blue());
    println!();

    // Content
    println!("{}", response.content);
    println!();

    // Metadata
    println!("{}", "─".repeat(80).dimmed());
    println!("{} {}", "Model:".bright_cyan(), response.model);
    println!("{} {}", "Finish Reason:".bright_cyan(), format!("{:?}", response.finish_reason));
    println!(
        "{} {} prompt + {} completion = {} total",
        "Tokens:".bright_cyan(),
        response.usage.prompt_tokens.to_string().yellow(),
        response.usage.completion_tokens.to_string().yellow(),
        response.usage.total_tokens.to_string().green().bold()
    );
    println!("{} {}", "Created:".bright_cyan(), response.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("{}", "━".repeat(80).bright_blue());
    println!();

    Ok(())
}

/// Display as compact JSON
fn display_json(response: &CompletionResponse) -> Result<()> {
    let json = serde_json::to_string(response)?;
    println!("{}", json);
    Ok(())
}

/// Display as pretty-printed JSON
fn display_json_pretty(response: &CompletionResponse) -> Result<()> {
    let json = serde_json::to_string_pretty(response)?;
    println!("{}", json);
    Ok(())
}

/// Display only the content (plain text)
fn display_plain(response: &CompletionResponse) -> Result<()> {
    print!("{}", response.content);
    Ok(())
}

/// Streaming output handler
pub struct StreamingOutput {
    format: OutputFormat,
    total_chars: usize,
    start_time: std::time::Instant,
}

impl StreamingOutput {
    /// Create a new streaming output handler
    pub fn new(format: OutputFormat) -> Self {
        Self {
            format,
            total_chars: 0,
            start_time: std::time::Instant::now(),
        }
    }

    /// Display the initial header for streaming output
    pub fn display_header(&self, model: &str) {
        match self.format {
            OutputFormat::Pretty => {
                println!();
                println!("{}", "━".repeat(80).bright_blue());
                println!("{} {}", "Streaming Response".bright_blue().bold(), model.dimmed());
                println!("{}", "━".repeat(80).bright_blue());
                println!();
            }
            OutputFormat::Plain => {} // No header for plain
            OutputFormat::Json | OutputFormat::JsonPretty => {
                print!("{{\"chunks\":[");
            }
        }
    }

    /// Display a chunk of streaming content
    pub fn display_chunk(&mut self, content: &str, is_first: bool) -> Result<()> {
        self.total_chars += content.len();

        match self.format {
            OutputFormat::Pretty | OutputFormat::Plain => {
                print!("{}", content);
                use std::io::Write;
                std::io::stdout().flush()?;
            }
            OutputFormat::Json | OutputFormat::JsonPretty => {
                if !is_first {
                    print!(",");
                }
                print!("\"{}\"", content.replace('\"', "\\\""));
            }
        }

        Ok(())
    }

    /// Display the final footer with statistics
    pub fn display_footer(&self, finish_reason: Option<&str>) {
        let elapsed = self.start_time.elapsed();

        match self.format {
            OutputFormat::Pretty => {
                println!("\n");
                println!("{}", "─".repeat(80).dimmed());
                if let Some(reason) = finish_reason {
                    println!("{} {}", "Finish Reason:".bright_cyan(), reason);
                }
                println!(
                    "{} {} chars in {:.2}s ({:.0} chars/s)",
                    "Stats:".bright_cyan(),
                    self.total_chars.to_string().yellow(),
                    elapsed.as_secs_f64(),
                    self.total_chars as f64 / elapsed.as_secs_f64().max(0.001)
                );
                println!("{}", "━".repeat(80).bright_blue());
                println!();
            }
            OutputFormat::Plain => {
                // No footer for plain text
            }
            OutputFormat::Json | OutputFormat::JsonPretty => {
                println!("],\"elapsed_ms\":{}}}", elapsed.as_millis());
            }
        }
    }
}

/// Display an error in a user-friendly way
pub fn display_error(error: &str, suggestion: Option<&str>) {
    eprintln!();
    eprintln!("{} {}", "Error:".red().bold(), error.red());
    if let Some(hint) = suggestion {
        eprintln!();
        eprintln!("{} {}", "Hint:".yellow().bold(), hint.yellow());
    }
    eprintln!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_test_bench_core::providers::{FinishReason, TokenUsage};

    fn create_test_response() -> CompletionResponse {
        CompletionResponse {
            id: "test-123".to_string(),
            content: "Hello, world!".to_string(),
            model: "gpt-4".to_string(),
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            finish_reason: FinishReason::Stop,
            created_at: chrono::Utc::now(),
            metadata: serde_json::Value::Null,
        }
    }

    #[test]
    fn test_display_json() {
        let response = create_test_response();
        assert!(display_json(&response).is_ok());
    }

    #[test]
    fn test_display_json_pretty() {
        let response = create_test_response();
        assert!(display_json_pretty(&response).is_ok());
    }

    #[test]
    fn test_display_plain() {
        let response = create_test_response();
        assert!(display_plain(&response).is_ok());
    }

    #[test]
    fn test_streaming_output() {
        let mut output = StreamingOutput::new(OutputFormat::Plain);
        output.display_header("gpt-4");
        assert!(output.display_chunk("Hello", true).is_ok());
        assert!(output.display_chunk(" world", false).is_ok());
        output.display_footer(Some("stop"));
    }
}
