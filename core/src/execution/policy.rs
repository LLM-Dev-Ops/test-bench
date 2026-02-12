//! Policy gate types for the Agentics Foundational Execution Unit.
//!
//! Defines the declarative policy that classifies simulation intents
//! as mandatory, recommended, or optional execution dependencies.
//! When a mandatory intent is bypassed, an ExemptionRecord must be
//! produced as a first-class artifact in the execution graph.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// All recognized simulation intents.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SimulationIntent {
    Benchmark,
    Compare,
    Evaluate,
    Complete,
    Optimize,
    LoadRamp,
    Spike,
    Soak,
    ExtremeInput,
    Adversarial,
    RateLimitProbe,
    TimeoutBoundary,
    TokenLimit,
    ContextOverflow,
    MalformedRequest,
}

/// Gate disposition for a simulation intent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GateDisposition {
    Mandatory,
    Recommended,
    Optional,
}

/// A single policy rule binding an intent to a gate disposition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub intent: SimulationIntent,
    pub disposition: GateDisposition,
    pub rationale: String,
}

/// The complete execution policy. Declarative, JSON-serializable,
/// inspectable at rest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPolicy {
    pub policy_version: String,
    pub updated_at: DateTime<Utc>,
    pub description: String,
    pub rules: Vec<PolicyRule>,
    pub default_disposition: GateDisposition,
}

/// An exemption record. Created when a mandatory intent is explicitly
/// waived. This is a first-class record in the execution graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExemptionRecord {
    pub intent: SimulationIntent,
    pub reason: String,
    pub granted_by: String,
    pub granted_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    pub execution_id: String,
    pub parent_span_id: String,
}

/// Exemption request from callers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExemptionRequest {
    pub intent: SimulationIntent,
    pub reason: String,
    pub granted_by: String,
}

impl ExecutionPolicy {
    /// Look up the disposition for a given intent.
    pub fn disposition_for(&self, intent: &SimulationIntent) -> &GateDisposition {
        self.rules
            .iter()
            .find(|r| &r.intent == intent)
            .map(|r| &r.disposition)
            .unwrap_or(&self.default_disposition)
    }

    /// Returns true if the intent is mandatory.
    pub fn is_mandatory(&self, intent: &SimulationIntent) -> bool {
        matches!(self.disposition_for(intent), GateDisposition::Mandatory)
    }
}

/// Build the default execution policy.
pub fn default_policy() -> ExecutionPolicy {
    ExecutionPolicy {
        policy_version: "1.0.0".to_string(),
        updated_at: Utc::now(),
        description: "Default policy: benchmark, compare, evaluate, and all stress test intents \
                      are mandatory execution dependencies."
            .to_string(),
        rules: vec![
            PolicyRule { intent: SimulationIntent::Benchmark, disposition: GateDisposition::Mandatory, rationale: "Benchmark results must appear in execution graph for auditability".to_string() },
            PolicyRule { intent: SimulationIntent::Compare, disposition: GateDisposition::Mandatory, rationale: "Model comparisons are decision-critical and must be traced".to_string() },
            PolicyRule { intent: SimulationIntent::Evaluate, disposition: GateDisposition::Mandatory, rationale: "Evaluations are quality gates and must be traced".to_string() },
            PolicyRule { intent: SimulationIntent::Complete, disposition: GateDisposition::Recommended, rationale: "Completions should be traced but may be used for ad-hoc exploration".to_string() },
            PolicyRule { intent: SimulationIntent::Optimize, disposition: GateDisposition::Recommended, rationale: "Optimization is advisory; should be traced but not hard-gated".to_string() },
            PolicyRule { intent: SimulationIntent::LoadRamp, disposition: GateDisposition::Mandatory, rationale: "Load ramp results are critical for capacity planning".to_string() },
            PolicyRule { intent: SimulationIntent::Spike, disposition: GateDisposition::Mandatory, rationale: "Spike test results inform resilience decisions".to_string() },
            PolicyRule { intent: SimulationIntent::Soak, disposition: GateDisposition::Mandatory, rationale: "Soak test results inform stability decisions".to_string() },
            PolicyRule { intent: SimulationIntent::ExtremeInput, disposition: GateDisposition::Mandatory, rationale: "Extreme input handling must be auditable".to_string() },
            PolicyRule { intent: SimulationIntent::Adversarial, disposition: GateDisposition::Mandatory, rationale: "Adversarial test results are security-critical".to_string() },
            PolicyRule { intent: SimulationIntent::RateLimitProbe, disposition: GateDisposition::Mandatory, rationale: "Rate limit probing affects operational parameters".to_string() },
            PolicyRule { intent: SimulationIntent::TimeoutBoundary, disposition: GateDisposition::Mandatory, rationale: "Timeout boundary results set SLA parameters".to_string() },
            PolicyRule { intent: SimulationIntent::TokenLimit, disposition: GateDisposition::Mandatory, rationale: "Token limit results inform context window policies".to_string() },
            PolicyRule { intent: SimulationIntent::ContextOverflow, disposition: GateDisposition::Mandatory, rationale: "Context overflow results are input validation critical".to_string() },
            PolicyRule { intent: SimulationIntent::MalformedRequest, disposition: GateDisposition::Mandatory, rationale: "Error handling behavior must be auditable".to_string() },
        ],
        default_disposition: GateDisposition::Recommended,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mandatory_intent_detected() {
        let policy = default_policy();
        assert!(policy.is_mandatory(&SimulationIntent::Benchmark));
        assert!(policy.is_mandatory(&SimulationIntent::Compare));
        assert!(policy.is_mandatory(&SimulationIntent::Evaluate));
        assert!(policy.is_mandatory(&SimulationIntent::LoadRamp));
        assert!(policy.is_mandatory(&SimulationIntent::Adversarial));
    }

    #[test]
    fn test_recommended_intent_detected() {
        let policy = default_policy();
        assert!(!policy.is_mandatory(&SimulationIntent::Complete));
        assert!(!policy.is_mandatory(&SimulationIntent::Optimize));
        assert_eq!(
            policy.disposition_for(&SimulationIntent::Complete),
            &GateDisposition::Recommended
        );
    }

    #[test]
    fn test_policy_roundtrip_json() {
        let policy = default_policy();
        let json = serde_json::to_string(&policy).unwrap();
        let parsed: ExecutionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.policy_version, "1.0.0");
        assert_eq!(parsed.rules.len(), 15);
    }

    #[test]
    fn test_exemption_record_serialization() {
        let record = ExemptionRecord {
            intent: SimulationIntent::Benchmark,
            reason: "Emergency hotfix".to_string(),
            granted_by: "operator:alice".to_string(),
            granted_at: Utc::now(),
            expires_at: None,
            execution_id: "EXEMPTED".to_string(),
            parent_span_id: "EXEMPTED".to_string(),
        };
        let json = serde_json::to_string(&record).unwrap();
        let parsed: ExemptionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.intent, SimulationIntent::Benchmark);
        assert_eq!(parsed.reason, "Emergency hotfix");
    }

    #[test]
    fn test_simulation_intent_serde() {
        let intent = SimulationIntent::RateLimitProbe;
        let json = serde_json::to_string(&intent).unwrap();
        assert_eq!(json, "\"rate_limit_probe\"");
        let parsed: SimulationIntent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, SimulationIntent::RateLimitProbe);
    }
}
