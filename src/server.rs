use crate::decider::SeedDecider;
use crate::knowledge::KnowledgeStore;
use crate::types::{CandidateAction, GeometryMetrics};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, UnixListener};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum IpcRequest {
    #[serde(rename = "evaluate")]
    Evaluate {
        candidate: CandidateAction,
        geometry: Option<GeometryMetrics>,
    },
    #[serde(rename = "learn")]
    Learn {
        concept: String,
        domain: String,
        vector: Vec<f32>,
    },
    #[serde(rename = "status")]
    Status,
}

#[derive(Debug, Serialize)]
pub struct IpcResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

pub async fn run_ipc_server(
    socket_path: &str,
    tcp_addr: Option<&str>,
    decider: Arc<SeedDecider>,
    knowledge: Arc<KnowledgeStore>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Clean up existing socket file if present
    if std::path::Path::new(socket_path).exists() {
        let _ = std::fs::remove_file(socket_path);
    }

    let unix_listener = UnixListener::bind(socket_path)?;
    println!(
        "⚡ [seed-core] Unix Domain Socket active at: {}",
        socket_path
    );

    if let Some(addr) = tcp_addr {
        let tcp_listener = TcpListener::bind(addr).await?;
        println!("⚡ [seed-core] TCP IPC listener active at: {}", addr);
        let dec_clone = Arc::clone(&decider);
        let kn_clone = Arc::clone(&knowledge);
        tokio::spawn(async move {
            loop {
                if let Ok((mut stream, _)) = tcp_listener.accept().await {
                    let d = Arc::clone(&dec_clone);
                    let k = Arc::clone(&kn_clone);
                    tokio::spawn(async move {
                        let (reader, mut writer) = stream.split();
                        let mut lines = BufReader::new(reader).lines();
                        while let Ok(Some(line)) = lines.next_line().await {
                            let resp = handle_request(&line, &d, &k);
                            let json_resp = serde_json::to_string(&resp).unwrap_or_default() + "\n";
                            let _ = writer.write_all(json_resp.as_bytes()).await;
                        }
                    });
                }
            }
        });
    }

    loop {
        let (mut stream, _) = unix_listener.accept().await?;
        let d = Arc::clone(&decider);
        let k = Arc::clone(&knowledge);
        tokio::spawn(async move {
            let (reader, mut writer) = stream.split();
            let mut lines = BufReader::new(reader).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let resp = handle_request(&line, &d, &k);
                let json_resp = serde_json::to_string(&resp).unwrap_or_default() + "\n";
                let _ = writer.write_all(json_resp.as_bytes()).await;
            }
        });
    }
}

fn handle_request(
    line: &str,
    decider: &SeedDecider,
    knowledge: &KnowledgeStore,
) -> IpcResponse<serde_json::Value> {
    match serde_json::from_str::<IpcRequest>(line) {
        Ok(IpcRequest::Evaluate {
            candidate,
            geometry,
        }) => {
            let geom = geometry.unwrap_or_default();
            let verdict = decider.evaluate(&candidate, &geom);
            IpcResponse {
                success: true,
                data: Some(serde_json::to_value(verdict).unwrap()),
                error: None,
            }
        }
        Ok(IpcRequest::Learn {
            concept,
            domain,
            vector,
        }) => {
            let neuron_id = knowledge.learn(concept, domain, vector);
            IpcResponse {
                success: true,
                data: Some(
                    serde_json::json!({ "neuron_id": neuron_id, "total_neurons": knowledge.len() }),
                ),
                error: None,
            }
        }
        Ok(IpcRequest::Status) => IpcResponse {
            success: true,
            data: Some(serde_json::json!({
                "status": "OPERATIONAL",
                "total_neurons": knowledge.len(),
                "audit_blocks": decider.audit_chain_len(),
                "version": "0.1.0"
            })),
            error: None,
        },
        Err(e) => IpcResponse {
            success: false,
            data: None,
            error: Some(format!("Invalid IPC JSON: {}", e)),
        },
    }
}
