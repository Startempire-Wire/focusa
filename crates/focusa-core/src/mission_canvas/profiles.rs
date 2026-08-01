use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceProfileDefinition {
    pub profile_id: String,
    pub revision: u64,
    pub display_name: String,
    pub candidate_contribution_ids: Vec<String>,
    pub density: String,
    pub terminology_registry_ref: String,
    pub renderer_registry_ref: String,
    pub domain_semantic_binding_registry_ref: Option<String>,
    pub viability_rule_revision: String,
    pub installed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActivityModeDefinition {
    pub activity_mode_id: String,
    pub revision: u64,
    pub display_name: String,
    pub candidate_contribution_ids: Vec<String>,
    pub terminology_overrides_ref: Option<String>,
    pub viability_rule_revision: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegistryDefinition {
    pub registry_kind: String,
    pub entry_id: String,
    pub revision: u64,
    pub schema_ref: String,
    pub payload_ref: String,
    pub required_capabilities: Vec<String>,
    pub required_permissions: Vec<String>,
    pub enabled: bool,
    pub payload: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DomainPack {
    pub pack_id: String,
    pub version: String,
    pub profile: WorkspaceProfileDefinition,
    pub activities: Vec<ActivityModeDefinition>,
    pub registry_entries: Vec<RegistryDefinition>,
}

#[derive(Clone, Debug, Default)]
pub struct CompositionRegistry {
    pub profiles: BTreeMap<String, WorkspaceProfileDefinition>,
    pub activities: BTreeMap<String, ActivityModeDefinition>,
    pub panels: BTreeMap<String, RegistryDefinition>,
    pub home_canvases: BTreeMap<String, RegistryDefinition>,
    pub work_surface_renderers: BTreeMap<String, RegistryDefinition>,
    pub artifact_renderers: BTreeMap<String, RegistryDefinition>,
    pub terminology: BTreeMap<String, RegistryDefinition>,
    pub domain_semantics: BTreeMap<String, RegistryDefinition>,
    installed_domain_packs: BTreeMap<String, DomainPack>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error("unknown workspace profile: {0}")]
    UnknownProfile(String),
    #[error("unknown activity mode: {0}")]
    UnknownActivity(String),
    #[error("domain pack id must use domain.<name> namespace")]
    InvalidDomainPackId,
    #[error("domain pack profile id must be namespaced by pack id")]
    InvalidDomainProfileId,
    #[error("domain pack already installed: {0}")]
    DomainPackAlreadyInstalled(String),
}

impl CompositionRegistry {
    pub fn builtin() -> Self {
        let mut registry = Self::default();
        for profile in builtin_profiles() {
            registry
                .profiles
                .insert(profile.profile_id.clone(), profile);
        }
        for activity in builtin_activities() {
            registry
                .activities
                .insert(activity.activity_mode_id.clone(), activity);
        }
        for entry in builtin_registry_entries() {
            let target = match entry.registry_kind.as_str() {
                "PanelRegistry" => &mut registry.panels,
                "HomeCanvasRegistry" => &mut registry.home_canvases,
                "WorkSurfaceRendererRegistry" => &mut registry.work_surface_renderers,
                "ArtifactRendererRegistry" => &mut registry.artifact_renderers,
                "TerminologyRegistry" => &mut registry.terminology,
                "DomainSemanticBindingRegistry" => &mut registry.domain_semantics,
                _ => continue,
            };
            target.insert(entry.entry_id.clone(), entry);
        }
        registry
    }

    pub fn compose_candidate_ids(
        &self,
        profile_id: &str,
        activity_mode_id: &str,
        available_contribution_ids: &BTreeSet<String>,
    ) -> Result<Vec<String>, RegistryError> {
        let profile = self
            .profiles
            .get(profile_id)
            .ok_or_else(|| RegistryError::UnknownProfile(profile_id.into()))?;
        let activity = self
            .activities
            .get(activity_mode_id)
            .ok_or_else(|| RegistryError::UnknownActivity(activity_mode_id.into()))?;
        let profile_ids = profile
            .candidate_contribution_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut candidates = activity
            .candidate_contribution_ids
            .iter()
            .filter(|id| profile_ids.contains(*id) && available_contribution_ids.contains(*id))
            .cloned()
            .collect::<Vec<_>>();
        candidates.sort();
        Ok(candidates)
    }

    pub fn viable_profiles(
        &self,
        activity_mode_id: &str,
        available_contribution_ids: &BTreeSet<String>,
    ) -> Vec<&WorkspaceProfileDefinition> {
        self.profiles
            .values()
            .filter(|profile| {
                profile.installed
                    && self
                        .compose_candidate_ids(
                            &profile.profile_id,
                            activity_mode_id,
                            available_contribution_ids,
                        )
                        .is_ok_and(|candidates| !candidates.is_empty())
            })
            .collect()
    }

    pub fn install_domain_pack(&mut self, pack: DomainPack) -> Result<(), RegistryError> {
        if !pack.pack_id.starts_with("domain.") {
            return Err(RegistryError::InvalidDomainPackId);
        }
        if !pack
            .profile
            .profile_id
            .starts_with(&format!("{}.", pack.pack_id))
        {
            return Err(RegistryError::InvalidDomainProfileId);
        }
        if self.installed_domain_packs.contains_key(&pack.pack_id) {
            return Err(RegistryError::DomainPackAlreadyInstalled(pack.pack_id));
        }
        self.profiles
            .insert(pack.profile.profile_id.clone(), pack.profile.clone());
        for activity in &pack.activities {
            self.activities
                .insert(activity.activity_mode_id.clone(), activity.clone());
        }
        for entry in &pack.registry_entries {
            if entry.registry_kind == "DomainSemanticBindingRegistry" {
                self.domain_semantics
                    .insert(entry.entry_id.clone(), entry.clone());
            }
        }
        self.installed_domain_packs
            .insert(pack.pack_id.clone(), pack);
        Ok(())
    }
}

fn profile(id: &str, display_name: &str, candidates: &[&str]) -> WorkspaceProfileDefinition {
    WorkspaceProfileDefinition {
        profile_id: id.into(),
        revision: 1,
        display_name: display_name.into(),
        candidate_contribution_ids: candidates.iter().map(|value| (*value).into()).collect(),
        density: "standard".into(),
        terminology_registry_ref: format!("registry:terminology:{id}"),
        renderer_registry_ref: "registry:renderer:builtin".into(),
        domain_semantic_binding_registry_ref: Some(format!("registry:semantics:{id}")),
        viability_rule_revision: "profile-viability:v1".into(),
        installed: true,
    }
}

fn activity(id: &str, display_name: &str, candidates: &[&str]) -> ActivityModeDefinition {
    ActivityModeDefinition {
        activity_mode_id: id.into(),
        revision: 1,
        display_name: display_name.into(),
        candidate_contribution_ids: candidates.iter().map(|value| (*value).into()).collect(),
        terminology_overrides_ref: Some(format!("registry:terminology:activity:{id}")),
        viability_rule_revision: "activity-viability:v1".into(),
    }
}

fn builtin_profiles() -> Vec<WorkspaceProfileDefinition> {
    vec![
        profile(
            "general",
            "General",
            &[
                "contribution:pi-session",
                "contribution:project-overview",
                "contribution:work-rail",
                "contribution:follow-up-queue",
                "contribution:controls",
            ],
        ),
        profile(
            "software",
            "Software Engineering",
            &[
                "contribution:pi-session",
                "contribution:project-overview",
                "contribution:context",
                "contribution:role",
                "contribution:interview",
                "contribution:spec",
                "contribution:tasks",
                "contribution:silent-sessions",
                "contribution:document",
                "contribution:research",
                "contribution:evidence",
                "contribution:history",
                "contribution:controls",
                "contribution:work-rail",
                "contribution:steering-queue",
                "contribution:follow-up-queue",
            ],
        ),
        profile(
            "legal",
            "Legal",
            &[
                "contribution:document",
                "contribution:research",
                "contribution:evidence",
                "contribution:authority-inspector",
                "contribution:history",
                "contribution:controls",
                "contribution:work-rail",
                "contribution:follow-up-queue",
            ],
        ),
        profile(
            "markets",
            "Markets",
            &[
                "contribution:market-overview",
                "contribution:research",
                "contribution:evidence",
                "contribution:risk-inspector",
                "contribution:history",
                "contribution:controls",
                "contribution:steering-queue",
            ],
        ),
        profile(
            "research",
            "Research",
            &[
                "contribution:project-overview",
                "contribution:document",
                "contribution:research",
                "contribution:evidence",
                "contribution:history",
                "contribution:controls",
                "contribution:work-rail",
            ],
        ),
        profile(
            "custom",
            "Custom",
            &["contribution:pi-session", "contribution:controls"],
        ),
    ]
}

fn builtin_activities() -> Vec<ActivityModeDefinition> {
    vec![
        activity(
            "overview",
            "Overview",
            &[
                "contribution:pi-session",
                "contribution:project-overview",
                "contribution:market-overview",
                "contribution:work-rail",
                "contribution:controls",
            ],
        ),
        activity(
            "context",
            "Context",
            &[
                "contribution:context",
                "contribution:authority-inspector",
                "contribution:risk-inspector",
            ],
        ),
        activity(
            "role",
            "Role",
            &["contribution:role", "contribution:authority-inspector"],
        ),
        activity(
            "interview",
            "Interview",
            &[
                "contribution:interview",
                "contribution:steering-queue",
                "contribution:follow-up-queue",
            ],
        ),
        activity(
            "spec",
            "Spec",
            &["contribution:spec", "contribution:evidence"],
        ),
        activity(
            "tasks",
            "Tasks / Work",
            &[
                "contribution:tasks",
                "contribution:work-rail",
                "contribution:steering-queue",
                "contribution:follow-up-queue",
            ],
        ),
        activity(
            "sessions",
            "Sessions",
            &["contribution:pi-session", "contribution:silent-sessions"],
        ),
        activity(
            "documents",
            "Documents",
            &["contribution:document", "contribution:authority-inspector"],
        ),
        activity(
            "research",
            "Research",
            &["contribution:research", "contribution:evidence"],
        ),
        activity(
            "evidence",
            "Evidence",
            &["contribution:evidence", "contribution:authority-inspector"],
        ),
        activity("history", "History", &["contribution:history"]),
        activity("controls", "Controls", &["contribution:controls"]),
    ]
}

fn registry_entry(kind: &str, id: &str, payload: Value) -> RegistryDefinition {
    RegistryDefinition {
        registry_kind: kind.into(),
        entry_id: id.into(),
        revision: 1,
        schema_ref: "registry-entry.schema.json".into(),
        payload_ref: format!("registry-entry:{id}@1"),
        required_capabilities: vec![],
        required_permissions: vec![],
        enabled: true,
        payload,
    }
}

fn builtin_registry_entries() -> Vec<RegistryDefinition> {
    vec![
        registry_entry(
            "PanelRegistry",
            "panel:focusa-inspector",
            json!({"contribution_id":"contribution:focusa-inspector"}),
        ),
        registry_entry(
            "PanelRegistry",
            "panel:work-rail",
            json!({"contribution_id":"contribution:work-rail"}),
        ),
        registry_entry(
            "HomeCanvasRegistry",
            "home:project-overview",
            json!({"contribution_id":"contribution:project-overview"}),
        ),
        registry_entry(
            "WorkSurfaceRendererRegistry",
            "renderer:pi-session@v1",
            json!({"kind":"pi_session"}),
        ),
        registry_entry(
            "WorkSurfaceRendererRegistry",
            "renderer:document@v1",
            json!({"kind":"document"}),
        ),
        registry_entry(
            "WorkSurfaceRendererRegistry",
            "renderer:research@v1",
            json!({"kind":"research"}),
        ),
        registry_entry(
            "WorkSurfaceRendererRegistry",
            "renderer:evidence@v1",
            json!({"kind":"evidence"}),
        ),
        registry_entry(
            "ArtifactRendererRegistry",
            "renderer:artifact:json@v1",
            json!({"media_type":"application/json"}),
        ),
        registry_entry(
            "ArtifactRendererRegistry",
            "renderer:artifact:markdown@v1",
            json!({"media_type":"text/markdown"}),
        ),
        registry_entry(
            "TerminologyRegistry",
            "terminology:general",
            json!({"work_item":"Work item","evidence":"Evidence"}),
        ),
        registry_entry(
            "TerminologyRegistry",
            "terminology:legal",
            json!({"work_item":"Matter","evidence":"Authority"}),
        ),
        registry_entry(
            "TerminologyRegistry",
            "terminology:markets",
            json!({"work_item":"Position","evidence":"Market evidence"}),
        ),
        registry_entry(
            "DomainSemanticBindingRegistry",
            "semantics:software",
            json!({"task":"provider_item","session":"agent_session"}),
        ),
        registry_entry(
            "DomainSemanticBindingRegistry",
            "semantics:legal",
            json!({"document":"matter_document","evidence":"authority"}),
        ),
        registry_entry(
            "DomainSemanticBindingRegistry",
            "semantics:markets",
            json!({"document":"instrument_note","evidence":"market_source"}),
        ),
        registry_entry(
            "DomainSemanticBindingRegistry",
            "semantics:research",
            json!({"document":"source","evidence":"finding"}),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_activity_cartesian_composition_is_deterministic_and_viable() {
        let registry = CompositionRegistry::builtin();
        let available = registry
            .profiles
            .values()
            .flat_map(|profile| profile.candidate_contribution_ids.iter().cloned())
            .collect::<BTreeSet<_>>();
        for profile in registry.profiles.values() {
            for activity in registry.activities.values() {
                let first = registry
                    .compose_candidate_ids(
                        &profile.profile_id,
                        &activity.activity_mode_id,
                        &available,
                    )
                    .unwrap();
                let second = registry
                    .compose_candidate_ids(
                        &profile.profile_id,
                        &activity.activity_mode_id,
                        &available,
                    )
                    .unwrap();
                assert_eq!(first, second);
            }
        }
        assert!(registry.viable_profiles("overview", &available).len() >= 5);
    }

    #[test]
    fn domain_pack_install_is_namespaced_and_idempotent() {
        let mut registry = CompositionRegistry::builtin();
        let pack = DomainPack {
            pack_id: "domain.healthcare".into(),
            version: "1.0.0".into(),
            profile: profile(
                "domain.healthcare.clinical",
                "Clinical",
                &["contribution:clinical-record"],
            ),
            activities: vec![activity(
                "domain.healthcare.review",
                "Clinical Review",
                &["contribution:clinical-record"],
            )],
            registry_entries: vec![registry_entry(
                "DomainSemanticBindingRegistry",
                "semantics:clinical",
                json!({"document":"clinical_record"}),
            )],
        };
        registry.install_domain_pack(pack.clone()).unwrap();
        assert!(registry.profiles.contains_key("domain.healthcare.clinical"));
        assert!(matches!(
            registry.install_domain_pack(pack),
            Err(RegistryError::DomainPackAlreadyInstalled(_))
        ));
    }
}
