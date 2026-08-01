use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{layout::LayoutNode, model::MissionCanvasScope};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
pub struct ProfileLayoutMemory {
    pub memory_id: String,
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

pub fn reduce_layout_memory(
    previous: Option<&ProfileLayoutMemory>,
    scope: MissionCanvasScope,
    profile_id: &str,
    activity_mode_id: &str,
    viewport_class: &str,
    layout: &LayoutNode,
    eligible_contribution_ids: &BTreeSet<String>,
    interaction: &PreservedInteractionState,
    idempotency_key: &str,
    updated_at: &str,
) -> ProfileLayoutMemory {
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
        MissionCanvasScope {
            project_root: "/tmp/focusa".into(),
            continuity_id: "mission-canvas".into(),
            instance_id: None,
            session_id: "session:1".into(),
            attachment_id: "attachment:1".into(),
            working_subpath_id: None,
        }
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
            scope(),
            "software",
            "overview",
            "standard",
            &first_layout,
            &["contribution:a".into(), "contribution:b".into()]
                .into_iter()
                .collect(),
            &interaction,
            "memory:1",
            "2026-07-30T12:00:00Z",
        );
        let second_layout = LayoutNode::Single {
            node_id: "layout:a".into(),
            contribution_id: "contribution:a".into(),
        };
        let second = reduce_layout_memory(
            Some(&first),
            scope(),
            "software",
            "overview",
            "standard",
            &second_layout,
            &["contribution:a".into()].into_iter().collect(),
            &interaction,
            "memory:2",
            "2026-07-30T12:01:00Z",
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
