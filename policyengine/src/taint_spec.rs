//! NewRow-Print! taint specification for policy-critical data.
//!
//! This module is generated/kept in sync with `policy/policy-taint-spec.aln`
//! and is consumed by an external static analyzer or lints. It does not
//! perform runtime checks in the hot path; instead, it encodes the allowed
//! writers/readers and banned patterns for CapabilityState, ReversalContext,
//! PolicyStack, and related types.

#![allow(dead_code)]

/// Marker attributes (expanded via a proc-macro crate in your build).
/// Here we declare them so they type-check in the core without depending
/// on the macro implementation.
pub use nr_taint_macros::{
    nr_taint_critical,        // #[nr_taint_critical]
    nr_taint_trusted_writer,  // #[nr_taint_trusted_writer]
    nr_taint_trusted_reader,  // #[nr_taint_trusted_reader]
    nr_taint_diag_join,       // #[nr_taint_diag_join]
};

/// Enumerates the fully-qualified names of policy-critical types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CriticalType {
    CapabilityState,
    CapabilityTransitionRequest,
    Decision,
    DecisionReason,
    PolicyStack,
    RoleSet,
    ReversalPolicyFlags,
    ReversalContext,
    RoHScore,
}

/// Allowed writers of critical types (pure kernels and state executor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrustedWriter {
    ReversalConditionsEvaluate,  // policyengine::reversalconditions::evaluate_reversal
    CapabilityTransitionEvaluate, // alncore::CapabilityTransitionRequest::evaluate
    CapabilityGuardApply,        // policyengine::capability_guard::apply_transition
    SovereignAuditRecord,        // sovereign_audit::record_decision
}

/// Allowed read-only consumers of critical types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrustedReader {
    EnvelopeModule,   // crate::envelope::*
    TreeOfLifeModule, // crate::treeoflife::*
    AutoChurchModule, // crate::autochurch::*
    NeuroprintModule, // crate::neuroprint::*
}

/// Banned language patterns around critical types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BannedPattern {
    UnsafeFn,
    RawPtr,
    FfiWrite,
    DynTraitCritical,
    GlobalMutable,
}

/// Diagnostic sources considered tainted.
/// They may only flow into `compute_no_safer_alternative`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticSource {
    TreeOfLifeView,
    TreeOfLifeDiagnostics,
    NeuroprintView,
    AutoChurchDiagnostics,
    EnvelopeContextView,
}

/// Single audited join point where diagnostics may influence
/// downgrade decisions by setting `nosaferalternative`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticJoinPoint {
    ComputeNoSaferAlternative,
}

#[derive(Debug, Clone)]
pub struct TaintPolicy {
    pub critical_types: &'static [CriticalType],
    pub trusted_writers: &'static [TrustedWriter],
    pub trusted_readers: &'static [TrustedReader],
    pub banned_patterns: &'static [BannedPattern],
    pub diagnostic_sources: &'static [DiagnosticSource],
    pub diagnostic_join: DiagnosticJoinPoint,
}

pub const TAINT_POLICY: TaintPolicy = TaintPolicy {
    critical_types: &[
        CriticalType::CapabilityState,
        CriticalType::CapabilityTransitionRequest,
        CriticalType::Decision,
        CriticalType::DecisionReason,
        CriticalType::PolicyStack,
        CriticalType::RoleSet,
        CriticalType::ReversalPolicyFlags,
        CriticalType::ReversalContext,
        CriticalType::RoHScore,
    ],
    trusted_writers: &[
        TrustedWriter::ReversalConditionsEvaluate,
        TrustedWriter::CapabilityTransitionEvaluate,
        TrustedWriter::CapabilityGuardApply,
        TrustedWriter::SovereignAuditRecord,
    ],
    trusted_readers: &[
        TrustedReader::EnvelopeModule,
        TrustedReader::TreeOfLifeModule,
        TrustedReader::AutoChurchModule,
        TrustedReader::NeuroprintModule,
    ],
    banned_patterns: &[
        BannedPattern::UnsafeFn,
        BannedPattern::RawPtr,
        BannedPattern::FfiWrite,
        BannedPattern::DynTraitCritical,
        BannedPattern::GlobalMutable,
    ],
    diagnostic_sources: &[
        DiagnosticSource::TreeOfLifeView,
        DiagnosticSource::TreeOfLifeDiagnostics,
        DiagnosticSource::NeuroprintView,
        DiagnosticSource::AutoChurchDiagnostics,
        DiagnosticSource::EnvelopeContextView,
    ],
    diagnostic_join: DiagnosticJoinPoint::ComputeNoSaferAlternative,
};

