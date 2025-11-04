//! Model definitions for all supported LLM providers
//!
//! This module provides comprehensive model catalogs for each provider,
//! including model IDs, capabilities, and metadata.

use serde::{Deserialize, Serialize};

/// Model capability flags
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelCapabilities {
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
    pub supports_vision: bool,
    pub max_context_tokens: usize,
    pub max_output_tokens: usize,
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: &'static str,
    pub display_name: &'static str,
    pub provider: &'static str,
    pub capabilities: ModelCapabilities,
    pub cost_per_1k_input: f64,
    pub cost_per_1k_output: f64,
}

// ================================
// OpenAI Models
// ================================

/// OpenAI GPT-4o (Latest flagship model)
pub const GPT_4O: &str = "gpt-4o";
pub const GPT_4O_2024_11_20: &str = "gpt-4o-2024-11-20";
pub const GPT_4O_2024_08_06: &str = "gpt-4o-2024-08-06";
pub const GPT_4O_2024_05_13: &str = "gpt-4o-2024-05-13";

/// OpenAI GPT-4o-mini (Faster, more affordable variant)
pub const GPT_4O_MINI: &str = "gpt-4o-mini";
pub const GPT_4O_MINI_2024_07_18: &str = "gpt-4o-mini-2024-07-18";

/// OpenAI o1 models (Reasoning models)
pub const O1: &str = "o1";
pub const O1_PREVIEW: &str = "o1-preview";
pub const O1_PREVIEW_2024_09_12: &str = "o1-preview-2024-09-12";
pub const O1_MINI: &str = "o1-mini";
pub const O1_MINI_2024_09_12: &str = "o1-mini-2024-09-12";

/// OpenAI GPT-4 Turbo
pub const GPT_4_TURBO: &str = "gpt-4-turbo";
pub const GPT_4_TURBO_2024_04_09: &str = "gpt-4-turbo-2024-04-09";
pub const GPT_4_TURBO_PREVIEW: &str = "gpt-4-turbo-preview";
pub const GPT_4_0125_PREVIEW: &str = "gpt-4-0125-preview";
pub const GPT_4_1106_PREVIEW: &str = "gpt-4-1106-preview";

/// OpenAI GPT-4 (Classic)
pub const GPT_4: &str = "gpt-4";
pub const GPT_4_0613: &str = "gpt-4-0613";

/// OpenAI GPT-3.5 Turbo
pub const GPT_35_TURBO: &str = "gpt-3.5-turbo";
pub const GPT_35_TURBO_0125: &str = "gpt-3.5-turbo-0125";
pub const GPT_35_TURBO_1106: &str = "gpt-3.5-turbo-1106";

// ================================
// Anthropic Claude Models
// ================================

/// Claude Sonnet 4 (Latest generation - if available)
pub const CLAUDE_SONNET_4: &str = "claude-sonnet-4";
pub const CLAUDE_SONNET_4_20250514: &str = "claude-sonnet-4-20250514";

/// Claude 3.5 Sonnet (Current flagship)
pub const CLAUDE_3_5_SONNET_LATEST: &str = "claude-3-5-sonnet-latest";
pub const CLAUDE_3_5_SONNET_20241022: &str = "claude-3-5-sonnet-20241022";
pub const CLAUDE_3_5_SONNET_20240620: &str = "claude-3-5-sonnet-20240620";

/// Claude 3.5 Haiku (Fast and affordable)
pub const CLAUDE_3_5_HAIKU_LATEST: &str = "claude-3-5-haiku-latest";
pub const CLAUDE_3_5_HAIKU_20241022: &str = "claude-3-5-haiku-20241022";

/// Claude 3 Opus (Most capable Claude 3 model)
pub const CLAUDE_3_OPUS_LATEST: &str = "claude-3-opus-latest";
pub const CLAUDE_3_OPUS_20240229: &str = "claude-3-opus-20240229";

/// Claude 3 Sonnet
pub const CLAUDE_3_SONNET_20240229: &str = "claude-3-sonnet-20240229";

/// Claude 3 Haiku
pub const CLAUDE_3_HAIKU_20240307: &str = "claude-3-haiku-20240307";

// ================================
// Google Gemini Models
// ================================

/// Gemini 2.0 (Latest generation)
pub const GEMINI_2_0_FLASH_EXP: &str = "gemini-2.0-flash-exp";
pub const GEMINI_2_0_FLASH_THINKING_EXP: &str = "gemini-2.0-flash-thinking-exp-1219";

