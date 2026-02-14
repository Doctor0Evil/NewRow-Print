//! Micro-unit fairness diagnostics for the Jetson-Line / Tree-of-Life stack.
//!
//! - Defines a DeedEvent / MicroUnit type bound to Tree-of-Life / NATURE views.
//! - Provides a pure check_tree_of_life_fairness(..) function that NEVER actuates.
//! - Intended to sit beside ReversalConditions: it labels deeds; it does not block them.

use serde::{Deserialize, Serialize};

/// Scalar rails in [0, 1] for a single site, projected from BiophysicalEnvelopeSpec
/// and Tree-of-Life views (RoH, DECAY, LIFEFORCE, FEAR, PAIN, POWER, CHURCH, etc.).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TreeOfLifeRails {
    /// Rate-of-Harm (RoH), already normalized and clamped.
    pub roh: f32,
    /// DECAY = RoH / 0.3, clamped to [0, 1].
    pub decay: f32,
    /// LIFEFORCE = 1 - DECAY, clamped to [0, 1].
    pub lifeforce: f32,
    /// FEAR asset in [0, 1].
    pub fear: f32,
    /// PAIN asset in [0, 1].
    pub pain: f32,
    /// POWER and CHURCH assets in [0, 1] (corridor-view, not wallet balances).
    pub power: f32,
    pub church: f32,
    /// UNFAIRDRAIN diagnostic flag for this site.
    pub unfair_drain: bool,
    /// CALM_STABLE / OVERLOADED / RECOVERY predicates.
    pub calm_stable: bool,
    pub overloaded: bool,
    pub recovery: bool,
}

/// Minimal deed kind set focused on fairness semantics.
/// Extend as needed; keep this enum #[non_exhaustive] in real code.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeedKind {
    Help,
    Repair,
    Support,
    DeployCleanTech,
    Colonize,
    Conflict,
    Abstain,
    Unknown,
}

/// Cause context: why the deed happened, as seen in the log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CauseContext {
    /// Optional short rule / policy identifier (e.g., "JUST_CAUSE_WINDOW").
    pub rule_id: Option<String>,
    /// Free-form explanatory tag (e.g., "defensive", "opportunistic").
    pub intent_tag: Option<String>,
}

/// Site snapshot for fairness analysis at one tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSnapshot {
    /// Index on the 1-D Jetson-Line.
    pub index: u32,
    /// Tree-of-Life scalar rails at this tick.
    pub rails: TreeOfLifeRails,
}

/// Fairness-focused judgement labels; this is advisory-only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessJudgement {
    /// True if the deed decreased expected harm / overload for vulnerable peers.
    pub fairness_positive: bool,
    /// True if the deed likely shifted load onto already drained peers.
    pub fairness_negative: bool,
    /// True if the deed is ethically ambiguous from a fairness perspective.
    pub fairness_ambiguous: bool,
    /// Human-readable explanation for logs and W-cycle reflections.
    pub rationale: String,
}

/// One micro-unit: the smallest fairness-complete slice of reality for a deed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeedEvent {
    /// Global tick on Jetson-Line / MicroSociety.
    pub tick: u64,
    /// Sites involved in the deed (actor and targets).
    pub sites: Vec<SiteSnapshot>,
    /// Deed kind (Help, Colonize, Repair, etc.).
    pub kind: DeedKind,
    /// Cause / rule context (defensive intent, last-resort window, etc.).
    pub cause: CauseContext,
    /// Optional pre/post flags for W-cycle binding; here we just store a stable id.
    pub w_cycle_id: Option<String>,
}

/// Fairness bands / thresholds for Tree-of-Life rails.
/// These are policy-configurable and live in config/ALN, not hard-coded doctrine.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FairnessPolicy {
    /// Maximum allowed RoH for a state to count as "not already overloaded".
    pub roh_safe_max: f32,
    /// Threshold under which LIFEFORCE is considered dangerously low.
    pub lifeforce_low_max: f32,
    /// Max FEAR for "safe to bear more load".
    pub fear_safe_max: f32,
    /// Multiplier k in POWER <= k * CHURCH.
    pub power_church_k: f32,
}

impl Default for FairnessPolicy {
    fn default() -> Self {
        Self {
            roh_safe_max: 0.30,
            lifeforce_low_max: 0.40,
            fear_safe_max: 0.60,
            power_church_k: 2.0,
        }
    }
}

/// Pure helper: check POWER <= k * CHURCH.
/// Uses corridor-view assets; returns true if the cap holds.
fn power_within_church_cap(rails: &TreeOfLifeRails, k: f32) -> bool {
    // If church is near-zero, we treat any non-zero power as suspect.
    if rails.church <= f32::EPSILON {
        return rails.power <= 0.0;
    }
    rails.power <= k * rails.church
}

