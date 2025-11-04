use anyhow::{Context, Result};
use clap::Subcommand;
use inquire::{Confirm, Select, Text};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Initialize configuration with interactive setup wizard
    Init {
        /// Provider to configure (openai, anthropic, etc.)
        #[arg(long)]
        provider: Option<String>,

        /// Skip interactive prompts and use defaults
        #[arg(long)]
        non_interactive: bool,
    },

    /// Show current configuration
    Show,

    /// Validate configuration file
    Validate {
        /// Path to config file (default: ~/.config/llm-test-bench/config.toml)
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    version: String,
    providers: HashMap<String, ProviderConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    defaults: Option<Defaults>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProviderConfig {
    #[serde(rename = "type")]
    provider_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    defaults: Option<ProviderDefaults>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProviderDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Defaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    output_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_enabled: Option<bool>,
}

pub async fn execute(cmd: ConfigCommands, verbose: bool) -> Result<()> {
    match cmd {
        ConfigCommands::Init {
            provider,
            non_interactive,
        } => init_config(provider, non_interactive, verbose).await,
        ConfigCommands::Show => show_config(verbose).await,
        ConfigCommands::Validate { config } => validate_config(config, verbose).await,
    }
}

async fn init_config(
    provider_filter: Option<String>,
    non_interactive: bool,
    verbose: bool,
) -> Result<()> {
    println!("🚀 LLM Test Bench Configuration Wizard\n");

    let config_path = get_config_path()?;

    // Check if config already exists
    if config_path.exists() {
        if !non_interactive {
            let overwrite = Confirm::new("Configuration file already exists. Overwrite?")
                .with_default(false)
                .prompt()
                .context("Failed to get user confirmation")?;

            if !overwrite {
                println!("Configuration initialization cancelled.");
                return Ok(());
            }
        }
    }

    let mut config = Config {
        version: "1.0".to_string(),
        providers: HashMap::new(),
        defaults: Some(Defaults {
            output_dir: Some("./test-results".to_string()),
            cache_enabled: Some(true),
        }),
    };

    // Interactive provider selection
    let providers_to_configure = if non_interactive {
        vec!["openai"]
    } else {
        let available_providers = vec!["openai", "anthropic", "local", "skip"];
        let mut selected_providers = Vec::new();

        loop {
            let provider = Select::new(
                "Select a provider to configure (or 'skip' to finish):",
                available_providers.clone(),
            )
            .prompt()
            .context("Failed to get provider selection")?;

            if provider == "skip" {
                break;
            }

            if !selected_providers.contains(&provider) {
                selected_providers.push(provider);
            }

            if selected_providers.len() >= 3 {
                break;
            }
        }

        selected_providers
    };

    // Configure each selected provider
    for provider_name in providers_to_configure {
        if let Some(ref filter) = provider_filter {
            if provider_name != filter {
                continue;
            }
        }

        println!("\n📋 Configuring {} provider:", provider_name);

        let provider_config = if non_interactive {
            create_default_provider_config(provider_name)
        } else {
            configure_provider_interactive(provider_name)?
        };

        config.providers.insert(provider_name.to_string(), provider_config);
    }

    // Ensure config directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create configuration directory")?;
    }

    // Write configuration to file
    let toml_string = toml::to_string_pretty(&config)
        .context("Failed to serialize configuration")?;

    fs::write(&config_path, toml_string)
        .context("Failed to write configuration file")?;

    println!("\n✅ Configuration saved to: {}", config_path.display());
    println!("\n📝 Next steps:");
    println!("   1. Set your API keys as environment variables:");

    for (provider_name, _) in &config.providers {
        let env_var = format!("{}_API_KEY", provider_name.to_uppercase());
        println!("      export {}=your-api-key-here", env_var);
    }

    println!("   2. Run your first test:");
    println!("      llm-test-bench test <provider> --prompt \"Hello, world!\" --model <model>");
    println!("\n   Or run 'llm-test-bench config show' to view your configuration.");

    if verbose {
        println!("\nConfiguration file location: {}", config_path.display());
    }

    Ok(())
}

fn configure_provider_interactive(provider_name: &str) -> Result<ProviderConfig> {
    let use_env_var = Confirm::new("Use environment variable for API key?")
        .with_default(true)
        .prompt()
        .context("Failed to get API key preference")?;

    let api_key = if use_env_var {
        None // Will use environment variable
    } else {
        let key = Text::new("Enter API key:")
            .with_help_message("Warning: Storing API keys in config file is not recommended")
            .prompt()
            .context("Failed to get API key")?;
        Some(key)
    };

    let model = Text::new("Default model:")
        .with_default(get_default_model(provider_name))
        .prompt()
        .context("Failed to get default model")?;

    let temperature = Text::new("Default temperature (0.0-1.0):")
        .with_default("0.7")
        .prompt()
        .context("Failed to get temperature")?
        .parse::<f32>()
        .context("Invalid temperature value")?;

    Ok(ProviderConfig {
        provider_type: provider_name.to_string(),
        api_key,
        base_url: None,
        defaults: Some(ProviderDefaults {
            model: Some(model),
            temperature: Some(temperature),
            max_tokens: Some(4096),
        }),
    })
}

