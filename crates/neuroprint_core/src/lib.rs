use serde::{Deserialize, Serialize};
use capability_core::CapabilityState;
use envelope_core::BiophysicalEnvelopeSnapshot;
use roh_model::RoHProjection;

/// View-only input for a single neuromorphic snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroPrintInput {
    pub capability_state: CapabilityState,
    pub roh: RoHProjection,
    pub envelope: BiophysicalEnvelopeSnapshot,
    /// Optional evolve index / learning counter.
    pub evolve_index: Option<u64>,
    /// Optional epoch index within session.
    pub epoch_index: Option<u64>,
}

/// Human-binary compatible TREE view, normalized to [0.0, 1.0].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroPrintView {
    // TREE assets – read-only diagnostics.
    pub blood: f32,
    pub oxygen: f32,
    pub wave: f32,
    pub time: f32,
    pub decay: f32,
    pub lifeforce: f32,
    pub brain: f32,
    pub smart: f32,
    pub evolve: f32,
    pub power: f32,
    pub tech: f32,
    pub fear: f32,
    pub pain: f32,
    pub nano: f32,
    /// Optional advisory labels (e.g., NATURE tokens as strings).
    pub labels: Vec<String>,
}
