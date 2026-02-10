use crate::alnroles::{RoleSet, can_revert_capability};
use crate::alncore::{
    CapabilityState,
    CapabilityTransitionRequest,
    Decision,
    DecisionReason,
    PolicyStack,
};
use crate::rohmodel::RoHScore;
use crate::envelope::EnvelopeContextView;
use crate::policyreversal::ReversalPolicyFlags;

/// Pure, side-effect-free context for evaluating neuromorph reversals.
///
/// This is the minimal state tuple the kernel needs, matching the
/// ALN SECTION,REVERSAL-POLICY + ROLE/ROLE-COMPOSITION surface and
/// BiophysicalEnvelopeSpec / RoH model.
#[derive(Debug, Clone)]
pub struct ReversalContext<'a> {
    /// The original capability transition request (from -> to, consent, etc.).
    pub base: &'a CapabilityTransitionRequest,

    /// Risk-of-Harm score before the proposed transition.
    pub roh_before: RoHScore,
    /// Risk-of-Harm score after the proposed transition (counterfactual / predicted).
    pub roh_after: RoHScore,

    /// Shard-level reversal policy flags frozen in ALN.
    pub reversal_flags: ReversalPolicyFlags,

    /// Active role set, including Host, OrganicCpuOwner, Regulator, SovereignKernel, etc.
    pub roles: &'a RoleSet,

    /// Pre-conjoined policy result: BASEMEDICAL ∧ BASEENGINEERING ∧ JURISLOCAL ∧ QUANTUMAISAFETY.
    pub policystack: &'a PolicyStack,

    /// View over envelope outputs for this subject/session.
    pub envelope_ctx: &'a EnvelopeContextView,

    /// Conservative boolean derived by biophysical logic + Tree-of-Life evidence.
    /// True only if all non-reversal mitigations (tighten, pause, rest) have been exhausted.
    pub no_safer_alternative: bool,
}

