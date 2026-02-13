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

fn comparable(a: &SubjectSnapshot, b: &SubjectSnapshot) -> bool {
    // Same capability tier.
    if a.capability_tier != b.capability_tier {
        return false;
    }

    // Same jurisdiction/policy context tag.
    if a.policy_view.jurisdiction_tag != b.policy_view.jurisdiction_tag {
        return false;
    }

    // Same high-level task tag (e.g., "lesson_01").
    if a.task_tag != b.task_tag {
        return false;
    }

    true
}

/// Compute advisory UNFAIRDRAIN flags over a set of SubjectSnapshot records.
/// Pure function: no I/O, no capability or policy mutations.
/// Intended usage: log post-processing or simulation diagnostics.
pub fn compute_unfair_drain(
    cfg: &UnfairDrainConfig,
    snapshots: &[SubjectSnapshot],
) -> Vec<UnfairDrainFlag> {
    // Group snapshots by subject_id for sliding-window analysis.
    let mut by_subject: HashMap<String, Vec<&SubjectSnapshot>> = HashMap::new();
    for snap in snapshots {
        by_subject
            .entry(snap.subject_id.clone())
            .or_default()
            .push(snap);
    }

    let mut flags = Vec::new();

    for (subject_id, mut series) in by_subject {
        // Sort by time within subject.
        series.sort_by_key(|s| s.t_ms);

        // For each snapshot in this subject's series, compute window-based metrics.
        for (idx, &snap) in series.iter().enumerate() {
            let t_center = snap.t_ms;
            let t_start = t_center - cfg.window_ms;

            // 1. Collect this subject's window frames.
            let mut self_count = 0usize;
            let mut self_overload_count = 0usize;
            let mut self_budget_sum = 0f32;

            for &s in series.iter() {
                if s.t_ms >= t_start && s.t_ms <= t_center {
                    self_count += 1;
                    self_budget_sum += 0.5 * (s.lifeforce + s.oxygen);
                    if s.overloaded {
                        self_overload_count += 1;
                    }
                }
            }

            if self_count == 0 {
                continue;
            }

            let self_budget_avg = self_budget_sum / self_count as f32;
            let self_overload_frac = self_overload_count as f32 / self_count as f32;

            // 2. Build peer group at this time across all subjects.
            let mut peer_budgets: Vec<f32> = Vec::new();

            for other in snapshots {
                // Time window for peer is aligned to t_center; same window width for simplicity.
                if other.t_ms >= t_start && other.t_ms <= t_center {
                    if comparable(snap, other) {
                        let b = 0.5 * (other.lifeforce + other.oxygen);
                        peer_budgets.push(b);
                    }
                }
            }

            if peer_budgets.is_empty() {
                // No peers: cannot assess unfairness; default to no unfair drain.
                flags.push(UnfairDrainFlag {
                    subject_id: subject_id.clone(),
                    t_ms: t_center,
                    unfair_drain: false,
                    budget: self_budget_avg,
                    peer_median_budget: self_budget_avg,
                    overload_fraction: self_overload_frac,
                });
                continue;
            }

            peer_budgets.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mid = peer_budgets.len() / 2;
            let peer_median = if peer_budgets.len() % 2 == 0 {
                0.5 * (peer_budgets[mid - 1] + peer_budgets[mid])
            } else {
                peer_budgets[mid]
            };

            // 3. Apply UNFAIRDRAIN predicate:
            //     B_s(t) <= Med_G(t) - delta_unfair
            //  AND overload_frac_s(t) >= overload_frac_min
            let budget_deficit = peer_median - self_budget_avg;
            let unfair = budget_deficit >= cfg.delta_unfair
                && self_overload_frac >= cfg.overload_frac_min;

            flags.push(UnfairDrainFlag {
                subject_id: subject_id.clone(),
                t_ms: t_center,
                unfair_drain: unfair,
                budget: self_budget_avg,
                peer_median_budget: peer_median,
                overload_fraction: self_overload_frac,
            });
        }
    }

    flags
}
