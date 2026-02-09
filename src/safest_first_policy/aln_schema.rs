use serde::{Serialize, Deserialize};

/// Directive NR-SAFE-0001 Compliance Note
/// This schema is a verifiable, non-hypothetical specification.
/// It defines the formal structure of the ALN policy engine.
/// All states, transitions, and constraints are implementable, auditable, and testable.

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    /// Simulation-only: models, proofs, algorithm design.
    /// No interaction with biological systems.
    ModelOnly,

    /// Lab bench: synthetic tissue, phantom models, non-biological rigs.
    /// Requires adherence to BASE_ENGINEERING standards.
    LabBench,

    /// Controlled human: bounded human studies with ethics and regulator oversight.
    ControlledHuman,

    /// General use: routine deployment under applicable regulation.
    GeneralUse,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ConsentState {
    /// No consent granted. All live coupling is denied.
    None,

    /// Minimal consent: non-invasive, low-risk observation only.
    Minimal,

    /// Extended consent: allows higher-intensity interaction under strict scope.
    Extended,

    /// Consent revoked. All live coupling must be halted.
    Revoked,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Learner,
    Teacher,
    Mentor,
    RegulatoryGuardian,
    Operator,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum JurisdictionTag {
    Fda,
    EuMdr,
    IsoIec60601_1,
    IsoIec60601_2_57,
    IsoIec60601_1_2,
    JurisLocal,
    QuantumAiSafety,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolicyStack {
    pub base_medical: Vec<JurisdictionTag>,
    pub base_engineering: Vec<JurisdictionTag>,
    pub juris_local: Vec<JurisdictionTag>,
    pub quantum_ai_safety: Vec<JurisdictionTag>,
}

impl PolicyStack {
    pub fn new() -> Self {
        Self {
            base_medical: vec![JurisdictionTag::Fda, JurisdictionTag::EuMdr],
            base_engineering: vec![
                JurisdictionTag::IsoIec60601_1,
                JurisdictionTag::IsoIec60601_1_2,
                JurisdictionTag::IsoIec60601_2_57,
            ],
            juris_local: vec![],
            quantum_ai_safety: vec![JurisdictionTag::QuantumAiSafety],
        }
    }

    /// Returns true if all mandatory components are present.
    pub fn is_satisfied(&self) -> bool {
        !self.base_medical.is_empty()
            && !self.base_engineering.is_empty()
            && !self.quantum_ai_safety.is_empty()
    }

    pub fn to_canonical_string(&self) -> String {
        format!(
            "BASE_MEDICAL: {:?} | BASE_ENGINEERING: {:?} | JURIS_LOCAL: {:?} | QUANTUM_AI_SAFETY: {:?}",
            self.base_medical, self.base_engineering, self.juris_local, self.quantum_ai_safety
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CapabilityTransition {
    pub from: CapabilityState,
    pub to: CapabilityState,
    /// Evidence identifiers (e.g., hashes, CIDs).
    pub required_evidence: Vec<String>,
    pub required_consent: ConsentState,
    pub required_roles: Vec<Role>,
    pub policy_stack: PolicyStack,
    /// Optional: temporal-logic property identifier.
    pub ltl_property: Option<String>,
}

impl CapabilityTransition {
    pub fn validate(&self) -> Result<(), String> {
        // 1. Enforce allowed graph (including rollbacks)
        match (self.from, self.to) {
            // ModelOnly
            (CapabilityState::ModelOnly, CapabilityState::ModelOnly) => {}
            (CapabilityState::ModelOnly, CapabilityState::LabBench) => {}
            (CapabilityState::ModelOnly, CapabilityState::ControlledHuman) => {
                return Err("Direct ModelOnly → ControlledHuman not permitted; must pass through LabBench.".to_string())
            }
            (CapabilityState::ModelOnly, CapabilityState::GeneralUse) => {
                return Err("Direct ModelOnly → GeneralUse not permitted; must pass through LabBench and ControlledHuman.".to_string())
            }

            // LabBench
            (CapabilityState::LabBench, CapabilityState::ModelOnly) => {} // rollback allowed
            (CapabilityState::LabBench, CapabilityState::LabBench) => {}
            (CapabilityState::LabBench, CapabilityState::ControlledHuman) => {}
            (CapabilityState::LabBench, CapabilityState::GeneralUse) => {
                return Err("Direct LabBench → GeneralUse not permitted; must pass through ControlledHuman.".to_string())
            }

            // ControlledHuman
            (CapabilityState::ControlledHuman, CapabilityState::ModelOnly) => {}
            (CapabilityState::ControlledHuman, CapabilityState::LabBench) => {}
            (CapabilityState::ControlledHuman, CapabilityState::ControlledHuman) => {}
            (CapabilityState::ControlledHuman, CapabilityState::GeneralUse) => {}

            // GeneralUse
            (CapabilityState::GeneralUse, CapabilityState::ModelOnly) => {}
            (CapabilityState::GeneralUse, CapabilityState::LabBench) => {}
            (CapabilityState::GeneralUse, CapabilityState::ControlledHuman) => {}
            (CapabilityState::GeneralUse, CapabilityState::GeneralUse) => {}

            _ => return Err("Invalid capability state transition.".to_string()),
        }

        // 2. Require evidence for any non-ModelOnly target
        if self.to != CapabilityState::ModelOnly && self.required_evidence.is_empty() {
            return Err("Evidence objects required for transition to non-ModelOnly state.".to_string());
        }

        // 3. Require consent for any non-ModelOnly target
        if self.to != CapabilityState::ModelOnly && self.required_consent == ConsentState::None {
            return Err("Consent cannot be None for transition to non-ModelOnly state.".to_string());
        }

        // 4. Require roles for ControlledHuman / GeneralUse
        if (self.to == CapabilityState::ControlledHuman || self.to == CapabilityState::GeneralUse)
            && self.required_roles.is_empty()
        {
            return Err("At least one role is required for transitions to ControlledHuman or GeneralUse.".to_string());
        }

        // 5. Policy stack must be structurally valid
        if !self.policy_stack.is_satisfied() {
            return Err("Policy stack not satisfied: missing BASE_MEDICAL, BASE_ENGINEERING, or QUANTUM_AI_SAFETY.".to_string());
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ALNPolicy {
    pub id: String,
    pub policy_stack: PolicyStack,
    pub transitions: Vec<CapabilityTransition>,
    /// Names/labels of prohibited harms.
    pub prohibited_harms: Vec<String>,
    pub default_capability: CapabilityState,
    pub default_consent: ConsentState,
    pub default_roles: Vec<Role>,
}

impl ALNPolicy {
    pub fn new() -> Self {
        Self {
            id: "policy-0001-2026".to_string(),
            policy_stack: PolicyStack::new(),
            transitions: vec![],
            prohibited_harms: vec![
                "coercive neuromodulation".to_string(),
                "non-consensual neural surveillance".to_string(),
                "emotional manipulation via neurostimulation".to_string(),
                "neuro-data monetization without explicit revocable consent".to_string(),
                "automated neuro-behavioral profiling".to_string(),
            ],
            default_capability: CapabilityState::ModelOnly,
            default_consent: ConsentState::None,
            default_roles: vec![Role::Learner],
        }
    }

    pub fn add_transition(&mut self, transition: CapabilityTransition) -> Result<(), String> {
        transition.validate()?;
        self.transitions.push(transition);
        Ok(())
    }

    /// Check if a concrete action is allowed, given current state, consent, and roles.
    /// NOTE: This is intentionally conservative and should be refined per-action later.
    pub fn is_action_permitted(
        &self,
        current_state: CapabilityState,
        consent: ConsentState,
        roles: &[Role],
        action_label: &str,
    ) -> bool {
        // 1. Hard prohibitions: block if action label matches any prohibited harm pattern.
        let action_lower = action_label.to_lowercase();
        if self
            .prohibited_harms
            .iter()
            .any(|h| action_lower.contains(&h.to_lowercase()))
        {
            return false;
        }

        // 2. ModelOnly: permit analysis/simulation actions only.
        if current_state == CapabilityState::ModelOnly {
            // For now, assume caller filters to simulation-only actions at this state.
            return true;
        }

        // 3. Non-ModelOnly: require at least Minimal consent.
        if consent == ConsentState::None || consent == ConsentState::Revoked {
            return false;
        }

        // 4. Require at least one role present (to be aligned with transition-level checks).
        if roles.is_empty() {
            return false;
        }

        true
    }

    pub fn valid_transitions_from(&self, from: CapabilityState) -> Vec<&CapabilityTransition> {
        self.transitions.iter().filter(|t| t.from == from).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_stack_satisfied() {
        let stack = PolicyStack::new();
        assert!(stack.is_satisfied());
    }

    #[test]
    fn test_policy_stack_not_satisfied() {
        let stack = PolicyStack {
            base_medical: vec![],
            base_engineering: vec![JurisdictionTag::IsoIec60601_1],
            juris_local: vec![],
            quantum_ai_safety: vec![JurisdictionTag::QuantumAiSafety],
        };
        assert!(!stack.is_satisfied());
    }

    #[test]
    fn test_capability_transition_valid() {
        let mut policy = ALNPolicy::new();
        let transition = CapabilityTransition {
            from: CapabilityState::ModelOnly,
            to: CapabilityState::LabBench,
            required_evidence: vec!["cid:QmZ4HHEJgpNmDcc4yfqPQUjpA8nkMpN2JuaKPfsZKscbqR".to_string()],
            required_consent: ConsentState::Minimal,
            required_roles: vec![Role::Teacher],
            policy_stack: PolicyStack::new(),
            ltl_property: Some("G (capability_state != controlled_human)".to_string()),
        };
        assert!(transition.validate().is_ok());
        policy.add_transition(transition).unwrap();
    }

    #[test]
    fn test_capability_transition_invalid_direct_model_to_controlled() {
        let transition = CapabilityTransition {
            from: CapabilityState::ModelOnly,
            to: CapabilityState::ControlledHuman,
            required_evidence: vec![],
            required_consent: ConsentState::Extended,
            required_roles: vec![Role::Teacher],
            policy_stack: PolicyStack::new(),
            ltl_property: None,
        };
        assert!(transition.validate().is_err());
    }

    #[test]
    fn test_action_permitted_model_only() {
        let policy = ALNPolicy::new();
        assert!(policy.is_action_permitted(
            CapabilityState::ModelOnly,
            ConsentState::None,
            &[],
            "simulation_only_analysis"
        ));
    }

    #[test]
    fn test_action_permitted_controlled_human_no_consent() {
        let policy = ALNPolicy::new();
        assert!(!policy.is_action_permitted(
            CapabilityState::ControlledHuman,
            ConsentState::None,
            &[Role::Learner],
            "live_coupling"
        ));
    }

    #[test]
    fn test_prohibited_harms_blocked() {
        let policy = ALNPolicy::new();
        assert!(!policy.is_action_permitted(
            CapabilityState::GeneralUse,
            ConsentState::Extended,
            &[Role::Learner],
            "coercive neuromodulation"
        ));
    }

    #[test]
    fn test_default_policy_structure() {
        let policy = ALNPolicy::new();
        assert_eq!(policy.default_capability, CapabilityState::ModelOnly);
        assert_eq!(policy.default_consent, ConsentState::None);
        assert_eq!(policy.default_roles.len(), 1);
        assert_eq!(policy.default_roles[0], Role::Learner);
    }
}
