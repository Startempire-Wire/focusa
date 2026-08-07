use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::workstream_context::{WorkstreamContext, WorkstreamContextError};

use super::{
    layout::LayoutNode,
    model::MissionCanvasScope,
    persistence::{MissionCanvasStore, MissionCanvasStoreError},
    reducer::RecompositionReceipt,
};

pub const LAYOUT_MEMORY_UPDATE_OPERATION: &str = "focusa.mission_canvas.layout_memory.update";
pub const LAYOUT_MEMORY_UPDATE_PERMISSION: &str = "mission_canvas:write";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlacementMemory {
    pub contribution_id: String,
    pub preferred_regions: Vec<String>,
    pub preferred_order: u32,
    pub minimum_span: u8,
    pub maximum_span: u8,
    pub preferred_adjacency: Vec<String>,
    pub last_compatible_layout_node_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileLayoutMemory {
    pub memory_id: String,
    #[serde(flatten)]
    pub scope: MissionCanvasScope,
    pub profile_id: String,
    pub activity_mode_id: String,
    pub viewport_class: String,
    pub placements: Vec<PlacementMemory>,
    pub absent_contribution_ids: Vec<String>,
    pub focused_semantic_target: Option<String>,
    pub memory_revision: u64,
    pub idempotency_key: String,
    pub updated_at: String,
}

/// Validate a persisted layout-memory DTO before an API adapter exposes it to
/// a generated client.  Layout memory is keyed by the complete authority
/// context and by the profile/activity/viewport tuple; a Workstream-only
/// storage partition is not permission to return a subordinate or profile
/// belonging to another binding.
pub fn validate_profile_layout_memory(
    memory: &ProfileLayoutMemory,
    expected_scope: &MissionCanvasScope,
    expected_profile_id: &str,
    expected_activity_mode_id: &str,
    expected_viewport_class: &str,
) -> Result<(), &'static str> {
    validate_memory_scope(&memory.scope)?;
    validate_memory_scope(expected_scope)?;
    if memory.scope != *expected_scope {
        return Err("scope_mismatch");
    }
    if expected_profile_id.trim().is_empty() || memory.profile_id != expected_profile_id {
        return Err("profile_mismatch");
    }
    if expected_activity_mode_id.trim().is_empty()
        || memory.activity_mode_id != expected_activity_mode_id
    {
        return Err("activity_mode_mismatch");
    }
    if !is_viewport_class(expected_viewport_class)
        || memory.viewport_class != expected_viewport_class
    {
        return Err("viewport_class_mismatch");
    }
    if memory.memory_id
        != format!(
            "layout-memory:{}:{}:{}",
            memory.profile_id, memory.activity_mode_id, memory.viewport_class
        )
    {
        return Err("memory_id_mismatch");
    }
    if memory.idempotency_key.trim().is_empty() {
        return Err("idempotency_key_missing");
    }
    if DateTime::parse_from_rfc3339(&memory.updated_at).is_err() {
        return Err("updated_at_invalid");
    }
    if memory.placements.iter().any(|placement| {
        !is_contribution_id(&placement.contribution_id)
            || placement.preferred_regions.is_empty()
            || placement
                .preferred_regions
                .iter()
                .any(|region| !is_region(region))
            || placement
                .preferred_regions
                .iter()
                .enumerate()
                .any(|(index, region)| placement.preferred_regions[..index].contains(region))
            || placement
                .preferred_adjacency
                .iter()
                .any(|contribution_id| !is_contribution_id(contribution_id))
            || placement
                .preferred_adjacency
                .iter()
                .enumerate()
                .any(|(index, contribution_id)| {
                    placement.preferred_adjacency[..index].contains(contribution_id)
                })
            || placement.minimum_span == 0
            || placement.maximum_span == 0
            || placement.minimum_span > placement.maximum_span
            || placement.minimum_span > 12
            || placement.maximum_span > 12
    }) {
        return Err("placement_invalid");
    }
    let placement_ids = memory
        .placements
        .iter()
        .map(|placement| placement.contribution_id.as_str())
        .collect::<BTreeSet<_>>();
    if placement_ids.len() != memory.placements.len() {
        return Err("placement_duplicate");
    }
    let absent_ids = memory
        .absent_contribution_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if absent_ids.len() != memory.absent_contribution_ids.len()
        || memory
            .absent_contribution_ids
            .iter()
            .any(|contribution_id| !is_contribution_id(contribution_id))
    {
        return Err("absent_contribution_invalid");
    }
    if memory
        .placements
        .iter()
        .any(|placement| absent_ids.contains(placement.contribution_id.as_str()))
    {
        return Err("placement_absent_overlap");
    }
    Ok(())
}