/// Evaluate a capability transition with respect to neuromorph evolution reversal.
///
/// This function is total and side-effect-free:
/// - It does not perform IO.
/// - It does not mutate capability, consent, roles, envelopes, or logs.
/// - It returns a Decision that callers can log into .evolve.jsonl / .donutloop.aln.
///
/// Invariants enforced:
/// - MODEL_ONLY / LAB_BENCH research is never blocked.
/// - RoH is monotone and ≤ 0.30 in CapControlledHuman, except when downgrade reduces RoH.
/// - Neuromorph evolution downgrades are forbidden by default:
///   only allowed if allowneuromorphreversal && can_revert_capability && policystack.ok().
pub fn evaluate_reversal(ctx: &ReversalContext<'_>) -> Decision {
    // 1. If this is not a downgrade in the neuromorph evolution lattice, allow and delegate
    //    to the base capability evaluator at a higher layer.
    if !is_neuromorph_downgrade(ctx.base.from, ctx.base.to) {
        return Decision {
            allowed: true,
            reason: DecisionReason::Allowed,
        };
    }

    // 2. RoH monotonicity / ceiling in CapControlledHuman.
    //
    // For CapControlledHuman, we enforce:
    // - roh_after.value <= roh_before.value (no hidden relaxation of biophysical strain), OR
    //   explicitly documented exception when the downgrade is *reducing* RoH.
    // - roh_after.value <= 0.30 as the neurorights ceiling.
    if matches!(ctx.base.from, CapabilityState::CapControlledHuman) {
        let before = ctx.roh_before.value;
        let after = ctx.roh_after.value;

        // If downgrade would *increase* RoH or exceed the ceiling, deny.
        if (after > before) && !reduces_capability_and_roh(ctx) {
            return Decision {
                allowed: false,
                reason: DecisionReason::DeniedRoHViolation,
            };
        }
        if after > 0.30 {
            return Decision {
                allowed: false,
                reason: DecisionReason::DeniedRoHViolation,
            };
        }
    }

    // 3. Deny neuromorph evolution downgrades by default at Tier-1:
    //    allowneuromorphreversal is a non-waivable false unless explicitly flipped in ALN.
    if !ctx.reversal_flags.allow_neuromorph_reversal {
        return Decision {
            allowed: false,
            reason: DecisionReason::DeniedReversalNotAllowedInTier,
        };
    }

    // 4. Sovereignty gate:
    //    Neuromorph evolution reversal is allowed only when the composite
    //    NEUROMORPH-GOD / NeuromorphSovereign predicate holds AND the shard-level
    //    canrevertcapability condition is satisfied.
    //
    // can_revert_capability encodes:
    // neuromorphgodsatisfied(roles, quorum) ∧ explicitreversalorder ∧ nosaferalternative
    let required_reg_quorum = ctx.reversal_flags.required_regulator_quorum();

    let can_revert = can_revert_capability(
        ctx.roles,
        required_reg_quorum,
        ctx.reversal_flags.explicit_reversal_order,
        ctx.no_safer_alternative,
    );

    if !can_revert {
        // Distinguish sovereignty vs. no-safer-alternative where possible.
        if !ctx.roles.neuromorph_god_satisfied(required_reg_quorum) {
            return Decision {
                allowed: false,
                reason: DecisionReason::DeniedIllegalDowngradeByNonRegulator,
            };
        }
        if !ctx.reversal_flags.explicit_reversal_order || !ctx.no_safer_alternative {
            return Decision {
                allowed: false,
                reason: DecisionReason::DeniedNoSaferAlternativeNotProved,
            };
        }
        // Fallback: generic consent / roles failure.
        return Decision {
            allowed: false,
            reason: DecisionReason::DeniedConsentOrRoles,
        };
    }

    // 5. PolicyStack gate:
    //    BASEMEDICAL ∧ BASEENGINEERING ∧ JURISLOCAL ∧ QUANTUMAISAFETY must all pass.
    if !ctx.policystack.all_pass() {
        return Decision {
            allowed: false,
            reason: DecisionReason::DeniedPolicyStackFailure,
        };
    }

    // 6. Envelope advisory context:
    //
    // EnvelopeContextView and Tree-of-Life diagnostics may mark
    // requires_downgrade / request_capability_downgrade, but they remain advisory
    // controllers. We *do not* allow them to bypass sovereignty or policy checks.
    //
    // Here we assert only that, if a downgrade is being granted, the envelope
    // context at least indicates that a downgrade is consistent with its own
    // recommendation; otherwise we deny as an illegal downgrade.
    if !ctx.envelope_ctx.request_capability_downgrade {
        return Decision {
            allowed: false,
            reason: DecisionReason::DeniedIllegalDowngradeByNonRegulator,
        };
    }

    // If all guards pass, this downgrade is allowed as a last-resort, sovereign, policy-checked
    // neuromorph evolution reversal, to be logged and attested in .evolve.jsonl / .donutloop.aln.
    Decision {
        allowed: true,
        reason: DecisionReason::Allowed,
    }
}

/// Helper: determine whether a transition is a neuromorph evolution downgrade
/// in the capability lattice. This is intentionally narrow: only downgrades
/// that reduce neuromorph evolution rights are treated as "high-stakes".
fn is_neuromorph_downgrade(from: CapabilityState, to: CapabilityState) -> bool {
    use CapabilityState::*;

    matches!(
        (from, to),
        (CapControlledHuman, CapLabBench)
            | (CapControlledHuman, CapModelOnly)
            | (CapGeneralUse, CapControlledHuman)
            | (CapGeneralUse, CapLabBench)
            | (CapGeneralUse, CapModelOnly)
    )
}

/// Helper: true when this downgrade reduces both capability tier and RoH,
/// allowing an exception to strict RoH monotonicity for safety-increasing
/// reversals. This keeps RoH "safe-first" while permitting emergency rollback.
fn reduces_capability_and_roh(ctx: &ReversalContext<'_>) -> bool {
    let from = ctx.base.from;
    let to = ctx.base.to;
    let before = ctx.roh_before.value;
    let after = ctx.roh_after.value;

    is_neuromorph_downgrade(from, to) && (after < before)
}
