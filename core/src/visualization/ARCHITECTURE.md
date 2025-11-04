# Visualization Module Architecture

## Overview

The visualization module provides a clean, layered architecture for generating interactive HTML dashboards.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    User Application                          │
│  (CLI, Web Service, Integration Tests)                      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                 DashboardGenerator                           │
│  • Template management (Tera)                                │
│  • Configuration handling                                    │
│  • HTML generation                                           │
│  • File export                                               │
└──────────┬───────────────────────────┬──────────────────────┘
           │                           │
           ▼                           ▼
┌──────────────────────┐    ┌─────────────────────────────────┐
│  ChartDataFormatter  │    │      DashboardData             │
│  • Latency histogram │    │  • Summary cards               │
│  • Metrics radar     │    │  • Result rows                 │
│  • Comparison bar    │    │  • Chart data JSON             │
│  • Scatter plot      │    │  • Metadata                    │
│  • Trend analysis    │    └─────────────────────────────────┘
│  • Status pie        │
└──────────┬───────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│              Benchmark Results (Input)                       │
│  • BenchmarkResults                                          │
│  • TestResult[]                                              │
│  • ResultSummary                                             │
└─────────────────────────────────────────────────────────────┘

                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                  Tera Templates                              │
│  ┌────────────────────────────────────────────┐             │
│  │  base.html (Base template + CSS)           │             │
│  └────────────────────────────────────────────┘             │
│           │                                                  │
│           ├─► benchmark_results.html                        │
│           ├─► comparison.html                               │
│           ├─► trend_analysis.html                           │
│           └─► cost_analysis.html                            │
└─────────────────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                Embedded Assets                               │
│  • Chart.js 4.4.0 (201KB)                                   │
│  • CSS (responsive, dark mode)                               │
│  • JavaScript (chart initialization)                         │
└─────────────────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│           Self-Contained HTML Output                         │
│  • Single file (<500KB)                                     │
│  • No external dependencies                                  │
│  • Works offline                                             │
│  • Responsive & interactive                                  │
└─────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

### 1. DashboardGenerator
**File**: `dashboard.rs` (525 lines)

**Responsibilities**:
- Initialize Tera template engine with embedded templates
- Validate and apply DashboardConfig
- Orchestrate data preparation
- Render HTML from templates
- Export to filesystem

**Key Types**:
- `DashboardGenerator` - Main generator struct
- `DashboardType` - Enum for dashboard variants
- `DashboardConfig` - Configuration builder
- `Theme` - Color theme selection
- `DashboardData` - Template context data

**Public API**:
```rust
pub fn new() -> Result<Self>
pub fn generate_benchmark_dashboard(&self, results: &BenchmarkResults, config: &DashboardConfig) -> Result<String>
pub fn generate_comparison_dashboard(&self, all_results: &[BenchmarkResults], config: &DashboardConfig) -> Result<String>
pub fn generate_trend_dashboard(&self, historical_results: &[BenchmarkResults], config: &DashboardConfig) -> Result<String>
pub fn export_to_file(&self, html: &str, output_path: &Path) -> Result<()>
```

### 2. ChartDataFormatter
**File**: `charts.rs` (426 lines)

**Responsibilities**:
- Convert BenchmarkResults to Chart.js JSON format
- Calculate histogram bins and distributions
- Aggregate metrics for radar charts
- Prepare time series data
- Format labels and datasets

**Chart Types**:
- Bar charts (latency histograms, comparisons)
- Line charts (trends over time)
- Radar charts (multi-dimensional metrics)
- Scatter plots (cost vs quality)
- Pie/Doughnut charts (status distribution)

**Public API**:
```rust
pub fn format_latency_histogram(results: &BenchmarkResults, bin_count: usize) -> Value
pub fn format_metrics_radar(results: &BenchmarkResults) -> Value
pub fn format_comparison_bar(all_results: &[BenchmarkResults]) -> Value
pub fn format_cost_quality_scatter(all_results: &[BenchmarkResults]) -> Value
pub fn format_trend_analysis(historical_results: &[BenchmarkResults]) -> Value
pub fn format_status_distribution(results: &BenchmarkResults) -> Value
```

