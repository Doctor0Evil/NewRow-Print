#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionReason {
    Allowed,
    DeniedInsufficientConsent,
    DeniedConsentRevoked,
    DeniedPolicyStackFailure,
    DeniedMissingEvidence,
    DeniedIllegalDowngradeByNonRegulator,
    DeniedNoSaferAlternativeNotProved,
    DeniedReversalNotAllowedInTier,
    DeniedRoHViolation,
    // New, explicit code for permanently disabled reversals:
    DeniedNeuromorphReversalProhibited,
    DeniedUnknown,
}
