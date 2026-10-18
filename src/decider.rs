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
    ssrf_patterns: Vec<Regex>,
    escalate_patterns: Vec<Regex>,
    dangerous_payload_patterns: Vec<Regex>,
    system_dir_patterns: Vec<Regex>,
}

impl SeedDecider {
    pub fn new(profile_mode: &str, write_target: &str) -> Self {
        let destructive = vec![
            Regex::new(r"\brm\s+-[rf]{1,2}\s+[/~]").unwrap(),
            Regex::new(r"\brm\s+-[rf]{1,2}\s+\*").unwrap(),
            Regex::new(r"\bmkfs\b").unwrap(),
            Regex::new(r"\bdd\s+if=").unwrap(),
            Regex::new(r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;").unwrap(), // forkbomb
            Regex::new(r"\bchmod\s+777\b").unwrap(),
            Regex::new(r"\bchown\b").unwrap(),
            Regex::new(r"\bsudo\b").unwrap(),
            Regex::new(r"\bsu\b").unwrap(),
            Regex::new(r"/dev/sd[a-z]").unwrap(),
            Regex::new(r"/dev/nvme").unwrap(),
            Regex::new(r">\s*/dev/null\s*2>&1\s*&\s*disown").unwrap(),
            Regex::new(r"\b(shutdown|reboot|poweroff|init\s+0)\b").unwrap(),
        ];

        let secret = vec![
            Regex::new(r"\.env\b").unwrap(),
            Regex::new(r"id_rsa").unwrap(),
            Regex::new(r"id_ed25519").unwrap(),
            Regex::new(r"/\.ssh/").unwrap(),
            Regex::new(r"/\.aws/").unwrap(),
            Regex::new(r"/\.gnupg/").unwrap(),
            Regex::new(r"/\.config/gh/").unwrap(),
            Regex::new(r"/etc/shadow").unwrap(),
            Regex::new(r"/etc/passwd").unwrap(),
            Regex::new(r"/etc/sudoers").unwrap(),
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

        let ssrf = vec![
            Regex::new(r"\b(localhost|127\.\d+\.\d+\.\d+|0\.0\.0\.0|169\.254\.169\.254)\b")
                .unwrap(),
            Regex::new(
                r"\b(10\.\d+\.\d+\.\d+|192\.168\.\d+\.\d+|172\.(1[6-9]|2\d|3[01])\.\d+\.\d+)\b",
            )
            .unwrap(),
            Regex::new(r"\[::1\]").unwrap(),
        ];

        let escalate = vec![
            Regex::new(r"\bgit\s+push.*--force\b").unwrap(),
            Regex::new(r"\b(npm|cargo)\s+publish\b").unwrap(),
            Regex::new(r"\bapt(-get)?\s+(install|remove)\b").unwrap(),
            Regex::new(r"\b(systemctl|service)\s+(restart|stop)\b").unwrap(),
            Regex::new(r"\bpkill\b").unwrap(),
            Regex::new(r"\bkillall\b").unwrap(),
        ];

        let dangerous_payload = vec![Regex::new(r"\.(exe|sh|bin|elf|appimage)$").unwrap()];

        let system_dir = vec![
            Regex::new(r"^/etc(/|$)").unwrap(),
            Regex::new(r"^/boot(/|$)").unwrap(),
            Regex::new(r"^/root(/|$)").unwrap(),
            Regex::new(r"^/sys(/|$)").unwrap(),
            Regex::new(r"^/proc(/|$)").unwrap(),
            Regex::new(r"^/dev(/|$)").unwrap(),
            Regex::new(r"^/usr(/|$)").unwrap(),
        ];

        Self {
            profile_mode: profile_mode.to_string(),
            write_target: write_target.to_string(),
            audit_chain: Arc::new(RwLock::new(vec!["GENESIS_SEED_JFP_2026_RUST".to_string()])),
            recent_history: Arc::new(RwLock::new(Vec::new())),
            destructive_patterns: destructive,
            secret_patterns: secret,
            network_patterns: network,
            ssrf_patterns: ssrf,
            escalate_patterns: escalate,
            dangerous_payload_patterns: dangerous_payload,
            system_dir_patterns: system_dir,
        }
    }

    pub fn evaluate(&self, candidate: &CandidateAction, geom: &GeometryMetrics) -> DeciderVerdict {
        let start = Instant::now();

        let cmd = candidate
            .action_args
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let target_path = candidate
            .action_args
            .get("path")
            .or_else(|| candidate.action_args.get("target"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let url = candidate
            .action_args
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 1. HARD CONSTITUTIONAL RULE: Secret & Credential Protection (All profiles)
        for pat in &self.secret_patterns {
            let matches_path = pat.is_match(target_path) && !target_path.ends_with(".env.example");
            let matches_cmd = pat.is_match(cmd) && !cmd.contains(".env.example");
            let matches_url = pat.is_match(url);
            if matches_path || matches_cmd || matches_url {
                return self.finalize(
                    candidate.step_id,
                    VerdictStatus::VetoDenied,
                    ActionDirective::Refuse,
                    ConfidenceLevel::High,
                    &format!(
                        "VETO: access or exfiltration of protected secret [{}] strictly denied",
                        pat
                    ),
                    start,
                );
            }
        }

        // 2. HARD CONSTITUTIONAL RULE: Destructive System Mutations (All profiles)
        if candidate.action_type == ActionType::CommandExec {
            for pat in &self.destructive_patterns {
                if pat.is_match(cmd) {
                    return self.finalize(
                        candidate.step_id,
                        VerdictStatus::VetoDenied,
                        ActionDirective::Refuse,
                        ConfidenceLevel::High,
                        &format!(
                            "VETO: destructive mutation vector [{}] strictly prohibited on host",
                            pat
                        ),
                        start,
                    );
                }
            }
        }

        // 3. HARD CONSTITUTIONAL RULE: Filesystem System Directory Protection
        if candidate.action_type == ActionType::FileWrite {
            if target_path.contains("..") {
                return self.finalize(
                    candidate.step_id,
                    VerdictStatus::VetoDenied,
                    ActionDirective::Refuse,
                    ConfidenceLevel::High,
                    "VETO: path traversal escape detected",
                    start,
                );
            }
            for sys_pat in &self.system_dir_patterns {
                if sys_pat.is_match(target_path) {
                    return self.finalize(
                        candidate.step_id,
                        VerdictStatus::VetoDenied,
                        ActionDirective::Refuse,
                        ConfidenceLevel::High,
                        &format!("VETO: system directory escape [{}] prohibited", sys_pat),
                        start,
                    );
                }
            }
            if self.profile_mode == "OFFLINE_STRICT"
                && self.write_target == "scratch"
                && !target_path.starts_with("scratch/")
                && !target_path.starts_with("./scratch/")
                && !target_path.starts_with("dist/")
            {
                return self.finalize(
                    candidate.step_id,
                    VerdictStatus::VetoDenied,
                    ActionDirective::Refuse,
                    ConfidenceLevel::High,
                    &format!(
                        "VETO: write target restricted to [{}] sandbox in OFFLINE_STRICT",
                        self.write_target
                    ),
                    start,
                );
            }
        }

        // 4. BROWSER ACTION CONSTITUTIONAL RULES
        if candidate.action_type == ActionType::BrowserAction {
            if url.starts_with("file://")
                || url.starts_with("chrome://")
                || url.starts_with("javascript:")
            {
                return self.finalize(
                    candidate.step_id,
                    VerdictStatus::VetoDenied,
                    ActionDirective::Refuse,
                    ConfidenceLevel::High,
                    "VETO: browser internal/local protocol blocked",
                    start,
                );
            }
            for ssrf in &self.ssrf_patterns {
                if ssrf.is_match(url) {
                    return self.finalize(
                        candidate.step_id,
                        VerdictStatus::VetoDenied,
                        ActionDirective::Refuse,
                        ConfidenceLevel::High,
                        &format!(
                            "VETO: browser SSRF or private network vector [{}] blocked",
                            ssrf
                        ),
                        start,
                    );
                }
            }
            for payload in &self.dangerous_payload_patterns {
                if payload.is_match(url) {
                    return self.finalize(
                        candidate.step_id,
                        VerdictStatus::VetoDenied,
                        ActionDirective::Refuse,
                        ConfidenceLevel::High,
                        &format!(
                            "VETO: executable payload download [{}] prohibited in browser",
                            payload
                        ),
                        start,
                    );
                }
            }
        }

        // 5. NETWORK ISOLATION vs HOST_SOVEREIGN POLICIES
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
                for pat in &self.network_patterns {
                    if pat.is_match(cmd) {
                        return self.finalize(
                            candidate.step_id,
                            VerdictStatus::VetoDenied,
                            ActionDirective::Refuse,
                            ConfidenceLevel::High,
                            &format!(
                                "VETO: outbound network pattern [{}] detected in command",
                                pat
                            ),
                            start,
                        );
                    }
                }
            }
        } else if self.profile_mode == "HOST_SOVEREIGN" {
            // In HOST_SOVEREIGN mode: Block SSRF / private targets in network calls
            if candidate.action_type == ActionType::NetworkFetch {
                for ssrf in &self.ssrf_patterns {
                    if ssrf.is_match(url) || ssrf.is_match(target_path) {
                        return self.finalize(
                            candidate.step_id,
                            VerdictStatus::VetoDenied,
                            ActionDirective::Refuse,
                            ConfidenceLevel::High,
                            &format!("VETO: network SSRF / private target [{}] blocked", ssrf),
                            start,
                        );
                    }
                }
            }
            // Check for high-impact commands that require human escalation
            if candidate.action_type == ActionType::CommandExec {
                for ssrf in &self.ssrf_patterns {
                    if ssrf.is_match(cmd) {
                        return self.finalize(
                            candidate.step_id,
                            VerdictStatus::VetoDenied,
                            ActionDirective::Refuse,
                            ConfidenceLevel::High,
                            &format!("VETO: command attempts private network attack [{}]", ssrf),
                            start,
                        );
                    }
                }
                for esc in &self.escalate_patterns {
                    if esc.is_match(cmd) {
                        return self.finalize(
                            candidate.step_id,
                            VerdictStatus::EscalateParliament,
                            ActionDirective::Escalate,
                            ConfidenceLevel::High,
                            &format!(
                                "ESCALATE: high-impact host mutation [{}] requires human approval",
                                esc
                            ),
                            start,
                        );
                    }
                }
            }
        }

        // 6. MATHEMATICAL CONSTITUTIONAL RULE: Generic Hub Dominance Veto
        let learned_count = geom
            .activated_domains
            .iter()
            .filter(|d| *d == "learned")
            .count();
        if learned_count >= 3 {
            return self.finalize(
                candidate.step_id,
                VerdictStatus::VetoDenied,
                ActionDirective::Refuse,
                ConfidenceLevel::Mid,
                &format!(
                    "VETO: learned_hub_dominance={}/5 unanchored extrapolation",
                    learned_count
                ),
                start,
            );
        }

        // 7. MATHEMATICAL CONSTITUTIONAL RULE: Geometric Surprise / High LID (Hallucination Cloud)
        if geom.lid > 0.70 {
            return self.finalize(
                candidate.step_id,
                VerdictStatus::EscalateParliament,
                ActionDirective::Escalate,
                ConfidenceLevel::Mid,
                &format!(
                    "ESCALATE: geometric_surprise lid={:.2}>0.70 (hallucination cloud)",
                    geom.lid
                ),
                start,
            );
        }

        // 8. MATHEMATICAL CONSTITUTIONAL RULE: Staleness Veto
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
                &format!(
                    "ESCALATE: knowledge staleness median_age={:.0}d exceeds threshold",
                    median_age
                ),
                start,
            );
        }

        // 9. ADAPTIVE LOOP DETECTION: Streak-Guard Repetitive Failure Veto
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
                    &format!(
                        "VETO: repetitive failure loop detected ({} repeated attempts)",
                        recent_fails + 1
                    ),
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
        self.recent_history
            .write()
            .push((reason.to_string(), verdict.clone()));

        // Cryptographic Tamper-Evident SHA-256 Proof Chain
        let mut chain = self.audit_chain.write();
        let prev_hash = chain.last().cloned().unwrap_or_default();
        let payload = format!(
            "{}:{:?}:{:?}:{}:{}",
            step_id, verdict, action, reason, prev_hash
        );
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
