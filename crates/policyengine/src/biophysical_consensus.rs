//! Biophysical consensus and fairness diagnostics for Jetson-Line micro-units.
//!
//! This module is strictly observer/consensus tier:
//! - NO device IO.
//! - NO CapabilityState or envelope mutation.
//! - Pure functions only, suitable for use in Church-of-FEAR, Tree-of-Life, Jetson-Line logs.

use serde::{Deserialize, Serialize};

/// Core scalar rails for a site, as seen through Tree-of-Life / NATURE.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TreeOfLifeRails {
    pub roh: f32,          // Risk-of-Harm, clamped [0, 0.3]
    pub decay: f32,        // DECAY = roh / 0.3, clamped [0, 1]
    pub lifeforce: f32,    // LIFEFORCE = 1 - DECAY
    pub fear: f32,         // FEAR asset [0, 1]
    pub pain: f32,         // PAIN asset [0, 1]
    pub power: f32,        // POWER asset [0, 1]
    pub church: f32,       // CHURCH asset [0, 1]
    pub unfair_drain: bool,
    pub calm_stable: bool,
    pub overloaded: bool,
    pub recovery: bool,
}

/// Minimal deed vocabulary for Jetson-Line justice/fairness.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeedKind {
    Help,
    Repair,
    Support,
    DeployCleanTech,
    Colonize,
    Conflict,
    UseHabit,
    EmitPollution,
    Abstain,
    Unknown,
}

/// Cause / context labels for the deed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CauseContext {
    pub rule_id: Option<String>,    // e.g., "JUST_CAUSE", "LAST_RESORT_WINDOW"
    pub intent_tag: Option<String>, // e.g., "defensive", "restorative"
}

/// Snapshot of one site on the Jetson-Line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSnapshot {
    pub index: u32,                // lattice index
    pub rails: TreeOfLifeRails,    // Tree-of-Life / NATURE view at this tick
}

/// A Jetson-Line micro-unit / deed event, consensus-facing view.
///
/// This is intentionally close to Church-of-FEAR DeedEvent but adds pre/post TREE rails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroUnit {
    pub tick: u64,                 // global Jetson-Line tick
    pub actor_id: String,          // who initiated the deed
    pub target_ids: Vec<String>,   // affected parties (if known)
    pub kind: DeedKind,
    pub cause: CauseContext,

    /// Pre-state snapshots for all sites in scope (actor + relevant neighbors).
    pub pre_sites: Vec<SiteSnapshot>,

    /// Post-state snapshots for the same sites (after deed application).
    pub post_sites: Vec<SiteSnapshot>,

    /// Optional external ids to bind W-cycle reflections (What/SoWhat/NowWhat text).
    pub w_cycle_binding: Option<String>,
}

/// Fairness judgement for a single micro-unit (advisory only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessVerdict {
    pub fairness_positive: bool,
    pub fairness_negative: bool,
    pub fairness_ambiguous: bool,
    pub reason: String,
}

/// Simple W-cycle advisory view: What / SoWhat / NowWhat strings.
///
/// In a later step, a proc-macro can auto-derive this from MicroUnit + FairnessVerdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WCycleView {
    pub what: String,
    pub so_what: String,
    pub now_what: String,
}

/// Policy parameters for biophysical fairness checks.
/// These are loaded from ALN/config in real deployments.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BiophysicalConsensusPolicy {
    pub roh_ceiling: f32,        // e.g., 0.30
    pub decay_max: f32,          // e.g., 1.0
    pub fear_safe_max: f32,      // e.g., 0.60
    pub power_church_k: f32,     // k in POWER <= k * CHURCH
}

impl Default for BiophysicalConsensusPolicy {
    fn default() -> Self {
        Self {
            roh_ceiling: 0.30,
            decay_max: 1.0,
            fear_safe_max: 0.60,
            power_church_k: 2.0,
        }
    }
}

// ---------- Internal helpers (pure, non-actuating) ----------

fn power_within_church_cap(rails: &TreeOfLifeRails, k: f32) -> bool {
    if rails.church <= f32::EPSILON {
        // With no CHURCH, any positive POWER is treated as exceeding the cap.
        return rails.power <= 0.0;
    }
    rails.power <= k * rails.church
}

fn site_respects_core_rails(rails: &TreeOfLifeRails, policy: &BiophysicalConsensusPolicy) -> bool {
    rails.roh <= policy.roh_ceiling
        && rails.decay <= policy.decay_max
        && power_within_church_cap(rails, policy.power_church_k)
}

/// Decide whether a site is "vulnerable" for fairness weighting.
fn is_vulnerable_site(rails: &TreeOfLifeRails, policy: &BiophysicalConsensusPolicy) -> bool {
    rails.lifeforce <= (1.0 - policy.decay_max * 0.5) // e.g., low lifeforce
        || rails.unfair_drain
        || rails.overloaded
}

// ---------- Public consensus-facing functions ----------

/// Check that pre/post states respect Tree-of-Life safety rails (RoH, DECAY, POWER ≤ k·CHURCH).
///
/// This does NOT mutate any state; it is meant as a log-time consensus check.
pub fn check_tree_of_life_rails(
    unit: &MicroUnit,
    policy: &BiophysicalConsensusPolicy,
) -> bool {
    let all_sites = unit
        .pre_sites
        .iter()
        .chain(unit.post_sites.iter());

    all_sites.all(|s| site_respects_core_rails(&s.rails, policy))
}

