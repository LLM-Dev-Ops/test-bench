//! Execution span types and span manager.
//!
//! Spans are append-only, causally ordered via parent_span_id,
//! and JSON-serializable without loss.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::context::ExecutionContext;
use super::REPO_NAME;

/// Span type discriminator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpanType {
    /// Repository-level span.
    Repo,
    /// Agent-level span.
    Agent,
}

/// Span status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpanStatus {
    /// Currently executing.
    Running,
    /// Completed successfully.
    Completed,
    /// Failed with error(s).
    Failed,
}

/// A single execution span within the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSpan {
    /// Discriminator: "repo" or "agent".
    #[serde(rename = "type")]
    pub span_type: SpanType,

    /// Unique span identifier.
    pub span_id: String,

    /// Parent span (Core span for repo, repo span for agents).
    pub parent_span_id: String,

    /// Start time.
    pub start_time: DateTime<Utc>,

    /// End time (set on completion/failure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,

    /// Current status.
    pub status: SpanStatus,

    /// Name of this repository.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_name: Option<String>,

    /// Name of the agent that executed (agent spans only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,

    /// Failure reasons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reasons: Option<Vec<String>>,
}

/// An artifact produced during execution.
/// MUST be attached to the agent-level span that produced it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionArtifact {
    /// Stable reference: ID, URI, hash, or filename.
    pub reference: String,

    /// Artifact type descriptor.
    pub artifact_type: String,

    /// Span ID of the agent that produced this artifact.
    pub agent_span_id: String,

    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Complete execution result returned by this repo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult<T: Serialize> {
    /// The repo-level span.
    pub repo_span: ExecutionSpan,

    /// Nested agent-level spans.
    pub agent_spans: Vec<ExecutionSpan>,

    /// Artifacts produced.
    pub artifacts: Vec<ExecutionArtifact>,

    /// The operation's return data (if successful).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// Whether the execution is valid per invariants.
    pub valid: bool,

    /// Validation errors (if invalid).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<Vec<String>>,
}

/// Manages execution spans for a single repo-level invocation.
pub struct SpanManager {
    repo_span: ExecutionSpan,
    agent_spans: HashMap<String, ExecutionSpan>,
    artifacts: Vec<ExecutionArtifact>,
    active_agents: std::collections::HashSet<String>,
    finalized: bool,
}

impl SpanManager {
    /// Create a new SpanManager for an incoming execution.
    pub fn new(ctx: &ExecutionContext) -> Self {
        let repo_span = ExecutionSpan {
            span_type: SpanType::Repo,
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: ctx.parent_span_id.clone(),
            start_time: Utc::now(),
            end_time: None,
            status: SpanStatus::Running,
            repo_name: Some(REPO_NAME.to_string()),
            agent_name: None,
            failure_reasons: None,
        };

        Self {
            repo_span,
            agent_spans: HashMap::new(),
            artifacts: Vec::new(),
            active_agents: std::collections::HashSet::new(),
            finalized: false,
        }
    }

    /// Get the repo-level span ID.
    pub fn repo_span_id(&self) -> &str {
        &self.repo_span.span_id
    }

    /// Start a new agent-level span.
    pub fn start_agent(&mut self, agent_name: &str) -> ExecutionSpan {
        assert!(!self.finalized, "Cannot start agent after finalization");

        let span = ExecutionSpan {
            span_type: SpanType::Agent,
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: self.repo_span.span_id.clone(),
            start_time: Utc::now(),
            end_time: None,
            status: SpanStatus::Running,
            repo_name: Some(REPO_NAME.to_string()),
            agent_name: Some(agent_name.to_string()),
            failure_reasons: None,
        };

        let id = span.span_id.clone();
        self.active_agents.insert(id.clone());
        self.agent_spans.insert(id, span.clone());
        span
    }

