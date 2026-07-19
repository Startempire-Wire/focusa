//! Provider-neutral WorkItem graph evaluation for the governed Work Loop.
//!
//! Adapters return snapshots; this module alone decides dependency readiness
//! and ordering. Provider commands must never become scheduler authority.

use super::{WorkItem, WorkItemProvider, WorkItemQuery, WorkItemStatus};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct WorkItemKey {
    provider: WorkItemProvider,
    project_root: PathBuf,
    provider_item_id: String,
}

impl WorkItemKey {
    fn from_item(item: &WorkItem) -> Self {
        Self {
            provider: item.provider,
            project_root: item.project_root.clone(),
            provider_item_id: item.provider_item_id.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkItemReadiness {
    pub ready: Vec<WorkItem>,
    pub blocked: Vec<BlockedWorkItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockedWorkItem {
    pub item: WorkItem,
    pub reason: String,
}

fn parent_matches(item: &WorkItem, query: &WorkItemQuery) -> bool {
    match &query.parent {
        Some(parent) => item.parent.as_ref().is_some_and(|candidate| {
            candidate.provider == parent.provider
                && candidate.provider_item_id == parent.provider_item_id
                && candidate.project_root == parent.project_root
        }),
        None => true,
    }
}

fn has_reachable_cycle(
    key: &WorkItemKey,
    items: &HashMap<WorkItemKey, &WorkItem>,
    visiting: &mut HashSet<WorkItemKey>,
    complete: &mut HashSet<WorkItemKey>,
) -> bool {
    if visiting.contains(key) {
        return true;
    }
    if complete.contains(key) {
        return false;
    }
    let Some(item) = items.get(key) else {
        return false;
    };
    visiting.insert(key.clone());
    for dependency in &item.dependencies {
        let dependency_key = WorkItemKey {
            provider: dependency.provider,
            project_root: dependency.project_root.clone(),
            provider_item_id: dependency.provider_item_id.clone(),
        };
        if items.contains_key(&dependency_key)
            && has_reachable_cycle(&dependency_key, items, visiting, complete)
        {
            visiting.remove(key);
            return true;
        }
    }
    visiting.remove(key);
    complete.insert(key.clone());
    false
}

/// Evaluate one provider snapshot using only Focusa's canonical graph rules.
/// Unknown, missing, cross-project, and cyclic dependencies fail closed.
pub fn evaluate_readiness(items: &[WorkItem], query: &WorkItemQuery) -> WorkItemReadiness {
    let by_key: HashMap<WorkItemKey, &WorkItem> = items
        .iter()
        .map(|item| (WorkItemKey::from_item(item), item))
        .collect();
    let mut ready = Vec::new();
    let mut blocked = Vec::new();

    for item in items
        .iter()
        .filter(|item| parent_matches(item, query))
        .filter(|item| !item.is_terminal())
    {
        let reason = if item.project_root != query.project_root {
            Some(format!(
                "cross_project_item:{}",
                item.project_root.to_string_lossy()
            ))
        } else if !matches!(
            item.provider_status,
            WorkItemStatus::Open | WorkItemStatus::InProgress
        ) {
            Some(format!("provider_status:{:?}", item.provider_status))
        } else if let Some(reason) = item.blocked_reason.as_deref() {
            Some(format!("provider_blocked:{reason}"))
        } else if has_reachable_cycle(
            &WorkItemKey::from_item(item),
            &by_key,
            &mut HashSet::new(),
            &mut HashSet::new(),
        ) {
            Some("dependency_cycle".to_string())
        } else {
            item.dependencies.iter().find_map(|dependency| {
                if dependency.project_root != query.project_root {
                    return Some(format!(
                        "cross_project_dependency:{}:{}",
                        dependency.project_root.to_string_lossy(),
                        dependency.provider_item_id
                    ));
                }
                let key = WorkItemKey {
                    provider: dependency.provider,
                    project_root: dependency.project_root.clone(),
                    provider_item_id: dependency.provider_item_id.clone(),
                };
                match by_key.get(&key) {
                    None => Some(format!(
                        "missing_dependency:{}",
                        dependency.provider_item_id
                    )),
                    Some(dependency_item) if !dependency_item.is_terminal() => Some(format!(
                        "dependency_incomplete:{}:{:?}",
                        dependency.provider_item_id, dependency_item.provider_status
                    )),
                    Some(_) => None,
                }
            })
        };

        if let Some(reason) = reason {
            blocked.push(BlockedWorkItem {
                item: item.clone(),
                reason,
            });
        } else {
            ready.push(item.clone());
        }
    }

    ready.sort_by(|left, right| {
        left.priority
            .cmp(&right.priority)
            .then_with(|| left.provider_item_id.cmp(&right.provider_item_id))
    });
    ready.truncate(query.limit.max(1));
    blocked.sort_by(|left, right| left.item.provider_item_id.cmp(&right.item.provider_item_id));
    WorkItemReadiness { ready, blocked }
}

pub fn select_next_ready(items: &[WorkItem], query: &WorkItemQuery) -> Option<WorkItem> {
    evaluate_readiness(items, query).ready.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::work_item::WorkItemRef;

    fn item(id: &str, status: WorkItemStatus, priority: i32) -> WorkItem {
        WorkItem {
            provider: WorkItemProvider::None,
            provider_item_id: id.into(),
            project_root: PathBuf::from("/project"),
            provider_status: status,
            title: id.into(),
            priority,
            parent: None,
            dependencies: vec![],
            acceptance_criteria: vec![],
            spec_refs: vec![],
            blocked_reason: None,
            url: None,
            revision: None,
        }
    }

    fn reference(id: &str, root: &Path) -> WorkItemRef {
        WorkItemRef {
            provider: WorkItemProvider::None,
            provider_item_id: id.into(),
            project_root: root.to_path_buf(),
            external_url: None,
        }
    }

    fn query() -> WorkItemQuery {
        WorkItemQuery {
            project_root: PathBuf::from("/project"),
            parent: None,
            limit: 100,
        }
    }

    #[test]
    fn core_selects_ready_items_without_provider_specific_authority() {
        let items = vec![
            item("later", WorkItemStatus::Open, 3),
            item("first", WorkItemStatus::Open, 0),
        ];
        assert_eq!(
            select_next_ready(&items, &query())
                .unwrap()
                .provider_item_id,
            "first"
        );
    }

    #[test]
    fn completed_dependencies_unlock_priority_ordered_work() {
        let done = item("dep", WorkItemStatus::Closed, 0);
        let mut next = item("next", WorkItemStatus::Open, 1);
        next.dependencies
            .push(reference("dep", Path::new("/project")));
        let result = evaluate_readiness(&[next, done], &query());
        assert_eq!(result.ready[0].provider_item_id, "next");
    }

    #[test]
    fn missing_dependency_fails_closed() {
        let mut next = item("next", WorkItemStatus::Open, 0);
        next.dependencies
            .push(reference("absent", Path::new("/project")));
        let result = evaluate_readiness(&[next], &query());
        assert!(result.ready.is_empty());
        assert_eq!(result.blocked[0].reason, "missing_dependency:absent");
    }

    #[test]
    fn cross_project_dependency_fails_closed() {
        let mut next = item("next", WorkItemStatus::Open, 0);
        next.dependencies
            .push(reference("foreign", Path::new("/other")));
        let result = evaluate_readiness(&[next], &query());
        assert!(result.ready.is_empty());
        assert!(
            result.blocked[0]
                .reason
                .starts_with("cross_project_dependency:")
        );
    }

    #[test]
    fn dependency_cycle_fails_closed() {
        let mut left = item("left", WorkItemStatus::Open, 0);
        let mut right = item("right", WorkItemStatus::Open, 0);
        left.dependencies
            .push(reference("right", Path::new("/project")));
        right
            .dependencies
            .push(reference("left", Path::new("/project")));
        let result = evaluate_readiness(&[left, right], &query());
        assert!(result.ready.is_empty());
        assert!(
            result
                .blocked
                .iter()
                .all(|entry| entry.reason == "dependency_cycle")
        );
    }

    #[test]
    fn parent_filter_uses_typed_project_scope() {
        let parent = reference("root", Path::new("/project"));
        let mut child = item("child", WorkItemStatus::Open, 0);
        child.parent = Some(parent.clone());
        let mut foreign = item("foreign", WorkItemStatus::Open, 0);
        foreign.parent = Some(reference("root", Path::new("/other")));
        let result = evaluate_readiness(
            &[child, foreign],
            &WorkItemQuery {
                project_root: PathBuf::from("/project"),
                parent: Some(parent),
                limit: 100,
            },
        );
        assert_eq!(result.ready.len(), 1);
        assert_eq!(result.ready[0].provider_item_id, "child");
    }
}