/// Gemini 1.5 Pro (Extended context)
pub const GEMINI_1_5_PRO: &str = "gemini-1.5-pro";
pub const GEMINI_1_5_PRO_LATEST: &str = "gemini-1.5-pro-latest";
pub const GEMINI_1_5_PRO_002: &str = "gemini-1.5-pro-002";
pub const GEMINI_1_5_PRO_001: &str = "gemini-1.5-pro-001";

/// Gemini 1.5 Flash (Faster variant)
pub const GEMINI_1_5_FLASH: &str = "gemini-1.5-flash";
pub const GEMINI_1_5_FLASH_LATEST: &str = "gemini-1.5-flash-latest";
pub const GEMINI_1_5_FLASH_002: &str = "gemini-1.5-flash-002";
pub const GEMINI_1_5_FLASH_001: &str = "gemini-1.5-flash-001";
pub const GEMINI_1_5_FLASH_8B: &str = "gemini-1.5-flash-8b";

/// Gemini 1.0 Pro
pub const GEMINI_PRO: &str = "gemini-pro";
pub const GEMINI_PRO_VISION: &str = "gemini-pro-vision";

// ================================
// Model Lists
// ================================

/// All supported OpenAI models
pub const OPENAI_MODELS: &[&str] = &[
    // GPT-4o
    GPT_4O,
    GPT_4O_2024_11_20,
    GPT_4O_2024_08_06,
    GPT_4O_2024_05_13,
    // GPT-4o-mini
    GPT_4O_MINI,
    GPT_4O_MINI_2024_07_18,
    // o1 reasoning models
    O1,
    O1_PREVIEW,
    O1_PREVIEW_2024_09_12,
    O1_MINI,
    O1_MINI_2024_09_12,
    // GPT-4 Turbo
    GPT_4_TURBO,
    GPT_4_TURBO_2024_04_09,
    GPT_4_TURBO_PREVIEW,
    GPT_4_0125_PREVIEW,
    GPT_4_1106_PREVIEW,
    // GPT-4 Classic
    GPT_4,
    GPT_4_0613,
    // GPT-3.5 Turbo
    GPT_35_TURBO,
    GPT_35_TURBO_0125,
    GPT_35_TURBO_1106,
];

/// All supported Anthropic Claude models
pub const ANTHROPIC_MODELS: &[&str] = &[
    // Claude Sonnet 4
    CLAUDE_SONNET_4,
    CLAUDE_SONNET_4_20250514,
    // Claude 3.5 Sonnet
    CLAUDE_3_5_SONNET_LATEST,
    CLAUDE_3_5_SONNET_20241022,
    CLAUDE_3_5_SONNET_20240620,
    // Claude 3.5 Haiku
    CLAUDE_3_5_HAIKU_LATEST,
    CLAUDE_3_5_HAIKU_20241022,
    // Claude 3 Opus
    CLAUDE_3_OPUS_LATEST,
    CLAUDE_3_OPUS_20240229,
    // Claude 3 Sonnet
    CLAUDE_3_SONNET_20240229,
    // Claude 3 Haiku
    CLAUDE_3_HAIKU_20240307,
];

/// All supported Google Gemini models
pub const GOOGLE_MODELS: &[&str] = &[
    // Gemini 2.0
    GEMINI_2_0_FLASH_EXP,
    GEMINI_2_0_FLASH_THINKING_EXP,
    // Gemini 1.5 Pro
    GEMINI_1_5_PRO,
    GEMINI_1_5_PRO_LATEST,
    GEMINI_1_5_PRO_002,
    GEMINI_1_5_PRO_001,
    // Gemini 1.5 Flash
    GEMINI_1_5_FLASH,
    GEMINI_1_5_FLASH_LATEST,
    GEMINI_1_5_FLASH_002,
    GEMINI_1_5_FLASH_001,
    GEMINI_1_5_FLASH_8B,
    // Gemini 1.0
    GEMINI_PRO,
    GEMINI_PRO_VISION,
];