/// Pure helper: classify sites as vulnerable (for fairness weighting).
fn is_vulnerable_site(rails: &TreeOfLifeRails, policy: &FairnessPolicy) -> bool {
    (rails.lifeforce <= policy.lifeforce_low_max)
        || rails.unfair_drain
        || rails.overloaded
}

/// Core fairness check: classify a DeedEvent under Tree-of-Life fairness rails.
///
/// This function:
/// - NEVER mutates capability or envelopes.
/// - ONLY labels the deed as fairness-positive / negative / ambiguous.
/// - Is suitable for use in observer layers (Church-of-FEAR, MicroSociety metrics, W-cycle).
pub fn check_tree_of_life_fairness(
    event: &DeedEvent,
    policy: &FairnessPolicy,
) -> FairnessJudgement {
    // Partition sites into "actor" (first index) and "peers" (rest).
    let mut fairness_positive = false;
    let mut fairness_negative = false;
    let mut rationale_parts: Vec<String> = Vec::new();

    if event.sites.is_empty() {
        return FairnessJudgement {
            fairness_positive: false,
            fairness_negative: false,
            fairness_ambiguous: true,
            rationale: "no sites attached to deed; fairness cannot be evaluated".to_string(),
        };
    }

    // Simplest assumption: first site is actor; others are peers/targets.
    let actor = &event.sites[0];
    let peers = &event.sites[1..];

    // Check Tree-of-Life caps for actor.
    if !power_within_church_cap(&actor.rails, policy.power_church_k) {
        fairness_negative = true;
        rationale_parts.push(format!(
            "actor site {} violates POWER <= k·CHURCH cap",
            actor.index
        ));
    }

    // Assess fairness based on deed kind and peer vulnerability.
    match event.kind {
        DeedKind::Help | DeedKind::Repair | DeedKind::Support | DeedKind::DeployCleanTech => {
            // Helping vulnerable peers while staying within caps is fairness-positive.
            let mut helped_vulnerable = false;
            for peer in peers {
                if is_vulnerable_site(&peer.rails, policy) {
                    helped_vulnerable = true;
                    if power_within_church_cap(&peer.rails, policy.power_church_k)
                        && peer.rails.roh <= policy.roh_safe_max
                    {
                        // Peer is vulnerable but not pushed beyond rails: good.
                        fairness_positive = true;
                        rationale_parts.push(format!(
                            "deed {:?} supports vulnerable site {} without breaching caps",
                            event.kind, peer.index
                        ));
                    } else {
                        fairness_negative = true;
                        rationale_parts.push(format!(
                            "deed {:?} touches vulnerable site {} at or beyond safety caps",
                            event.kind, peer.index
                        ));
                    }
                }
            }
            if !helped_vulnerable && peers.is_empty() {
                // Self-care deeds in overloaded states should not be penalized.
                if is_vulnerable_site(&actor.rails, policy) {
                    fairness_positive = true;
                    rationale_parts.push(
                        "self-directed help/repair on an overloaded actor site".to_string(),
                    );
                }
            }
        }

        DeedKind::Colonize | DeedKind::Conflict => {
            // Colonize / Conflict is only fairness-compatible if directed against
            // a segment that is *already* attacking or persistently draining peers,
            // and if post-state rails will remain inside corridor. Here we only
            // see pre-state; so we flag based on vulnerability + UNFAIRDRAIN.
            for peer in peers {
                if is_vulnerable_site(&peer.rails, policy) && !peer.rails.unfair_drain {
                    fairness_negative = true;
                    rationale_parts.push(format!(
                        "deed {:?} targets vulnerable non-draining site {}",
                        event.kind, peer.index
                    ));
                } else if peer.rails.unfair_drain {
                    fairness_positive = true;
                    rationale_parts.push(format!(
                        "deed {:?} targets unfair-drain site {} (defensive corridor)",
                        event.kind, peer.index
                    ));
                }
            }
        }

        DeedKind::Abstain | DeedKind::Unknown => {
            // Abstain / Unknown remains ambiguous; log rails but do not score.
            rationale_parts.push(format!(
                "deed {:?} treated as fairness-ambiguous; no scoring applied",
                event.kind
            ));
        }
    }

    // Intent tags can tip ambiguous cases but must not override caps.
    if let Some(intent) = &event.cause.intent_tag {
        if intent.eq_ignore_ascii_case("defensive") && fairness_negative && fairness_positive {
            rationale_parts.push(
                "intent=defensive; keeping both positive and negative flags for transparency"
                    .to_string(),
            );
        }
        if intent.eq_ignore_ascii_case("restorative") && !fairness_negative {
            fairness_positive = true;
            rationale_parts.push("intent=restorative with no cap violations".to_string());
        }
    }

    // Consolidate into a tri-state classification.
    let fairness_ambiguous = !(fairness_positive ^ fairness_negative);

    FairnessJudgement {
        fairness_positive,
        fairness_negative,
        fairness_ambiguous,
        rationale: rationale_parts.join("; "),
    }
}