    /// End an agent-level span.
    pub fn end_agent(
        &mut self,
        span_id: &str,
        status: SpanStatus,
        failure_reasons: Option<Vec<String>>,
    ) {
        if let Some(span) = self.agent_spans.get_mut(span_id) {
            span.end_time = Some(Utc::now());
            span.status = status;
            span.failure_reasons = failure_reasons;
            self.active_agents.remove(span_id);
        }
    }

    /// Attach an artifact to an agent span.
    pub fn add_artifact(&mut self, artifact: ExecutionArtifact) {
        assert!(
            self.agent_spans.contains_key(&artifact.agent_span_id),
            "Cannot attach artifact to unknown agent span: {}",
            artifact.agent_span_id
        );
        self.artifacts.push(artifact);
    }

    /// Finalize the execution and produce the result.
    pub fn finalize<T: Serialize>(mut self, data: Option<T>) -> ExecutionResult<T> {
        assert!(!self.finalized, "Already finalized");
        self.finalized = true;

        let mut validation_errors = Vec::new();

        // Enforce: at least one agent span
        if self.agent_spans.is_empty() {
            validation_errors
                .push("No agent-level spans were emitted. Execution is INVALID.".to_string());
        }

        // Force-end running agents
        let active: Vec<String> = self.active_agents.iter().cloned().collect();
        for id in &active {
            self.end_agent(
                id,
                SpanStatus::Failed,
                Some(vec!["Agent was still running at finalization".to_string()]),
            );
            if let Some(name) = self.agent_spans.get(id).and_then(|s| s.agent_name.as_ref()) {
                validation_errors.push(format!("Agent '{}' was still running at finalization", name));
            }
        }

        let any_failed = self.agent_spans.values().any(|s| s.status == SpanStatus::Failed);
        let valid = validation_errors.is_empty();

        self.repo_span.end_time = Some(Utc::now());
        self.repo_span.status = if !valid || any_failed {
            SpanStatus::Failed
        } else {
            SpanStatus::Completed
        };

        if self.repo_span.status == SpanStatus::Failed {
            self.repo_span.failure_reasons = if !validation_errors.is_empty() {
                Some(validation_errors.clone())
            } else {
                Some(
                    self.agent_spans
                        .values()
                        .filter(|s| s.status == SpanStatus::Failed)
                        .flat_map(|s| {
                            s.failure_reasons
                                .clone()
                                .unwrap_or_else(|| vec!["Agent failed".to_string()])
                        })
                        .collect(),
                )
            };
        }

        ExecutionResult {
            repo_span: self.repo_span,
            agent_spans: self.agent_spans.into_values().collect(),
            artifacts: self.artifacts,
            data: if valid { data } else { None },
            valid,
            validation_errors: if validation_errors.is_empty() {
                None
            } else {
                Some(validation_errors)
            },
        }
    }

