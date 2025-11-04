// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Analytics and optimization for benchmark results.
//!
//! This module provides comprehensive statistical analysis and cost optimization
//! capabilities for LLM benchmarking results. It includes tools for:
//!
//! - Statistical testing (t-tests, Mann-Whitney U, effect sizes)
//! - Cost optimization and recommendations
//! - Trend analysis and regression detection
//!
//! # Examples
//!
//! ## Statistical Analysis
//!
//! ```
//! use llm_test_bench_core::analytics::StatisticalAnalyzer;
//!
//! let analyzer = StatisticalAnalyzer::new(0.95);
//! let baseline = vec![100.0, 110.0, 105.0, 95.0, 102.0];
//! let improved = vec![90.0, 88.0, 92.0, 85.0, 87.0];
//!
//! let result = analyzer.t_test(&baseline, &improved).unwrap();
//! if result.is_significant {
//!     println!("Improvement is statistically significant!");
//! }
//! ```
//!
//! ## Cost Optimization
//!
//! ```
//! use llm_test_bench_core::analytics::CostOptimizer;
//!
//! let optimizer = CostOptimizer::new(0.95); // 95% quality threshold
//! // Analyze results and get recommendations...
//! ```

pub mod statistics;
pub mod cost_optimizer;

#[cfg(test)]
mod tests;

pub use statistics::{
    StatisticalAnalyzer, TTestResult, MannWhitneyResult, SignificanceTest,
};
pub use cost_optimizer::{
    CostOptimizer, CostRecommendation, ExpensivePattern, PatternType,
    OptimizationSuggestion, ImplementationEffort,
};