fn create_default_provider_config(provider_name: &str) -> ProviderConfig {
    ProviderConfig {
        provider_type: provider_name.to_string(),
        api_key: None,
        base_url: None,
        defaults: Some(ProviderDefaults {
            model: Some(get_default_model(provider_name).to_string()),
            temperature: Some(0.7),
            max_tokens: Some(4096),
        }),
    }
}

fn get_default_model(provider_name: &str) -> &'static str {
    match provider_name {
        "openai" => "gpt-4",
        "anthropic" => "claude-sonnet-4-20250514",
        "local" => "llama2",
        _ => "default",
    }
}

async fn show_config(verbose: bool) -> Result<()> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        println!("⚠️  No configuration file found at: {}", config_path.display());
        println!("\nRun 'llm-test-bench config init' to create one.");
        return Ok(());
    }

    let config_content = fs::read_to_string(&config_path)
        .context("Failed to read configuration file")?;

    let config: Config = toml::from_str(&config_content)
        .context("Failed to parse configuration file")?;

    println!("📋 Current Configuration:\n");
    println!("Version: {}", config.version);
    println!("\nProviders:");

    for (name, provider) in &config.providers {
        println!("  • {} ({})", name, provider.provider_type);
        if let Some(ref defaults) = provider.defaults {
            if let Some(ref model) = defaults.model {
                println!("    Model: {}", model);
            }
            if let Some(temp) = defaults.temperature {
                println!("    Temperature: {}", temp);
            }
        }
        if provider.api_key.is_some() {
            println!("    API Key: [stored in config - not recommended]");
        } else {
            println!("    API Key: ${}_API_KEY (environment variable)", name.to_uppercase());
        }
    }

    if verbose {
        println!("\nFull configuration:");
        println!("{}", config_content);
    }

    println!("\nConfig file: {}", config_path.display());

    Ok(())
}

async fn validate_config(config_path: Option<PathBuf>, verbose: bool) -> Result<()> {
    let path = config_path.unwrap_or_else(|| get_config_path().unwrap());

    if !path.exists() {
        anyhow::bail!("Configuration file not found: {}", path.display());
    }

    let config_content = fs::read_to_string(&path)
        .context("Failed to read configuration file")?;

    let config: Config = toml::from_str(&config_content)
        .context("Failed to parse configuration file")?;

    println!("✅ Configuration file is valid!");
    println!("\nValidation results:");
    println!("  • Version: {}", config.version);
    println!("  • Providers configured: {}", config.providers.len());

    for (name, provider) in &config.providers {
        println!("\n  Provider: {}", name);

        // Check for API key
        if provider.api_key.is_some() {
            println!("    ⚠️  Warning: API key stored in config file (consider using environment variables)");
        } else {
            let env_var = format!("{}_API_KEY", name.to_uppercase());
            if std::env::var(&env_var).is_ok() {
                println!("    ✅ API key found in environment: {}", env_var);
            } else {
                println!("    ⚠️  Warning: API key not found. Set {} environment variable", env_var);
            }
        }

        // Validate provider type
        match provider.provider_type.as_str() {
            "openai" | "anthropic" | "local" => {
                println!("    ✅ Provider type '{}' is supported", provider.provider_type);
            }
            _ => {
                println!("    ⚠️  Unknown provider type: {}", provider.provider_type);
            }
        }

        // Check defaults
        if let Some(ref defaults) = provider.defaults {
            if let Some(temp) = defaults.temperature {
                if !(0.0..=2.0).contains(&temp) {
                    println!("    ⚠️  Temperature {} is outside recommended range (0.0-2.0)", temp);
                }
            }
        }
    }

    if verbose {
        println!("\nConfiguration content:");
        println!("{}", config_content);
    }

    Ok(())
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to determine config directory")?
        .join("llm-test-bench");

    Ok(config_dir.join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_models() {
        assert_eq!(get_default_model("openai"), "gpt-4");
        assert_eq!(get_default_model("anthropic"), "claude-sonnet-4-20250514");
        assert_eq!(get_default_model("local"), "llama2");
    }

    #[test]
    fn test_config_serialization() {
        let mut providers = HashMap::new();
        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                provider_type: "openai".to_string(),
                api_key: None,
                base_url: None,
                defaults: Some(ProviderDefaults {
                    model: Some("gpt-4".to_string()),
                    temperature: Some(0.7),
                    max_tokens: Some(4096),
                }),
            },
        );

        let config = Config {
            version: "1.0".to_string(),
            providers,
            defaults: Some(Defaults {
                output_dir: Some("./test-results".to_string()),
                cache_enabled: Some(true),
            }),
        };

        let toml_string = toml::to_string(&config).unwrap();
        assert!(toml_string.contains("version = \"1.0\""));
        assert!(toml_string.contains("[providers.openai]"));
    }
}
