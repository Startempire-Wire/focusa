use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use super::model::{CandidateContribution, ContributionKind};

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
#[serde(deny_unknown_fields)]
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
    #[error("domain pack version must be non-empty")]
    InvalidDomainPackVersion,
    #[error("domain pack profile must expose at least one contribution")]
    EmptyDomainPackProfile,
    #[error("domain pack must expose at least one activity mode")]
    EmptyDomainPackActivities,
    #[error("domain pack activity must expose at least one contribution: {0}")]
    EmptyDomainPackActivity(String),
    #[error("domain pack has no viable profile/activity contribution")]
    NoViableDomainContribution,
    #[error("domain pack contains duplicate activity: {0}")]
    DuplicateDomainActivity(String),
    #[error("domain pack contains duplicate registry entry: {0}")]
    DuplicateDomainRegistryEntry(String),
    #[error("domain pack registry kind is unsupported: {0}")]
    UnsupportedDomainRegistryKind(String),
    #[error("domain pack registry entry is invalid: {0}")]
    InvalidDomainRegistryEntry(String),
    #[error("domain pack entry collides with an existing registry entry: {0}")]
    DomainPackEntryCollision(String),
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

    pub fn validate_domain_pack(&self, pack: &DomainPack) -> Result<(), RegistryError> {
        if !pack.pack_id.starts_with("domain.")
            || pack
                .pack_id
                .strip_prefix("domain.")
                .is_none_or(|suffix| suffix.is_empty())
        {
            return Err(RegistryError::InvalidDomainPackId);
        }
        if pack.version.trim().is_empty() {
            return Err(RegistryError::InvalidDomainPackVersion);
        }
        let profile_prefix = format!("{}.", pack.pack_id);
        if !pack.profile.profile_id.starts_with(&profile_prefix)
            || pack
                .profile
                .profile_id
                .strip_prefix(&profile_prefix)
                .is_none_or(|suffix| suffix.is_empty())
        {
            return Err(RegistryError::InvalidDomainProfileId);
        }
        if self.installed_domain_packs.contains_key(&pack.pack_id) {
            return Err(RegistryError::DomainPackAlreadyInstalled(
                pack.pack_id.clone(),
            ));
        }
        if pack.profile.candidate_contribution_ids.is_empty() {
            return Err(RegistryError::EmptyDomainPackProfile);
        }
        if pack.activities.is_empty() {
            return Err(RegistryError::EmptyDomainPackActivities);
        }
        let profile_candidates = pack
            .profile
            .candidate_contribution_ids
            .iter()
            .collect::<BTreeSet<_>>();
        let mut activity_ids = BTreeSet::new();
        let mut viable = false;
        let activity_prefix = format!("{}.", pack.pack_id);
        for activity in &pack.activities {
            if activity.activity_mode_id.trim().is_empty()
                || activity.display_name.trim().is_empty()
                || !activity.activity_mode_id.starts_with(&activity_prefix)
            {
                return Err(RegistryError::EmptyDomainPackActivity(
                    activity.activity_mode_id.clone(),
                ));
            }
            if !activity_ids.insert(activity.activity_mode_id.clone()) {
                return Err(RegistryError::DuplicateDomainActivity(
                    activity.activity_mode_id.clone(),
                ));
            }
            if activity.candidate_contribution_ids.is_empty() {
                return Err(RegistryError::EmptyDomainPackActivity(
                    activity.activity_mode_id.clone(),
                ));
            }
            viable |= activity
                .candidate_contribution_ids
                .iter()
                .any(|candidate| profile_candidates.contains(candidate));
            if self.activities.contains_key(&activity.activity_mode_id) {
                return Err(RegistryError::DomainPackEntryCollision(
                    activity.activity_mode_id.clone(),
                ));
            }
        }
        if !viable {
            return Err(RegistryError::NoViableDomainContribution);
        }
        if self.profiles.contains_key(&pack.profile.profile_id) {
            return Err(RegistryError::DomainPackEntryCollision(
                pack.profile.profile_id.clone(),
            ));
        }
        let mut entry_ids = BTreeSet::new();
        for entry in &pack.registry_entries {
            if entry.entry_id.trim().is_empty()
                || entry.schema_ref.trim().is_empty()
                || entry.payload_ref.trim().is_empty()
                || entry.payload.is_null()
            {
                return Err(RegistryError::InvalidDomainRegistryEntry(
                    entry.entry_id.clone(),
                ));
            }
            if !entry.enabled {
                return Err(RegistryError::InvalidDomainRegistryEntry(format!(
                    "{} is disabled",
                    entry.entry_id
                )));
            }
            if !entry_ids.insert(entry.entry_id.clone()) {
                return Err(RegistryError::DuplicateDomainRegistryEntry(
                    entry.entry_id.clone(),
                ));
            }
            if self.registry_contains(&entry.registry_kind, &entry.entry_id) {
                return Err(RegistryError::DomainPackEntryCollision(
                    entry.entry_id.clone(),
                ));
            }
            if self.registry_map(&entry.registry_kind).is_none() {
                return Err(RegistryError::UnsupportedDomainRegistryKind(
                    entry.registry_kind.clone(),
                ));
            }
        }
        Ok(())
    }

    pub fn install_domain_pack(&mut self, pack: DomainPack) -> Result<(), RegistryError> {
        self.validate_domain_pack(&pack)?;
        self.profiles
            .insert(pack.profile.profile_id.clone(), pack.profile.clone());
        for activity in &pack.activities {
            self.activities
                .insert(activity.activity_mode_id.clone(), activity.clone());
        }
        for entry in &pack.registry_entries {
            if let Some(target) = self.registry_map_mut(&entry.registry_kind) {
                target.insert(entry.entry_id.clone(), entry.clone());
            }
        }
        self.installed_domain_packs
            .insert(pack.pack_id.clone(), pack);
        Ok(())
    }

    fn registry_map(&self, registry_kind: &str) -> Option<&BTreeMap<String, RegistryDefinition>> {
        match registry_kind {
            "PanelRegistry" => Some(&self.panels),
            "HomeCanvasRegistry" => Some(&self.home_canvases),
            "WorkSurfaceRendererRegistry" => Some(&self.work_surface_renderers),
            "ArtifactRendererRegistry" => Some(&self.artifact_renderers),
            "TerminologyRegistry" => Some(&self.terminology),
            "DomainSemanticBindingRegistry" => Some(&self.domain_semantics),
            _ => None,
        }
    }

    fn registry_map_mut(
        &mut self,
        registry_kind: &str,
    ) -> Option<&mut BTreeMap<String, RegistryDefinition>> {
        match registry_kind {
            "PanelRegistry" => Some(&mut self.panels),
            "HomeCanvasRegistry" => Some(&mut self.home_canvases),
            "WorkSurfaceRendererRegistry" => Some(&mut self.work_surface_renderers),
            "ArtifactRendererRegistry" => Some(&mut self.artifact_renderers),
            "TerminologyRegistry" => Some(&mut self.terminology),
            "DomainSemanticBindingRegistry" => Some(&mut self.domain_semantics),
            _ => None,
        }
    }

    fn registry_contains(&self, registry_kind: &str, entry_id: &str) -> bool {
        self.registry_map(registry_kind)
            .is_some_and(|entries| entries.contains_key(entry_id))
    }
}

