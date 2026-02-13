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
/// Pure, non-actuating projection from governed state to NeuroPrintView.
pub fn neuroprint_from_snapshot(input: &NeuroPrintInput) -> NeuroPrintView {
    // Internal helpers use only envelope + RoH + capability, never mutate them.
    let blood = clamp01(map_blood(&input.envelope));
    let oxygen = clamp01(map_oxygen(&input.envelope));
    let wave = clamp01(map_wave(&input.envelope));
    let time = clamp01(map_time(&input.envelope));

    // RoH-based assets; RoHProjection enforces roh_after <= roh_ceiling <= 0.3.
    let roh_norm = clamp01(input.roh.after / input.roh.ceiling);
    let decay = roh_norm;
    let lifeforce = 1.0 - roh_norm;

    let brain = clamp01(map_brain(&input.capability_state));
    let smart = clamp01(map_smart(&input.capability_state));
    let evolve = clamp01(map_evolve(&input.capability_state, input.evolve_index));
    let power = clamp01(map_power(&input.envelope));
    let tech = clamp01(map_tech(&input.envelope));
    let fear = clamp01(map_fear(&input.envelope));
    let pain = clamp01(map_pain(&input.envelope));
    let nano = clamp01(map_nano(input.evolve_index));

    let labels = Vec::new(); // NATURE labels can be attached by a separate module.

    NeuroPrintView {
        blood,
        oxygen,
        wave,
        time,
        decay,
        lifeforce,
        brain,
        smart,
        evolve,
        power,
        tech,
        fear,
        pain,
        nano,
        labels,
    }
}

#[macro_export]
macro_rules! neuroprint {
    ($input:expr) => {
        $crate::neuroprint_from_snapshot(&$input)
    };
}