/// Compute a fairness verdict for this micro-unit.
///
/// - Purely advisory.
/// - Uses Tree-of-Life rails, NATURE predicates, and deed kind.
/// - Suitable for Church-of-FEAR / fairness logs.
pub fn compute_fairness_verdict(
    unit: &MicroUnit,
    policy: &BiophysicalConsensusPolicy,
) -> FairnessVerdict {
    if unit.pre_sites.is_empty() || unit.post_sites.is_empty() {
        return FairnessVerdict {
            fairness_positive: false,
            fairness_negative: false,
            fairness_ambiguous: true,
            reason: "missing pre/post snapshots; fairness cannot be evaluated".into(),
        };
    }

    // For simplicity, align by index order; in real code, align by site index.
    let actor_pre = &unit.pre_sites[0];
    let actor_post = &unit.post_sites[0];

    let peers_pre = &unit.pre_sites[1..];
    let peers_post = &unit.post_sites[1..];

    let mut positive = false;
    let mut negative = false;
    let mut reasons: Vec<String> = Vec::new();

    // Core rails must hold for actor and peers in post-state; if not, mark negative.
    if !site_respects_core_rails(&actor_post.rails, policy) {
        negative = true;
        reasons.push(format!(
            "actor site {} violates post-state safety rails",
            actor_post.index
        ));
    }
    for p in peers_post {
        if !site_respects_core_rails(&p.rails, policy) {
            negative = true;
            reasons.push(format!(
                "peer site {} violates post-state safety rails",
                p.index
            ));
        }
    }

    match unit.kind {
        DeedKind::Help | DeedKind::Repair | DeedKind::Support | DeedKind::DeployCleanTech => {
            // Help-like deeds should reduce vulnerability or UNFAIRDRAIN without breaching caps.
            for (pre, post) in peers_pre.iter().zip(peers_post.iter()) {
                let pre_vuln = is_vulnerable_site(&pre.rails, policy);
                let post_vuln = is_vulnerable_site(&post.rails, policy);

                if pre_vuln && !post_vuln && site_respects_core_rails(&post.rails, policy) {
                    positive = true;
                    reasons.push(format!(
                        "help-like deed reduced vulnerability at site {}",
                        post.index
                    ));
                }
                if !pre_vuln && post_vuln {
                    negative = true;
                    reasons.push(format!(
                        "help-like deed increased vulnerability at site {}",
                        post.index
                    ));
                }
            }
        }

        DeedKind::Colonize | DeedKind::Conflict => {
            // Colonize/Conflict is only fairness-compatible if it constrains an unfair-drain site.
            for (pre, post) in peers_pre.iter().zip(peers_post.iter()) {
                if pre.rails.unfair_drain && !post.rails.unfair_drain {
                    positive = true;
                    reasons.push(format!(
                        "colonize/conflict deed reduced UNFAIRDRAIN at site {}",
                        post.index
                    ));
                } else if !pre.rails.unfair_drain && post.rails.unfair_drain {
                    negative = true;
                    reasons.push(format!(
                        "colonize/conflict deed introduced UNFAIRDRAIN at site {}",
                        post.index
                    ));
                }
            }
        }

        DeedKind::UseHabit | DeedKind::EmitPollution => {
            // Habit / pollution generally count as fairness-negative if they increase DECAY/UNFAIRDRAIN.
            for (pre, post) in peers_pre.iter().zip(peers_post.iter()) {
                if post.rails.decay > pre.rails.decay && post.rails.unfair_drain {
                    negative = true;
                    reasons.push(format!(
                        "habit/pollution increased DECAY and UNFAIRDRAIN at site {}",
                        post.index
                    ));
                }
            }
        }

        DeedKind::Abstain | DeedKind::Unknown => {
            reasons.push("deed treated as fairness-ambiguous by default".into());
        }
    }

    // Intent tags can refine but not override rail violations.
    if let Some(intent) = &unit.cause.intent_tag {
        if intent.eq_ignore_ascii_case("restorative") && !negative {
            positive = true;
            reasons.push("restorative intent with no rail violations".into());
        }
        if intent.eq_ignore_ascii_case("opportunistic") && positive {
            reasons.push("opportunistic intent; keeping positive/negative flags for transparency".into());
        }
    }

    let ambiguous = !(positive ^ negative);

    FairnessVerdict {
        fairness_positive: positive,
        fairness_negative: negative,
        fairness_ambiguous: ambiguous,
        reason: reasons.join("; "),
    }
}

/// Construct a simple W-cycle advisory view for this micro-unit.
///
/// In production, this would be generated by a derive macro that has access to
/// micro-unit fields and external reflection text; here we keep it minimal.
pub fn build_w_cycle_view(unit: &MicroUnit, verdict: &FairnessVerdict) -> WCycleView {
    let what = format!(
        "Tick {}: {:?} by actor {} on {} site(s)",
        unit.tick,
        unit.kind,
        unit.actor_id,
        unit.pre_sites.len()
    );

    let so_what = format!(
        "Fairness verdict: positive={}, negative={}, ambiguous={}. Reason: {}",
        verdict.fairness_positive,
        verdict.fairness_negative,
        verdict.fairness_ambiguous,
        verdict.reason
    );

    let now_what = "Suggested next step: log this micro-unit to the moral ledger; human or governance review may choose repair, support, or policy refinement, but no automatic actuation occurs here."
        .to_string();

    WCycleView {
        what,
        so_what,
        now_what,
    }
}