/// A bounded mutation command for the generated layout-memory operation. The
/// request carries a complete ProfileLayoutMemory representation, while
/// `expected_memory_revision` comes only from the transport If-Match header.
/// Core decides the persisted revision and Receipt; neither is inferred by
/// Desktop or by a selected Work Surface.
#[derive(Clone, Debug)]
pub struct LayoutMemoryUpdateCommand {
    pub context: WorkstreamContext,
    pub scope: MissionCanvasScope,
    pub memory: ProfileLayoutMemory,
    pub expected_memory_revision: u64,
    pub permissions: BTreeSet<String>,
}

#[derive(Debug, Error)]
pub enum LayoutMemoryUpdateError {
    #[error("layout-memory Workstream context is invalid: {0}")]
    Context(#[from] WorkstreamContextError),
    #[error("layout-memory scope is invalid: {0}")]
    Scope(&'static str),
    #[error("layout-memory operation requires permission: {0}")]
    PermissionDenied(String),
    #[error("layout-memory operation requires a non-empty idempotency key")]
    IdempotencyKeyRequired,
    #[error("layout-memory payload is invalid: {0}")]
    MemoryInvalid(&'static str),
    #[error("layout-memory revision is stale: expected {expected}, submitted {submitted}")]
    RevisionConflict { expected: u64, submitted: u64 },
    #[error("layout-memory revision overflow")]
    RevisionOverflow,
    #[error("layout-memory serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("layout-memory persistence failed: {0}")]
    Store(#[from] MissionCanvasStoreError),
}

/// Core-owned implementation of `layout_memory.update`. It validates the
/// exact WorkstreamContext, semantic placement payload, permission, and
/// optimistic revision before delegating one atomic idempotent write to the
/// MissionCanvas store. It never resolves eligibility or composes a layout.
#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutMemoryUpdateService;

impl LayoutMemoryUpdateService {
    pub fn update(
        &self,
        store: &MissionCanvasStore,
        command: &LayoutMemoryUpdateCommand,
    ) -> Result<RecompositionReceipt, LayoutMemoryUpdateError> {
        validate_layout_memory_update_command(command)?;

        let submitted_revision = command.memory.memory_revision;
        if submitted_revision == 0 {
            return Err(LayoutMemoryUpdateError::MemoryInvalid(
                "memory_revision_invalid",
            ));
        }
        let next_revision = command
            .expected_memory_revision
            .checked_add(1)
            .ok_or(LayoutMemoryUpdateError::RevisionOverflow)?;
        // Accept both generated representations used by existing reducers:
        // the body may describe the current version (the server advances it),
        // or the next version already produced by reduce_layout_memory. In
        // both cases If-Match remains the sole expected stored revision.
        let persisted_revision = if submitted_revision == command.expected_memory_revision {
            next_revision
        } else if submitted_revision == next_revision {
            submitted_revision
        } else {
            return Err(LayoutMemoryUpdateError::RevisionConflict {
                expected: command.expected_memory_revision,
                submitted: submitted_revision,
            });
        };

        let request_digest = layout_memory_digest(&command.memory)?;
        let mut memory = command.memory.clone();
        memory.memory_revision = persisted_revision;
        memory.updated_at = Utc::now().to_rfc3339();
        store
            .update_layout_memory(
                &memory,
                command.expected_memory_revision,
                &request_digest,
                &command.context.authority.authority_ref,
            )
            .map_err(Into::into)
    }
}

fn validate_layout_memory_update_command(
    command: &LayoutMemoryUpdateCommand,
) -> Result<(), LayoutMemoryUpdateError> {
    command.context.validate()?;
    command
        .scope
        .validate()
        .map_err(LayoutMemoryUpdateError::Scope)?;
    validate_layout_memory_context(&command.context, &command.scope)?;
    if !has_layout_memory_permission(&command.permissions) {
        return Err(LayoutMemoryUpdateError::PermissionDenied(
            LAYOUT_MEMORY_UPDATE_PERMISSION.into(),
        ));
    }
    if command.memory.idempotency_key.trim().is_empty()
        || command.memory.idempotency_key.len() > 200
    {
        return Err(LayoutMemoryUpdateError::IdempotencyKeyRequired);
    }
    validate_profile_layout_memory(
        &command.memory,
        &command.scope,
        &command.memory.profile_id,
        &command.memory.activity_mode_id,
        &command.memory.viewport_class,
    )
    .map_err(LayoutMemoryUpdateError::MemoryInvalid)?;
    Ok(())
}

fn validate_layout_memory_context(
    context: &WorkstreamContext,
    scope: &MissionCanvasScope,
) -> Result<(), LayoutMemoryUpdateError> {
    if context.workstream != scope.workstream {
        return Err(WorkstreamContextError::WorkstreamMismatch.into());
    }
    let expected_continuity = scope.continuity_id.clone().or_else(|| {
        scope
            .attachment
            .as_ref()
            .and_then(|attachment| attachment.continuity_id.clone())
    });
    if context.continuity_id != expected_continuity {
        return Err(WorkstreamContextError::ContinuityMismatch.into());
    }
    if context.attachment != scope.attachment {
        return Err(WorkstreamContextError::WorkstreamMismatch.into());
    }
    let expected_binding = scope.workspace_binding_id.clone().or_else(|| {
        scope
            .attachment
            .as_ref()
            .map(|attachment| attachment.workspace_binding_id.clone())
    });
    if context.workspace_binding_id != expected_binding {
        return Err(WorkstreamContextError::WorkspaceBindingMismatch.into());
    }
    Ok(())
}

fn has_layout_memory_permission(permissions: &BTreeSet<String>) -> bool {
    permissions.contains(LAYOUT_MEMORY_UPDATE_PERMISSION)
        || permissions.contains("mission_canvas:*")
        || permissions.contains("admin:*")
        || permissions.contains("*")
}

/// Stable digest used by the Receipt and Evidence for one canonical memory
/// representation. It includes exact authority and semantic placement data;
/// it is not a client-side layout or renderer digest.
pub fn layout_memory_digest(memory: &ProfileLayoutMemory) -> Result<String, serde_json::Error> {
    Ok(format!(
        "sha256:{}",
        hex::encode(Sha256::digest(serde_json::to_vec(memory)?))
    ))
}

fn validate_memory_scope(scope: &MissionCanvasScope) -> Result<(), &'static str> {
    scope.validate()?;
    if scope.work_surface_id.is_some() && scope.attachment.is_none() {
        return Err("attachment_missing");
    }
    Ok(())
}

fn is_viewport_class(value: &str) -> bool {
    matches!(
        value,
        "minimum" | "compact" | "standard" | "productive" | "wide" | "reference_capture"
    )
}

fn is_contribution_id(value: &str) -> bool {
    let Some(suffix) = value.strip_prefix("contribution:") else {
        return false;
    };
    let mut chars = suffix.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    suffix.len() <= 160
        && (first.is_ascii_lowercase() || first.is_ascii_digit())
        && chars.all(|value| {
            value.is_ascii_lowercase() || value.is_ascii_digit() || "._:-".contains(value)
        })
}

fn is_region(value: &str) -> bool {
    matches!(
        value,
        "primary"
            | "secondary"
            | "inspector"
            | "rail"
            | "queue"
            | "composer"
            | "navigation"
            | "overlay"
    )
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DraftSnapshot {
    pub draft_id: String,
    pub content: String,
    pub recipient_ref: String,
    pub revision: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreservedInteractionState {
    pub focused_semantic_target: Option<String>,
    pub drafts: BTreeMap<String, DraftSnapshot>,
}

pub struct LayoutMemoryReduction<'a> {
    pub scope: MissionCanvasScope,
    pub profile_id: &'a str,
    pub activity_mode_id: &'a str,
    pub viewport_class: &'a str,
    pub layout: &'a LayoutNode,
    pub eligible_contribution_ids: &'a BTreeSet<String>,
    pub interaction: &'a PreservedInteractionState,
    pub idempotency_key: &'a str,
    pub updated_at: &'a str,
}

pub fn reduce_layout_memory(
    previous: Option<&ProfileLayoutMemory>,
    reduction: LayoutMemoryReduction<'_>,
) -> ProfileLayoutMemory {
    let LayoutMemoryReduction {
        scope,
        profile_id,
        activity_mode_id,
        viewport_class,
        layout,
        eligible_contribution_ids,
        interaction,
        idempotency_key,
        updated_at,
    } = reduction;
    let previous_placements = previous
        .map(|memory| {
            memory
                .placements
                .iter()
                .map(|placement| (placement.contribution_id.clone(), placement.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut layout_locations = BTreeMap::new();
    collect_locations(layout, &mut layout_locations);
    let known_ids = previous_placements
        .keys()
        .cloned()
        .chain(eligible_contribution_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let absent_contribution_ids = known_ids
        .difference(eligible_contribution_ids)
        .cloned()
        .collect::<Vec<_>>();
    let mut placements = known_ids
        .iter()
        .enumerate()
        .map(|(order, contribution_id)| {
            let mut placement = previous_placements
                .get(contribution_id)
                .cloned()
                .unwrap_or_else(|| PlacementMemory {
                    contribution_id: contribution_id.clone(),
                    preferred_regions: vec!["primary".into()],
                    preferred_order: order as u32,
                    minimum_span: 1,
                    maximum_span: 12,
                    preferred_adjacency: vec![],
                    last_compatible_layout_node_id: None,
                });
            if let Some(node_id) = layout_locations.get(contribution_id) {
                placement.last_compatible_layout_node_id = Some(node_id.clone());
            }
            placement
        })
        .collect::<Vec<_>>();
    placements.sort_by(|left, right| {
        left.preferred_order
            .cmp(&right.preferred_order)
            .then_with(|| left.contribution_id.cmp(&right.contribution_id))
    });
    ProfileLayoutMemory {
        memory_id: format!("layout-memory:{profile_id}:{activity_mode_id}:{viewport_class}"),
        scope,
        profile_id: profile_id.into(),
        activity_mode_id: activity_mode_id.into(),
        viewport_class: viewport_class.into(),
        placements,
        absent_contribution_ids,
        focused_semantic_target: interaction.focused_semantic_target.clone(),
        memory_revision: previous.map_or(1, |memory| memory.memory_revision + 1),
        idempotency_key: idempotency_key.into(),
        updated_at: updated_at.into(),
    }
}

pub fn restore_return_placement<'a>(
    memory: &'a ProfileLayoutMemory,
    contribution_id: &str,
) -> Option<&'a PlacementMemory> {
    memory
        .placements
        .iter()
        .find(|placement| placement.contribution_id == contribution_id)
}

pub fn migrate_legacy_layout(
    source: &Value,
    scope: MissionCanvasScope,
    profile_id: &str,
    activity_mode_id: &str,
    viewport_class: &str,
    updated_at: &str,
) -> ProfileLayoutMemory {
    let legacy_ids = source
        .get("panes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|pane| pane.get("contribution_id").and_then(Value::as_str))
        .collect::<Vec<_>>();
    let placements = legacy_ids
        .iter()
        .enumerate()
        .map(|(index, contribution_id)| PlacementMemory {
            contribution_id: (*contribution_id).into(),
            preferred_regions: vec!["primary".into()],
            preferred_order: index as u32,
            minimum_span: 1,
            maximum_span: 12,
            preferred_adjacency: vec![],
            last_compatible_layout_node_id: None,
        })
        .collect();
    ProfileLayoutMemory {
        memory_id: format!("layout-memory:{profile_id}:{activity_mode_id}:{viewport_class}"),
        scope,
        profile_id: profile_id.into(),
        activity_mode_id: activity_mode_id.into(),
        viewport_class: viewport_class.into(),
        placements,
        absent_contribution_ids: vec![],
        focused_semantic_target: source
            .get("focused_semantic_target")
            .and_then(Value::as_str)
            .map(str::to_owned),
        memory_revision: 1,
        idempotency_key: "legacy-layout-migration:v1".into(),
        updated_at: updated_at.into(),
    }
}

fn collect_locations(node: &LayoutNode, output: &mut BTreeMap<String, String>) {
    match node {
        LayoutNode::Single {
            node_id,
            contribution_id,
        } => {
            output.insert(contribution_id.clone(), node_id.clone());
        }
        LayoutNode::Split { children, .. }
        | LayoutNode::Stack { children, .. }
        | LayoutNode::Grid { children, .. } => {
            for child in children {
                collect_locations(child, output);
            }
        }
        LayoutNode::Tabs {
            node_id,
            contribution_ids,
            ..
        } => {
            for contribution_id in contribution_ids {
                output.insert(contribution_id.clone(), node_id.clone());
            }
        }
        LayoutNode::Inspector {
            node_id,
            primary,
            inspector_contribution_ids,
            ..
        } => {
            collect_locations(primary, output);
            for contribution_id in inspector_contribution_ids {
                output.insert(contribution_id.clone(), node_id.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> MissionCanvasScope {
        let legacy = crate::scoped_state::ScopeRef::project(
            "project:focusa",
            "/workspace/focusa",
            "Focusa",
            "host-a:worktree-main",
        )
        .unwrap();
        let workstream = crate::workstream_identity::WorkstreamKey::new(
            crate::workstream_identity::ScopeRef::project(legacy).unwrap(),
            crate::workstream_identity::WorkstreamId::parse("ws:mission-canvas").unwrap(),
        );
        let continuity =
            crate::workstream_identity::ContinuityId::parse("continuity:mission-canvas").unwrap();
        let attachment = crate::workstream_identity::AttachmentKey::new(
            workstream.clone(),
            Some(continuity),
            crate::workstream_identity::InstanceId::parse("instance:pi").unwrap(),
            crate::workstream_identity::SessionId::parse("session:1").unwrap(),
            crate::workstream_identity::AttachmentId::parse("attachment:1").unwrap(),
            crate::workstream_identity::WorkspaceBindingId::parse("workspace:mission-canvas")
                .unwrap(),
        );
        MissionCanvasScope::new(workstream, Some(attachment)).unwrap()
    }

    #[test]
    fn remembers_absence_and_restores_return_order() {
        let first_layout = LayoutNode::Stack {
            node_id: "layout:root".into(),
            children: vec![
                LayoutNode::Single {
                    node_id: "layout:a".into(),
                    contribution_id: "contribution:a".into(),
                },
                LayoutNode::Single {
                    node_id: "layout:b".into(),
                    contribution_id: "contribution:b".into(),
                },
            ],
        };
        let interaction = PreservedInteractionState {
            focused_semantic_target: Some("focus:a".into()),
            drafts: BTreeMap::new(),
        };
        let first = reduce_layout_memory(
            None,
            LayoutMemoryReduction {
                scope: scope(),
                profile_id: "software",
                activity_mode_id: "overview",
                viewport_class: "standard",
                layout: &first_layout,
                eligible_contribution_ids: &["contribution:a".into(), "contribution:b".into()]
                    .into_iter()
                    .collect(),
                interaction: &interaction,
                idempotency_key: "memory:1",
                updated_at: "2026-07-30T12:00:00Z",
            },
        );
        let second_layout = LayoutNode::Single {
            node_id: "layout:a".into(),
            contribution_id: "contribution:a".into(),
        };
        let second = reduce_layout_memory(
            Some(&first),
            LayoutMemoryReduction {
                scope: scope(),
                profile_id: "software",
                activity_mode_id: "overview",
                viewport_class: "standard",
                layout: &second_layout,
                eligible_contribution_ids: &["contribution:a".into()].into_iter().collect(),
                interaction: &interaction,
                idempotency_key: "memory:2",
                updated_at: "2026-07-30T12:01:00Z",
            },
        );
        assert_eq!(second.absent_contribution_ids, vec!["contribution:b"]);
        assert_eq!(
            restore_return_placement(&second, "contribution:b")
                .unwrap()
                .preferred_order,
            1
        );
        assert_eq!(second.focused_semantic_target.as_deref(), Some("focus:a"));
    }
}
