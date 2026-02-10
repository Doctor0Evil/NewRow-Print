use crate::HiveMindFenceFrame;

pub enum FenceSink {
    Hud,
    AiChat,
    OfflineAnalytics,
    NoSaEvidence,      // for computenosaferalternative evidence bundles
}

pub fn write_frame(frame: &HiveMindFenceFrame, sink: FenceSink) -> Result<(), LogError> {
    match sink {
        FenceSink::Hud | FenceSink::AiChat | FenceSink::OfflineAnalytics => {
            append_jsonl("hivemind-fence-view.jsonl", frame)
        }
        FenceSink::NoSaEvidence => {
            append_jsonl("hivemind-fence-evidence.jsonl", frame)
        }
    }
}
