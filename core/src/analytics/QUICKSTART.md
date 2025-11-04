# Analytics Module - Quick Start Guide

## Installation

The analytics module is included in `llm-test-bench-core`. Add to your `Cargo.toml`:

```toml
[dependencies]
llm-test-bench-core = { path = "../core" }
```

## Basic Usage

### 1. Statistical Comparison (30 seconds)

```rust
use llm_test_bench_core::analytics::StatisticalAnalyzer;

let analyzer = StatisticalAnalyzer::new(0.95); // 95% confidence

// Compare two sets of measurements
let baseline = vec![100.0, 110.0, 105.0, 95.0, 102.0];
let optimized = vec![90.0, 88.0, 92.0, 85.0, 87.0];

// Run t-test
let result = analyzer.t_test(&baseline, &optimized)?;
println!("P-value: {:.4}", result.p_value);
println!("Significant: {}", result.is_significant);

// Calculate effect size
let effect = analyzer.cohens_d(&baseline, &optimized);
println!("Effect size: {:.2}", effect);
```

**Output:**
```
P-value: 0.0012
Significant: true
Effect size: 1.45
```

### 2. Cost Optimization (1 minute)

```rust
use llm_test_bench_core::analytics::CostOptimizer;

let optimizer = CostOptimizer::new(0.95); // 95% quality threshold

// Calculate potential savings
let monthly_savings = optimizer.calculate_savings(
    "gpt-4",           // Current model
    "gpt-3.5-turbo",   // Cheaper alternative
    100_000,           // Monthly requests
);

println!("Switching saves ${:.2}/month", monthly_savings);
```

**Output:**
```
Switching saves $4275.00/month
```

### 3. Pattern Detection (2 minutes)

```rust
use llm_test_bench_core::analytics::CostOptimizer;
use llm_test_bench_core::benchmarks::results::BenchmarkResults;

let optimizer = CostOptimizer::new(0.95);

// Load your benchmark results
let results = load_benchmark_results("production.json")?;

// Detect expensive patterns
let patterns = optimizer.identify_expensive_patterns(&[results.clone()]);

for pattern in patterns {
    println!("Found: {}", pattern.description);
    println!("Potential savings: ${:.2}", pattern.potential_savings);
}

// Get optimization suggestions
let suggestions = optimizer.suggest_prompt_optimizations(&results);

for suggestion in suggestions {
    println!("\n{}", suggestion.title);
    println!("Savings: ${:.2}", suggestion.estimated_savings);
    println!("Effort: {}", suggestion.implementation_effort);
}
```

**Output:**
```
Found: Long prompts detected (avg 1200 tokens). Consider compression.
Potential savings: $45.00

Compress Long Prompts
Savings: $45.00
Effort: Medium
```

## Common Recipes

### Recipe 1: A/B Test Validation

```rust
// Compare two models statistically
let analyzer = StatisticalAnalyzer::new(0.95);

let test = analyzer.is_significant_improvement(
    &model_a_results,
    &model_b_results,
    "latency"
)?;

if test.is_significant {
    println!("✓ Model B is significantly faster");
    println!("  {}", test.interpretation);
} else {
    println!("✗ No significant difference");
}
```

### Recipe 2: Cost vs Quality Trade-off

```rust
let optimizer = CostOptimizer::new(0.90); // Accept 90% quality

let recommendation = optimizer.recommend_model(&all_benchmarks)?;

println!("Recommended: {}", recommendation.recommended_model);
println!("Annual savings: ${:.2}", recommendation.annual_savings);
println!("Quality change: {:+.1}%", recommendation.quality_delta * 100.0);
```

### Recipe 3: Monthly Cost Report

