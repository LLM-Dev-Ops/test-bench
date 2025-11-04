# Analytics Module

Production-grade statistical analysis and cost optimization for LLM benchmarking.

## Overview

The analytics module provides comprehensive tools for analyzing benchmark results, including:

- **Statistical Testing**: Determine if performance differences are statistically significant
- **Cost Optimization**: Identify opportunities to reduce LLM costs while maintaining quality
- **Effect Size Analysis**: Measure the practical significance of improvements
- **Pattern Detection**: Identify expensive usage patterns automatically

## Modules

### `statistics.rs` - Statistical Analysis

Provides rigorous statistical testing to determine if observed differences are significant or due to random variation.

#### Key Components

**StatisticalAnalyzer**
- Configurable confidence level (default: 95%)
- Multiple statistical tests
- Effect size calculations
- Plain-language interpretations

#### Statistical Tests

**1. Two-Sample T-Test (Welch's t-test)**

Used to compare means of two samples. Doesn't assume equal variances.

```rust
use llm_test_bench_core::analytics::StatisticalAnalyzer;

let analyzer = StatisticalAnalyzer::new(0.95);
let baseline = vec![100.0, 110.0, 105.0, 95.0, 102.0];
let optimized = vec![90.0, 88.0, 92.0, 85.0, 87.0];

let result = analyzer.t_test(&baseline, &optimized)?;
println!("P-value: {:.4}", result.p_value);
println!("Significant: {}", result.is_significant);
```

**When to use**: Data is approximately normally distributed, comparing means.

**2. Mann-Whitney U Test (Non-parametric)**

Compares distributions without assuming normality. More robust to outliers.

```rust
let result = analyzer.mann_whitney_u(&baseline, &optimized)?;
println!("U-statistic: {:.2}", result.u_statistic);
println!("Significant: {}", result.is_significant);
```

**When to use**: Skewed data, outliers present, or small samples.

**3. Effect Size (Cohen's d)**

Measures the magnitude of difference, independent of sample size.

```rust
let effect = analyzer.cohens_d(&baseline, &optimized);
println!("Effect size: {:.2}", effect);
```

**Interpretation:**
- |d| < 0.2: Negligible effect
- |d| = 0.2-0.5: Small effect
- |d| = 0.5-0.8: Medium effect
- |d| > 0.8: Large effect

#### High-Level Analysis

**Comprehensive Significance Testing**

```rust
use llm_test_bench_core::benchmarks::results::BenchmarkResults;

let test = analyzer.is_significant_improvement(
    &baseline_results,
    &comparison_results,
    "latency"
)?;

if test.is_significant {
    println!("{}", test.interpretation);
    // Output: "Statistically significant improvement in latency (p=0.0023).
    //          Effect size is large (d=1.23). latency changed by 15.2%
    //          (from 1000.00 to 848.00)."
}
```

### `cost_optimizer.rs` - Cost Optimization

Analyzes benchmark results to identify cost savings opportunities while maintaining quality thresholds.

#### Key Components

**CostOptimizer**
- Quality-constrained optimization
- Model recommendations
- Savings calculations
- Pattern detection

#### Model Recommendations

```rust
use llm_test_bench_core::analytics::CostOptimizer;

let optimizer = CostOptimizer::new(0.95); // 95% quality threshold

let recommendation = optimizer.recommend_model(&benchmark_results)?;

println!("Recommended: {}", recommendation.recommended_model);
println!("Monthly savings: ${:.2}", recommendation.monthly_savings);
println!("Annual savings: ${:.2}", recommendation.annual_savings);
println!("Quality delta: {:.1}%", recommendation.quality_delta * 100.0);
println!("Reasoning: {}", recommendation.reasoning);
```

#### Savings Calculations

```rust
let monthly_savings = optimizer.calculate_savings(
    "gpt-4",              // Current model
    "gpt-3.5-turbo",      // Recommended model
    100_000,              // Monthly requests
);

println!("Switching could save ${:.2}/month", monthly_savings);
```

#### Pattern Detection

Automatically identifies expensive usage patterns:

```rust
let patterns = optimizer.identify_expensive_patterns(&historical_results);

for pattern in patterns {
    println!("{}: {}", pattern.pattern_type, pattern.description);
    println!("Potential savings: ${:.2}", pattern.potential_savings);
}
```

**Detected Patterns:**
- **Long Prompts**: Prompts > 1000 tokens
- **Verbose Responses**: Completions > 500 tokens
- **Expensive Models**: High-cost models for high-success tasks
- **Suboptimal Settings**: Temperature/parameter issues

#### Optimization Suggestions

```rust
let suggestions = optimizer.suggest_prompt_optimizations(&results);

for suggestion in suggestions {
    println!("{}", suggestion.title);
    println!("{}", suggestion.description);
    println!("Estimated savings: ${:.2}", suggestion.estimated_savings);
    println!("Effort: {}", suggestion.implementation_effort);
}
```

## Usage Examples

### Example 1: Compare Two Models

```rust
use llm_test_bench_core::analytics::{StatisticalAnalyzer, CostOptimizer};

// Run benchmarks
let gpt4_results = run_benchmark("gpt-4", test_suite)?;
let gpt35_results = run_benchmark("gpt-3.5-turbo", test_suite)?;

// Statistical comparison
let analyzer = StatisticalAnalyzer::new(0.95);
let latency_test = analyzer.is_significant_improvement(
    &gpt4_results,
    &gpt35_results,
    "latency"
)?;

println!("Latency: {}", latency_test.interpretation);

// Cost analysis
let optimizer = CostOptimizer::new(0.90);
let recommendation = optimizer.recommend_model(&[gpt4_results, gpt35_results])?;

println!("Recommendation: {}", recommendation.recommended_model);
println!("Annual savings: ${:.2}", recommendation.annual_savings);
```

### Example 2: Optimize Existing Usage

```rust
// Analyze current usage patterns
let current_results = load_benchmark_results("production-jan-2024.json")?;

let optimizer = CostOptimizer::new(0.95);

// Identify expensive patterns
let patterns = optimizer.identify_expensive_patterns(&[current_results.clone()]);
for pattern in &patterns {
    println!("Found: {} - ${:.2} potential savings",
             pattern.description, pattern.potential_savings);
}

// Get optimization suggestions
let suggestions = optimizer.suggest_prompt_optimizations(&current_results);
for suggestion in &suggestions {
    println!("\n{}", suggestion.title);
    println!("{}", suggestion.description);
    println!("Savings: ${:.2} | Effort: {}",
             suggestion.estimated_savings, suggestion.implementation_effort);
}
```

### Example 3: Track Improvements Over Time

```rust
// Load historical benchmarks
let baseline = load_benchmark("2024-01-baseline.json")?;
let after_optimization = load_benchmark("2024-02-optimized.json")?;

let analyzer = StatisticalAnalyzer::new(0.95);

// Analyze each metric
for metric in &["latency", "cost", "tokens"] {
    let test = analyzer.is_significant_improvement(
        &baseline,
        &after_optimization,
        metric
    )?;

    println!("\n{}: ", metric);
    println!("  Significant: {}", test.is_significant);
    println!("  P-value: {:.4}", test.p_value);
    println!("  Effect size: {:.2}", test.effect_size);
    println!("  {}", test.interpretation);
}
```

## Best Practices

### Statistical Analysis

1. **Always report effect sizes** alongside p-values
   - P-values only indicate if a difference exists
   - Effect sizes indicate if the difference matters

2. **Use appropriate tests**
   - T-test for normal data
   - Mann-Whitney U for skewed data or outliers
   - Consider sample size (need ≥ 2 per group)

3. **Consider practical significance**
   - A statistically significant 0.1ms improvement may not be practically meaningful
   - Use effect size and domain knowledge to assess importance

4. **Account for multiple comparisons**
   - Testing multiple metrics increases false positive rate
   - Consider Bonferroni correction for multiple tests

### Cost Optimization

1. **Set appropriate quality thresholds**
   - Don't sacrifice critical quality for cost savings
   - Consider business impact of quality changes

2. **Validate recommendations**
   - Test cheaper models on representative samples
   - Monitor quality in production

3. **Iterate on optimizations**
   - Start with low-effort suggestions
   - Measure impact before moving to high-effort changes

4. **Monitor continuously**
   - Re-run analysis monthly or quarterly
   - Pricing and model capabilities change over time

## Testing

The analytics module includes 40+ comprehensive tests covering:

- Statistical calculations (validated against R/Python)
- Cost optimization logic
- Pattern detection accuracy
- Edge cases (empty data, outliers, ties)
- Integration scenarios

Run tests with:
```bash
cargo test --package llm-test-bench-core analytics
```

## Model Pricing

The cost optimizer uses current pricing for:

**OpenAI:**
- GPT-4: $0.03/$0.06 per 1K tokens (prompt/completion)
- GPT-4 Turbo: $0.01/$0.03 per 1K tokens
- GPT-4o: $0.005/$0.015 per 1K tokens
- GPT-4o Mini: $0.00015/$0.0006 per 1K tokens
- GPT-3.5 Turbo: $0.0015/$0.002 per 1K tokens

**Anthropic:**
- Claude 3 Opus: $0.015/$0.075 per 1K tokens
- Claude 3 Sonnet: $0.003/$0.015 per 1K tokens
- Claude 3.5 Sonnet: $0.003/$0.015 per 1K tokens
- Claude 3 Haiku: $0.00025/$0.00125 per 1K tokens

*Pricing accurate as of January 2025. Check provider websites for current rates.*

## Performance

- Statistical tests: < 10ms for typical sample sizes (n < 1000)
- Cost analysis: < 100ms for full benchmark results
- Pattern detection: < 50ms per benchmark
- All operations are single-threaded and deterministic

## References

### Statistical Methods

- Welch's t-test: Welch, B. L. (1947). "The generalization of 'Student's' problem when several different population variances are involved"
- Mann-Whitney U: Mann, H. B.; Whitney, D. R. (1947). "On a test of whether one of two random variables is stochastically larger than the other"
- Cohen's d: Cohen, J. (1988). "Statistical Power Analysis for the Behavioral Sciences"

### Implementation

The statistical functions are implemented from first principles with validation against:
- R: `t.test()`, `wilcox.test()`, `effsize::cohen.d()`
- Python: `scipy.stats.ttest_ind()`, `scipy.stats.mannwhitneyu()`
- Online calculators for edge cases

## Contributing

When adding new statistical tests or optimization heuristics:

1. Include comprehensive unit tests
2. Validate against reference implementations
3. Document when to use the new feature
4. Update this README with examples
5. Consider edge cases (empty data, outliers, etc.)
