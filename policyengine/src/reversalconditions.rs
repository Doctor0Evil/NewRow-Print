// ReversalConditions kernel for NewRow-Print!
//
// Purpose:
// - Enforce neuromorph evolution monotone by default.
// - Allow downgrade/reversal ONLY when:
//   * allow_neuromorph_reversal == true (Tier-1 policy flag),
//   * NeuromorphSovereign (NEUROMORPH_GOD) quorum is satisfied,
//   * explicit_reversal_order == true (typed owner/quorum order),
//   * no_safer_alternative == true (Tier-2 envelopes exhausted all soft mitigations),
//   * PolicyStack passes, consent is valid, and RoH invariants hold.
//
// This module is pure and side-effect free: it does not write logs,
// mutate capability state, or touch hardware. It only returns a Decision.

use serde::{Deserialize, Serialize};

use crate::alncore::{
    CapabilityState,
    ConsentState,
    Decision,
    DecisionReason,
    Jurisdiction,
    PolicyStack,
    Role,
    CapabilityTransitionRequest,
};

/// Flags coming from the ALN SECTION,REVERSAL-POLICY shard.
/// These are already defined at the ALN level; this struct is the Rust mirror.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReversalPolicyFlags {
    /// Global, non-waivable Tier-1 flag:
    /// when false, neuromorph evolution downgrades are forbidden in this tier.
    pub allow_neuromorph_reversal: bool,
    /// Explicit, typewritten / quorum-signed owner order in .stake.aln / consent ledger.
    pub explicit_reversal_order: bool,
    /// Derived by Tier-2 envelopes and Tree-of-Life evidence:
    /// true only if all non-reversal mitigations (tighten, pause, rest) failed
    /// and persistent risk / RoH≈0.3 has been observed.
    pub no_safer_alternative: bool,
}

/// Minimal view of envelope / Tree-of-Life context needed for ReversalConditions.
/// This keeps biophysical logic in its own module; here we consume only derived flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvelopeContextView {
    /// True if any active envelope recommends downgrade (requiresdowngrade).
    pub requires_downgrade: bool,
    /// True if a requestcapabilitydowngrade output has been formed
    /// (requires_downgrade && autodowngradeenabled && ownerdowngradeapproved).
    pub request_capability_downgrade: bool,
    /// True when neurodimensional balance is within envelopes (no multi-axis RISK, RoH < ceiling).
    pub balance_maintained: bool,
}

/// Role set wrapper so we can evaluate NeuromorphSovereign predicates cleanly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleSet {
    pub roles: Vec<Role>,
    /// Number of regulator roles required for quorum.
    pub required_regulator_quorum: u8,
}

impl RoleSet {
    fn has_role(&self, role: Role) -> bool {
        self.roles.iter().any(|r| *r == role)
    }

    fn count_role(&self, role: Role) -> u8 {
        self.roles.iter().filter(|r| **r == role).count() as u8
    }
}

/// NeuromorphSovereign / NEUROMORPH_GOD composite:
/// Host AND OrganicCpuOwner AND SovereignKernel AND regulator quorum.
pub fn neuromorph_god_satisfied(role_set: &RoleSet) -> bool {
    let has_host = role_set.has_role(Role::Host);
    let has_owner = role_set.has_role(Role::OrganicCpuOwner);
    let has_kernel = role_set.has_role(Role::SovereignKernel);
    let reg_count = role_set.count_role(Role::Regulator);

    has_host && has_owner && has_kernel && reg_count >= role_set.required_regulator_quorum
}

/// Helper: is this transition a neuromorph evolution downgrade?
/// By convention, evolution downgrades are those that reduce CapabilityState
/// along the main lattice used for human-coupled neuromorph capability.
fn is_neuromorph_evolution_downgrade(from: CapabilityState, to: CapabilityState) -> bool {
    // We treat the enum discriminants as ordered from least to most powerful,
    // consistent with alncore.rs: CapModelOnly < CapLabBench < CapControlledHuman < CapGeneralUse.
    (to as u8) < (from as u8)
}

/// Helper: basic consent checks for any live-coupling / downgrade operation.
fn consent_ok(consent: ConsentState) -> bool {
    use ConsentState::*;
    match consent {
        ConsentState::ConsentRevoked => false,
        ConsentState::ConsentNone => false,
        ConsentState::ConsentMinimal | ConsentState::ConsentExtended => true,
    }
}