/// Select only installed profiles that can compose at least one meaningful
/// contribution in the exact activity mode and resolved projection supplied
/// by Core.  The eligible set is already capability/permission/content gated;
/// this function deliberately does not infer missing content or substitute a
/// placeholder profile.
pub fn meaningful_profiles_for_projection(
    profiles: &[WorkspaceProfileDefinition],
    activity: &ActivityModeDefinition,
    eligible_contribution_ids: &BTreeSet<String>,
) -> Vec<WorkspaceProfileDefinition> {
    let activity_contribution_ids = activity
        .candidate_contribution_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut viable = profiles
        .iter()
        .filter(|profile| {
            profile
                .candidate_contribution_ids
                .iter()
                .any(|candidate_id| {
                    activity_contribution_ids.contains(candidate_id)
                        && eligible_contribution_ids.contains(candidate_id)
                })
                && profile.installed
        })
        .cloned()
        .collect::<Vec<_>>();
    viable.sort_by(|left, right| left.profile_id.cmp(&right.profile_id));
    viable
}

/// Return only registered activity modes that can produce meaningful content
/// for the exact Core-resolved profile and projection.  Activity navigation is
/// a projection over canonical eligibility, not a second activity resolver:
/// an activity with no profile-compatible eligible contribution is omitted
/// rather than exposed as dead chrome or a placeholder choice.
pub fn meaningful_activities_for_projection(
    activities: &[ActivityModeDefinition],
    profile: &WorkspaceProfileDefinition,
    eligible_contribution_ids: &BTreeSet<String>,
) -> Vec<ActivityModeDefinition> {
    let profile_contribution_ids = profile
        .candidate_contribution_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut viable = activities
        .iter()
        .filter(|activity| {
            !activity.activity_mode_id.trim().is_empty()
                && !activity.display_name.trim().is_empty()
                && !activity.viability_rule_revision.trim().is_empty()
                && activity
                    .candidate_contribution_ids
                    .iter()
                    .any(|candidate_id| {
                        profile_contribution_ids.contains(candidate_id)
                            && eligible_contribution_ids.contains(candidate_id)
                    })
        })
        .cloned()
        .collect::<Vec<_>>();
    viable.sort_by(|left, right| left.activity_mode_id.cmp(&right.activity_mode_id));
    viable
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
                "contribution:steering-queue",
                "contribution:follow-up-queue",
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

/// Canonical candidate catalog for the builtin registry.  These are the
/// scope-neutral generated DTOs the resolver evaluates for a scope that has
/// no installed domain pack yet — a fresh Workstream must resolve its first
/// projection without requiring a prior install, otherwise the generated
/// resolve endpoint deadlocks (candidates only exist after a resolve that
/// needs candidates).
pub fn builtin_candidates() -> Vec<CandidateContribution> {
    let kind_of = |contribution_id: &str| -> ContributionKind {
        match contribution_id {
            "contribution:pi-session"
            | "contribution:project-overview"
            | "contribution:market-overview"
            | "contribution:context"
            | "contribution:role"
            | "contribution:interview"
            | "contribution:spec"
            | "contribution:tasks"
            | "contribution:silent-sessions"
            | "contribution:document"
            | "contribution:research"
            | "contribution:evidence"
            | "contribution:history" => ContributionKind::FocusedWorkSurface,
            "contribution:focusa-inspector"
            | "contribution:authority-inspector"
            | "contribution:risk-inspector" => ContributionKind::Inspector,
            "contribution:work-rail" => ContributionKind::WorkRail,
            "contribution:steering-queue" => ContributionKind::SteeringQueue,
            "contribution:follow-up-queue" => ContributionKind::FollowUpQueue,
            "contribution:prompt-editor" => ContributionKind::PromptEditor,
            "contribution:controls" => ContributionKind::ToolbarControl,
            _ => ContributionKind::FocusedWorkSurface,
        }
    };
    let renderer_of = |contribution_id: &str| -> String {
        let known = [
            ("contribution:pi-session", "renderer:pi-session@v1"),
            (
                "contribution:focusa-inspector",
                "renderer:focusa-inspector@v1",
            ),
            ("contribution:work-rail", "renderer:work-rail@v1"),
            ("contribution:document", "renderer:document@v1"),
            ("contribution:research", "renderer:research@v1"),
            ("contribution:evidence", "renderer:evidence@v1"),
        ];
        known
            .iter()
            .find(|(id, _)| *id == contribution_id)
            .map(|(_, renderer)| renderer.to_string())
            .unwrap_or_else(|| {
                format!(
                    "renderer:{}",
                    contribution_id.trim_start_matches("contribution:")
                )
            })
    };
    let mut ids = BTreeSet::new();
    for profile in builtin_profiles() {
        ids.extend(profile.candidate_contribution_ids);
    }
    for activity in builtin_activities() {
        ids.extend(activity.candidate_contribution_ids);
    }
    // Prompt Editor is a stable Mission Canvas lane. Exact target resolution
    // and operation availability still gate interaction; empty applicability
    // sets deliberately make the renderer available across all profiles/modes.
    ids.insert("contribution:prompt-editor".into());
    ids.into_iter()
        .map(|contribution_id| {
            let semantic = format!("semantic:{}", contribution_id.trim_start_matches("contribution:"));
            let data_ref_kind = match kind_of(&contribution_id) {
                ContributionKind::WorkRail => "work_rail",
                ContributionKind::SteeringQueue | ContributionKind::FollowUpQueue => "queue",
                ContributionKind::Inspector => "inspector",
                _ => "work_surface",
            };
            CandidateContribution {
                contribution_id: contribution_id.clone(),
                kind: kind_of(&contribution_id),
                semantic_binding_id: semantic,
                renderer_binding_id: renderer_of(&contribution_id),
                priority: 10,
                applicable_profile_ids: vec![],
                applicable_activity_mode_ids: vec![],
                canonical_content_refs: vec![json!({
                    "kind": data_ref_kind,
                    "ref": format!("surface:{}", contribution_id.trim_start_matches("contribution:")),
                    "revision": 1,
                    "freshness": "current",
                })],
                required_capabilities: vec![],
                required_permissions: vec![],
                required_operations: vec![],
                geometry: json!({
                    "preferred_regions": ["primary", "inspector"],
                    "minimum_span": 1,
                    "maximum_span": 12,
                    "preferred_order": 10,
                    "merge_policy": "compatible",
                    "tab_policy": "preferred",
                    "inspector_side": "profile_default",
                }),
            }
        })
        .collect()
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
            "renderer:work-rail@v1",
            json!({"kind":"work_rail","semantic_binding_id":"semantic:work-rail"}),
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
            "semantic:work-rail",
            json!({"object":"work_rail","renderer_binding_id":"renderer:work-rail@v1"}),
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
    fn meaningful_profile_listing_is_installed_activity_and_projection_bounded() {
        let activity = activity(
            "overview",
            "Overview",
            &["contribution:live", "contribution:shared"],
        );
        let mut unavailable = profile("unavailable", "Unavailable", &["contribution:missing"]);
        unavailable.installed = false;
        let profiles = vec![
            profile("shared", "Shared", &["contribution:shared"]),
            profile("live", "Live", &["contribution:live"]),
            profile("missing", "Missing", &["contribution:missing"]),
            unavailable,
        ];
        let eligible = BTreeSet::from(["contribution:live".to_owned()]);

        let viable = meaningful_profiles_for_projection(&profiles, &activity, &eligible);
        assert_eq!(
            viable
                .iter()
                .map(|profile| profile.profile_id.as_str())
                .collect::<Vec<_>>(),
            vec!["live"]
        );
        assert!(
            meaningful_profiles_for_projection(&profiles, &activity, &BTreeSet::new()).is_empty()
        );
    }

    #[test]
    fn meaningful_activity_listing_is_registered_profile_and_projection_bounded() {
        let profile = profile(
            "software",
            "Software Engineering",
            &["contribution:live", "contribution:shared"],
        );
        let activities = vec![
            activity(
                "tasks",
                "Tasks / Work",
                &["contribution:live", "contribution:missing"],
            ),
            activity("empty", "Empty", &["contribution:missing"]),
            activity("unbound", "Unbound", &["contribution:other"]),
            activity("shared", "Shared", &["contribution:shared"]),
        ];
        let eligible = BTreeSet::from(["contribution:live".to_owned()]);

        let viable = meaningful_activities_for_projection(&activities, &profile, &eligible);
        assert_eq!(
            viable
                .iter()
                .map(|activity| activity.activity_mode_id.as_str())
                .collect::<Vec<_>>(),
            vec!["tasks"]
        );
        assert!(
            meaningful_activities_for_projection(&activities, &profile, &BTreeSet::new())
                .is_empty()
        );
    }

    #[test]
    fn meaningful_activity_listing_omits_malformed_unregistered_shape() {
        let profile = profile("software", "Software Engineering", &["contribution:live"]);
        let activities = vec![
            ActivityModeDefinition {
                activity_mode_id: String::new(),
                revision: 1,
                display_name: "Unnamed".into(),
                candidate_contribution_ids: vec!["contribution:live".into()],
                terminology_overrides_ref: None,
                viability_rule_revision: "activity-viability:v1".into(),
            },
            ActivityModeDefinition {
                activity_mode_id: "missing-rule".into(),
                revision: 1,
                display_name: "Missing rule".into(),
                candidate_contribution_ids: vec!["contribution:live".into()],
                terminology_overrides_ref: None,
                viability_rule_revision: String::new(),
            },
        ];

        assert!(
            meaningful_activities_for_projection(
                &activities,
                &profile,
                &BTreeSet::from(["contribution:live".to_owned()]),
            )
            .is_empty()
        );
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
