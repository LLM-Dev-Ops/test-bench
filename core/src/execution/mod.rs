//! # Execution Span Module - Agentics Foundational Execution Unit
//!
//! This module ensures the repository NEVER executes silently.
//! All externally-invoked operations emit agent-level execution spans
//! and integrate into a hierarchical ExecutionGraph produced by a Core.
//!
//! ## Span Hierarchy Invariant
//!
//! ```text
//! Core
//!   └─ Repo (this repo)
//!       └─ Agent (one or more)
//! ```
//!
//! If no agent span exists, execution is INVALID.

pub mod context;
pub mod span;
pub mod middleware;

pub use context::{ExecutionContext, validate_execution_context, ExecutionContextError};
pub use span::{
    ExecutionSpan, SpanType, SpanStatus, ExecutionArtifact,
    ExecutionResult, SpanManager,
};
pub use middleware::ExecutionLayer;

/// The canonical name for this repository.
pub const REPO_NAME: &str = "llm-test-bench";