/// Core ReversalConditions evaluation function.
///
/// Inputs:
/// - base: the original capability transition request (from, to, requester role, consent, PolicyStack, evidence).
/// - rev_flags: ALN reversal policy flags (allow_neuromorph_reversal, explicit_reversal_order, no_safer_alternative).
/// - roh_before / roh_after: scalar RoH values before and after the proposed transition.
/// - envelopectx: biophysical envelope context (requires_downgrade, request_capability_downgrade, balance_maintained).
/// - role_set: roles involved in the signed downgrade proposal (Host, OrganicCpuOwner, Regulator, SovereignKernel, etc.).
///
/// Output:
/// - Decision { allowed, reason } with explicit, audit-friendly reason codes.
pub fn evaluate_reversal(
    base: &CapabilityTransitionRequest,
    rev_flags: ReversalPolicyFlags,
    roh_before: f32,
    roh_after: f32,
    envelopectx: EnvelopeContextView,
    role_set: &RoleSet,
) -> Decision {
    use CapabilityState::*;
    use ConsentState::*;
    use DecisionReason::*;

    // 1. If this is NOT a downgrade, delegate to the normal capability engine.
    if !is_neuromorph_evolution_downgrade(base.from, base.to) {
        // For upgrades / same-level or non-evolution downgrades (e.g., bench-only),
        // keep existing logic in alncore.rs; ReversalConditions is not active here.
        return base.evaluate();
    }

    // 2. For neuromorph evolution downgrades, apply high-bar logic.

    // 2.1 PolicyStack must pass.
    if !base.policy_stack.all_pass() {
        return Decision::deny(DeniedPolicyStackFailure);
    }

    // 2.2 Consent must be at least minimal and not revoked.
    if !consent_ok(base.effective_consent) {
        return match base.effective_consent {
            ConsentRevoked => Decision::deny(DeniedConsentRevoked),
            ConsentNone => Decision::deny(DeniedInsufficientConsent),
            _ => Decision::deny(DeniedInsufficientConsent),
        };
    }

    // 2.3 RoH invariants: monotone and bounded by ceiling (0.30) in human-coupled tiers.
    // We assume roh_before/after are already derived from envelope axes with weights summing to 1.0.
    if roh_after > roh_before {
        // Downgrade path must not increase RoH.
        return Decision::deny(DeniedRoHInvariantViolation);
    }
    // A stricter check can enforce roh_after <= 0.30 when in CapControlledHuman or above:
    if matches!(base.from, CapControlledHuman | CapGeneralUse) && roh_after > 0.30 {
        return Decision::deny(DeniedRoHInvariantViolation);
    }

    // 2.4 Global Tier-1 reversal flag: if false, evolution downgrades are forbidden in this tier.
    if !rev_flags.allow_neuromorph_reversal {
        return Decision::deny(DeniedReversalNotAllowedInTier);
    }

    // 2.5 Sovereignty: NeuromorphSovereign / NEUROMORPH_GOD composite must be satisfied.
    if !neuromorph_god_satisfied(role_set) {
        return Decision::deny(DeniedIllegalDowngradeByNonRegulator);
    }

    // 2.6 Envelope / microspace: downgrade should be considered only if envelopes truly request it.
    // This respects the "freedom within microspace-distancing": envelopes can protect,
    // but ONLY recommend downgrades, never enforce them.
    if !envelopectx.requires_downgrade {
        // No strong biophysical recommendation; keep evolution monotone.
        return Decision::deny(DeniedNoSaferAlternativeNotProved);
    }

    // Request flag is optional as a stronger signal; if it exists, enforce it.
    if !envelopectx.request_capability_downgrade {
        // Biophysical layer has not passed the owner-gated downgrade request condition.
        return Decision::deny(DeniedNoSaferAlternativeNotProved);
    }

    // 2.7 Explicit owner/quorum order and biophysical "no safer alternative" proof.
    if !rev_flags.explicit_reversal_order || !rev_flags.no_safer_alternative {
        return Decision::deny(DeniedNoSaferAlternativeNotProved);
    }

    // 2.8 Final check: jurisdiction identifier may further restrict reversals in some regions.
    // This is a hook for JURISLOCAL / QUANTUMAISAFETY extensions; here we treat it as a no-op
    // except for placeholder audit reasons when future rules are added.
    match base.jurisdiction {
        Jurisdiction::GlobalBaseline
        | Jurisdiction::UsFda
        | Jurisdiction::EuMdr
        | Jurisdiction::LocalCustom => {
            // No extra denial here; jurisdictional predicates are encoded in PolicyStack.
        }
    }

    // If all checks pass, we allow the neuromorph evolution downgrade as
    // a last-resort, owner-quorum, biophysically justified action.
    Decision::allow()
}

// --------- Extended DecisionReason integration ---------
//
// NOTE: The core DecisionReason enum is defined in alncore.rs.
// It must be extended there with the following variants:
//
//   DeniedReversalNotAllowedInTier,
//   DeniedRoHInvariantViolation,
//   DeniedNoSaferAlternativeNotProved,
//   DeniedIllegalDowngradeByNonRegulator
//
// This module assumes those variants exist and uses them for explicit auditability.
//
// Example extension (to be placed in alncore.rs):
//
// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub enum DecisionReason {
//     Allowed,
//     DeniedInsufficientConsent,
//     DeniedConsentRevoked,
//     DeniedPolicyStackFailure,
//     DeniedMissingEvidence,
//     DeniedIllegalDowngradeByNonRegulator,
//     DeniedReversalNotAllowedInTier,
//     DeniedRoHInvariantViolation,
//     DeniedNoSaferAlternativeNotProved,
//     DeniedUnknown,
// }
//
// impl Decision {
//     pub fn allow() -> Self {
//         Decision { allowed: true, reason: DecisionReason::Allowed }
//     }
//
//     pub fn deny(reason: DecisionReason) -> Self {
//         Decision { allowed: false, reason }
//     }
// }
// -------------------------------------------------------
