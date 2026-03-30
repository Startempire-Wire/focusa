//! Agent Skills — docs/34-35
//!
//! 18 skills in 4 categories.
//! Prohibited list: agents CANNOT execute certain operations.
//! Skills are READ-ONLY inspection + guarded proposals.

use crate::types::*;

/// Skill registry.
#[derive(Debug, Default)]
pub struct SkillRegistry {
    pub skills: Vec<AgentSkill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: default_skills(),
        }
    }

    /// Register a custom skill.
    pub fn register(&mut self, skill: AgentSkill) -> Result<(), String> {
        // Check prohibited list.
        if PROHIBITED_SKILLS.contains(&skill.id.as_str()) {
            return Err(format!("Skill '{}' is prohibited", skill.id));
        }
        if self.skills.iter().any(|s| s.id == skill.id) {
            return Err(format!("Skill '{}' already registered", skill.id));
        }
        self.skills.push(skill);
        Ok(())
    }

    /// Enable/disable a skill.
    pub fn set_enabled(&mut self, skill_id: &str, enabled: bool) -> Result<(), String> {
        let skill = self
            .skills
            .iter_mut()
            .find(|s| s.id == skill_id)
            .ok_or_else(|| format!("Skill '{}' not found", skill_id))?;
        skill.enabled = enabled;
        Ok(())
    }

    /// Get all enabled skills.
    pub fn enabled_skills(&self) -> Vec<&AgentSkill> {
        self.skills.iter().filter(|s| s.enabled).collect()
    }

    /// Check if a skill is allowed.
    pub fn is_allowed(skill_id: &str) -> bool {
        !PROHIBITED_SKILLS.contains(&skill_id)
    }

    /// Get skills by category.
    pub fn by_category(&self, cat: SkillCategory) -> Vec<&AgentSkill> {
        self.skills.iter().filter(|s| s.category == cat).collect()
    }
}

/// Default 18 skills per spec.
fn default_skills() -> Vec<AgentSkill> {
    vec![
        // ─── Cognition Inspection (8 read-only) ─────────────────────
        skill(
            "inspect_focus",
            "Inspect Focus State",
            SkillCategory::CognitionInspection,
            "/v1/focus/state",
        ),
        skill(
            "inspect_stack",
            "Inspect Focus Stack",
            SkillCategory::CognitionInspection,
            "/v1/focus/stack",
        ),
        skill(
            "inspect_gate",
            "Inspect Focus Gate",
            SkillCategory::CognitionInspection,
            "/v1/gate/candidates",
        ),
        skill(
            "inspect_lineage",
            "Inspect CLT",
            SkillCategory::CognitionInspection,
            "/v1/clt/path",
        ),
        skill(
            "inspect_memory",
            "Inspect Memory",
            SkillCategory::CognitionInspection,
            "/v1/memory",
        ),
        skill(
            "inspect_constitution",
            "Inspect Constitution",
            SkillCategory::CognitionInspection,
            "/v1/constitution/active",
        ),
        skill(
            "inspect_autonomy",
            "Inspect Autonomy",
            SkillCategory::CognitionInspection,
            "/v1/autonomy",
        ),
        skill(
            "inspect_uxp",
            "Inspect UXP/UFI",
            SkillCategory::CognitionInspection,
            "/v1/uxp",
        ),
        // ─── Telemetry Metrics (4 read-only) ────────────────────────
        skill(
            "metrics_tokens",
            "Token Usage Metrics",
            SkillCategory::TelemetryMetrics,
            "/v1/telemetry/tokens",
        ),
        skill(
            "metrics_cost",
            "Cost Metrics",
            SkillCategory::TelemetryMetrics,
            "/v1/telemetry/cost",
        ),
        skill(
            "metrics_efficiency",
            "Efficiency Metrics",
            SkillCategory::TelemetryMetrics,
            "/v1/telemetry/efficiency",
        ),
        skill(
            "metrics_session",
            "Session Metrics",
            SkillCategory::TelemetryMetrics,
            "/v1/telemetry/session",
        ),
        // ─── Explanation & Traceability (2 read-only) ───────────────
        skill(
            "explain_decision",
            "Explain Decision",
            SkillCategory::ExplanationTraceability,
            "/v1/explain/decision",
        ),
        skill(
            "trace_lineage",
            "Trace Lineage Path",
            SkillCategory::ExplanationTraceability,
            "/v1/clt/trace",
        ),
        // ─── Proposal Request (4 guarded) ───────────────────────────
        skill(
            "propose_focus",
            "Propose Focus Change",
            SkillCategory::ProposalRequest,
            "/v1/proposals/focus",
        ),
        skill(
            "propose_thesis",
            "Propose Thesis Update",
            SkillCategory::ProposalRequest,
            "/v1/proposals/thesis",
        ),
        skill(
            "propose_autonomy",
            "Propose Autonomy Adjustment",
            SkillCategory::ProposalRequest,
            "/v1/proposals/autonomy",
        ),
        skill(
            "propose_constitution",
            "Propose Constitution Revision",
            SkillCategory::ProposalRequest,
            "/v1/proposals/constitution",
        ),
    ]
}

fn skill(id: &str, name: &str, category: SkillCategory, endpoint: &str) -> AgentSkill {
    AgentSkill {
        id: id.into(),
        name: name.into(),
        category,
        description: name.into(),
        api_endpoint: endpoint.into(),
        permission_class: match category {
            SkillCategory::CognitionInspection
            | SkillCategory::TelemetryMetrics
            | SkillCategory::ExplanationTraceability => PermissionClass::Read,
            SkillCategory::ProposalRequest => PermissionClass::Command,
        },
        enabled: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_18_skills() {
        let registry = SkillRegistry::new();
        assert_eq!(registry.skills.len(), 18);
    }

    #[test]
    fn test_prohibited_skill_blocked() {
        let mut registry = SkillRegistry::new();
        let skill = AgentSkill {
            id: "set_focus_state".into(),
            name: "Prohibited".into(),
            category: SkillCategory::CognitionInspection,
            description: "Should fail".into(),
            api_endpoint: "/bad".into(),
            permission_class: PermissionClass::Command,
            enabled: true,
        };
        assert!(registry.register(skill).is_err());
    }

    #[test]
    fn test_enable_disable() {
        let mut registry = SkillRegistry::new();
        registry.set_enabled("inspect_focus", false).unwrap();
        assert_eq!(registry.enabled_skills().len(), 17);
    }
}
