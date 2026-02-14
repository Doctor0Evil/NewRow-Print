pub mod reversalconditions {
    use crate::alncore::{CapabilityState, PolicyStack, RoleSet, Decision, DecisionReason};
    use crate::reversal_policy::ReversalPolicyFlags;
    use crate::envelope::EnvelopeContextView;

    // Sealing module
    mod sealed {
        pub trait Sealed {}
    }

    /// Read-only context passed into the kernel.
    pub struct ReversalContext<'a> {
        pub from: CapabilityState,
        pub to: CapabilityState,
        pub roh_before: f32,
        pub roh_after: f32,
        pub roles: &'a RoleSet,
        pub reversal_flags: &'a ReversalPolicyFlags,
        pub policystack: &'a PolicyStack,
        pub envelope_ctx: &'a EnvelopeContextView,
        pub nosaferalternative: bool,
    }

    pub trait ReversalEvaluator: sealed::Sealed {
        fn evaluate_reversal(&self, ctx: &ReversalContext) -> Decision;
    }

    pub struct KernelEvaluator;

    impl sealed::Sealed for KernelEvaluator {}

    impl ReversalEvaluator for KernelEvaluator {
        fn evaluate_reversal(&self, ctx: &ReversalContext) -> Decision {
            // 1) Non-neuromorph or non-downgrade transitions: delegate
            if !is_neuromorph_downgrade(ctx.from, ctx.to) {
                return Decision::Allowed;
            }

            // 2) RoH invariants in CapControlledHuman, except safety-improving rollback
            if matches!(ctx.from, CapabilityState::CapControlledHuman) {
                if !reduces_capability_and_roh(ctx) {
                    if ctx.roh_after > ctx.roh_before || ctx.roh_after > 0.30 {
                        return Decision::denied(DecisionReason::DeniedRoHViolation);
                    }
                }
            }

            // 3) Tier-1 flag: downgrades forbidden by default
            if !ctx.reversal_flags.allow_neuromorph_reversal {
                return Decision::denied(DecisionReason::DeniedReversalNotAllowedInTier);
            }

            // 4) Sovereign quorum and explicit order + no-safer-alternative
            if !ctx.roles.neuromorph_god_satisfied(ctx.reversal_flags.required_regulator_quorum) {
                return Decision::denied(DecisionReason::DeniedIllegalDowngradeByNonRegulator);
            }

            if !ctx.reversal_flags.explicit_reversal_order || !ctx.nosaferalternative {
                return Decision::denied(DecisionReason::DeniedNoSaferAlternativeNotProved);
            }

            // 5) PolicyStack gate
            if !ctx.policystack.all_pass() {
                return Decision::denied(DecisionReason::DeniedPolicyStackFailure);
            }

            // 6) Envelope recommendation must be consistent (advisory, not overriding)
            if !ctx.envelope_ctx.request_capability_downgrade {
                return Decision::denied(DecisionReason::DeniedIllegalDowngradeByNonRegulator);
            }

            Decision::Allowed
        }
    }

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

    fn reduces_capability_and_roh(ctx: &ReversalContext) -> bool {
        is_neuromorph_downgrade(ctx.from, ctx.to) && ctx.roh_after <= ctx.roh_before
    }
}
