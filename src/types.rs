use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionType {
    FileRead,
    FileWrite,
    CommandExec,
    NetworkFetch,
    PlanStep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateAction {
    pub step_id: u64,
    pub intent: String,
    pub action_type: ActionType,
    pub action_args: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub planner_thought: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryMetrics {
    pub lid: f32,                     // Local Intrinsic Dimensionality [0.0 - 1.0] (> 0.70 = hallucination cloud)
    pub cos_gap: f32,                 // Separation gap between top-1 and top-2
    pub etm: f32,                     // Evidence Trust Metric [0.0 - 1.0]
    pub activated_domains: Vec<String>,
    pub activated_ages: Vec<u32>,     // Age in days
}

impl Default for GeometryMetrics {
    fn default() -> Self {
        Self {
            lid: 0.25,
            cos_gap: 0.45,
            etm: 0.90,
            activated_domains: vec!["code".into(), "jfp".into()],
            activated_ages: vec![1, 2, 5],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerdictStatus {
    Approved,
    VetoDenied,
    EscalateParliament,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionDirective {
    Execute,
    Refuse,
    Escalate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConfidenceLevel {
    Low,
    Mid,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeciderVerdict {
    pub step_id: u64,
    pub verdict: VerdictStatus,
    pub action: ActionDirective,
    pub confidence: ConfidenceLevel,
    pub reason: String,
    pub latency_us: u64,              // Microseconds
    pub audit_hash: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neuron {
    pub id: String,
    pub concept: String,
    pub domain: String,
    pub vector: Vec<f32>,
    pub age_days: u32,
    pub uses: u64,
    pub created_at: String,
}
