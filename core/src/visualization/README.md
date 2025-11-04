# Visualization Module

Production-quality HTML dashboard generation with interactive Chart.js visualizations for LLM Test Bench.

## Features

- **Self-Contained HTML**: Single-file dashboards (<500KB) with embedded CSS, JavaScript, and Chart.js
- **Responsive Design**: Mobile-friendly layouts with CSS Grid and Flexbox
- **Dark Mode Support**: Automatic theme switching based on system preferences
- **Interactive Charts**: 6+ chart types using Chart.js 4.4.0
- **Fast Generation**: Template-based rendering with Tera (<3s for 100 tests)
- **Cross-Browser Compatible**: Works on all modern browsers

## Module Structure

```
visualization/
├── mod.rs                    # Module exports and documentation
├── dashboard.rs              # Main dashboard generator
├── charts.rs                 # Chart data formatting
├── templates/               # Tera HTML templates
│   ├── base.html            # Base template with CSS
│   ├── benchmark_results.html
│   ├── comparison.html
│   ├── trend_analysis.html
│   └── cost_analysis.html
└── assets/
    └── chartjs.min.js       # Chart.js 4.4.0 (201KB)
```

## Dashboard Types

### 1. Benchmark Results Dashboard
Single benchmark run analysis with:
- Summary cards (total tests, success rate, latency, cost)
- Latency distribution histogram
- Test status pie chart
- Evaluation metrics radar chart
- Detailed results table

### 2. Model Comparison Dashboard
Side-by-side comparison of multiple models:
- Multi-metric bar chart (success rate, latency)
- Cost vs quality scatter plot
- Model rankings table
- Performance comparison grid

### 3. Trend Analysis Dashboard
Performance trends over time:
- Dual-axis time series chart
- Success rate trend line
- Latency trend line
- Key insights section

### 4. Cost Analysis Dashboard
Cost efficiency analysis:
- Cost breakdown by model
- Cost per request comparison
- Efficiency score radar
- Optimization recommendations

## Chart Types

The module supports these Chart.js chart types:

1. **Bar Chart** - Latency histograms, comparisons
2. **Line Chart** - Trend analysis, time series
3. **Radar Chart** - Evaluation metrics, efficiency scores
4. **Scatter Plot** - Cost vs quality analysis
5. **Pie/Doughnut Chart** - Status distribution
6. **Multi-axis Charts** - Combined metrics visualization

## Usage Examples

### Basic Dashboard Generation

```rust
use llm_test_bench_core::visualization::{DashboardGenerator, DashboardConfig};
use llm_test_bench_core::benchmarks::results::BenchmarkResults;
use std::path::Path;

fn generate_dashboard(results: &BenchmarkResults) -> anyhow::Result<()> {
    // Create generator
    let generator = DashboardGenerator::new()?;

    // Use default config
    let config = DashboardConfig::default();

    // Generate HTML
    let html = generator.generate_benchmark_dashboard(results, &config)?;

    // Export to file
    generator.export_to_file(&html, Path::new("dashboard.html"))?;

    Ok(())
}
```

### Custom Configuration

```rust
use llm_test_bench_core::visualization::{DashboardConfig, Theme};

let config = DashboardConfig {
    title: "Production Benchmark Results".to_string(),
    theme: Theme::Dark,
    max_data_points: 500,
    chart_colors: vec![
        "rgb(59, 130, 246)".to_string(),   // Blue
        "rgb(16, 185, 129)".to_string(),   // Green
        "rgb(245, 158, 11)".to_string(),   // Orange
    ],
};
```

### Model Comparison

```rust
fn compare_models(
    results: &[BenchmarkResults]
) -> anyhow::Result<String> {
    let generator = DashboardGenerator::new()?;
    let config = DashboardConfig::default();

    generator.generate_comparison_dashboard(results, &config)
}
```

### Trend Analysis

```rust
fn analyze_trends(
    historical_results: &[BenchmarkResults]
) -> anyhow::Result<String> {
    let generator = DashboardGenerator::new()?;
    let config = DashboardConfig::default();

    generator.generate_trend_dashboard(historical_results, &config)
}
```

## Custom Chart Data

You can generate chart data separately for custom dashboards:

```rust
use llm_test_bench_core::visualization::charts::ChartDataFormatter;

// Create chart data
let latency_data = ChartDataFormatter::format_latency_histogram(&results, 10);
let metrics_data = ChartDataFormatter::format_metrics_radar(&results);
let status_data = ChartDataFormatter::format_status_distribution(&results);

// Serialize to JSON for templates
let json = serde_json::to_string(&latency_data)?;
```

## Template Customization

Templates use Tera syntax and can be extended. The base template provides:

- CSS variables for theming
- Responsive grid layouts
- Reusable component styles
- Dark mode support

### CSS Variables

```css
:root {
    --primary: #3b82f6;
    --success: #10b981;
    --warning: #f59e0b;
    --danger: #ef4444;
    --info: #06b6d4;
}
```

## Performance

- **Generation Speed**: <3s for 100 test results
- **File Size**: <500KB for complete dashboard with 100+ data points
- **Browser Rendering**: <100ms initial render
- **Chart Animation**: 60 FPS smooth animations

## Browser Support

- Chrome/Edge: Latest 2 versions
- Firefox: Latest 2 versions
- Safari: Latest 2 versions
- Mobile browsers: iOS Safari, Chrome Mobile

## Testing

The module includes 20+ comprehensive tests covering:

- Template rendering
- Chart data formatting
- HTML generation
- File export
- Configuration handling
- Data serialization

Run tests with:
```bash
cargo test --package llm-test-bench-core --lib visualization
```

## Dependencies

- **tera 1.20**: Template engine for HTML generation
- **serde_json**: JSON serialization for chart data
- **Chart.js 4.4.0**: Embedded JavaScript charting library

## Design Principles

1. **Self-Contained**: No external dependencies at runtime
2. **Responsive**: Mobile-first design approach
3. **Accessible**: Semantic HTML and ARIA labels
4. **Fast**: Optimized rendering and minimal JavaScript
5. **Maintainable**: Clean separation of concerns

## Future Enhancements

Potential additions:
- Export to PNG/PDF
- Real-time data updates (WebSocket)
- Custom color themes
- Additional chart types (heatmaps, sankey diagrams)
- Interactive filtering and sorting
- Data export (CSV, JSON)

## License

Licensed under Apache 2.0 or MIT license.
