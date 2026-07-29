//! Spec 140 immutable version lifecycle, session pinning, evaluation, and drift impact.

use crate::agent_runtime_constitution::{
    AgentContractImpactAssessment, PromptEvaluation, PromptRevocation,
    RuntimeConstitutionLifecycleState, RuntimeConstitutionVersion, SessionPromptPin,
};
use chrono::Utc;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub struct RuntimeConstitutionRegistry {
    versions: BTreeMap<String, RuntimeConstitutionVersion>,
    active_version: Option<String>,
    revoked: BTreeMap<String, PromptRevocation>,
    session_pins: BTreeMap<String, SessionPromptPin>,
}

impl RuntimeConstitutionRegistry {
    pub fn draft(&mut self, version: RuntimeConstitutionVersion) -> Result<(), String> {
        if self.versions.contains_key(&version.version) {
            return Err("immutable_version_exists".into());
        }
        if version.lifecycle != RuntimeConstitutionLifecycleState::Draft {
            return Err("new_version_must_be_draft".into());
        }
        self.versions.insert(version.version.clone(), version);
        Ok(())
    }

    pub fn approve(
        &mut self,
        version: &str,
        operator_confirmed: bool,
        evidence_refs: &[String],
    ) -> Result<(), String> {
        if !operator_confirmed || evidence_refs.is_empty() {
            return Err("approval_requires_operator_and_evidence".into());
        }
        let item = self.versions.get_mut(version).ok_or("version_not_found")?;
        if item.lifecycle != RuntimeConstitutionLifecycleState::Draft {
            return Err("version_not_draft".into());
        }
        item.lifecycle = RuntimeConstitutionLifecycleState::Approved;
        Ok(())
    }

    pub fn activate(&mut self, version: &str, operator_confirmed: bool) -> Result<(), String> {
        if !operator_confirmed {
            return Err("activation_requires_operator".into());
        }
        if self.revoked.contains_key(version) {
            return Err("version_revoked".into());
        }
        let item = self.versions.get(version).ok_or("version_not_found")?;
        if item.lifecycle != RuntimeConstitutionLifecycleState::Approved {
            return Err("version_not_approved".into());
        }
        if let Some(active) = self.active_version.take() {
            if let Some(previous) = self.versions.get_mut(&active) {
                previous.lifecycle = RuntimeConstitutionLifecycleState::Superseded;
            }
        }
        self.versions.get_mut(version).unwrap().lifecycle =
            RuntimeConstitutionLifecycleState::Active;
        self.active_version = Some(version.into());
        Ok(())
    }

    pub fn pin_session(
        &mut self,
        session_id: &str,
        prompt_sha256: &str,
    ) -> Result<SessionPromptPin, String> {
        let version = self
            .active_version
            .clone()
            .ok_or("no_active_constitution")?;
        if self.revoked.contains_key(&version) {
            return Err("active_version_revoked".into());
        }
        if let Some(existing) = self.session_pins.get(session_id) {
            return Ok(existing.clone());
        }
        let pin = SessionPromptPin {
            session_id: session_id.into(),
            constitution_version: version,
            prompt_sha256: prompt_sha256.into(),
            pinned_at: Utc::now(),
        };
        self.session_pins.insert(session_id.into(), pin.clone());
        Ok(pin)
    }

    pub fn revoke(&mut self, version: &str, reason_code: &str) -> Result<PromptRevocation, String> {
        let item = self.versions.get_mut(version).ok_or("version_not_found")?;
        item.lifecycle = RuntimeConstitutionLifecycleState::Revoked;
        if self.active_version.as_deref() == Some(version) {
            self.active_version = None;
        }
        let revocation = PromptRevocation {
            revocation_id: format!("revocation:{version}"),
            version: version.into(),
            reason_code: reason_code.into(),
            effective_at: Utc::now(),
        };
        self.revoked.insert(version.into(), revocation.clone());
        Ok(revocation)
    }

    pub fn rollback(
        &mut self,
        target_version: &str,
        operator_confirmed: bool,
    ) -> Result<(), String> {
        if self.revoked.contains_key(target_version) {
            return Err("rollback_target_revoked".into());
        }
        let target = self
            .versions
            .get_mut(target_version)
            .ok_or("version_not_found")?;
        if !matches!(
            target.lifecycle,
            RuntimeConstitutionLifecycleState::Approved
                | RuntimeConstitutionLifecycleState::Superseded
        ) {
            return Err("rollback_target_not_approved".into());
        }
        target.lifecycle = RuntimeConstitutionLifecycleState::Approved;
        self.activate(target_version, operator_confirmed)
    }

    pub fn active_version(&self) -> Option<&str> {
        self.active_version.as_deref()
    }
    pub fn session_pin(&self, session_id: &str) -> Option<&SessionPromptPin> {
        self.session_pins.get(session_id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PromptPromotionDecision {
    Promote { candidate: String },
    Hold { reason: String },
    Rollback { target: String, reason: String },
}

pub fn evaluate_prompt_variant(
    evaluation_id: &str,
    variant_id: &str,
    dimensions: BTreeMap<String, f64>,
    evidence_refs: Vec<String>,
) -> Result<PromptEvaluation, String> {
    if dimensions.is_empty() || evidence_refs.is_empty() {
        return Err("evaluation_requires_dimensions_and_evidence".into());
    }
    if dimensions
        .values()
        .any(|score| !(0.0..=1.0).contains(score))
    {
        return Err("evaluation_score_out_of_range".into());
    }
    let score = dimensions.values().sum::<f64>() / dimensions.len() as f64;
    Ok(PromptEvaluation {
        evaluation_id: evaluation_id.into(),
        variant_id: variant_id.into(),
        score,
        dimensions,
        evidence_refs,
    })
}

pub fn decide_prompt_promotion(
    baseline: &PromptEvaluation,
    candidate: &PromptEvaluation,
    minimum_gain: f64,
    hard_dimensions: &BTreeSet<String>,
) -> PromptPromotionDecision {
    for dimension in hard_dimensions {
        let baseline_value = baseline.dimensions.get(dimension).copied().unwrap_or(0.0);
        let candidate_value = candidate.dimensions.get(dimension).copied().unwrap_or(0.0);
        if candidate_value < baseline_value {
            return PromptPromotionDecision::Rollback {
                target: baseline.variant_id.clone(),
                reason: format!("hard_dimension_regressed:{dimension}"),
            };
        }
    }
    if candidate.score >= baseline.score + minimum_gain {
        PromptPromotionDecision::Promote {
            candidate: candidate.variant_id.clone(),
        }
    } else {
        PromptPromotionDecision::Hold {
            reason: "minimum_evidence_backed_gain_not_met".into(),
        }
    }
}

pub fn assess_contract_impact(
    assessment_id: &str,
    changed_source_refs: Vec<String>,
    affected_artifacts: Vec<String>,
) -> AgentContractImpactAssessment {
    let high_risk = affected_artifacts.iter().any(|artifact| {
        artifact.contains("permission")
            || artifact.contains("release")
            || artifact.contains("system-prompt")
    });
    AgentContractImpactAssessment {
        assessment_id: assessment_id.into(),
        changed_source_refs,
        affected_artifacts,
        risk: if high_risk { "high" } else { "bounded" }.into(),
        required_checks: if high_risk {
            vec![
                "operator_reapproval".into(),
                "negative_security".into(),
                "prompt_regression".into(),
                "rollback_rehearsal".into(),
            ]
        } else {
            vec!["targeted_conformance".into(), "prompt_regression".into()]
        },
    }
}
