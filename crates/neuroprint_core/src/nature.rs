use serde::{Deserialize, Serialize};
use crate::NeuroPrintView;

/// Configuration for NATURE predicates, loaded from ALN/JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatureConfig {
    pub calm_stable: CalmStableConfig,
    pub overloaded: OverloadedConfig,
    pub recovery: RecoveryConfig,
    pub unfair_drain: UnfairDrainConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalmStableConfig {
    pub window_epochs: u64,
    pub lifeforce_min: f32,
    pub fear_max: f32,
    pub pain_max: f32,
    pub decay_max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverloadedConfig {
    pub window_epochs: u64,
    pub decay_min: f32,
    pub power_min: f32,
    pub lifeforce_max: f32,
    pub fear_min: f32,
    pub pain_min: f32,
}

// Similar structs for RecoveryConfig and UnfairDrainConfig ...

/// Evaluated NATURE tokens for a given epoch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatureLabels {
    pub calm_stable: bool,
    pub overloaded: bool,
    pub recovery: bool,
    pub unfair_drain: bool,
}

pub fn eval_nature_labels(
    history: &[NeuroPrintView],
    cfg: &NatureConfig,
) -> NatureLabels {
    NatureLabels {
        calm_stable: eval_calm_stable(history, &cfg.calm_stable),
        overloaded: eval_overloaded(history, &cfg.overloaded),
        recovery: eval_recovery(history, &cfg.recovery),
        unfair_drain: eval_unfair_drain(history, &cfg.unfair_drain),
    }
}