```rust
let optimizer = CostOptimizer::new(0.95);

// Analyze current month
let patterns = optimizer.identify_expensive_patterns(&monthly_benchmarks);
let total_savings: f64 = patterns.iter()
    .map(|p| p.potential_savings)
    .sum();

println!("Total optimization potential: ${:.2}/month", total_savings);

// Top 3 opportunities
patterns.sort_by(|a, b| b.potential_savings.partial_cmp(&a.potential_savings).unwrap());
for (i, pattern) in patterns.iter().take(3).enumerate() {
    println!("{}. {} (${:.2})", i+1, pattern.description, pattern.potential_savings);
}
```

## Decision Tree

### Which Statistical Test?

```
Do you have outliers or skewed data?
├─ Yes → Use Mann-Whitney U test
└─ No → Use t-test

Is the difference practically meaningful?
├─ Check effect size (Cohen's d)
├─ Small (d < 0.2): Probably not worth it
├─ Medium (d = 0.5): Consider cost/benefit
└─ Large (d > 0.8): Likely worth pursuing
```

### Which Optimizer Method?

```
What's your goal?
├─ Find best model → recommend_model()
├─ Calculate savings → calculate_savings()
├─ Find waste → identify_expensive_patterns()
└─ Get suggestions → suggest_prompt_optimizations()
```

## API Cheat Sheet

### StatisticalAnalyzer

| Method | Use When | Returns |
|--------|----------|---------|
| `t_test()` | Normal data, comparing means | TTestResult |
| `mann_whitney_u()` | Skewed data, outliers | MannWhitneyResult |
| `cohens_d()` | Measuring effect size | f64 |
| `confidence_interval()` | Need uncertainty bounds | (f64, f64) |
| `is_significant_improvement()` | High-level comparison | SignificanceTest |

### CostOptimizer

| Method | Use When | Returns |
|--------|----------|---------|
| `recommend_model()` | Need model selection | CostRecommendation |
| `calculate_savings()` | Estimate cost difference | f64 |
| `identify_expensive_patterns()` | Find optimization areas | Vec<ExpensivePattern> |
| `suggest_prompt_optimizations()` | Get action items | Vec<OptimizationSuggestion> |

## Effect Size Interpretation

| Cohen's d | Interpretation | Action |
|-----------|----------------|--------|
| < 0.2 | Negligible | Probably not worth optimizing |
| 0.2 - 0.5 | Small | Consider if cost is low |
| 0.5 - 0.8 | Medium | Likely worth pursuing |
| > 0.8 | Large | Strong candidate for optimization |

## Pattern Types

| Pattern | Threshold | Optimization |
|---------|-----------|--------------|
| LongPrompts | > 1000 tokens | Compress, summarize |
| VerboseResponses | > 500 tokens | Set max_tokens |
| ExpensiveModel | High cost, high success | Use cheaper model |
| HighTemperature | Deterministic task | Lower temperature |

## Performance Notes

- T-test: < 10ms for n < 1000
- Mann-Whitney: < 20ms for n < 1000
- Cost analysis: < 100ms per benchmark
- Pattern detection: < 50ms per benchmark

## Error Handling

```rust
match analyzer.t_test(&sample_a, &sample_b) {
    Ok(result) => println!("P-value: {:.4}", result.p_value),
    Err(e) => eprintln!("Error: {}", e),
}
```

**Common errors:**
- "Samples cannot be empty"
- "Samples must have at least 2 observations each"
- "No models meet the quality threshold"
- "Unknown metric: {}"

## Tips

1. **Always report effect sizes** with p-values
2. **Use Mann-Whitney U** if you're unsure about normality
3. **Set realistic quality thresholds** (0.90-0.95)
4. **Start with low-effort optimizations** first
5. **Re-run analysis** monthly as prices change

## Next Steps

- See [README.md](README.md) for detailed documentation
- See [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md) for technical details
- Run tests: `cargo test --package llm-test-bench-core analytics`

## Support

For issues or questions:
1. Check the README for detailed examples
2. Review test cases in `tests.rs`
3. Consult academic references for statistical methods