### 3. Templates
**Directory**: `templates/` (5 files, 1,145 lines)

**Structure**:
```
base.html (329 lines)
├── CSS (responsive, dark mode, print styles)
├── HTML structure
└── JavaScript placeholder

benchmark_results.html (191 lines)
├── extends base.html
├── Summary cards grid
├── Chart containers (latency, status, metrics)
└── Detailed results table

comparison.html (189 lines)
├── extends base.html
├── Comparison metrics
├── Side-by-side charts
└── Rankings table

trend_analysis.html (214 lines)
├── extends base.html
├── Time series charts
├── Trend indicators
└── Insights section

cost_analysis.html (222 lines)
├── extends base.html
├── Cost breakdown
├── Efficiency charts
└── Optimization recommendations
```

**Template Features**:
- Tera syntax ({% ... %}, {{ ... }})
- Template inheritance ({% extends ... %})
- Blocks ({% block content %})
- Filters (| round, | safe)
- Loops ({% for ... %})
- Conditionals ({% if ... %})

### 4. Assets
**Directory**: `assets/` (1 file, 201KB)

**chartjs.min.js**:
- Chart.js 4.4.0 (latest stable)
- Embedded via `include_str!` macro
- No CDN dependency
- Supports all chart types
- 201KB minified

## Data Flow

### Benchmark Dashboard Generation

```
1. User creates BenchmarkResults
   └─► Contains: TestResult[], timestamp, provider info

2. User calls generate_benchmark_dashboard()
   └─► With: results, config

3. DashboardGenerator.prepare_benchmark_data()
   ├─► Creates summary cards (4 cards)
   ├─► Formats result rows for table
   ├─► Calls ChartDataFormatter.format_latency_histogram()
   ├─► Calls ChartDataFormatter.format_metrics_radar()
   └─► Calls ChartDataFormatter.format_status_distribution()

4. ChartDataFormatter returns JSON Values
   └─► serde_json::Value (Chart.js compatible)

5. DashboardData struct populated
   ├─► title, dataset_name, provider_name
   ├─► summary_cards[]
   ├─► results[]
   ├─► latency_data_json
   ├─► metrics_data_json
   ├─► status_data_json
   └─► chartjs_code (embedded Chart.js)

6. Tera.render("benchmark_results.html", context)
   ├─► Extends base.html
   ├─► Fills content block
   ├─► Injects chart data
   └─► Initializes Chart.js

7. Returns complete HTML string
   └─► Self-contained, <500KB

8. Optional: export_to_file()
   └─► Writes to filesystem
```

## Template Inheritance Flow

```
base.html
├── Defines structure:
│   ├── <head> with CSS
│   ├── {% block content %}
│   ├── Chart.js script
│   └── {% block charts %}
│
├── benchmark_results.html
│   ├── {% extends "base.html" %}
│   ├── {% block content %}
│   │   ├── Header
│   │   ├── Summary cards
│   │   ├── Chart containers
│   │   └── Results table
│   └── {% block charts %}
│       └── Chart.js initialization
│
├── comparison.html
│   ├── {% extends "base.html" %}
│   ├── {% block content %}
│   │   └── Comparison-specific layout
│   └── {% block charts %}
│       └── Comparison charts
│
├── trend_analysis.html
│   └── (similar structure)
│
└── cost_analysis.html
    └── (similar structure)
```

## Configuration System

```
DashboardConfig
├── title: String
│   └─► Displayed in header
│
├── theme: Theme
│   ├─► Light: Light mode
│   ├─► Dark: Dark mode
│   └─► Auto: System preference
│
├── max_data_points: usize
│   └─► Limit chart data points
│
└── chart_colors: Vec<String>
    └─► Custom color palette
```

## Error Handling

