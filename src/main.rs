use clap::{Parser, Subcommand};
use seed_core::decider::SeedDecider;
use seed_core::knowledge::KnowledgeStore;
use seed_core::server::run_ipc_server;
use seed_core::types::{ActionType, CandidateAction, GeometryMetrics, VerdictStatus};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "seed-core")]
#[command(about = "Deterministic Decision Engine, Real-Time Episodic Memory & JFP Gatekeeper in Rust")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate a candidate action step through the deterministic JFP Decider
    Eval {
        /// Intent description
        #[arg(short, long)]
        intent: String,

        /// Action type: file_read, file_write, command_exec, network_fetch, plan_step
        #[arg(short, long)]
        action: String,

        /// Target path or command argument
        #[arg(short, long)]
        target: Option<String>,

        /// Local Intrinsic Dimensionality [0.0 - 1.0]
        #[arg(long, default_value_t = 0.25)]
        lid: f32,

        /// Evidence Trust Metric [0.0 - 1.0]
        #[arg(long, default_value_t = 0.90)]
        etm: f32,
    },
    /// Run background IPC daemon for real-time agent evaluation (Viper Chat connector)
    Serve {
        /// Unix domain socket path
        #[arg(short, long, default_value = "/tmp/seed-core.sock")]
        socket: String,

        /// Optional TCP address (e.g. 127.0.0.1:4040)
        #[arg(short, long)]
        tcp: Option<String>,
    },
    /// Add knowledge neuron to real-time episodic memory
    Learn {
        /// Concept name / title
        #[arg(short, long)]
        concept: String,

        /// Domain tag (e.g. code, jfp, security, osint)
        #[arg(short, long)]
        domain: String,
    },
    /// Print system status and verified audit chain length
    Status,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let decider = Arc::new(SeedDecider::new("OFFLINE_STRICT", "scratch"));
    let knowledge = Arc::new(KnowledgeStore::new());

    match cli.command {
        Commands::Eval { intent, action, target, lid, etm } => {
            let act_type = match action.to_lowercase().as_str() {
                "file_read" => ActionType::FileRead,
                "file_write" => ActionType::FileWrite,
                "command_exec" => ActionType::CommandExec,
                "network_fetch" => ActionType::NetworkFetch,
                _ => ActionType::PlanStep,
            };

            let mut args = HashMap::new();
            if let Some(t) = target {
                if act_type == ActionType::CommandExec {
                    args.insert("command".to_string(), serde_json::Value::String(t));
                } else {
                    args.insert("path".to_string(), serde_json::Value::String(t));
                }
            }

            let candidate = CandidateAction {
                step_id: 1,
                intent,
                action_type: act_type,
                action_args: args,
                planner_thought: "CLI interactive evaluation".into(),
            };

            let geom = GeometryMetrics {
                lid,
                cos_gap: 0.45,
                etm,
                activated_domains: vec!["code".into(), "jfp".into()],
                activated_ages: vec![1, 2],
            };

            let verdict = decider.evaluate(&candidate, &geom);

            println!("\n🧠 SEED-CORE DETERMINISTIC VERDICT");
            println!("================================================================================");
            let icon = match verdict.verdict {
                VerdictStatus::Approved => "✅ APPROVED",
                VerdictStatus::VetoDenied => "🚫 VETO_DENIED",
                VerdictStatus::EscalateParliament => "⚠️  ESCALATE_PARLIAMENT",
            };
            println!("⚖️  Status:         {}", icon);
            println!("🎯 Directive:      {:?}", verdict.action);
            println!("🛡️  Confidence:     {:?}", verdict.confidence);
            println!("⚡ Latency:        {} µs ({:.3} ms)", verdict.latency_us, verdict.latency_us as f64 / 1000.0);
            println!("💬 Reason:         {}", verdict.reason);
            println!("🔗 SHA-256 Proof:  {}", verdict.audit_hash);
            println!("================================================================================\n");
        }
        Commands::Serve { socket, tcp } => {
            println!("🚀 Launching SEED-Core IPC Engine (Rust / Tokio)...");
            run_ipc_server(&socket, tcp.as_deref(), decider, knowledge).await?;
        }
        Commands::Learn { concept, domain } => {
            // Mock synthetic 384-dim normalized vector for real-time demonstration
            let vec: Vec<f32> = (0..384).map(|i| ((i as f32) * 0.01).sin()).collect();
            let neuron_id = knowledge.learn(concept.clone(), domain.clone(), vec);
            println!("🧠 [seed-core] Real-time knowledge crystallized!");
            println!("   - Neuron ID: {}", neuron_id);
            println!("   - Concept:   {}", concept);
            println!("   - Domain:    {}", domain);
            println!("   - Total Memory Neurons: {}", knowledge.len());
        }
        Commands::Status => {
            println!("\n⚡ SEED-Core Engine Status: OPERATIONAL");
            println!("   - Profile:        OFFLINE_STRICT (viper-boxd.profile.v0)");
            println!("   - Target:         scratch confinement");
            println!("   - Audit Blocks:   {} verified cryptographic links", decider.audit_chain_len());
            println!("   - Version:        0.1.0 (Rust 2021)\n");
        }
    }

    Ok(())
}
