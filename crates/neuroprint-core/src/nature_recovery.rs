use crate::treeoflife::TreeOfLifeView; // DECAY, LIFEFORCE, POWER, FEAR, PAIN 0.0–1.0
use crate::nature_overloaded::is_overloaded;   // existing NATURE::OVERLOADED
use crate::config::NatureRecoveryConfig;      // window lengths, thresholds

/// Pure, non-actuating diagnostic: logs RECOVERY as a boolean label only.
/// Inputs: immutable slice of consecutive TreeOfLifeView epochs, newest last.
/// Assumes each view was computed under RoH <= 0.3, ΔT and energy ceilings
/// already enforced by BiophysicalEnvelopeSpec and RoH model.
pub fn is_recovery(
    history: &[TreeOfLifeView],
    cfg: &NatureRecoveryConfig,
) -> bool {
    let len = history.len();
    if len == 0 { return false; }

    let w  = cfg.window_len_epochs as usize;
    let g  = cfg.recovery_gap_epochs as usize;
    let wr = cfg.recovery_window_epochs as usize;

    if len < w + g + wr {
        return false; // not enough history to evaluate
    }

    // Indices: ... [past_window][gap][recent_window] ... latest
    let recent_end   = len;
    let recent_start = recent_end - wr;
    let gap_start    = recent_start - g;
    let past_start   = gap_start - w;

    let past = &history[past_start..gap_start];
    let recent = &history[recent_start..recent_end];

    // 1) Recent OVERLOADED fraction in past window
    let overloaded_frac = past.iter()
        .filter(|v| is_overloaded(v, cfg.overloaded_cfg))
        .count() as f32 / past.len() as f32;

    if overloaded_frac < cfg.min_overloaded_fraction {
        return false;
    }

    // Helper to average a scalar field over a slice
    fn avg<F>(views: &[TreeOfLifeView], f: F) -> f32
    where
        F: Fn(&TreeOfLifeView) -> f32
    {
        if views.is_empty() { return 0.0; }
        let sum: f32 = views.iter().map(f).sum();
        (sum / views.len() as f32).clamp(0.0, 1.0)
    }

    // 2) Past vs recent averages in 5D microspace
    let decay_past      = avg(past,   |v| v.decay);
    let decay_recent    = avg(recent, |v| v.decay);
    let lf_past         = avg(past,   |v| v.lifeforce);
    let lf_recent       = avg(recent, |v| v.lifeforce);
    let fear_past       = avg(past,   |v| v.fear);
    let fear_recent     = avg(recent, |v| v.fear);
    let pain_past       = avg(past,   |v| v.pain);
    let pain_recent     = avg(recent, |v| v.pain);

    let delta_decay   = decay_past - decay_recent;
    let delta_lf      = lf_recent - lf_past;
    let delta_fear    = fear_past - fear_recent;
    let delta_pain    = pain_past - pain_recent;

    delta_decay   >= cfg.delta_decay_min
        && delta_lf   >= cfg.delta_lifeforce_min
        && delta_fear >= cfg.delta_fear_min
        && delta_pain >= cfg.delta_pain_min
}