/// Get model metadata by model ID
pub fn get_model_metadata(model_id: &str) -> Option<ModelMetadata> {
    match model_id {
        // OpenAI GPT-4o
        GPT_4O | GPT_4O_2024_11_20 => Some(ModelMetadata {
            id: model_id,
            display_name: "GPT-4o",
            provider: "openai",
            capabilities: ModelCapabilities {
                supports_streaming: true,
                supports_function_calling: true,
                supports_vision: true,
                max_context_tokens: 128_000,
                max_output_tokens: 16_384,
            },
            cost_per_1k_input: 0.0025,
            cost_per_1k_output: 0.01,
        }),

        // OpenAI GPT-4o-mini
        GPT_4O_MINI | GPT_4O_MINI_2024_07_18 => Some(ModelMetadata {
            id: model_id,
            display_name: "GPT-4o-mini",
            provider: "openai",
            capabilities: ModelCapabilities {
                supports_streaming: true,
                supports_function_calling: true,
                supports_vision: true,
                max_context_tokens: 128_000,
                max_output_tokens: 16_384,
            },
            cost_per_1k_input: 0.00015,
            cost_per_1k_output: 0.0006,
        }),

        // Anthropic Claude 3.5 Sonnet
        CLAUDE_3_5_SONNET_LATEST | CLAUDE_3_5_SONNET_20241022 | CLAUDE_3_5_SONNET_20240620 => Some(ModelMetadata {
            id: model_id,
            display_name: "Claude 3.5 Sonnet",
            provider: "anthropic",
            capabilities: ModelCapabilities {
                supports_streaming: true,
                supports_function_calling: true,
                supports_vision: true,
                max_context_tokens: 200_000,
                max_output_tokens: 8_192,
            },
            cost_per_1k_input: 0.003,
            cost_per_1k_output: 0.015,
        }),

        // Anthropic Claude 3.5 Haiku
        CLAUDE_3_5_HAIKU_LATEST | CLAUDE_3_5_HAIKU_20241022 => Some(ModelMetadata {
            id: model_id,
            display_name: "Claude 3.5 Haiku",
            provider: "anthropic",
            capabilities: ModelCapabilities {
                supports_streaming: true,
                supports_function_calling: true,
                supports_vision: true,
                max_context_tokens: 200_000,
                max_output_tokens: 8_192,
            },
            cost_per_1k_input: 0.001,
            cost_per_1k_output: 0.005,
        }),

        // Google Gemini 2.0 Flash
        GEMINI_2_0_FLASH_EXP => Some(ModelMetadata {
            id: model_id,
            display_name: "Gemini 2.0 Flash",
            provider: "google",
            capabilities: ModelCapabilities {
                supports_streaming: true,
                supports_function_calling: true,
                supports_vision: true,
                max_context_tokens: 1_048_576,
                max_output_tokens: 8_192,
            },
            cost_per_1k_input: 0.0,  // Experimental pricing
            cost_per_1k_output: 0.0,
        }),

        // Google Gemini 1.5 Pro
        GEMINI_1_5_PRO | GEMINI_1_5_PRO_LATEST | GEMINI_1_5_PRO_002 => Some(ModelMetadata {
            id: model_id,
            display_name: "Gemini 1.5 Pro",
            provider: "google",
            capabilities: ModelCapabilities {
                supports_streaming: true,
                supports_function_calling: true,
                supports_vision: true,
                max_context_tokens: 2_097_152,
                max_output_tokens: 8_192,
            },
            cost_per_1k_input: 0.00125,
            cost_per_1k_output: 0.005,
        }),

        // Google Gemini 1.5 Flash
        GEMINI_1_5_FLASH | GEMINI_1_5_FLASH_LATEST | GEMINI_1_5_FLASH_002 => Some(ModelMetadata {
            id: model_id,
            display_name: "Gemini 1.5 Flash",
            provider: "google",
            capabilities: ModelCapabilities {
                supports_streaming: true,
                supports_function_calling: true,
                supports_vision: true,
                max_context_tokens: 1_048_576,
                max_output_tokens: 8_192,
            },
            cost_per_1k_input: 0.000075,
            cost_per_1k_output: 0.0003,
        }),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_models_list() {
        assert!(OPENAI_MODELS.len() > 0);
        assert!(OPENAI_MODELS.contains(&GPT_4O));
        assert!(OPENAI_MODELS.contains(&GPT_4O_MINI));
        assert!(OPENAI_MODELS.contains(&O1_PREVIEW));
    }

    #[test]
    fn test_anthropic_models_list() {
        assert!(ANTHROPIC_MODELS.len() > 0);
        assert!(ANTHROPIC_MODELS.contains(&CLAUDE_3_5_SONNET_LATEST));
        assert!(ANTHROPIC_MODELS.contains(&CLAUDE_3_5_HAIKU_LATEST));
    }

    #[test]
    fn test_google_models_list() {
        assert!(GOOGLE_MODELS.len() > 0);
        assert!(GOOGLE_MODELS.contains(&GEMINI_2_0_FLASH_EXP));
        assert!(GOOGLE_MODELS.contains(&GEMINI_1_5_PRO));
    }

    #[test]
    fn test_get_model_metadata() {
        let metadata = get_model_metadata(GPT_4O);
        assert!(metadata.is_some());

        let metadata = metadata.unwrap();
        assert_eq!(metadata.provider, "openai");
        assert!(metadata.capabilities.supports_streaming);
    }
}