```
anyhow::Result<T>
├── Template errors
│   ├── Template not found
│   ├── Syntax errors
│   └── Rendering failures
│
├── File I/O errors
│   ├── Permission denied
│   ├── Disk full
│   └── Invalid path
│
└── Data errors
    ├── Empty results
    ├── Invalid metrics
    └── Serialization failures
```

All errors include context via `.context()` for debugging.

## Performance Characteristics

### Time Complexity
- Dashboard generation: O(n) where n = number of tests
- Chart data formatting: O(n) per chart
- Template rendering: O(1) (cached templates)

### Space Complexity
- Memory: O(n) for storing results
- Output size: O(n) but limited by max_data_points
- Typical: 250-350KB for 100 tests

### Optimization Strategies
1. **Template Caching**: Tera caches parsed templates
2. **Lazy JSON Serialization**: Only serialize when needed
3. **Histogram Binning**: Reduces data points for large datasets
4. **Embedded Assets**: No network requests

## Extension Points

### Adding New Dashboard Types
```rust
// 1. Add variant to DashboardType
pub enum DashboardType {
    MyNewDashboard,
}

// 2. Create template
templates/my_new_dashboard.html

// 3. Add generation method
impl DashboardGenerator {
    pub fn generate_my_dashboard(...) -> Result<String> {
        // Implementation
    }
}
```

### Adding New Chart Types
```rust
// 1. Add formatting function
impl ChartDataFormatter {
    pub fn format_my_chart(data: &Data) -> Value {
        json!({
            "labels": [...],
            "datasets": [...]
        })
    }
}

// 2. Use in template
<script>
new Chart(ctx, {
    type: 'myChartType',
    data: {{ my_chart_data_json | safe }}
});
</script>
```

## Testing Strategy

### Unit Tests (15)
- Test each chart formatter independently
- Test configuration handling
- Test data serialization
- Test edge cases (empty, large datasets)

### Integration Tests (20)
- Test complete dashboard generation
- Test file export
- Test HTML validity
- Test performance benchmarks
- Test size constraints

### Manual Testing
- Visual inspection in multiple browsers
- Responsive design testing
- Dark mode verification
- Chart interactivity

## Security Considerations

1. **No User Input in Templates**: All data is properly escaped by Tera
2. **No External Resources**: No CDN dependencies (XSS prevention)
3. **File System Safety**: Validates paths before writing
4. **Content Security**: No inline event handlers
5. **Data Sanitization**: Proper escaping of test IDs and content

## Browser Compatibility

### Required Features
- ES6 JavaScript (for Chart.js)
- CSS Grid and Flexbox
- CSS Custom Properties (variables)
- prefers-color-scheme media query

### Polyfills
Not required - modern browsers only

### Tested Browsers
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+
- Mobile Safari (iOS 14+)
- Chrome Mobile (Android)

## Deployment

### Build
```bash
cargo build --package llm-test-bench-core
```

### Usage
```rust
use llm_test_bench_core::visualization::DashboardGenerator;

let generator = DashboardGenerator::new()?;
let html = generator.generate_benchmark_dashboard(&results, &config)?;
```

### Distribution
- Templates embedded in binary (no separate files)
- Chart.js embedded in binary (no CDN)
- Single binary deployment

## Maintenance

### Adding New Features
1. Update appropriate module (charts.rs or dashboard.rs)
2. Add tests
3. Update documentation
4. Update examples

### Updating Dependencies
- Tera: Check template syntax compatibility
- Chart.js: Download new version, test compatibility

### Versioning
Follow semantic versioning:
- Major: Breaking API changes
- Minor: New features (backward compatible)
- Patch: Bug fixes

## Summary

The visualization module provides a well-architected, production-ready solution for generating interactive dashboards. Key strengths:

- **Clean Architecture**: Clear separation of concerns
- **Extensibility**: Easy to add new dashboard and chart types
- **Performance**: Optimized for speed and size
- **Reliability**: Comprehensive testing and error handling
- **Usability**: Simple API with sensible defaults
- **Maintainability**: Well-documented and organized code
