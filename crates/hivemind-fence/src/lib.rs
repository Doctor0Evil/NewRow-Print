use serde::{Serialize, Deserialize};
use capability_core::{CapabilityStateView};          // readonly view
use envelope_core::{BiophysicalEnvelopeSnapshot};    // readonly view
use treeoflife_core::{TreeOfLifeView};               // readonly view
use roh_core::RoHProjection;                         // rohbefore/after/ceiling

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveMindFenceFrame {
    pub subject_id: String,
    pub epoch_ms: i64,
    pub capability: CapabilityStateView,      // view-only
    pub roh: RoHProjection,                  // RoH ≤ 0.3 invariant already enforced
    pub tol_view: TreeOfLifeView,            // BLOOD/OXYGEN/DECAY/FEAR/PAIN, etc.
    pub unfairdrain_index: f32,              // scalar metric
    pub subject_unfairdrain_flag: bool,
    pub subject_unfairstress_flag: bool,
    pub cohort_imbalance_index: f32,
    pub collective_imbalance_flag: bool,
    pub cohort_cooldown_advised: bool,
    pub juristags: Vec<String>,              // e.g. ["USFDA","EUMDR","CHILENEURORIGHTS2023"]
    pub hivehash: Option<String>,            // filled by logging layer, not by fence logic
}

pub trait HiveMindFenceView {
    /// Pure, non-actuating diagnostic over immutable snapshots.
    fn compute_advisories(
        &self,
        subject_id: &str,
        epoch_ms: i64,
        roh: &RoHProjection,
        envelope: &BiophysicalEnvelopeSnapshot,
        tol_view: &TreeOfLifeView,
        cohort_stats: &CohortStatsView,
    ) -> HiveMindFenceFrame;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortStatsView {
    pub peer_subjects: Vec<PeerSnapshot>,    // already-logged, readonly
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerSnapshot {
    pub subject_id: String,
    pub capability: CapabilityStateView,
    pub tol_view: TreeOfLifeView,
}
