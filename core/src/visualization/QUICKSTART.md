# Visualization Module - Quick Start Guide

## 🚀 Quick Start (30 seconds)

```rust
use llm_test_bench_core::visualization::{DashboardGenerator, DashboardConfig};

// 1. Create generator
let generator = DashboardGenerator::new()?;

// 2. Generate dashboard
let html = generator.generate_benchmark_dashboard(&results, &DashboardConfig::default())?;

// 3. Export to file
generator.export_to_file(&html, Path::new("dashboard.html"))?;
```

## 📊 Dashboard Types

### Benchmark Results
```rust
generator.generate_benchmark_dashboard(&results, &config)?;
```
Shows: latency distribution, status pie chart, metrics radar, detailed table

### Model Comparison
```rust
generator.generate_comparison_dashboard(&[results1, results2], &config)?;
```
Shows: side-by-side metrics, cost vs quality scatter, rankings

### Trend Analysis
```rust
generator.generate_trend_dashboard(&historical_results, &config)?;
```
Shows: performance over time, success rate trends, latency trends

### Cost Analysis
```rust
// Use comparison dashboard with cost focus
generator.generate_comparison_dashboard(&results, &config)?;
```
Shows: cost breakdown, efficiency metrics, optimization tips

## 🎨 Customization

```rust
let config = DashboardConfig {
    title: "My Custom Dashboard".to_string(),
    theme: Theme::Dark,  // Light, Dark, or Auto
    max_data_points: 500,
    chart_colors: vec![
        "rgb(59, 130, 246)".to_string(),   // Blue
        "rgb(16, 185, 129)".to_string(),   // Green
    ],
};
```

## 📈 Chart Types Available

1. **Bar Chart** - `format_latency_histogram()`
2. **Line Chart** - `format_trend_analysis()`
3. **Radar Chart** - `format_metrics_radar()`
4. **Scatter Plot** - `format_cost_quality_scatter()`
5. **Pie/Doughnut** - `format_status_distribution()`
6. **Comparison Bar** - `format_comparison_bar()`

## 🧪 Run Demo

```bash
# Generate example dashboards
cargo run --example visualization_demo

# Open generated files
open demo_benchmark.html
open demo_comparison.html
open demo_trends.html
```

## ✅ Key Features

- ✅ Self-contained HTML (<500KB)
- ✅ Responsive (mobile, tablet, desktop)
- ✅ Dark/light mode
- ✅ Interactive charts
- ✅ Fast generation (<3s)
- ✅ No external dependencies

## 📚 Full Documentation

See [README.md](./README.md) for complete API documentation and examples.

## 🐛 Troubleshooting

**Issue**: Template not found
```rust
// Solution: Ensure templates are embedded
let generator = DashboardGenerator::new()?;
```

**Issue**: Chart.js not loading
```rust
// Solution: Chart.js is embedded via include_str!
// No external CDN needed
```

**Issue**: File too large
```rust
// Solution: Reduce data points
let config = DashboardConfig {
    max_data_points: 100,
    ..Default::default()
};
```

## 🎯 Common Patterns

### Generate and Save
```rust
let html = generator.generate_benchmark_dashboard(&results, &config)?;
generator.export_to_file(&html, Path::new("output.html"))?;
```

### Generate Multiple Dashboards
```rust
for (i, result) in results.iter().enumerate() {
    let html = generator.generate_benchmark_dashboard(result, &config)?;
    generator.export_to_file(&html, Path::new(&format!("dashboard_{}.html", i)))?;
}
```

### Custom Chart Data
```rust
use llm_test_bench_core::visualization::charts::ChartDataFormatter;

let chart_data = ChartDataFormatter::format_latency_histogram(&results, 15);
let json = serde_json::to_string(&chart_data)?;
```

## 📦 Dependencies

Only requires:
- `tera = "1.20"` (template engine)

Chart.js is embedded (no runtime dependency).

## 🏁 Next Steps

1. Read [README.md](./README.md) for detailed API docs
2. Run `cargo run --example visualization_demo`
3. Customize templates in `templates/` directory
4. Add your own chart types in `charts.rs`
