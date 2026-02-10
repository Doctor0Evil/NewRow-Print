use serde::{Deserialize, Serialize};

use crate::hivemind_fence_log::{
    append_hivemind_fence_view, FenceState, HiveMindFenceLogConfig, HiveMindFenceLogError,
    HiveMindFenceView,
};

/// Minimal, readonly snapshot input for HIVEMIND-FENCE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveMindFenceInput {
    pub view_id: String,
    pub subject_id: String,
    pub cohort_id: Option<String>,
    pub epoch_index: i64,
    /// Current global RoH score (0.0..=0.30 in CapControlledHuman contexts).
    pub roh_score: f32,
    /// Subject-level TREE asset scores (0.0..=1.0) from Tree-of-Life, if available.
    pub tol_fear: Option<f32>,
    pub tol_pain: Option<f32>,
    pub tol_decay: Option<f32>,
    pub tol_lifeforce: Option<f32>,
    /// Cohort aggregates for fairness (e.g., mean/median TREE assets).
    pub cohort_mean_fear: Option<f32>,
    pub cohort_mean_pain: Option<f32>,
    /// Cohort dispersion metrics (e.g., Gini) for TREE assets.
    pub cohort_decay_gini: Option<f32>,
    pub cohort_fear_gini: Option<f32>,
    pub cohort_pain_gini: Option<f32>,
    /// Previous hash in the hivemind-fence-view.jsonl WORM chain.
    pub prev_hexstamp: String,
    /// Optional external anchor (e.g., Googolswarm transaction id).
    pub anchor_id: Option<String>,
    /// ISO-8601 UTC timestamp, provided by caller.
    pub timestamp_utc: String,
}

/// Advisory-only threshold configuration for HIVEMIND-FENCE indices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveMindFenceConfig {
    /// Threshold above which subject unfairdrain_index is considered WARN.
    pub unfairdrain_warn: f32,
    /// Threshold above which subject unfairdrain_index is considered RISK.
    pub unfairdrain_risk: f32,
    /// Threshold on cohort dispersion metrics to flag collective imbalance.
    pub cohesion_gini_warn: f32,
    pub cohesion_gini_risk: f32,
    /// RoH level at which cohort-wide cooldown is advised (e.g., 0.25).
    pub roh_cooldown_threshold: f32,
}

impl Default for HiveMindFenceConfig {
    fn default() -> Self {
        Self {
            unfairdrain_warn: 0.15,
            unfairdrain_risk: 0.30,
            cohesion_gini_warn: 0.20,
            cohesion_gini_risk: 0.35,
            roh_cooldown_threshold: 0.25,
        }
    }
}

/// Pure evaluator namespace for HIVEMIND-FENCE.
pub struct HiveMindFence;

impl HiveMindFence {
    /// Compute a HiveMindFenceView from a snapshot and thresholds,
    /// then append it to the hivemind-fence-view JSONL WORM log.
    ///
    /// Invariants:
    /// - No capability, consent, or envelope state is mutated.
    /// - hexstamp is a content hash over the view payload plus prev_hexstamp.
    pub fn evaluate_and_log(
        log_cfg: &HiveMindFenceLogConfig,
        cfg: &HiveMindFenceConfig,
        input: &HiveMindFenceInput,
    ) -> Result<(), HiveMindFenceLogError> {
        let unfairdrain_index =
            Self::compute_unfairdrain_index(input.tol_decay, input.tol_lifeforce);
        let (unfairfear_index, unfairpain_index) =
            Self::compute_unfairstress_indices(input.tol_fear, input.tol_pain, input.cohort_mean_fear, input.cohort_mean_pain);

        let subject_unfairdrain_state =
            Self::classify_fence_state(unfairdrain_index, cfg.unfairdrain_warn, cfg.unfairdrain_risk);
        let subject_unfairstress_state = Self::classify_fence_state(
            Self::max_opt(unfairfear_index, unfairpain_index),
            cfg.unfairdrain_warn,
            cfg.unfairdrain_risk,
        );

        let cohort_balance_state = Self::classify_fence_state(
            Self::max_three_opt(
                input.cohort_decay_gini,
                input.cohort_fear_gini,
                input.cohort_pain_gini,
            ),
            cfg.cohesion_gini_warn,
            cfg.cohesion_gini_risk,
        );

        let unfairdrain_flag =
            matches!(subject_unfairdrain_state, Some(FenceState::Risk));
        let collective_imbalance_flag =
            matches!(cohort_balance_state, Some(FenceState::Risk));
        let cohort_cooldown_advised =
            input.roh_score >= cfg.roh_cooldown_threshold || collective_imbalance_flag;

        let mut view = HiveMindFenceView {
            view_id: input.view_id.clone(),
            subject_id: input.subject_id.clone(),
            cohort_id: input.cohort_id.clone(),
            epoch_index: input.epoch_index,
            roh_score: input.roh_score,
            unfairdrain_index,
            unfairfear_index,
            unfairpain_index,
            cohort_decay_gini: input.cohort_decay_gini,
            cohort_fear_gini: input.cohort_fear_gini,
            cohort_pain_gini: input.cohort_pain_gini,
            subject_unfairdrain_state,
            subject_unfairstress_state,
            cohort_balance_state,
            unfairdrain_flag,
            collective_imbalance_flag,
            cohort_cooldown_advised,
            timestamp_utc: input.timestamp_utc.clone(),
            prev_hexstamp: input.prev_hexstamp.clone(),
            hexstamp: String::new(), // filled below
            anchor_id: input.anchor_id.clone(),
        };

        view.hexstamp = Self::compute_hexstamp(&view);

        append_hivemind_fence_view(log_cfg, &view)
    }

