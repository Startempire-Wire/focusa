use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::model::{CandidateContribution, ContributionKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LayoutNode {
    Single {
        node_id: String,
        contribution_id: String,
    },
    Split {
        node_id: String,
        orientation: SplitOrientation,
        ratio: f64,
        children: Vec<LayoutNode>,
    },
    Stack {
        node_id: String,
        children: Vec<LayoutNode>,
    },
    Grid {
        node_id: String,
        columns: u8,
        children: Vec<LayoutNode>,
    },
    Tabs {
        node_id: String,
        contribution_ids: Vec<String>,
        active_contribution_id: String,
    },
    Inspector {
        node_id: String,
        side: InspectorSide,
        primary: Box<LayoutNode>,
        inspector_contribution_ids: Vec<String>,
        span: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectorSide {
    Start,
    End,
}

#[derive(Clone, Debug)]
pub struct LayoutConstraints {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub minimum_primary_span: u8,
    pub inspector_side: InspectorSide,
    pub focused_contribution_id: Option<String>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum LayoutError {
    #[error("cannot resolve a layout without eligible contributions")]
    NoEligibleContributions,
    #[error("layout contains dead chrome at node {0}")]
    DeadChrome(String),
    #[error("layout references unknown contribution {0}")]
    UnknownContribution(String),
    #[error("invalid span constraints for contribution {0}")]
    InvalidSpan(String),
}

pub fn resolve_layout(
    candidates: &[CandidateContribution],
    constraints: &LayoutConstraints,
) -> Result<LayoutNode, LayoutError> {
    if candidates.is_empty() {
        return Err(LayoutError::NoEligibleContributions);
    }
    for candidate in candidates {
        validate_geometry(candidate)?;
    }
    let ids = candidates
        .iter()
        .map(|candidate| candidate.contribution_id.clone())
        .collect::<Vec<_>>();
    if constraints.viewport_width <= 1024 {
        let active = constraints
            .focused_contribution_id
            .as_ref()
            .filter(|focused| ids.contains(focused))
            .cloned()
            .unwrap_or_else(|| ids[0].clone());
        return Ok(LayoutNode::Tabs {
            node_id: "layout:compact-tabs".into(),
            contribution_ids: ids,
            active_contribution_id: active,
        });
    }

    let inspectors = candidates
        .iter()
        .filter(|candidate| {
            matches!(
                candidate.kind,
                ContributionKind::Inspector | ContributionKind::InspectorSection
            )
        })
        .map(|candidate| candidate.contribution_id.clone())
        .collect::<Vec<_>>();
    let primary_candidates = candidates
        .iter()
        .filter(|candidate| {
            !matches!(
                candidate.kind,
                ContributionKind::Inspector | ContributionKind::InspectorSection
            )
        })
        .collect::<Vec<_>>();
    let mut primary = layout_primary(&primary_candidates, constraints);
    if primary_candidates.is_empty() {
        let first = inspectors[0].clone();
        primary = LayoutNode::Single {
            node_id: "layout:primary-fallback".into(),
            contribution_id: first,
        };
    }
    if !inspectors.is_empty() && !primary_candidates.is_empty() {
        primary = LayoutNode::Inspector {
            node_id: "layout:inspector".into(),
            side: constraints.inspector_side,
            primary: Box::new(primary),
            inspector_contribution_ids: inspectors,
            span: 3,
        };
    }
    validate_no_dead_chrome(&primary, &ids.iter().cloned().collect())?;
    Ok(primary)
}

fn layout_primary(
    candidates: &[&CandidateContribution],
    constraints: &LayoutConstraints,
) -> LayoutNode {
    let children = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| LayoutNode::Single {
            node_id: format!("layout:item:{index}"),
            contribution_id: candidate.contribution_id.clone(),
        })
        .collect::<Vec<_>>();
    match children.len() {
        0 => LayoutNode::Stack {
            node_id: "layout:empty".into(),
            children,
        },
        1 => children.into_iter().next().unwrap(),
        2 if constraints.viewport_width < 1440 => LayoutNode::Split {
            node_id: "layout:primary-split".into(),
            orientation: if constraints.viewport_height < 800 {
                SplitOrientation::Horizontal
            } else {
                SplitOrientation::Vertical
            },
            ratio: 0.67,
            children,
        },
        count if constraints.viewport_width >= 1600 => LayoutNode::Grid {
            node_id: "layout:primary-grid".into(),
            columns: if count >= 4 { 3 } else { 2 },
            children,
        },
        _ => LayoutNode::Stack {
            node_id: "layout:primary-stack".into(),
            children,
        },
    }
}

fn validate_geometry(candidate: &CandidateContribution) -> Result<(), LayoutError> {
    let minimum = candidate
        .geometry
        .get("minimum_span")
        .and_then(|value| value.as_u64())
        .unwrap_or(1);
    let maximum = candidate
        .geometry
        .get("maximum_span")
        .and_then(|value| value.as_u64())
        .unwrap_or(12);
    if minimum == 0 || maximum > 12 || minimum > maximum {
        return Err(LayoutError::InvalidSpan(candidate.contribution_id.clone()));
    }
    Ok(())
}

pub fn validate_no_dead_chrome(
    node: &LayoutNode,
    eligible_contribution_ids: &BTreeSet<String>,
) -> Result<(), LayoutError> {
    match node {
        LayoutNode::Single {
            node_id,
            contribution_id,
        } => {
            if node_id.is_empty() {
                return Err(LayoutError::DeadChrome("single".into()));
            }
            if !eligible_contribution_ids.contains(contribution_id) {
                return Err(LayoutError::UnknownContribution(contribution_id.clone()));
            }
        }
        LayoutNode::Split {
            node_id, children, ..
        } => {
            if children.len() != 2 {
                return Err(LayoutError::DeadChrome(node_id.clone()));
            }
            for child in children {
                validate_no_dead_chrome(child, eligible_contribution_ids)?;
            }
        }
        LayoutNode::Stack { node_id, children }
        | LayoutNode::Grid {
            node_id, children, ..
        } => {
            if children.is_empty() {
                return Err(LayoutError::DeadChrome(node_id.clone()));
            }
            for child in children {
                validate_no_dead_chrome(child, eligible_contribution_ids)?;
            }
        }
        LayoutNode::Tabs {
            node_id,
            contribution_ids,
            active_contribution_id,
        } => {
            if contribution_ids.is_empty() || !contribution_ids.contains(active_contribution_id) {
                return Err(LayoutError::DeadChrome(node_id.clone()));
            }
            for contribution_id in contribution_ids {
                if !eligible_contribution_ids.contains(contribution_id) {
                    return Err(LayoutError::UnknownContribution(contribution_id.clone()));
                }
            }
        }
        LayoutNode::Inspector {
            node_id,
            primary,
            inspector_contribution_ids,
            ..
        } => {
            if inspector_contribution_ids.is_empty() {
                return Err(LayoutError::DeadChrome(node_id.clone()));
            }
            validate_no_dead_chrome(primary, eligible_contribution_ids)?;
            for contribution_id in inspector_contribution_ids {
                if !eligible_contribution_ids.contains(contribution_id) {
                    return Err(LayoutError::UnknownContribution(contribution_id.clone()));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn candidate(id: &str, kind: ContributionKind) -> CandidateContribution {
        CandidateContribution {
            contribution_id: id.into(),
            kind,
            semantic_binding_id: format!("semantic:{id}"),
            renderer_binding_id: format!("renderer:{id}"),
            priority: 1,
            applicable_profile_ids: vec![],
            applicable_activity_mode_ids: vec![],
            canonical_content_refs: vec![json!({"ref": id})],
            required_capabilities: vec![],
            required_permissions: vec![],
            required_operations: vec![],
            geometry: json!({"minimum_span": 2, "maximum_span": 12}),
        }
    }

    #[test]
    fn compacts_to_tabs_and_preserves_focus() {
        let values = vec![
            candidate("contribution:primary", ContributionKind::FocusedWorkSurface),
            candidate("contribution:rail", ContributionKind::WorkRail),
        ];
        let layout = resolve_layout(
            &values,
            &LayoutConstraints {
                viewport_width: 1024,
                viewport_height: 720,
                minimum_primary_span: 8,
                inspector_side: InspectorSide::End,
                focused_contribution_id: Some("contribution:rail".into()),
            },
        )
        .unwrap();
        assert!(
            matches!(layout, LayoutNode::Tabs { active_contribution_id, .. } if active_contribution_id == "contribution:rail")
        );
    }

    #[test]
    fn omits_empty_inspector_structure() {
        let values = vec![candidate(
            "contribution:primary",
            ContributionKind::FocusedWorkSurface,
        )];
        let layout = resolve_layout(
            &values,
            &LayoutConstraints {
                viewport_width: 1440,
                viewport_height: 900,
                minimum_primary_span: 7,
                inspector_side: InspectorSide::End,
                focused_contribution_id: None,
            },
        )
        .unwrap();
        assert!(matches!(layout, LayoutNode::Single { .. }));
    }

    #[test]
    fn layout_property_vectors_never_create_dead_chrome() {
        for width in [1024, 1280, 1440, 1600, 1920] {
            for count in 1..=20 {
                let values = (0..count)
                    .map(|index| {
                        candidate(
                            &format!("contribution:item-{index}"),
                            if index % 5 == 0 {
                                ContributionKind::Inspector
                            } else {
                                ContributionKind::GeneratedSurface
                            },
                        )
                    })
                    .collect::<Vec<_>>();
                let eligible = values
                    .iter()
                    .map(|value| value.contribution_id.clone())
                    .collect();
                let layout = resolve_layout(
                    &values,
                    &LayoutConstraints {
                        viewport_width: width,
                        viewport_height: 900,
                        minimum_primary_span: 6,
                        inspector_side: InspectorSide::End,
                        focused_contribution_id: Some(values[0].contribution_id.clone()),
                    },
                )
                .unwrap();
                validate_no_dead_chrome(&layout, &eligible).unwrap();
            }
        }
    }
}
