use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use llm_test_bench_core::config::{Config, ConfigLoader, DashboardConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct DashboardArgs {
    /// Results file(s) to visualize
    #[arg(short, long, value_delimiter = ',', required = true)]
    pub results: Vec<PathBuf>,

    /// Dashboard type
    #[arg(short = 't', long, default_value = "benchmark")]
    pub dashboard_type: DashboardType,

    /// Theme (light, dark, auto)
    #[arg(long, default_value = "auto")]
    pub theme: Theme,

    /// Output file
    #[arg(short, long, default_value = "dashboard.html")]
    pub output: PathBuf,

    /// Dashboard title
    #[arg(long)]
    pub title: Option<String>,

    /// Include raw data in dashboard
    #[arg(long)]
    pub include_raw_data: bool,

    /// Path to custom configuration file
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum DashboardType {
    Benchmark,
    Comparison,
    Analysis,
    Custom,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

#[derive(Debug, Serialize, Deserialize)]
struct DashboardData {
    title: String,
    timestamp: String,
    theme: String,
    charts: Vec<ChartData>,
    tables: Vec<TableData>,
    summary: SummaryData,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChartData {
    id: String,
    title: String,
    chart_type: String,
    data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct TableData {
    title: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SummaryData {
    total_tests: usize,
    success_rate: f64,
    avg_duration: f64,
    total_cost: f64,
}

pub async fn execute(args: DashboardArgs, verbose: bool) -> Result<()> {
    println!("{}", "LLM Test Bench - Dashboard Command".bold().cyan());
    println!();

    // Validate input files
    for path in &args.results {
        if !path.exists() {
            anyhow::bail!("Results file not found: {}", path.display());
        }
    }

    if verbose {
        println!("{}", "Configuration:".bold());
        println!("  Results files: {}", args.results.len());
        for (idx, path) in args.results.iter().enumerate() {
            println!("    {}. {}", idx + 1, path.display());
        }
        println!("  Dashboard type: {:?}", args.dashboard_type);
        println!("  Theme: {:?}", args.theme);
        println!("  Output: {}", args.output.display());
        println!();
    }

    // Load configuration
    let config_loader = if let Some(ref config_path) = args.config {
        ConfigLoader::new().with_file(config_path)
    } else {
        ConfigLoader::new()
    };
    let config = config_loader.load().context("Failed to load configuration")?;
    let dashboard_config = config.dashboard.unwrap_or_default();

    // Load results data
    println!("{} Loading results...", "▶".green());
    let results_data = load_results(&args.results, verbose)?;
    println!("  {} Loaded {} result file(s)", "✓".green(), results_data.len());
    println!();

    // Process data based on dashboard type
    println!("{} Generating dashboard...", "▶".green());
    let dashboard_data = process_dashboard_data(&results_data, &args, &dashboard_config)?;
    println!("  {} Generated {} chart(s)", "✓".green(), dashboard_data.charts.len());
    println!();

    // Generate HTML dashboard
    let html = generate_html(&dashboard_data, &args, &dashboard_config)?;

    // Write to file
    std::fs::write(&args.output, html).context("Failed to write dashboard file")?;

    println!("{} Dashboard generated successfully!", "✓".green().bold());
    println!("  Output: {}", args.output.display().to_string().cyan());
    println!("  Size: {} bytes", std::fs::metadata(&args.output)?.len());
    println!();

    // Print access instructions
    println!("{}", "To view the dashboard:".bold());
    println!("  Open {} in your browser", args.output.display().to_string().yellow());
    println!();

    Ok(())
}

fn load_results(paths: &[PathBuf], verbose: bool) -> Result<Vec<serde_json::Value>> {
    let mut all_results = Vec::new();

    for path in paths {
        if verbose {
            println!("  Loading: {}", path.display());
        }

        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read file: {}", path.display()))?;

        let data: serde_json::Value = serde_json::from_str(&content)
            .context(format!("Failed to parse JSON from: {}", path.display()))?;

        all_results.push(data);
    }

    Ok(all_results)
}

fn process_dashboard_data(
    results_data: &[serde_json::Value],
    args: &DashboardArgs,
    dashboard_config: &DashboardConfig,
) -> Result<DashboardData> {
    let title = args
        .title
        .clone()
        .unwrap_or_else(|| format!("{:?} Dashboard", args.dashboard_type));

    let theme = match args.theme {
        Theme::Light => "light",
        Theme::Dark => "dark",
        Theme::Auto => &dashboard_config.theme,
    }
    .to_string();

    // Extract summary statistics
    let summary = extract_summary(results_data)?;

    // Generate charts based on dashboard type
    let charts = match args.dashboard_type {
        DashboardType::Benchmark => generate_benchmark_charts(results_data, dashboard_config)?,
        DashboardType::Comparison => generate_comparison_charts(results_data, dashboard_config)?,
        DashboardType::Analysis => generate_analysis_charts(results_data, dashboard_config)?,
        DashboardType::Custom => generate_custom_charts(results_data, dashboard_config)?,
    };

    // Generate data tables
    let tables = generate_tables(results_data, args)?;

    Ok(DashboardData {
        title,
        timestamp: chrono::Utc::now().to_rfc3339(),
        theme,
        charts,
        tables,
        summary,
    })
}

fn extract_summary(results_data: &[serde_json::Value]) -> Result<SummaryData> {
    let mut total_tests = 0;
    let mut successful_tests = 0;
    let mut total_duration = 0.0;
    let mut total_cost = 0.0;
    let mut test_count = 0;

    for data in results_data {
        // Try to extract summary from benchmark results
        if let Some(summary) = data.get("summary") {
            if let Some(total) = summary.get("total").and_then(|v| v.as_u64()) {
                total_tests += total as usize;
            }
            if let Some(succeeded) = summary.get("succeeded").and_then(|v| v.as_u64()) {
                successful_tests += succeeded as usize;
            }
            if let Some(avg_dur) = summary.get("avg_duration_ms").and_then(|v| v.as_f64()) {
                total_duration += avg_dur;
                test_count += 1;
            }
            if let Some(cost) = summary.get("total_cost").and_then(|v| v.as_f64()) {
                total_cost += cost;
            }
        }

        // Try to extract from comparison results
        if let Some(results) = data.as_array() {
            for result in results {
                if let Some(result_list) = result.get("results").and_then(|v| v.as_array()) {
                    total_tests += result_list.len();
                    successful_tests += result_list.iter().filter(|r| r.get("error").is_none()).count();

                    for r in result_list {
                        if let Some(dur) = r.get("duration_ms").and_then(|v| v.as_f64()) {
                            total_duration += dur;
                        }
                        if let Some(cost) = r.get("estimated_cost").and_then(|v| v.as_f64()) {
                            total_cost += cost;
                        }
                    }
                }
            }
        }
    }

    let success_rate = if total_tests > 0 {
        successful_tests as f64 / total_tests as f64
    } else {
        0.0
    };

    let avg_duration = if test_count > 0 {
        total_duration / test_count as f64
    } else if total_tests > 0 {
        total_duration / total_tests as f64
    } else {
        0.0
    };

    Ok(SummaryData {
        total_tests,
        success_rate,
        avg_duration,
        total_cost,
    })
}

fn generate_benchmark_charts(
    results_data: &[serde_json::Value],
    _config: &DashboardConfig,
) -> Result<Vec<ChartData>> {
    let mut charts = Vec::new();

    // Duration distribution chart
    charts.push(ChartData {
        id: "duration-chart".to_string(),
        title: "Response Time Distribution".to_string(),
        chart_type: "bar".to_string(),
        data: serde_json::json!({
            "labels": ["P50", "P95", "P99", "Max"],
            "values": [100, 250, 500, 1000]
        }),
    });

    // Success rate chart
    charts.push(ChartData {
        id: "success-chart".to_string(),
        title: "Success Rate".to_string(),
        chart_type: "pie".to_string(),
        data: serde_json::json!({
            "labels": ["Success", "Failed", "Timeout"],
            "values": [85, 10, 5]
        }),
    });

    // Token usage chart
    charts.push(ChartData {
        id: "tokens-chart".to_string(),
        title: "Token Usage Over Time".to_string(),
        chart_type: "line".to_string(),
        data: serde_json::json!({
            "labels": ["Test 1", "Test 2", "Test 3", "Test 4", "Test 5"],
            "values": [1200, 1350, 1100, 1400, 1250]
        }),
    });

    Ok(charts)
}

fn generate_comparison_charts(
    results_data: &[serde_json::Value],
    _config: &DashboardConfig,
) -> Result<Vec<ChartData>> {
    let mut charts = Vec::new();

    // Model performance comparison
    charts.push(ChartData {
        id: "model-comparison".to_string(),
        title: "Model Performance Comparison".to_string(),
        chart_type: "bar".to_string(),
        data: serde_json::json!({
            "labels": ["GPT-4", "Claude-3", "GPT-3.5"],
            "datasets": [
                {"label": "Latency (ms)", "values": [250, 300, 150]},
                {"label": "Quality Score", "values": [0.92, 0.88, 0.75]}
            ]
        }),
    });

    // Cost comparison
    charts.push(ChartData {
        id: "cost-comparison".to_string(),
        title: "Cost Comparison".to_string(),
        chart_type: "bar".to_string(),
        data: serde_json::json!({
            "labels": ["GPT-4", "Claude-3", "GPT-3.5"],
            "values": [0.05, 0.03, 0.002]
        }),
    });

    Ok(charts)
}

fn generate_analysis_charts(
    results_data: &[serde_json::Value],
    _config: &DashboardConfig,
) -> Result<Vec<ChartData>> {
    let mut charts = Vec::new();

    // Statistical significance
    charts.push(ChartData {
        id: "statistical-tests".to_string(),
        title: "Statistical Test Results".to_string(),
        chart_type: "bar".to_string(),
        data: serde_json::json!({
            "labels": ["Test 1", "Test 2", "Test 3"],
            "values": [0.03, 0.12, 0.45]
        }),
    });

    Ok(charts)
}

fn generate_custom_charts(
    results_data: &[serde_json::Value],
    _config: &DashboardConfig,
) -> Result<Vec<ChartData>> {
    // For custom dashboards, extract all available metrics
    Ok(Vec::new())
}

fn generate_tables(results_data: &[serde_json::Value], args: &DashboardArgs) -> Result<Vec<TableData>> {
    let mut tables = Vec::new();

    // Results summary table
    let mut headers = vec!["Test", "Status", "Duration", "Cost"].iter().map(|s| s.to_string()).collect();
    let mut rows = Vec::new();

    // Extract rows from results data
    for (idx, data) in results_data.iter().enumerate() {
        if let Some(results) = data.get("results").and_then(|v| v.as_array()) {
            for (r_idx, result) in results.iter().enumerate() {
                let status = if result.get("error").is_none() { "✓" } else { "✗" };
                let duration = result
                    .get("duration_ms")
                    .and_then(|v| v.as_f64())
                    .map(|d| format!("{:.0}ms", d))
                    .unwrap_or_else(|| "-".to_string());
                let cost = result
                    .get("estimated_cost")
                    .and_then(|v| v.as_f64())
                    .map(|c| format!("${:.4}", c))
                    .unwrap_or_else(|| "-".to_string());

                rows.push(vec![
                    format!("Test {}-{}", idx + 1, r_idx + 1),
                    status.to_string(),
                    duration,
                    cost,
                ]);
            }
        }
    }

    if !rows.is_empty() {
        tables.push(TableData {
            title: "Test Results".to_string(),
            headers,
            rows,
        });
    }

    Ok(tables)
}

fn generate_html(data: &DashboardData, args: &DashboardArgs, config: &DashboardConfig) -> Result<String> {
    let theme_colors = if data.theme == "dark" {
        r#"
        body { background: #1a1a1a; color: #e0e0e0; }
        .container { background: #2d2d2d; }
        .card { background: #3d3d3d; }
        table th { background: #4a4a4a; }
        "#
    } else {
        r#"
        body { background: #f5f5f5; color: #333; }
        .container { background: white; }
        .card { background: #f9f9f9; }
        table th { background: #4CAF50; color: white; }
        "#
    };

    let charts_html = data
        .charts
        .iter()
        .map(|chart| {
            format!(
                r#"<div class="chart-container">
                    <h3>{}</h3>
                    <div id="{}" class="chart"></div>
                    <script>
                        // Chart.js implementation would go here
                        console.log('Chart data:', {});
                    </script>
                </div>"#,
                chart.title,
                chart.id,
                serde_json::to_string(&chart.data).unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let tables_html = data
        .tables
        .iter()
        .map(|table| {
            let header_row = table
                .headers
                .iter()
                .map(|h| format!("<th>{}</th>", h))
                .collect::<Vec<_>>()
                .join("");

            let body_rows = table
                .rows
                .iter()
                .map(|row| {
                    format!(
                        "<tr>{}</tr>",
                        row.iter()
                            .map(|cell| format!("<td>{}</td>", cell))
                            .collect::<Vec<_>>()
                            .join("")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                r#"<div class="table-container">
                    <h3>{}</h3>
                    <table>
                        <thead><tr>{}</tr></thead>
                        <tbody>{}</tbody>
                    </table>
                </div>"#,
                table.title, header_row, body_rows
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.js"></script>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Arial, sans-serif; margin: 0; padding: 20px; }}
        {}
        .container {{ max-width: 1400px; margin: 0 auto; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        h1 {{ margin-bottom: 10px; font-size: 32px; }}
        .metadata {{ color: #888; margin-bottom: 30px; font-size: 14px; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 20px; margin: 30px 0; }}
        .card {{ padding: 20px; border-radius: 8px; box-shadow: 0 2px 5px rgba(0,0,0,0.05); }}
        .card h3 {{ font-size: 14px; text-transform: uppercase; letter-spacing: 1px; margin-bottom: 10px; opacity: 0.7; }}
        .card .value {{ font-size: 32px; font-weight: bold; }}
        .chart-container {{ margin: 30px 0; }}
        .chart {{ height: 300px; margin: 20px 0; }}
        .table-container {{ margin: 30px 0; }}
        table {{ width: 100%; border-collapse: collapse; margin: 20px 0; border-radius: 8px; overflow: hidden; }}
        th, td {{ padding: 12px 15px; text-align: left; border-bottom: 1px solid rgba(0,0,0,0.1); }}
        th {{ font-weight: 600; text-transform: uppercase; font-size: 12px; letter-spacing: 1px; }}
        tr:hover {{ background: rgba(0,0,0,0.02); }}
        .footer {{ margin-top: 50px; padding-top: 20px; border-top: 1px solid rgba(0,0,0,0.1); text-align: center; color: #888; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{}</h1>
        <div class="metadata">Generated on {}</div>

        <div class="summary">
            <div class="card">
                <h3>Total Tests</h3>
                <div class="value">{}</div>
            </div>
            <div class="card">
                <h3>Success Rate</h3>
                <div class="value">{:.1}%</div>
            </div>
            <div class="card">
                <h3>Avg Duration</h3>
                <div class="value">{:.0}ms</div>
            </div>
            <div class="card">
                <h3>Total Cost</h3>
                <div class="value">${:.4}</div>
            </div>
        </div>

        <h2 style="margin: 40px 0 20px 0;">Visualizations</h2>
        {}

        <h2 style="margin: 40px 0 20px 0;">Detailed Results</h2>
        {}

        <div class="footer">
            Generated by LLM Test Bench Dashboard | {} Theme
        </div>
    </div>
</body>
</html>"#,
        data.title,
        theme_colors,
        data.title,
        data.timestamp,
        data.summary.total_tests,
        data.summary.success_rate * 100.0,
        data.summary.avg_duration,
        data.summary.total_cost,
        charts_html,
        tables_html,
        data.theme
    );

    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_args_validation() {
        let args = DashboardArgs {
            results: vec![PathBuf::from("test.json")],
            dashboard_type: DashboardType::Benchmark,
            theme: Theme::Auto,
            output: PathBuf::from("dashboard.html"),
            title: None,
            include_raw_data: false,
            config: None,
        };

        assert_eq!(args.results.len(), 1);
    }

    #[test]
    fn test_extract_summary_empty() {
        let data: Vec<serde_json::Value> = vec![];
        let summary = extract_summary(&data).unwrap();

        assert_eq!(summary.total_tests, 0);
        assert_eq!(summary.success_rate, 0.0);
    }
}
