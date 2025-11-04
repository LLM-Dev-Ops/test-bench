// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Benchmark reporter implementation

use super::BenchmarkResult;
use std::fmt;

/// Benchmark reporter
pub struct BenchmarkReporter;

impl BenchmarkReporter {
    /// Create a new benchmark reporter
    pub fn new() -> Self {
        Self
    }

    /// Generate a report from benchmark results
    pub fn report(&self, results: &[BenchmarkResult]) -> String {
        let mut output = String::new();
        output.push_str("Benchmark Results\n");
        output.push_str("=================\n\n");

        for result in results {
            output.push_str(&format!("{}\n", result));
        }

        output
    }
}

impl Default for BenchmarkReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {}/{}: P50={:?}, P95={:?}, Success Rate={:.2}%",
            self.name,
            self.provider,
            self.model,
            self.latency.p50,
            self.latency.p95,
            self.success_rate * 100.0
        )
    }
}