/// Convenience helpers for the static analyzer (invoked out-of-band).
impl TaintPolicy {
    /// Returns true if the given fully-qualified type path is policy-critical.
    pub fn is_critical_type(&self, fq_type: &str) -> bool {
        match fq_type {
            "crate::alncore::CapabilityState" => true,
            "crate::alncore::CapabilityTransitionRequest" => true,
            "crate::alncore::Decision" => true,
            "crate::alncore::DecisionReason" => true,
            "crate::alncore::PolicyStack" => true,
            "crate::alnroles::RoleSet" => true,
            "crate::policy::reversal::ReversalPolicyFlags" => true,
            "crate::policyengine::reversalconditions::ReversalContext" => true,
            "crate::rohmodel::RoHScore" => true,
            _ => false,
        }
    }

    /// Returns true if `fn_path` is an allowed writer of critical types.
    pub fn is_trusted_writer(&self, fn_path: &str) -> bool {
        match fn_path {
            "crate::policyengine::reversalconditions::evaluate_reversal" => true,
            "crate::alncore::CapabilityTransitionRequest::evaluate" => true,
            "crate::policyengine::capability_guard::apply_transition" => true,
            "crate::sovereign_audit::record_decision" => true,
            _ => false,
        }
    }

    /// Returns true if `module_path` is allowed to read but never write.
    pub fn is_trusted_reader_module(&self, module_path: &str) -> bool {
        module_path.starts_with("crate::envelope")
            || module_path.starts_with("crate::treeoflife")
            || module_path.starts_with("crate::autochurch")
            || module_path.starts_with("crate::neuroprint")
    }

    /// Returns true if a given function path is the diagnostic join point.
    pub fn is_diag_join_point(&self, fn_path: &str) -> bool {
        fn_path == "crate::policy::reversal::compute_no_safer_alternative"
    }
}

// ---- Attribute usage on core types (examples) -----------------------------

use crate::alncore::{
    CapabilityState,
    CapabilityTransitionRequest,
    Decision,
    DecisionReason,
    PolicyStack,
};
use crate::alnroles::RoleSet;
use crate::policy::reversal::ReversalPolicyFlags;
use crate::rohmodel::RoHScore;
use crate::policyengine::reversalconditions::ReversalContext;

/// Mark core types as taint-critical so the analyzer treats them specially.
#[nr_taint_critical]
type T_CapabilityState = CapabilityState;

#[nr_taint_critical]
type T_CapabilityTransitionRequest = CapabilityTransitionRequest;

#[nr_taint_critical]
type T_Decision = Decision;

#[nr_taint_critical]
type T_DecisionReason = DecisionReason;

#[nr_taint_critical]
type T_PolicyStack = PolicyStack;

#[nr_taint_critical]
type T_RoleSet = RoleSet;

#[nr_taint_critical]
type T_ReversalPolicyFlags = ReversalPolicyFlags;

#[nr_taint_critical]
type T_ReversalContext = ReversalContext;

#[nr_taint_critical]
type T_RoHScore = RoHScore;

/// Mark the pure downgrade kernel as a trusted writer.
#[nr_taint_trusted_writer]
pub fn _taint_marker_reversalconditions_evaluate() {
    // The actual implementation lives in policyengine::reversalconditions;
    // this stub exists only to anchor the attribute.
}

/// Mark the capability state machine as a trusted writer.
#[nr_taint_trusted_writer]
pub fn _taint_marker_capability_transition_evaluate() {}

/// Mark the capability executor as a trusted writer.
#[nr_taint_trusted_writer]
pub fn _taint_marker_capability_guard_apply() {}

/// Mark the diagnostic join point.
#[nr_taint_diag_join]
pub fn _taint_marker_compute_no_safer_alternative() {}

/// Mark diagnostic modules as trusted readers (advisory only).
#[nr_taint_trusted_reader]
pub mod treeoflife_reader_marker {}

#[nr_taint_trusted_reader]
pub mod envelope_reader_marker {}

#[nr_taint_trusted_reader]
pub mod neuroprint_reader_marker {}

#[nr_taint_trusted_reader]
pub mod autochurch_reader_marker {}
