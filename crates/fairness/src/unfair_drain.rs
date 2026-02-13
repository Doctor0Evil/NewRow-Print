use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Minimal view of capability tiers, aligned with your CapabilityState.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityTier {
    ModelOnly,
    LabBench,
    ControlledHuman,
    GeneralUse,
}

/// Minimal view of jurisdiction / policy context for peer grouping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyStackView {
    pub jurisdiction_tag: String,   // e.g., "US_FDA", "EU_MDR", "GLOBAL_BASE"
    pub base_medical_ok: bool,
    pub base_engineering_ok: bool,
    pub juris_local_ok: bool,
    pub quantum_ai_safety_ok: bool,
}

/// Minimal role tag to help Comparables(s, s', t) if needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleTag {
    Teacher,
    Learner,
    Mentor,
    Operator,
    Other,
}

/// One snapshot per subject per epoch, derived from TreeOfLifeView and logs.
/// All floats are normalized 0.0–1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectSnapshot {
    pub subject_id: String,
    pub t_ms: i64,

    pub capability_tier: CapabilityTier,
    pub role: RoleTag,
    pub policy_view: PolicyStackView,

    // Budget components from TREE assets.
    pub lifeforce: f32,  // TREE.LIFEFORCE in [0, 1]
    pub oxygen:   f32,   // TREE.OXYGEN in [0, 1]

    // Overload indicator from NATURE/OVERLOADED.
    pub overloaded: bool,

    // Optional task/domain tag from .evolve.jsonl to refine Comparables.
    pub task_tag: String,
}

/// Configuration shard for UNFAIRDRAIN predicate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnfairDrainConfig {
    /// Length of the sliding window in milliseconds.
    pub window_ms: i64,
    /// Max allowed deficit below peer median budget.
    pub delta_unfair: f32,
    /// Minimum overload fraction required to flag unfair drain.
    pub overload_frac_min: f32,
}

/// Output flag: advisory-only UNFAIRDRAIN label per (subject, time).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnfairDrainFlag {
    pub subject_id: String,
    pub t_ms: i64,
    pub unfair_drain: bool,

    // Optional diagnostics for analysis / HUDs.
    pub budget: f32,
    pub peer_median_budget: f32,
    pub overload_fraction: f32,
}
