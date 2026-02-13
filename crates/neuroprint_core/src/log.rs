use serde::{Deserialize, Serialize};
use crate::{NeuroPrintView};
use crate::nature::NatureLabels;
use capability_core::CapabilityState;
use roh_model::RoHProjection;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroPrintLogEntry {
    pub timestamp_ms: u64,
    pub subject_id: String,
    pub epoch_index: u64,
    pub capability_state: CapabilityState,
    pub roh: RoHProjection,
    pub neuroprint: NeuroPrintView,
    pub nature: Option<NatureLabels>,
}