    /// Finalize as failed with explicit reasons.
    pub fn finalize_failure(mut self, reasons: Vec<String>) -> ExecutionResult<()> {
        self.finalized = true;

        // End any running agents
        let active: Vec<String> = self.active_agents.iter().cloned().collect();
        for id in &active {
            self.end_agent(
                id,
                SpanStatus::Failed,
                Some(reasons.clone()),
            );
        }

        self.repo_span.end_time = Some(Utc::now());
        self.repo_span.status = SpanStatus::Failed;
        self.repo_span.failure_reasons = Some(reasons.clone());

        ExecutionResult {
            repo_span: self.repo_span,
            agent_spans: self.agent_spans.into_values().collect(),
            artifacts: self.artifacts,
            data: None,
            valid: false,
            validation_errors: Some(reasons),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::context::ExecutionContext;

    fn test_ctx() -> ExecutionContext {
        ExecutionContext {
            execution_id: "exec-test".to_string(),
            parent_span_id: "core-span-test".to_string(),
        }
    }

    #[test]
    fn test_span_lifecycle() {
        let ctx = test_ctx();
        let mut mgr = SpanManager::new(&ctx);

        let agent = mgr.start_agent("test-agent");
        assert_eq!(agent.status, SpanStatus::Running);
        assert_eq!(agent.agent_name.as_deref(), Some("test-agent"));

        mgr.end_agent(&agent.span_id, SpanStatus::Completed, None);

        let result = mgr.finalize::<()>(None);
        assert!(result.valid);
        assert_eq!(result.repo_span.status, SpanStatus::Completed);
        assert_eq!(result.agent_spans.len(), 1);
    }

    #[test]
    fn test_no_agent_spans_is_invalid() {
        let ctx = test_ctx();
        let mgr = SpanManager::new(&ctx);
        let result = mgr.finalize::<()>(None);
        assert!(!result.valid);
        assert!(result.validation_errors.is_some());
    }

    #[test]
    fn test_artifact_attachment() {
        let ctx = test_ctx();
        let mut mgr = SpanManager::new(&ctx);

        let agent = mgr.start_agent("producer");
        mgr.add_artifact(ExecutionArtifact {
            reference: "output.json".to_string(),
            artifact_type: "results".to_string(),
            agent_span_id: agent.span_id.clone(),
            metadata: None,
        });
        mgr.end_agent(&agent.span_id, SpanStatus::Completed, None);

        let result = mgr.finalize::<()>(None);
        assert!(result.valid);
        assert_eq!(result.artifacts.len(), 1);
        assert_eq!(result.artifacts[0].reference, "output.json");
    }

    #[test]
    #[should_panic(expected = "Cannot attach artifact to unknown agent span")]
    fn test_artifact_wrong_span_panics() {
        let ctx = test_ctx();
        let mut mgr = SpanManager::new(&ctx);
        mgr.start_agent("real-agent");
        mgr.add_artifact(ExecutionArtifact {
            reference: "bad.json".to_string(),
            artifact_type: "results".to_string(),
            agent_span_id: "nonexistent-span".to_string(),
            metadata: None,
        });
    }

    #[test]
    fn test_failure_propagation() {
        let ctx = test_ctx();
        let mut mgr = SpanManager::new(&ctx);

        let agent = mgr.start_agent("failing-agent");
        mgr.end_agent(
            &agent.span_id,
            SpanStatus::Failed,
            Some(vec!["Something went wrong".to_string()]),
        );

        let result = mgr.finalize::<()>(None);
        assert!(result.valid); // Valid because agent span exists, even though agent failed
        assert_eq!(result.repo_span.status, SpanStatus::Failed);
    }

    #[test]
    fn test_finalize_failure() {
        let ctx = test_ctx();
        let mut mgr = SpanManager::new(&ctx);
        mgr.start_agent("doomed-agent");

        let result = mgr.finalize_failure(vec!["fatal error".to_string()]);
        assert!(!result.valid);
        assert_eq!(result.repo_span.status, SpanStatus::Failed);
    }

    #[test]
    fn test_multiple_agents() {
        let ctx = test_ctx();
        let mut mgr = SpanManager::new(&ctx);

        let a1 = mgr.start_agent("agent-1");
        let a2 = mgr.start_agent("agent-2");
        let a3 = mgr.start_agent("agent-3");

        mgr.end_agent(&a1.span_id, SpanStatus::Completed, None);
        mgr.end_agent(&a2.span_id, SpanStatus::Completed, None);
        mgr.end_agent(&a3.span_id, SpanStatus::Completed, None);

        let result = mgr.finalize(Some("all done".to_string()));
        assert!(result.valid);
        assert_eq!(result.agent_spans.len(), 3);
        assert_eq!(result.data.as_deref(), Some("all done"));
    }

    #[test]
    fn test_spans_are_causally_ordered() {
        let ctx = test_ctx();
        let mut mgr = SpanManager::new(&ctx);

        let agent = mgr.start_agent("child");
        mgr.end_agent(&agent.span_id, SpanStatus::Completed, None);

        let result = mgr.finalize::<()>(None);

        // Repo span parents to core span
        assert_eq!(result.repo_span.parent_span_id, "core-span-test");
        // Agent span parents to repo span
        assert_eq!(
            result.agent_spans[0].parent_span_id,
            result.repo_span.span_id
        );
    }
}