    /// Subject-level unfair drain index, normalized to 0.0..=1.0.
    /// Example: higher DECAY minus LIFEFORCE yields higher unfair drain.
    fn compute_unfairdrain_index(
        tol_decay: Option<f32>,
        tol_lifeforce: Option<f32>,
    ) -> Option<f32> {
        match (tol_decay, tol_lifeforce) {
            (Some(decay), Some(lifeforce)) => {
                let raw = decay - lifeforce;
                Some(Self::clamp01((raw + 1.0) * 0.5))
            }
            _ => None,
        }
    }

    /// Subject-level unfair fear/pain vs cohort means, normalized to 0.0..=1.0.
    fn compute_unfairstress_indices(
        tol_fear: Option<f32>,
        tol_pain: Option<f32>,
        cohort_mean_fear: Option<f32>,
        cohort_mean_pain: Option<f32>,
    ) -> (Option<f32>, Option<f32>) {
        let fear_idx = match (tol_fear, cohort_mean_fear) {
            (Some(fear), Some(mu_fear)) => {
                let delta = fear - mu_fear;
                Some(Self::clamp01((delta + 1.0) * 0.5))
            }
            _ => None,
        };

        let pain_idx = match (tol_pain, cohort_mean_pain) {
            (Some(pain), Some(mu_pain)) => {
                let delta = pain - mu_pain;
                Some(Self::clamp01((delta + 1.0) * 0.5))
            }
            _ => None,
        };

        (fear_idx, pain_idx)
    }

    fn classify_fence_state(
        idx: Option<f32>,
        warn_threshold: f32,
        risk_threshold: f32,
    ) -> Option<FenceState> {
        let value = idx?;
        if value >= risk_threshold {
            Some(FenceState::Risk)
        } else if value >= warn_threshold {
            Some(FenceState::Warn)
        } else {
            Some(FenceState::Info)
        }
    }

    fn max_opt(a: Option<f32>, b: Option<f32>) -> Option<f32> {
        match (a, b) {
            (Some(x), Some(y)) => Some(x.max(y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }

    fn max_three_opt(
        a: Option<f32>,
        b: Option<f32>,
        c: Option<f32>,
    ) -> Option<f32> {
        Self::max_opt(Self::max_opt(a, b), c)
    }

    fn clamp01(x: f32) -> f32 {
        if x < 0.0 {
            0.0
        } else if x > 1.0 {
            1.0
        } else {
            x
        }
    }

    /// Deterministic hexstamp over view content plus prev_hexstamp, with no I/O.
    /// Placeholder: wire to your existing hexstamp/H() utility in sovereignty core.
    fn compute_hexstamp(view: &HiveMindFenceView) -> String {
        use blake3::Hasher;

        let mut hasher = Hasher::new();
        // Note: prev_hexstamp is part of the chain, so include it explicitly.
        hasher.update(view.prev_hexstamp.as_bytes());

        // Serialize without the hexstamp field itself to avoid self-reference.
        let mut clone = view.clone();
        clone.hexstamp.clear();

        let payload = serde_json::to_vec(&clone)
            .expect("HiveMindFenceView serialization must not fail for hashing");
        hasher.update(&payload);

        let hash = hasher.finalize();
        format!("0xHMFENCE{}", hash.to_hex())
    }
}
