use anyhow::{anyhow, bail, Result};
use organiccpualn::donutloopledger::{DonutloopEntry, DonutloopLedger};
use organiccpualn::evolvestream::EvolutionProposalRecord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Consent depth required by a SMART token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConsentState {
    ConsentMinimal,
    ConsentExtended,
}

/// Minimal shape of a SMART token policy entry from `.smart.json`.
/// You can expand this as your JSON schema finalizes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartTokenPolicy {
    pub token_id: String,
    pub subject_id: String,
    pub scope: String,
    pub max_effect_size_l2: f32,
    pub requires_consent_state: ConsentState,
    pub expiry_utc: String,
}

/// In‑memory index of SMART policies keyed by token_id.
#[derive(Debug, Clone, Default)]
pub struct SmartPolicyIndex {
    by_token: HashMap<String, SmartTokenPolicy>,
}

impl SmartPolicyIndex {
    pub fn new(policies: Vec<SmartTokenPolicy>) -> Self {
        let mut by_token = HashMap::new();
        for p in policies {
            by_token.insert(p.token_id.clone(), p);
        }
        SmartPolicyIndex { by_token }
    }

    pub fn get(&self, token_id: &str) -> Option<&SmartTokenPolicy> {
        self.by_token.get(token_id)
    }
}

/// Effective consent snapshot for a subject and scope, resolved from your
/// ALN consent ledger by higher‑level code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsentSnapshot {
    pub subject_id: String,
    pub scope: String,
    pub consent_state: ConsentState,
    pub revoked: bool,
}

/// Read‑only view that the guard uses. You can back this with an ALN
/// shard loader elsewhere in sovereigntycore.
pub trait ConsentResolver {
    fn resolve_consent(&self, subject_id: &str, scope: &str) -> Result<ConsentSnapshot>;
}

/// Guard decision codes – reuse your existing GuardDecision if you prefer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SmartGuardDecision {
    Allowed,
    Rejected(String),
}

/// Evaluate SMART token + consent for a proposal.
/// Assumes `proposal.token_id` is already present in EvolutionProposalRecord.
pub fn evaluate_smart_and_consent(
    proposal: &EvolutionProposalRecord,
    smart_policies: &SmartPolicyIndex,
    consent_resolver: &dyn ConsentResolver,
) -> SmartGuardDecision {
    // Only guard SMART tokens; EVOLVE is handled elsewhere.
    if proposal.token_kind != "SMART" {
        return SmartGuardDecision::Allowed;
    }

    let token_id = match &proposal.token_id {
        Some(tid) => tid,
        None => {
            return SmartGuardDecision::Rejected(
                "SMART token guard: missing token_id on SMART proposal".to_string(),
            )
        }
    };

    let policy = match smart_policies.get(token_id.as_str()) {
        Some(p) => p,
        None => {
            return SmartGuardDecision::Rejected(format!(
                "SMART token guard: unknown token_id {}",
                token_id
            ))
        }
    };

    // Scope and subject must match.
    if policy.scope != proposal.scope {
        return SmartGuardDecision::Rejected(format!(
            "SMART token guard: scope mismatch token={}, token_scope={}, proposal_scope={}",
            token_id, policy.scope, proposal.scope
        ));
    }
    if policy.subject_id != proposal.subject_id {
        return SmartGuardDecision::Rejected(format!(
            "SMART token guard: subject mismatch token={}, token_subject={}, proposal_subject={}",
            token_id, policy.subject_id, proposal.subject_id
        ));
    }

    // Effect size bound.
    if proposal.effect_bounds.l2_delta_norm > policy.max_effect_size_l2 + 1e-6 {
        return SmartGuardDecision::Rejected(format!(
            "SMART token guard: effect size {} exceeds max_effect_size_l2 {} for token {}",
            proposal.effect_bounds.l2_delta_norm, policy.max_effect_size_l2, token_id
        ));
    }

    // Resolve consent for this subject/scope.
    let consent = match consent_resolver.resolve_consent(&proposal.subject_id, &proposal.scope) {
        Ok(c) => c,
        Err(e) => {
            return SmartGuardDecision::Rejected(format!(
                "SMART token guard: failed to resolve consent: {}",
                e
            ))
        }
    };

    if consent.revoked {
        return SmartGuardDecision::Rejected(
            "SMART token guard: consent revoked for subject/scope".to_string(),
        );
    }

    // Required consent depth.
    match (policy.requires_consent_state.clone(), consent.consent_state.clone()) {
        (ConsentState::ConsentMinimal, ConsentState::ConsentMinimal)
        | (ConsentState::ConsentMinimal, ConsentState::ConsentExtended)
        | (ConsentState::ConsentExtended, ConsentState::ConsentExtended) => {
            // OK – consent depth sufficient.
        }
        (ConsentState::ConsentExtended, ConsentState::ConsentMinimal) => {
            return SmartGuardDecision::Rejected(
                "SMART token guard: requires ConsentExtended but only ConsentMinimal present"
                    .to_string(),
            );
        }
    }

    SmartGuardDecision::Allowed
}

