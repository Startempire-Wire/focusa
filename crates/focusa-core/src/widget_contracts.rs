//! Governed widget contracts and the canonical widget catalog.
//!
//! Widgets are projections of registered Focusa primitives. This module owns
//! identity, surface, freshness, privacy, and mutation metadata only; query
//! execution remains in the daemon/API layer.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const WIDGET_DESCRIPTOR_SCHEMA: &str = "focusa.widget_descriptor.v1";
pub const WIDGET_QUERY_SCHEMA: &str = "focusa.widget_query.v1";
pub const WIDGET_PROJECTION_SCHEMA: &str = "focusa.widget_projection.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetSurface {
    Startpage,
    Sidepanel,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetSize {
    Compact,
    Wide,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetMutation {
    None,
    Governed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetPrivacyClass {
    PublicSafe,
    ProjectScoped,
    WorkstreamScoped,
    OrganizationScoped,
    Sensitive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetRefreshMode {
    OnRequest,
    EventPlusInterval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidgetRefreshPolicy {
    pub mode: WidgetRefreshMode,
    pub min_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidgetQueryContract {
    pub operation_id: String,
    pub request_schema_ref: String,
    pub response_schema_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidgetDescriptor {
    pub schema: String,
    pub widget_id: String,
    pub revision: u32,
    pub title: String,
    pub description: String,
    pub family: String,
    pub primitive_refs: Vec<String>,
    pub query: WidgetQueryContract,
    pub allowed_surfaces: Vec<WidgetSurface>,
    pub default_size: WidgetSize,
    pub supported_sizes: Vec<WidgetSize>,
    pub refresh_policy: WidgetRefreshPolicy,
    pub privacy_class: WidgetPrivacyClass,
    pub mutation: WidgetMutation,
    pub freshness_sla_ms: u64,
    pub fallback: String,
}

impl WidgetDescriptor {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != WIDGET_DESCRIPTOR_SCHEMA {
            return Err(format!("{} has unsupported schema", self.widget_id));
        }
        if !valid_identifier(&self.widget_id) || self.revision == 0 {
            return Err(format!("{} has invalid identity", self.widget_id));
        }
        if self.title.trim().is_empty() || self.description.trim().is_empty() {
            return Err(format!("{} requires title and description", self.widget_id));
        }
        if self.primitive_refs.is_empty() {
            return Err(format!("{} has no primitive refs", self.widget_id));
        }
        if self.query.operation_id.trim().is_empty()
            || self.query.request_schema_ref != WIDGET_QUERY_SCHEMA
            || self.query.response_schema_ref != WIDGET_PROJECTION_SCHEMA
        {
            return Err(format!("{} has invalid query contract", self.widget_id));
        }
        if self.allowed_surfaces.is_empty()
            || self.supported_sizes.is_empty()
            || !self.supported_sizes.contains(&self.default_size)
        {
            return Err(format!(
                "{} has invalid surface or size support",
                self.widget_id
            ));
        }
        if self.refresh_policy.min_interval_ms == 0 || self.freshness_sla_ms == 0 {
            return Err(format!("{} has invalid freshness policy", self.widget_id));
        }
        if self.mutation != WidgetMutation::None
            && self.allowed_surfaces.contains(&WidgetSurface::Wall)
        {
            return Err(format!("{} cannot expose mutation on wall", self.widget_id));
        }
        if self.fallback.trim().is_empty() {
            return Err(format!("{} requires a fallback", self.widget_id));
        }
        Ok(())
    }

    pub fn supports(&self, surface: WidgetSurface, size: WidgetSize) -> bool {
        self.allowed_surfaces.contains(&surface) && self.supported_sizes.contains(&size)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetProjectionStatus {
    Fresh,
    Stale,
    Degraded,
    Offline,
    Unauthorized,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidgetProjectionEnvelope<T> {
    pub schema: String,
    pub status: WidgetProjectionStatus,
    pub widget_id: String,
    pub widget_revision: u32,
    pub generated_at: String,
    pub fresh_until: String,
    pub source_revision: String,
    pub scope: String,
    pub data: T,
    pub evidence_refs: Vec<String>,
}

pub fn widget_catalog() -> Vec<WidgetDescriptor> {
    vec![
        descriptor(
            "focusa.focus.active",
            "Focus",
            "Current mission, objective, next action, and blockers.",
            "focus",
            vec!["focusa_trajectory_view", "focusa_workpoint_resume"],
            "focusa.widget.focus.active.read",
            vec![
                WidgetSurface::Startpage,
                WidgetSurface::Sidepanel,
                WidgetSurface::Wall,
            ],
            WidgetSize::Wide,
            WidgetPrivacyClass::WorkstreamScoped,
        ),
        descriptor(
            "focusa.workforce.roster",
            "Workforce",
            "Bounded agent lifecycle and health overview.",
            "workforce",
            vec!["focusa_silent_sessions", "focusa_work_loop_status"],
            "focusa.widget.workforce.roster.read",
            vec![
                WidgetSurface::Startpage,
                WidgetSurface::Sidepanel,
                WidgetSurface::Wall,
            ],
            WidgetSize::Wide,
            WidgetPrivacyClass::ProjectScoped,
        ),
        descriptor(
            "focusa.execution.workset",
            "Workset",
            "Requirement disposition and deterministic settlement summary.",
            "execution",
            vec!["focusa_workset_projection"],
            "focusa.widget.execution.workset.read",
            vec![
                WidgetSurface::Startpage,
                WidgetSurface::Sidepanel,
                WidgetSurface::Wall,
            ],
            WidgetSize::Wide,
            WidgetPrivacyClass::ProjectScoped,
        ),
        descriptor(
            "focusa.execution.callgraph",
            "Execution",
            "CallGraph run frontier, paths, joins, and settlement state.",
            "execution",
            vec!["focusa_callgraph_observe"],
            "focusa.widget.execution.callgraph.read",
            vec![WidgetSurface::Sidepanel, WidgetSurface::Wall],
            WidgetSize::Large,
            WidgetPrivacyClass::WorkstreamScoped,
        ),
        descriptor(
            "focusa.governance.activity",
            "Activity",
            "Recent bounded receipts, approvals, and completion signals.",
            "governance",
            vec!["focusa_silent_sessions", "focusa_evidence_capture"],
            "focusa.widget.governance.activity.read",
            vec![
                WidgetSurface::Startpage,
                WidgetSurface::Sidepanel,
                WidgetSurface::Wall,
            ],
            WidgetSize::Compact,
            WidgetPrivacyClass::ProjectScoped,
        ),
    ]
}

pub fn validate_widget_catalog(catalog: &[WidgetDescriptor]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for descriptor in catalog {
        descriptor.validate()?;
        if !ids.insert((&descriptor.widget_id, descriptor.revision)) {
            return Err(format!(
                "duplicate widget revision: {}",
                descriptor.widget_id
            ));
        }
    }
    Ok(())
}

fn descriptor(
    widget_id: &str,
    title: &str,
    description: &str,
    family: &str,
    primitive_refs: Vec<&str>,
    operation_id: &str,
    allowed_surfaces: Vec<WidgetSurface>,
    default_size: WidgetSize,
    privacy_class: WidgetPrivacyClass,
) -> WidgetDescriptor {
    WidgetDescriptor {
        schema: WIDGET_DESCRIPTOR_SCHEMA.to_owned(),
        widget_id: widget_id.to_owned(),
        revision: 1,
        title: title.to_owned(),
        description: description.to_owned(),
        family: family.to_owned(),
        primitive_refs: primitive_refs.into_iter().map(str::to_owned).collect(),
        query: WidgetQueryContract {
            operation_id: operation_id.to_owned(),
            request_schema_ref: WIDGET_QUERY_SCHEMA.to_owned(),
            response_schema_ref: WIDGET_PROJECTION_SCHEMA.to_owned(),
        },
        allowed_surfaces,
        default_size,
        supported_sizes: vec![WidgetSize::Compact, WidgetSize::Wide, WidgetSize::Large],
        refresh_policy: WidgetRefreshPolicy {
            mode: WidgetRefreshMode::EventPlusInterval,
            min_interval_ms: 5_000,
        },
        privacy_class,
        mutation: WidgetMutation::None,
        freshness_sla_ms: 30_000,
        fallback: "stale_snapshot_with_banner".to_owned(),
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '_'
                || character == '.'
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_catalog_is_grounded_and_valid() {
        let catalog = widget_catalog();
        assert_eq!(catalog.len(), 5);
        validate_widget_catalog(&catalog).expect("catalog must validate");
        assert!(
            catalog
                .iter()
                .all(|widget| widget.mutation == WidgetMutation::None)
        );
    }

    #[test]
    fn wall_support_is_explicit_and_mutation_is_rejected() {
        let mut widget = widget_catalog().remove(0);
        assert!(widget.supports(WidgetSurface::Wall, WidgetSize::Wide));
        widget.mutation = WidgetMutation::Governed;
        assert!(widget.validate().is_err());
    }

    #[test]
    fn duplicate_revisions_fail_closed() {
        let mut catalog = widget_catalog();
        catalog.push(catalog[0].clone());
        assert!(validate_widget_catalog(&catalog).is_err());
    }

    #[test]
    fn projection_status_does_not_collapse_offline_to_data() {
        let projection = WidgetProjectionEnvelope {
            schema: WIDGET_PROJECTION_SCHEMA.to_owned(),
            status: WidgetProjectionStatus::Offline,
            widget_id: "focusa.focus.active".to_owned(),
            widget_revision: 1,
            generated_at: "now".to_owned(),
            fresh_until: "past".to_owned(),
            source_revision: "ledger:1".to_owned(),
            scope: "project:test".to_owned(),
            data: Vec::<String>::new(),
            evidence_refs: Vec::new(),
        };
        assert_eq!(projection.status, WidgetProjectionStatus::Offline);
    }
}
