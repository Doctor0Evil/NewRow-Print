use serde::{Deserialize, Serialize};
use crate::alncore::{CapabilityState, Jurisdiction, PolicyStack, Decision, DecisionReason};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityGuardErrorKind {
    // Module / manifest problems
    UnknownModule,
    ManifestSchemaViolation,
    TierExceeded,
    ForbiddenTarget,

    // Grounding / dataset / standards problems
    MissingBiophysicalSourceId,
    MissingRegulatoryBasisId,
    MissingValidationEvidenceRef,
    UnverifiedBiophysicalArtifact,
    UnverifiedRegulatoryArtifact,
    UnverifiedValidationEvidence,

    // Policy / envelope / RoH problems
    PolicyStackNotSatisfied,
    EnvelopeMissing,
    EnvelopeViolation,
    RoHMonotonicityViolation,
    RoHCeilingExceeded,

    // Cryptographic / hash-chain / signature problems
    HashChainBroken,
    MissingRequiredSignatures,
    SignatureVerificationFailed,

    // Fallback
    InternalError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGuardError {
    pub kind: CapabilityGuardErrorKind,
    pub message: String,
}
