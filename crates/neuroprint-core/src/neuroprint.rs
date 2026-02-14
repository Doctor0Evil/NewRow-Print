use serde::{Deserialize, Serialize};

/// View-only snapshot of governed neuromorph state.
/// In your stack, this would be constructed from existing
/// CapabilityState, RoHProjection, and BiophysicalEnvelopeSnapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroPrintInput {
    pub subject_id: String,
    pub epoch_index: u64,
    // Core governed rails (simplified surface)
    pub roh_after: f32,      // normalized RoH after step (0.0–0.3 in spec, scaled here)
    pub roh_ceiling: f32,    // usually 0.3
    pub hr_norm: f32,        // 0.0–1.0 heart rate axis
    pub hrv_norm: f32,       // 0.0–1.0 HRV axis
    pub eeg_wave_norm: f32,  // 0.0–1.0 EEG bandpower/alphaCVE
    pub eda_norm: f32,       // 0.0–1.0 EDA arousal proxy
    pub motion_norm: f32,    // 0.0–1.0 motion / agitation proxy
    pub capability_tier: f32, // 0.0–1.0 discrete tier mapped to scalar
    pub evolve_index: f32,    // 0.0–1.0 normalized evolve count
    // 1D geometry from biofield / sentinel (already normalized)
    pub bio_1d_coord: f32,    // 0.0–1.0 position on your 1D manifold
    pub biofield_intensity: f32, // 0.0–1.0 local field load
}

/// TREE-style diagnostic view (all 0.0–1.0, read-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroPrintView {
    pub subject_id: String,
    pub epoch_index: u64,
    // Core TREE assets
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
    // 1D geometry rails
    pub bio_coord_1d: f32,
    pub biofield_load: f32,
    // Optional advisory labels (CALM_STABLE, OVERLOADED, etc.)
    pub nature_labels: Vec<String>,
}

fn clamp01(x: f32) -> f32 {
    if x.is_nan() {
        0.0
    } else if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

/// Map governed inputs + biofield 1D geometry into a TREE/NATURE view.
/// Pure function: NO side effects, NO capability writes.
pub fn neuroprint_from_snapshot(input: &NeuroPrintInput) -> NeuroPrintView {
    // RoH-based rails
    let roh_norm = if input.roh_ceiling > 0.0 {
        clamp01(input.roh_after / input.roh_ceiling)
    } else {
        0.0
    };
    let decay = roh_norm;
    let lifeforce = clamp01(1.0 - roh_norm);

    // Physiology
    let blood = clamp01(input.hr_norm);         // higher HR → higher load
    let oxygen = clamp01(input.hrv_norm);      // higher HRV → more reserve
    let wave = clamp01(input.eeg_wave_norm);   // cognitive load / engagement

    // Capability / evolution rails
    let brain = clamp01(input.capability_tier);
    let evolve = clamp01(input.evolve_index);
    let smart = clamp01(0.5 * brain + 0.5 * evolve);

    // Power / tech (simplified weighted loads)
    let power = clamp01(0.5 * input.hr_norm + 0.5 * input.eeg_wave_norm);
    let tech = clamp01(0.5 * brain + 0.5 * power);

    // Distress rails from EDA + motion
    let fear = clamp01(0.6 * input.eda_norm + 0.4 * input.hr_norm);
    let pain = clamp01(0.5 * input.motion_norm + 0.5 * input.eda_norm);

    // Nano rail: reuse evolve for now (you can specialize later)
    let nano = evolve;

    // 1D geometry from biofield / sentinel
    let bio_coord_1d = clamp01(input.bio_1d_coord);
    let biofield_load = clamp01(input.biofield_intensity);

    // Simple NATURE labelling (diagnostic only)
    let mut nature_labels = Vec::new();
    if lifeforce > 0.7 && fear < 0.3 && pain < 0.3 && decay < 0.3 {
        nature_labels.push("CALM_STABLE".to_string());
    }
    if decay > 0.7 || fear > 0.7 || pain > 0.7 {
        nature_labels.push("OVERLOADED".to_string());
    }
    // Example fairness hint using 1D geometry (still advisory)
    if biofield_load > 0.8 && lifeforce < 0.4 {
        nature_labels.push("LOCAL_1D_OVERLOAD".to_string());
    }

    NeuroPrintView {
        subject_id: input.subject_id.clone(),
        epoch_index: input.epoch_index,
        blood,
        oxygen,
        wave,
        time: clamp01(input.epoch_index as f32 / 10_000.0),
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
        bio_coord_1d,
        biofield_load,
        nature_labels,
    }
}

/// JSONL-friendly wrapper: turn a slice of inputs into newline-delimited views.
pub fn render_jsonl(inputs: &[NeuroPrintInput]) -> String {
    let mut out = String::new();
    for inp in inputs {
        let view = neuroprint_from_snapshot(inp);
        let line = serde_json::to_string(&view)
            .expect("NeuroPrintView must be serializable");
        out.push_str(&line);
        out.push('\n');
    }
    out
}
