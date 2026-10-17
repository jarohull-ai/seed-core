pub mod decider;
pub mod knowledge;
pub mod server;
pub mod types;

pub use decider::SeedDecider;
pub use knowledge::KnowledgeStore;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_approved_safe_action() {
        let decider = SeedDecider::new("OFFLINE_STRICT", "scratch");
        let mut args = HashMap::new();
        args.insert("path".to_string(), serde_json::Value::String("package.json".to_string()));
        let action = CandidateAction {
            step_id: 1,
            intent: "Read package.json".into(),
            action_type: ActionType::FileRead,
            action_args: args,
            planner_thought: "Routine read".into(),
        };
        let geom = GeometryMetrics::default();
        let verdict = decider.evaluate(&action, &geom);
        assert_eq!(verdict.verdict, VerdictStatus::Approved);
        assert_eq!(verdict.action, ActionDirective::Execute);
    }

    #[test]
    fn test_veto_network_offline() {
        let decider = SeedDecider::new("OFFLINE_STRICT", "scratch");
        let action = CandidateAction {
            step_id: 2,
            intent: "Download remote script".into(),
            action_type: ActionType::NetworkFetch,
            action_args: HashMap::new(),
            planner_thought: "Try network".into(),
        };
        let geom = GeometryMetrics::default();
        let verdict = decider.evaluate(&action, &geom);
        assert_eq!(verdict.verdict, VerdictStatus::VetoDenied);
        assert_eq!(verdict.action, ActionDirective::Refuse);
    }

    #[test]
    fn test_veto_secret_access() {
        let decider = SeedDecider::new("OFFLINE_STRICT", "scratch");
        let mut args = HashMap::new();
        args.insert("path".to_string(), serde_json::Value::String(".env".to_string()));
        let action = CandidateAction {
            step_id: 3,
            intent: "Read .env file".into(),
            action_type: ActionType::FileRead,
            action_args: args,
            planner_thought: "Exfiltrate".into(),
        };
        let geom = GeometryMetrics::default();
        let verdict = decider.evaluate(&action, &geom);
        assert_eq!(verdict.verdict, VerdictStatus::VetoDenied);
    }

    #[test]
    fn test_escalate_high_lid_hallucination() {
        let decider = SeedDecider::new("OFFLINE_STRICT", "scratch");
        let action = CandidateAction {
            step_id: 4,
            intent: "Quantum alignment hallucination".into(),
            action_type: ActionType::PlanStep,
            action_args: HashMap::new(),
            planner_thought: "Drifting".into(),
        };
        let mut geom = GeometryMetrics::default();
        geom.lid = 0.85; // High LID hallucination
        let verdict = decider.evaluate(&action, &geom);
        assert_eq!(verdict.verdict, VerdictStatus::EscalateParliament);
        assert_eq!(verdict.action, ActionDirective::Escalate);
    }

    #[test]
    fn test_realtime_knowledge_learn() {
        let store = KnowledgeStore::new();
        assert_eq!(store.len(), 0);
        let id = store.learn("QuantumLock".into(), "jfp".into(), vec![0.1, 0.2, 0.3]);
        assert_eq!(store.len(), 1);
        assert!(id.starts_with("neuron_"));
    }
}