/// Minimal rollback helper: when an already‑applied SMART change is later
/// discovered to violate consent, synthesize a compensating proposal and
/// apply it as a new ledger entry with lower RoH (monotone safety).
///
/// This does NOT mutate existing entries; it appends a corrective step.
pub fn synthesize_smart_rollback_entry(
    offending_entry: &DonutloopEntry,
    last_safe_entry: &DonutloopEntry,
    new_entry_id: &str,
    new_hexstamp: &str,
) -> Result<DonutloopEntry> {
    if offending_entry.subject_id != last_safe_entry.subject_id {
        bail!("rollback: subject_id mismatch between offending and last_safe entries");
    }

    // Enforce RoH monotonicity: rollback must not increase RoH relative to
    // last safe state; typically you set roh_after to last_safe.roh_after
    // or lower (tightening).
    if last_safe_entry.roh_after > offending_entry.roh_after + 1e-6 {
        bail!(
            "rollback: last_safe.roh_after ({}) is already lower than offending.roh_after ({}) – nothing to roll back",
            last_safe_entry.roh_after,
            offending_entry.roh_after
        );
    }

    let rollback_roh_after = last_safe_entry.roh_after;

    Ok(DonutloopEntry {
        entry_id: new_entry_id.to_string(),
        subject_id: offending_entry.subject_id.clone(),
        proposal_id: format!("rollback-{}", offending_entry.proposal_id),
        change_type: format!("rollback-{}", offending_entry.change_type),
        tsafe_mode: "Observe".to_string(),
        roh_before: offending_entry.roh_after,
        roh_after: rollback_roh_after,
        knowledge_factor: offending_entry.knowledge_factor,
        cybostate_factor: offending_entry.cybostate_factor,
        policy_refs: offending_entry.policy_refs.clone(),
        hexstamp: new_hexstamp.to_string(),
        timestamp_utc: chrono::Utc::now().to_rfc3339(),
        prev_hexstamp: offending_entry.hexstamp.clone(),
    })
}

/// Append a rollback entry to the ledger, enforcing existing ledger invariants.
pub fn append_rollback_to_ledger(
    ledger: &mut DonutloopLedger,
    rollback_entry: DonutloopEntry,
) -> Result<()> {
    ledger.append(rollback_entry)?;
    Ok(())
}

/// High‑level helper: given a consent‑violation detected after the fact,
/// construct and append a rollback entry that restores RoH toward the last
/// safe state without breaking the hash chain.
///
/// `ledger_tail_index` should point at the offending entry in the current
/// ledger; the last safe entry is typically the one immediately before it.
pub fn rollback_smart_violation(
    ledger: &mut DonutloopLedger,
    ledger_tail_index: usize,
    new_entry_id: &str,
    new_hexstamp: &str,
) -> Result<()> {
    let entries = ledger.entries();

    if ledger_tail_index == 0 || ledger_tail_index >= entries.len() {
        return Err(anyhow!("rollback: invalid ledger_tail_index {}", ledger_tail_index));
    }

    let offending_entry = &entries[ledger_tail_index];
    let last_safe_entry = &entries[ledger_tail_index - 1];

    let rollback = synthesize_smart_rollback_entry(
        offending_entry,
        last_safe_entry,
        new_entry_id,
        new_hexstamp,
    )?;
    append_rollback_to_ledger(ledger, rollback)?;
    Ok(())
}
