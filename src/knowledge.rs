use crate::types::{GeometryMetrics, Neuron};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct KnowledgeStore {
    neurons: Arc<RwLock<Vec<Neuron>>>,
}

impl Default for KnowledgeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeStore {
    pub fn new() -> Self {
        Self {
            neurons: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a new knowledge neuron in real-time (Episodic Memory Accretion)
    pub fn learn(&self, concept: String, domain: String, vector: Vec<f32>) -> String {
        let mut store = self.neurons.write();
        let id = format!("neuron_{:06x}", store.len() + 1);
        let neuron = Neuron {
            id: id.clone(),
            concept,
            domain,
            vector,
            age_days: 0,
            uses: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        store.push(neuron);
        id
    }

    pub fn len(&self) -> usize {
        self.neurons.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Fast Cosine similarity search against all active neurons
    pub fn query_top_k(&self, query_vec: &[f32], k: usize) -> (Vec<Neuron>, GeometryMetrics) {
        let store = self.neurons.read();
        if store.is_empty() {
            return (Vec::new(), GeometryMetrics::default());
        }

        let q_norm = norm(query_vec);
        if q_norm < 1e-9 {
            return (
                Vec::new(),
                GeometryMetrics {
                    lid: 1.0,
                    cos_gap: 0.0,
                    etm: 0.0,
                    activated_domains: vec!["empty".into()],
                    activated_ages: vec![0],
                },
            );
        }

        let mut scored: Vec<(f32, &Neuron)> = store
            .iter()
            .map(|n| {
                let dot: f32 = n
                    .vector
                    .iter()
                    .zip(query_vec.iter())
                    .map(|(a, b)| a * b)
                    .sum();
                let n_norm = norm(&n.vector);
                let cos = if n_norm > 1e-9 {
                    dot / (q_norm * n_norm)
                } else {
                    0.0
                };
                (cos, n)
            })
            .collect();

        // Sort descending by cosine similarity
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let top_k: Vec<Neuron> = scored.iter().take(k).map(|(_, n)| (*n).clone()).collect();
        let top_cosines: Vec<f32> = scored.iter().take(k).map(|(c, _)| *c).collect();

        let cos_gap = if top_cosines.len() >= 2 {
            (top_cosines[0] - top_cosines[1]).max(0.0)
        } else {
            0.5
        };

        // Estimate Local Intrinsic Dimensionality (LID)
        // High dispersion between distances indicates topological confusion / hallucination
        let lid = estimate_lid(&top_cosines);
        let etm = if !top_cosines.is_empty() {
            top_cosines[0].clamp(0.0, 1.0)
        } else {
            0.0
        };

        let domains: Vec<String> = top_k.iter().map(|n| n.domain.clone()).collect();
        let ages: Vec<u32> = top_k.iter().map(|n| n.age_days).collect();

        let geom = GeometryMetrics {
            lid,
            cos_gap,
            etm,
            activated_domains: domains,
            activated_ages: ages,
        };

        (top_k, geom)
    }
}

fn norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

fn estimate_lid(cosines: &[f32]) -> f32 {
    if cosines.len() < 3 {
        return 0.25;
    }
    // Transform cosines to pseudo-distances: d = 1.0 - cos
    let dists: Vec<f32> = cosines.iter().map(|c| (1.0 - c).max(1e-4)).collect();
    let r_w = dists[dists.len() - 1]; // Maximum radius in neighborhood
    if r_w <= 1e-6 {
        return 0.10;
    }

    let mut sum_log = 0.0;
    for &d in &dists[..dists.len() - 1] {
        sum_log += (d / r_w).ln();
    }
    let k = (dists.len() - 1) as f32;
    if sum_log.abs() < 1e-6 {
        return 0.50;
    }
    let raw_lid = -k / sum_log;
    // Normalize into [0.0 - 1.0] scale
    (raw_lid / 10.0).clamp(0.05, 0.99)
}
