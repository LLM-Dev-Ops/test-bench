use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use llm_test_bench_core::config::{AnalyticsConfig, ConfigLoader};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct OptimizeArgs {
    /// Current model being used
    #[arg(short = 'm', long, required = true)]
    pub current_model: String,

    /// Quality threshold (0.0-1.0)
    #[arg(short, long, default_value = "0.75")]
    pub quality_threshold: f64,

    /// Monthly request volume
    #[arg(short = 'r', long, required = true)]
    pub monthly_requests: usize,

    /// Historical results for analysis
    #[arg(long)]
    pub history: Option<PathBuf>,

    /// Maximum acceptable cost increase (%)
    #[arg(long, default_value = "10.0")]
    pub max_cost_increase: f64,

    /// Minimum required quality score
    #[arg(long, default_value = "0.70")]
    pub min_quality: f64,

    /// Include experimental models
    #[arg(long)]
    pub include_experimental: bool,

    /// Output format
    #[arg(short, long, default_value = "detailed")]
    pub output: OutputFormat,

    /// Save optimization report
    #[arg(long)]
    pub report_file: Option<PathBuf>,

    /// Path to custom configuration file
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Detailed,
    Summary,
    Json,
}

#[derive(Debug, Serialize, Deserialize)]
struct OptimizationReport {
    current_model: ModelAnalysis,
    recommendations: Vec<ModelRecommendation>,
    cost_savings: CostSavings,
    risk_assessment: RiskAssessment,
    summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelAnalysis {
    name: String,
    provider: String,
    monthly_cost: f64,
    avg_quality: f64,
    avg_latency: f64,
    tokens_per_request: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelRecommendation {
    rank: usize,
    model: String,
    provider: String,
    monthly_cost: f64,
    cost_savings_amount: f64,
    cost_savings_percent: f64,
    estimated_quality: f64,
    quality_change: f64,
    avg_latency: f64,
    latency_change: f64,
    reason: String,
    pros: Vec<String>,
    cons: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CostSavings {
    total_annual_savings: f64,
    best_recommendation_savings: f64,
    roi_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct RiskAssessment {
    overall_risk: String,
    quality_risk: String,
    cost_risk: String,
    recommendations: Vec<String>,
}

// Model pricing database (per 1K tokens)
struct ModelPricing {
    input_cost: f64,
    output_cost: f64,
}

pub async fn execute(args: OptimizeArgs, verbose: bool) -> Result<()> {
    println!("{}", "LLM Test Bench - Optimize Command".bold().cyan());
    println!();

    // Validate arguments
    if args.quality_threshold < 0.0 || args.quality_threshold > 1.0 {
        anyhow::bail!("Quality threshold must be between 0.0 and 1.0, got: {}", args.quality_threshold);
    }

    if args.monthly_requests == 0 {
        anyhow::bail!("Monthly requests must be greater than 0");
    }

    if verbose {
        println!("{}", "Configuration:".bold());
        println!("  Current model: {}", args.current_model);
        println!("  Quality threshold: {:.2}", args.quality_threshold);
        println!("  Monthly requests: {}", args.monthly_requests);
        println!("  Min quality: {:.2}", args.min_quality);
        println!();
    }

    // Load configuration
    let config_loader = if let Some(ref config_path) = args.config {
        ConfigLoader::new().with_file(config_path)
    } else {
        ConfigLoader::new()
    };
    let config = config_loader.load().context("Failed to load configuration")?;
    let analytics_config = config.analytics.unwrap_or_default();

    // Analyze current model
    println!("{} Analyzing current model: {}", "▶".green(), args.current_model.bold());
    let current_analysis = analyze_current_model(&args, &analytics_config)?;
    println!("  {} Monthly cost: ${:.2}", "ℹ".blue(), current_analysis.monthly_cost);
    println!("  {} Avg quality: {:.2}", "ℹ".blue(), current_analysis.avg_quality);
    println!("  {} Avg latency: {:.0}ms", "ℹ".blue(), current_analysis.avg_latency);
    println!();

    // Generate recommendations
    println!("{} Generating optimization recommendations...", "▶".green());
    let recommendations = generate_recommendations(&current_analysis, &args, &analytics_config)?;
    println!("  {} Found {} alternative model(s)", "✓".green(), recommendations.len());
    println!();

    // Calculate cost savings
    let cost_savings = calculate_cost_savings(&current_analysis, &recommendations)?;

    // Assess risks
    let risk_assessment = assess_risks(&current_analysis, &recommendations, &args)?;

    // Create report
    let summary = generate_summary(&current_analysis, &recommendations, &cost_savings);
    let report = OptimizationReport {
        current_model: current_analysis,
        recommendations,
        cost_savings,
        risk_assessment,
        summary,
    };

    // Display results
    display_optimization_report(&report, &args, verbose)?;

    // Save report if requested
    if let Some(ref report_path) = args.report_file {
        save_report(&report, report_path)?;
        println!();
        println!("{} Report saved to: {}", "✓".green(), report_path.display().to_string().cyan());
    }

    println!();
    println!("{} Optimization analysis complete!", "✓".green().bold());

    Ok(())
}

fn analyze_current_model(args: &OptimizeArgs, _config: &AnalyticsConfig) -> Result<ModelAnalysis> {
    let (provider, model) = parse_model_spec(&args.current_model)?;

    // Get pricing for current model
    let pricing = get_model_pricing(&provider, &model);

    // Estimate token usage (simplified - in production, use historical data)
    let avg_input_tokens = 500.0;
    let avg_output_tokens = 300.0;
    let tokens_per_request = avg_input_tokens + avg_output_tokens;

    // Calculate monthly cost
    let cost_per_request = (avg_input_tokens * pricing.input_cost + avg_output_tokens * pricing.output_cost) / 1000.0;
    let monthly_cost = cost_per_request * args.monthly_requests as f64;

    // Estimate quality based on model tier (simplified)
    let avg_quality = estimate_model_quality(&provider, &model);

    // Estimate latency based on model
    let avg_latency = estimate_model_latency(&provider, &model);

    Ok(ModelAnalysis {
        name: model,
        provider,
        monthly_cost,
        avg_quality,
        avg_latency,
        tokens_per_request,
    })
}

fn parse_model_spec(spec: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() == 2 {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        // Infer provider from model name
        if spec.contains("gpt") {
            Ok(("openai".to_string(), spec.to_string()))
        } else if spec.contains("claude") {
            Ok(("anthropic".to_string(), spec.to_string()))
        } else {
            anyhow::bail!("Cannot infer provider from model '{}'. Use format: provider:model", spec)
        }
    }
}

fn get_model_pricing(provider: &str, model: &str) -> ModelPricing {
    match (provider, model) {
        ("openai", m) if m.contains("gpt-4-turbo") => ModelPricing {
            input_cost: 10.0,
            output_cost: 30.0,
        },
        ("openai", m) if m.contains("gpt-4") => ModelPricing {
            input_cost: 30.0,
            output_cost: 60.0,
        },
        ("openai", m) if m.contains("gpt-3.5-turbo") => ModelPricing {
            input_cost: 0.5,
            output_cost: 1.5,
        },
        ("anthropic", m) if m.contains("opus") => ModelPricing {
            input_cost: 15.0,
            output_cost: 75.0,
        },
        ("anthropic", m) if m.contains("sonnet") => ModelPricing {
            input_cost: 3.0,
            output_cost: 15.0,
        },
        ("anthropic", m) if m.contains("haiku") => ModelPricing {
            input_cost: 0.25,
            output_cost: 1.25,
        },
        _ => ModelPricing {
            input_cost: 1.0,
            output_cost: 2.0,
        },
    }
}

fn estimate_model_quality(provider: &str, model: &str) -> f64 {
    match (provider, model) {
        ("openai", m) if m.contains("gpt-4") => 0.92,
        ("openai", m) if m.contains("gpt-3.5-turbo") => 0.78,
        ("anthropic", m) if m.contains("opus") => 0.94,
        ("anthropic", m) if m.contains("sonnet") => 0.88,
        ("anthropic", m) if m.contains("haiku") => 0.82,
        _ => 0.75,
    }
}

fn estimate_model_latency(provider: &str, model: &str) -> f64 {
    match (provider, model) {
        ("openai", m) if m.contains("gpt-4") && !m.contains("turbo") => 2500.0,
        ("openai", m) if m.contains("gpt-4-turbo") => 1200.0,
        ("openai", m) if m.contains("gpt-3.5") => 800.0,
        ("anthropic", m) if m.contains("opus") => 2000.0,
        ("anthropic", m) if m.contains("sonnet") => 1000.0,
        ("anthropic", m) if m.contains("haiku") => 500.0,
        _ => 1000.0,
    }
}

fn generate_recommendations(
    current: &ModelAnalysis,
    args: &OptimizeArgs,
    _config: &AnalyticsConfig,
) -> Result<Vec<ModelRecommendation>> {
    let mut recommendations = Vec::new();

    // Define candidate models
    let mut candidates = vec![
        ("openai", "gpt-4-turbo"),
        ("openai", "gpt-4"),
        ("openai", "gpt-3.5-turbo"),
        ("anthropic", "claude-3-opus"),
        ("anthropic", "claude-3-sonnet"),
        ("anthropic", "claude-3-haiku"),
    ];

    // Filter out current model
    candidates.retain(|(p, m)| {
        format!("{}:{}", p, m) != format!("{}:{}", current.provider, current.name)
    });

    // Analyze each candidate
    for (rank, (provider, model)) in candidates.iter().enumerate() {
        let pricing = get_model_pricing(provider, model);
        let quality = estimate_model_quality(provider, model);
        let latency = estimate_model_latency(provider, model);

        // Calculate cost
        let avg_input_tokens = 500.0;
        let avg_output_tokens = 300.0;
        let cost_per_request = (avg_input_tokens * pricing.input_cost + avg_output_tokens * pricing.output_cost) / 1000.0;
        let monthly_cost = cost_per_request * args.monthly_requests as f64;

        // Check if meets criteria
        if quality < args.min_quality {
            continue;
        }

        if quality < args.quality_threshold {
            continue;
        }

        let cost_savings_amount = current.monthly_cost - monthly_cost;
        let cost_savings_percent = (cost_savings_amount / current.monthly_cost) * 100.0;

        // Skip if cost increase exceeds threshold
        if cost_savings_amount < 0.0 && cost_savings_percent.abs() > args.max_cost_increase {
            continue;
        }

        let quality_change = quality - current.avg_quality;
        let latency_change = latency - current.avg_latency;

        // Generate reason
        let reason = if cost_savings_amount > 0.0 && quality_change >= 0.0 {
            format!("Saves ${:.2}/month while maintaining or improving quality", cost_savings_amount)
        } else if cost_savings_amount > 0.0 {
            format!("Saves ${:.2}/month ({:.1}% reduction)", cost_savings_amount, cost_savings_percent)
        } else if quality_change > 0.0 {
            format!("Improves quality by {:.2} points", quality_change)
        } else {
            "Alternative option".to_string()
        };

        // Generate pros and cons
        let mut pros = Vec::new();
        let mut cons = Vec::new();

        if cost_savings_amount > 0.0 {
            pros.push(format!("${:.2}/month cost savings ({:.1}%)", cost_savings_amount, cost_savings_percent));
        }
        if quality_change > 0.0 {
            pros.push(format!("+{:.2} quality improvement", quality_change));
        }
        if latency_change < 0.0 {
            pros.push(format!("{:.0}ms faster response time", latency_change.abs()));
        }

        if cost_savings_amount < 0.0 {
            cons.push(format!("${:.2}/month additional cost", cost_savings_amount.abs()));
        }
        if quality_change < 0.0 {
            cons.push(format!("{:.2} quality decrease", quality_change.abs()));
        }
        if latency_change > 0.0 {
            cons.push(format!("+{:.0}ms slower response time", latency_change));
        }

        recommendations.push(ModelRecommendation {
            rank: rank + 1,
            model: model.to_string(),
            provider: provider.to_string(),
            monthly_cost,
            cost_savings_amount,
            cost_savings_percent,
            estimated_quality: quality,
            quality_change,
            avg_latency: latency,
            latency_change,
            reason,
            pros,
            cons,
        });
    }

    // Sort by cost savings (descending)
    recommendations.sort_by(|a, b| {
        b.cost_savings_amount.partial_cmp(&a.cost_savings_amount).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Update ranks
    for (idx, rec) in recommendations.iter_mut().enumerate() {
        rec.rank = idx + 1;
    }

    Ok(recommendations)
}

fn calculate_cost_savings(
    _current: &ModelAnalysis,
    recommendations: &[ModelRecommendation],
) -> Result<CostSavings> {
    let best_savings = recommendations
        .first()
        .map(|r| r.cost_savings_amount)
        .unwrap_or(0.0);

    let total_annual_savings = best_savings * 12.0;

    let roi_percentage = if best_savings > 0.0 {
        100.0 // Simplified - assume minimal switching cost
    } else {
        0.0
    };

    Ok(CostSavings {
        total_annual_savings,
        best_recommendation_savings: best_savings,
        roi_percentage,
    })
}

fn assess_risks(
    current: &ModelAnalysis,
    recommendations: &[ModelRecommendation],
    args: &OptimizeArgs,
) -> Result<RiskAssessment> {
    let best_rec = recommendations.first();

    let quality_risk = if let Some(rec) = best_rec {
        if rec.quality_change < -0.05 {
            "High - Significant quality decrease expected".to_string()
        } else if rec.quality_change < 0.0 {
            "Medium - Minor quality decrease expected".to_string()
        } else {
            "Low - Quality maintained or improved".to_string()
        }
    } else {
        "N/A - No recommendations available".to_string()
    };

    let cost_risk = if let Some(rec) = best_rec {
        if rec.cost_savings_percent < -10.0 {
            "High - Significant cost increase".to_string()
        } else if rec.cost_savings_percent < 0.0 {
            "Medium - Minor cost increase".to_string()
        } else {
            "Low - Cost reduction expected".to_string()
        }
    } else {
        "N/A".to_string()
    };

    let overall_risk = if quality_risk.contains("High") || cost_risk.contains("High") {
        "High".to_string()
    } else if quality_risk.contains("Medium") || cost_risk.contains("Medium") {
        "Medium".to_string()
    } else {
        "Low".to_string()
    };

    let mut risk_recommendations = Vec::new();

    if overall_risk == "High" {
        risk_recommendations.push("Run A/B tests before full migration".to_string());
        risk_recommendations.push("Monitor quality metrics closely during transition".to_string());
    } else if overall_risk == "Medium" {
        risk_recommendations.push("Gradual rollout recommended (e.g., 10% -> 50% -> 100%)".to_string());
        risk_recommendations.push("Set up quality alerts before migration".to_string());
    } else {
        risk_recommendations.push("Safe to proceed with migration".to_string());
        risk_recommendations.push("Standard monitoring procedures sufficient".to_string());
    }

    Ok(RiskAssessment {
        overall_risk,
        quality_risk,
        cost_risk,
        recommendations: risk_recommendations,
    })
}

fn generate_summary(
    current: &ModelAnalysis,
    recommendations: &[ModelRecommendation],
    cost_savings: &CostSavings,
) -> String {
    if recommendations.is_empty() {
        format!(
            "No better alternatives found for {} that meet your quality threshold of {:.2}.",
            current.name, current.avg_quality
        )
    } else {
        let best = &recommendations[0];
        if cost_savings.best_recommendation_savings > 0.0 {
            format!(
                "Switch to {}:{} to save ${:.2}/month (${:.2}/year) while {} quality.",
                best.provider,
                best.model,
                cost_savings.best_recommendation_savings,
                cost_savings.total_annual_savings,
                if best.quality_change >= 0.0 { "maintaining or improving" } else { "slightly reducing" }
            )
        } else {
            format!(
                "Consider {}:{} for improved quality ({:.2} vs {:.2}), though at ${:.2}/month additional cost.",
                best.provider, best.model, best.estimated_quality, current.avg_quality, best.cost_savings_amount.abs()
            )
        }
    }
}

fn display_optimization_report(report: &OptimizationReport, args: &OptimizeArgs, verbose: bool) -> Result<()> {
    match args.output {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(report)?;
            println!("{}", json);
        }
        OutputFormat::Summary | OutputFormat::Detailed => {
            println!("{}", "Optimization Report".bold().cyan());
            println!("{}", "═".repeat(80).dimmed());
            println!();

            // Current model analysis
            println!("{}", "Current Model".bold());
            println!("  Model: {}:{}", report.current_model.provider, report.current_model.name);
            println!("  Monthly cost: ${:.2}", report.current_model.monthly_cost);
            println!("  Quality score: {:.2}", report.current_model.avg_quality);
            println!("  Avg latency: {:.0}ms", report.current_model.avg_latency);
            println!();

            // Recommendations
            if report.recommendations.is_empty() {
                println!("{}", "No Recommendations".yellow().bold());
                println!("  No alternative models found that meet your criteria.");
                println!("  Consider lowering quality threshold or accepting higher costs.");
            } else {
                println!("{} Recommendation{}", "Top".bold().green(), if report.recommendations.len() > 1 { "s" } else { "" });
                println!();

                let display_count = if args.output == OutputFormat::Summary {
                    3.min(report.recommendations.len())
                } else {
                    report.recommendations.len()
                };

                for rec in &report.recommendations[..display_count] {
                    let savings_color = if rec.cost_savings_amount > 0.0 { "green" } else { "yellow" };

                    println!("{}. {}:{}", rec.rank, rec.provider.bold(), rec.model.bold());
                    println!("   {}", rec.reason.dimmed());
                    println!("   Monthly cost: ${:.2} ({}${:.2}, {:.1}%)",
                        rec.monthly_cost,
                        if rec.cost_savings_amount > 0.0 { "-" } else { "+" },
                        rec.cost_savings_amount.abs(),
                        rec.cost_savings_percent.abs()
                    );
                    println!("   Quality: {:.2} ({}{:.2})",
                        rec.estimated_quality,
                        if rec.quality_change >= 0.0 { "+" } else { "" },
                        rec.quality_change
                    );
                    println!("   Latency: {:.0}ms ({}{:.0}ms)",
                        rec.avg_latency,
                        if rec.latency_change >= 0.0 { "+" } else { "" },
                        rec.latency_change
                    );

                    if verbose {
                        if !rec.pros.is_empty() {
                            println!("   Pros:");
                            for pro in &rec.pros {
                                println!("     + {}", pro.green());
                            }
                        }
                        if !rec.cons.is_empty() {
                            println!("   Cons:");
                            for con in &rec.cons {
                                println!("     - {}", con.yellow());
                            }
                        }
                    }
                    println!();
                }
            }

            // Cost savings
            if report.cost_savings.total_annual_savings > 0.0 {
                println!("{}", "Potential Savings".bold().green());
                println!("  Monthly: ${:.2}", report.cost_savings.best_recommendation_savings);
                println!("  Annual: ${:.2}", report.cost_savings.total_annual_savings);
                println!("  ROI: {:.0}%", report.cost_savings.roi_percentage);
                println!();
            }

            // Risk assessment
            println!("{}", "Risk Assessment".bold().yellow());
            let risk_color = match report.risk_assessment.overall_risk.as_str() {
                "High" => "red",
                "Medium" => "yellow",
                _ => "green",
            };
            print!("  Overall risk: ");
            match risk_color {
                "red" => println!("{}", report.risk_assessment.overall_risk.red()),
                "yellow" => println!("{}", report.risk_assessment.overall_risk.yellow()),
                _ => println!("{}", report.risk_assessment.overall_risk.green()),
            }
            println!("  Quality risk: {}", report.risk_assessment.quality_risk);
            println!("  Cost risk: {}", report.risk_assessment.cost_risk);
            println!();

            println!("{}", "Recommendations:".dimmed());
            for rec in &report.risk_assessment.recommendations {
                println!("  • {}", rec);
            }
            println!();

            // Summary
            println!("{}", "Summary".bold().cyan());
            println!("  {}", report.summary);
        }
    }

    Ok(())
}

fn save_report(report: &OptimizationReport, path: &PathBuf) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_model_spec() {
        let (provider, model) = parse_model_spec("openai:gpt-4").unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");

        let (provider, model) = parse_model_spec("gpt-4").unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");
    }

    #[test]
    fn test_model_pricing() {
        let pricing = get_model_pricing("openai", "gpt-4");
        assert!(pricing.input_cost > 0.0);
        assert!(pricing.output_cost > 0.0);
    }

    #[test]
    fn test_estimate_model_quality() {
        let quality = estimate_model_quality("openai", "gpt-4");
        assert!(quality >= 0.8 && quality <= 1.0);
    }
}
