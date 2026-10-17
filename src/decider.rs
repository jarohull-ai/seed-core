use crate::types::{
    ActionDirective, ActionType, CandidateAction, ConfidenceLevel, DeciderVerdict, GeometryMetrics,
    VerdictStatus,
};
use parking_lot::RwLock;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Instant;

pub struct SeedDecider {
    profile_mode: String,
    write_target: String,
    audit_chain: Arc<RwLock<Vec<String>>>,
    recent_history: Arc<RwLock<Vec<(String, VerdictStatus)>>>,
    destructive_patterns: Vec<Regex>,
    secret_patterns: Vec<Regex>,
    network_patterns: Vec<Regex>,
}

impl SeedDecider {
    pub fn new(profile_mode: &str, write_target: &str) -> Self {
        let destructive = vec![
            Regex::new(r"\brm\s+-[rf]{1,2}\s+[/~]").unwrap(),
            Regex::new(r"\bmkfs\b").unwrap(),
            Regex::new(r"\bdd\s+if=").unwrap(),
            Regex::new(r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;").unwrap(), // forkbomb
            Regex::new(r"\bchmod\s+777\b").unwrap(),
            Regex::new(r"\bchown\b").unwrap(),
            Regex::new(r"\bsudo\b").unwrap(),
            Regex::new(r"/dev/sd[a-z]").unwrap(),
            Regex::new(r">\s*/dev/null\s*2>&1\s*&\s*disown").unwrap(),
        ];

        let secret = vec![
            Regex::new(r"\.env\b").unwrap(),
            Regex::new(r"id_rsa").unwrap(),
            Regex::new(r"id_ed25519").unwrap(),
            Regex::new(r"/\.ssh/").unwrap(),
            Regex::new(r"/etc/shadow").unwrap(),
            Regex::new(r"/etc/passwd").unwrap(),
            Regex::new(r"api[-_]?key").unwrap(),
            Regex::new(r"secret[-_]?token").unwrap(),
        ];

        let network = vec![
            Regex::new(r"\bcurl\b").unwrap(),
            Regex::new(r"\bwget\b").unwrap(),
            Regex::new(r"\bnc\b").unwrap(),
            Regex::new(r"\bnetcat\b").unwrap(),
            Regex::new(r"/dev/tcp/").unwrap(),
            Regex::new(r"/dev/udp/").unwrap(),
            Regex::new(r"\bssh\b").unwrap(),
            Regex::new(r"\bscp\b").unwrap(),
            Regex::new(r"\brsync\b").unwrap(),
        ];

        Self {
            profile_mode: profile_mode.to_string(),
            write_target: write_target.to_string(),
            audit_chain: Arc::new(RwLock::new(vec!["GENESIS_SEED_JFP_2026_RUST".to_string()])),
            recent_history: Arc::new(RwLock::new(Vec::new())),
            destructive_patterns: destructive,
            secret_patterns: secret,
            network_patterns: network,
        }
    }

    pub fn evaluate(&self, candidate: &CandidateAction, geom: &GeometryMetrics) -> DeciderVerdict {
        let start = Instant::now();

        // 1. HARD CONSTITUTIONAL RULE: Network Isolation in OFFLINE_STRICT mode
        if self.profile_mode == "OFFLINE_STRICT" {
            if candidate.action_type == ActionType::NetworkFetch {
                return self.finalize(
                    candidate.step_id,
                    VerdictStatus::VetoDenied,
                    ActionDirective::Refuse,
                    ConfidenceLevel::High,
                    "VETO: network_fetch blocked in OFFLINE_STRICT mode",
                    start,
                );
            }
            if candidate.action_type == ActionType::CommandExec {
                let cmd = candidate
                    .action_args
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                for pat in &self.network_patterns {
                    if pat.is_match(cmd) {
                        return self.finalize(
                            candidate.step_id,
                            VerdictStatus::VetoDenied,
                            ActionDirective::Refuse,
                            ConfidenceLevel::High,
                            &format!("VETO: outbound network pattern [{}] detected in command", pat),
                            start,
                        );
                    }
                }
            }
        }

        // 2. HARD CONSTITUTIONAL RULE: Secret & Credential Protection
        let target_path = candidate
            .action_args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let cmd = candidate
            .action_args
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        for pat in &self.secret_patterns {
            if (pat.is_match(target_path) || pat.is_match(cmd)) && !target_path.ends_with(".env.example") {
                return self.finalize(
                    candidate.step_id,
                    VerdictStatus::VetoDenied,
                    ActionDirective::Refuse,
                    ConfidenceLevel::High,
                    &format!("VETO: access to protected credential/secret vector [{}] denied", pat),
                    start,
                );
            }
        }

        // 3. HARD CONSTITUTIONAL RULE: Filesystem Traversal & Write Target Confinement
        if candidate.action_type == ActionType::FileWrite {
            if target_path.contains("..") || target_path.starts_with("/etc") || target_path.starts_with("/root") {
                return self.finalize(
                    candidate.step_id,
                    VerdictStatus::VetoDenied,
                    ActionDirective::Refuse,
                    ConfidenceLevel::High,
                    "VETO: path traversal or system directory escape detected",
                    start,
                );
            }
            if self.write_target == "scratch"
                && !target_path.starts_with("scratch/")
                && !target_path.starts_with("./scratch/")
                && !target_path.starts_with("dist/")
            {
                return self.finalize(
                    candidate.step_id,
                    VerdictStatus::VetoDenied,
                    ActionDirective::Refuse,
                    ConfidenceLevel::High,
                    &format!("VETO: write target restricted to [{}] sandbox", self.write_target),
                    start,
                );
            }
        }

        // 4. HARD CONSTITUTIONAL RULE: Destructive System Mutations
        if candidate.action_type == ActionType::CommandExec {
            for pat in &self.destructive_patterns {
                if pat.is_match(cmd) {
                    return self.finalize(
                        candidate.step_id,
                        VerdictStatus::VetoDenied,
                        ActionDirective::Refuse,
                        ConfidenceLevel::High,
                        &format!("VETO: destructive mutation vector [{}] strictly prohibited", pat),
                        start,
                    );
                }
            }
        }

        // 5. MATHEMATICAL CONSTITUTIONAL RULE: Generic Hub Dominance Veto
        let learned_count = geom.activated_domains.iter().filter(|d| *d == "learned").count();
        if learned_count >= 3 {
            return self.finalize(
                candidate.step_id,
                VerdictStatus::VetoDenied,
                ActionDirective::Refuse,
                ConfidenceLevel::Mid,
                &format!("VETO: learned_hub_dominance={}/5 unanchored extrapolation", learned_count),
                start,
            );
        }

        // 6. MATHEMATICAL CONSTITUTIONAL RULE: Geometric Surprise / High LID (Hallucination Cloud)
        if geom.lid > 0.70 {
            return self.finalize(
                candidate.step_id,
                VerdictStatus::EscalateParliament,
                ActionDirective::Escalate,
                ConfidenceLevel::Mid,
                &format!("ESCALATE: geometric_surprise lid={:.2}>0.70 (hallucination cloud)", geom.lid),
                start,
            );
        }

        // 7. MATHEMATICAL CONSTITUTIONAL RULE: Staleness Veto
        let median_age = if !geom.activated_ages.is_empty() {
            let mut sorted = geom.activated_ages.clone();
            sorted.sort_unstable();
            sorted[sorted.len() / 2] as f32
        } else {
            0.0
        };
        if median_age > 30.0 && geom.etm < 0.65 {
            return self.finalize(
                candidate.step_id,
                VerdictStatus::EscalateParliament,
                ActionDirective::Escalate,
                ConfidenceLevel::Mid,
                &format!("ESCALATE: knowledge staleness median_age={:.0}d exceeds threshold", median_age),
                start,
            );
        }

        // 8. ADAPTIVE LOOP DETECTION: Streak-Guard Repetitive Failure Veto
        {
            let hist = self.recent_history.read();
            let recent_fails = hist
                .iter()
                .rev()
                .take(4)
                .filter(|(intent, v)| intent == &candidate.intent && *v != VerdictStatus::Approved)
                .count();
            if recent_fails >= 2 {
                return self.finalize(
                    candidate.step_id,
                    VerdictStatus::VetoDenied,
                    ActionDirective::Refuse,
                    ConfidenceLevel::High,
                    &format!("VETO: repetitive failure loop detected ({} repeated attempts)", recent_fails + 1),
                    start,
                );
            }
        }

        // Passed all gates: APPROVED
        let confidence = if geom.etm >= 0.75 && geom.lid <= 0.40 {
            ConfidenceLevel::High
        } else {
            ConfidenceLevel::Mid
        };
        let reason = format!(
            "VALID: etm={:.2}, lid={:.2}, gap={:.2}, domain_conformance=100%",
            geom.etm, geom.lid, geom.cos_gap
        );

        self.finalize(
            candidate.step_id,
            VerdictStatus::Approved,
            ActionDirective::Execute,
            confidence,
            &reason,
            start,
        )
    }

    fn finalize(
        &self,
        step_id: u64,
        verdict: VerdictStatus,
        action: ActionDirective,
        confidence: ConfidenceLevel,
        reason: &str,
        start: Instant,
    ) -> DeciderVerdict {
        let latency_us = start.elapsed().as_micros() as u64;

        // Record history for streak-guard
        self.recent_history.write().push((reason.to_string(), verdict.clone()));

        // Cryptographic Tamper-Evident SHA-256 Proof Chain
        let mut chain = self.audit_chain.write();
        let prev_hash = chain.last().cloned().unwrap_or_default();
        let payload = format!("{}:{:?}:{:?}:{}:{}", step_id, verdict, action, reason, prev_hash);
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        let audit_hash = format!("{:x}", hasher.finalize());
        chain.push(audit_hash.clone());

        DeciderVerdict {
            step_id,
            verdict,
            action,
            confidence,
            reason: reason.to_string(),
            latency_us,
            audit_hash,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn audit_chain_len(&self) -> usize {
        self.audit_chain.read().len()
    }
}
