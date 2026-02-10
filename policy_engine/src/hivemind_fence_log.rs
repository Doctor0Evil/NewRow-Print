use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Write};
use std::fs::{File, OpenOptions};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveMindFenceView {
    pub view_id: String,
    pub subject_id: String,
    pub cohort_id: Option<String>,
    pub epoch_index: i64,
    pub roh_score: f32,
    pub unfairdrain_index: Option<f32>,
    pub unfairfear_index: Option<f32>,
    pub unfairpain_index: Option<f32>,
    pub cohort_decay_gini: Option<f32>,
    pub cohort_fear_gini: Option<f32>,
    pub cohort_pain_gini: Option<f32>,
    pub subject_unfairdrain_state: Option<FenceState>,
    pub subject_unfairstress_state: Option<FenceState>,
    pub cohort_balance_state: Option<FenceState>,
    pub unfairdrain_flag: bool,
    pub collective_imbalance_flag: bool,
    pub cohort_cooldown_advised: bool,
    pub timestamp_utc: String,
    pub prev_hexstamp: String,
    pub hexstamp: String,
    pub anchor_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FenceState {
    Info,
    Warn,
    Risk,
}

/// Immutable configuration for hivemind-fence-view logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveMindFenceLogConfig {
    /// Append-only JSONL log path, e.g., "/logs/hivemind-fence-view.jsonl".
    pub storage_path: String,
    /// Genesis prev_hexstamp for the first row, e.g., "0xHMFENCE-GENESIS".
    pub genesis_hexstamp: String,
}

/// Result type for log append operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HiveMindFenceLogError {
    IoError(String),
    SerializationError(String),
}

/// Append a single HIVEMIND-FENCE view to the WORM JSONL log.
///
/// Invariants (enforced by calling code and storage layer):
/// - File at `config.storage_path` is mounted / configured append-only.
/// - `view.hexstamp` has been computed as H(payload_without_hexes || prev_hexstamp).
/// - `view.prev_hexstamp` is either the prior row's hexstamp or `config.genesis_hexstamp`.
///
/// This function never mutates capability, consent, envelope, or policy state.
/// It only appends a serialized line to the hivemind-fence-view.jsonl log.
pub fn append_hivemind_fence_view(
    config: &HiveMindFenceLogConfig,
    view: &HiveMindFenceView,
) -> Result<(), HiveMindFenceLogError> {
    let path = Path::new(&config.storage_path);

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| HiveMindFenceLogError::IoError(e.to_string()))?;

    let mut writer = BufWriter::new(file);

    let json = serde_json::to_string(view)
        .map_err(|e| HiveMindFenceLogError::SerializationError(e.to_string()))?;

    writer
        .write_all(json.as_bytes())
        .and_then(|_| writer.write_all(b"\n"))
        .map_err(|e| HiveMindFenceLogError::IoError(e.to_string()))
}
