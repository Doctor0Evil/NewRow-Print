use crate::aln_roles::{RoleSet, can_revert_capability};
use crate::aln_core::{
    CapabilityState,
    CapabilityTransitionRequest,
    Decision,
    DecisionReason,
    PolicyStack,
};
use crate::roh_model::RoHScore;
use crate::envelope::EnvelopeContextView;
use crate::policy_reversal::ReversalPolicyFlags;

#[derive(Debug, Clone)]
pub struct ReversalContext<'a> {
    pub base: &'a CapabilityTransitionRequest,
    pub cap_before: CapabilityState,
    pub cap_after: CapabilityState,
    pub roh_before: RoHScore,
    pub roh_after: RoHScore,
    pub reversal_flags: ReversalPolicyFlags,
    pub roles: &'a RoleSet,
    pub policy_stack: &'a PolicyStack,
    pub envelope_ctx: &'a EnvelopeContextView,
    pub is_diag_event: bool,
}

pub fn evaluate_reversal(ctx: &ReversalContext<'_>) -> Decision {
    // 1. Diagnostic isolation: diagnostics can never mutate capability.
    if ctx.is_diag_event && ctx.cap_after != ctx.cap_before {
        return Decision::Denied(
            DecisionReason::DeniedIllegalDowngradeByNonRegulator
        );
    }

    // 2. RoH invariants in CapControlledHuman: monotone & capped.
    if matches!(ctx.cap_before, CapabilityState::CapControlledHuman) {
        if ctx.roh_after.value > ctx.roh_before.value
            || ctx.roh_after.value > ctx.envelope_ctx.roh_ceiling()
        {
            return Decision::Denied(DecisionReason::DeniedRoHViolation);
        }
    }

    // 3. Check if this transition is a neuromorph-evolution downgrade.
    let is_evolution_downgrade = matches!(
        (ctx.cap_before, ctx.cap_after),
        (CapabilityState::CapControlledHuman, CapabilityState::CapLabBench)
            | (CapabilityState::CapControlledHuman, CapabilityState::CapModelOnly)
            | (CapabilityState::CapGeneralUse, CapabilityState::CapControlledHuman)
            | (CapabilityState::CapGeneralUse, CapabilityState::CapLabBench)
            | (CapabilityState::CapGeneralUse, CapabilityState::CapModelOnly)
    );

    // 4. Structural prohibition: all evolution downgrades are denied.
    if is_evolution_downgrade {
        return Decision::Denied(
            DecisionReason::DeniedIllegalDowngradeByNonRegulator
            // or a dedicated variant like:
            // DecisionReason::DeniedNeuromorphReversalProhibited
        );
    }

    // 5. Non-downgrade transitions fall through to the normal evaluator.
    // The ReversalConditions kernel is a guard; it never *allows* downgrades.
    crate::aln_core::evaluate_capability_transition(ctx.base, ctx.policy_stack)
}
