use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceKind {
    Primary,
    FeatureBranch,
    LinkedWorktree,
    Detached,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkingSubpathLifecycle {
    Active,
    Merging,
    Merged,
    Stale,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkingSubpath {
    pub working_subpath_id: String,
    pub workspace_root: String,
    pub workspace_kind: WorkspaceKind,
    pub git_common_dir_id: String,
    pub branch_or_ref: String,
    pub head_commit: String,
    pub beads_root: Option<String>,
    pub beads_prefix: Option<String>,
    pub lifecycle: WorkingSubpathLifecycle,
    pub created_from_ref: Option<String>,
    pub merge_target_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitWorkingContext {
    pub canonical_parent_root: String,
    pub active_worktree_root: String,
    pub git_common_dir: String,
    pub git_dir: String,
    pub working_subpath: WorkingSubpath,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WorkingSubpathError {
    #[error("working path does not exist: {0}")]
    MissingPath(String),
    #[error("git context command failed: {0}")]
    GitCommand(String),
    #[error("git common directory has no canonical parent: {0}")]
    MissingCanonicalParent(String),
}

pub fn resolve_git_working_context(
    start: &Path,
) -> Result<Option<GitWorkingContext>, WorkingSubpathError> {
    if !start.exists() {
        return Err(WorkingSubpathError::MissingPath(
            start.to_string_lossy().into_owned(),
        ));
    }
    let start = if start.is_file() {
        start.parent().unwrap_or(start)
    } else {
        start
    };
    let Some(workspace_root) = git_optional(start, &["rev-parse", "--show-toplevel"])? else {
        return Ok(None);
    };
    let workspace_root = canonical(Path::new(&workspace_root));
    let common_raw = git_required(
        start,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    let git_dir_raw = git_required(start, &["rev-parse", "--path-format=absolute", "--git-dir"])?;
    let common_dir = canonical(&resolve_git_path(start, &common_raw));
    let git_dir = canonical(&resolve_git_path(start, &git_dir_raw));
    let canonical_parent = canonical_parent_for_common_dir(&common_dir)?;
    let branch = git_required(start, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let head = git_required(start, &["rev-parse", "HEAD"])?;
    let common_id = stable_id("git-common", &[path_text(&common_dir)]);
    let workspace_kind = classify_workspace(&workspace_root, &canonical_parent, &branch);
    let subpath_id = if workspace_kind == WorkspaceKind::Primary {
        "primary".into()
    } else {
        stable_id(
            "working-subpath",
            &[
                common_id.clone(),
                path_text(&workspace_root),
                branch.clone(),
            ],
        )
    };
    let beads_root_path = canonical_parent.join(".beads");
    let beads_root = beads_root_path
        .is_dir()
        .then(|| path_text(&beads_root_path));
    let parent_prefix = beads_root
        .as_deref()
        .and_then(|_| read_parent_beads_prefix(&beads_root_path));
    let beads_prefix = parent_prefix.map(|prefix| {
        if subpath_id == "primary" {
            prefix
        } else {
            format!("{prefix}-wt-{}", short_id(&subpath_id))
        }
    });
    Ok(Some(GitWorkingContext {
        canonical_parent_root: path_text(&canonical_parent),
        active_worktree_root: path_text(&workspace_root),
        git_common_dir: path_text(&common_dir),
        git_dir: path_text(&git_dir),
        working_subpath: WorkingSubpath {
            working_subpath_id: subpath_id,
            workspace_root: path_text(&workspace_root),
            workspace_kind,
            git_common_dir_id: common_id,
            branch_or_ref: branch,
            head_commit: head,
            beads_root,
            beads_prefix,
            lifecycle: WorkingSubpathLifecycle::Active,
            created_from_ref: None,
            merge_target_ref: None,
        },
    }))
}

fn classify_workspace(workspace: &Path, parent: &Path, branch: &str) -> WorkspaceKind {
    if branch == "HEAD" {
        WorkspaceKind::Detached
    } else if workspace != parent {
        WorkspaceKind::LinkedWorktree
    } else if matches!(branch, "main" | "master") {
        WorkspaceKind::Primary
    } else {
        WorkspaceKind::FeatureBranch
    }
}

fn canonical_parent_for_common_dir(common_dir: &Path) -> Result<PathBuf, WorkingSubpathError> {
    if common_dir.file_name().and_then(|value| value.to_str()) == Some(".git") {
        return common_dir
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| WorkingSubpathError::MissingCanonicalParent(path_text(common_dir)));
    }
    Err(WorkingSubpathError::MissingCanonicalParent(path_text(
        common_dir,
    )))
}

fn resolve_git_path(start: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        start.join(path)
    }
}

fn git_optional(start: &Path, args: &[&str]) -> Result<Option<String>, WorkingSubpathError> {
    let output = Command::new("git").arg("-C").arg(start).args(args).output();
    let Ok(output) = output else {
        return Ok(None);
    };
    if !output.status.success() {
        return Ok(None);
    }
    Ok(nonempty_stdout(output.stdout))
}

fn git_required(start: &Path, args: &[&str]) -> Result<String, WorkingSubpathError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(start)
        .args(args)
        .output()
        .map_err(|error| WorkingSubpathError::GitCommand(error.to_string()))?;
    if !output.status.success() {
        return Err(WorkingSubpathError::GitCommand(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    nonempty_stdout(output.stdout).ok_or_else(|| {
        WorkingSubpathError::GitCommand(format!("empty output for {}", args.join(" ")))
    })
}

fn nonempty_stdout(bytes: Vec<u8>) -> Option<String> {
    let value = String::from_utf8_lossy(&bytes).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().trim_end_matches('/').to_string()
}

fn stable_id(namespace: &str, parts: &[String]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(namespace.as_bytes());
    for part in parts {
        hasher.update([0]);
        hasher.update(part.as_bytes());
    }
    format!("{namespace}:{}", hex::encode(hasher.finalize()))
}

fn short_id(id: &str) -> &str {
    id.rsplit(':').next().unwrap_or(id).get(..8).unwrap_or(id)
}

fn read_parent_beads_prefix(beads_root: &Path) -> Option<String> {
    let issues = std::fs::read_to_string(beads_root.join("issues.jsonl")).ok()?;
    for line in issues.lines() {
        let id = serde_json::from_str::<serde_json::Value>(line)
            .ok()?
            .get("id")?
            .as_str()?
            .to_string();
        if let Some((prefix, _)) = id.split_once('-') {
            return Some(prefix.to_string());
        }
    }
    None
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectBindingCandidate {
    pub project_root: String,
    pub active_worktree_root: Option<String>,
    pub canonical_parent_root: Option<String>,
    #[serde(default)]
    pub repo_fingerprint: Option<String>,
    #[serde(default)]
    pub project_fingerprint: Option<String>,
    pub score: u16,
    pub sources: Vec<String>,
    pub markers: Vec<String>,
    pub relationship: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectBindingDecision {
    pub status: String,
    #[serde(default)]
    pub schema: String,
    #[serde(default)]
    pub decision_id: String,
    #[serde(default)]
    pub state: String,
    pub selected_project_root: Option<String>,
    pub selected_active_worktree_root: Option<String>,
    #[serde(default)]
    pub selected_worktree_root: Option<String>,
    pub canonical_parent_root: Option<String>,
    #[serde(default)]
    pub continuity_id: Option<String>,
    pub ambiguous: bool,
    pub requires_confirmation: bool,
    pub reason: String,
    pub candidates: Vec<ProjectBindingCandidate>,
    #[serde(default)]
    pub evidence_sources: Vec<String>,
    #[serde(default)]
    pub evidence_revision: String,
    #[serde(default)]
    pub evidence_freshness: String,
    #[serde(default)]
    pub rejection_reasons: Vec<String>,
    #[serde(default)]
    pub scope_safety_policy_version: String,
    #[serde(default)]
    pub repo_fingerprint: Option<String>,
    #[serde(default)]
    pub project_fingerprint: Option<String>,
    #[serde(default)]
    pub confidence: String,
    #[serde(default)]
    pub verification_status: String,
    #[serde(default)]
    pub permitted_capability_tier: String,
    #[serde(default)]
    pub recovery_packet_ref: Option<String>,
    #[serde(default)]
    pub operator_decision_ref: Option<String>,
    #[serde(default)]
    pub effective_at: Option<String>,
    #[serde(default)]
    pub supersedes_decision_id: Option<String>,
    #[serde(default)]
    pub binding_receipt_id: Option<String>,
}

impl ProjectBindingDecision {
    pub fn mark_verified(mut self) -> Self {
        if self.ambiguous || self.selected_project_root.is_none() {
            return self;
        }
        self.state = "BOUND".to_string();
        self.status = "verified".to_string();
        self.verification_status = "verified".to_string();
        self.permitted_capability_tier = "scoped".to_string();
        self.requires_confirmation = false;
        self.rejection_reasons.clear();
        self.evidence_freshness = "current".to_string();
        self.evidence_revision = stable_id(
            "project-binding-evidence",
            &["BOUND".to_string(), self.evidence_revision.clone()],
        );
        self.decision_id = stable_id(
            "project-binding",
            std::slice::from_ref(&self.evidence_revision),
        );
        self.binding_receipt_id = Some(stable_id(
            "binding-receipt",
            std::slice::from_ref(&self.evidence_revision),
        ));
        self
    }
}

fn binding_markers(path: &Path) -> Vec<String> {
    let mut markers = Vec::new();
    if path.join(".focusa-project.json").is_file() {
        markers.push("focusa_marker".to_string());
    }
    if path.join(".git").exists() {
        markers.push("git".to_string());
    }
    if path.join(".beads").is_dir() {
        markers.push("beads".to_string());
    }
    markers
}

fn project_marker_fingerprint(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path.join(".focusa-project.json")).ok()?;
    let normalized = serde_json::from_str::<serde_json::Value>(&content)
        .ok()
        .and_then(|value| {
            value
                .get("project_id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or(content);
    Some(stable_id("project-marker", &[normalized]))
}

fn insert_binding_candidate(
    candidates: &mut BTreeMap<String, ProjectBindingCandidate>,
    path: &Path,
    source: &str,
    score: u16,
    relationship: &str,
) {
    let root = canonical(path);
    let root_text = path_text(&root);
    let git = resolve_git_working_context(&root).ok().flatten();
    let active_worktree_root = git
        .as_ref()
        .map(|context| context.active_worktree_root.clone())
        .filter(|value| !value.trim().is_empty());
    let canonical_parent_root = git
        .as_ref()
        .map(|context| context.canonical_parent_root.clone())
        .filter(|value| !value.trim().is_empty())
        .or_else(|| Some(root_text.clone()));
    let authoritative_root = active_worktree_root
        .clone()
        .unwrap_or_else(|| root_text.clone());
    let repo_fingerprint = git
        .as_ref()
        .map(|context| context.working_subpath.git_common_dir_id.clone());
    let project_fingerprint = project_marker_fingerprint(&root);
    let markers = binding_markers(&root);
    let entry = candidates
        .entry(authoritative_root.clone())
        .or_insert_with(|| ProjectBindingCandidate {
            project_root: authoritative_root,
            active_worktree_root,
            canonical_parent_root,
            repo_fingerprint,
            project_fingerprint,
            score,
            sources: Vec::new(),
            markers: Vec::new(),
            relationship: relationship.to_string(),
        });
    entry.score = entry.score.max(score);
    if !entry.sources.iter().any(|value| value == source) {
        if source == "explicit_project_root" {
            entry.sources.insert(0, source.to_string());
        } else {
            entry.sources.push(source.to_string());
        }
    }
    for marker in markers {
        if !entry.markers.contains(&marker) {
            entry.markers.push(marker);
        }
    }
}

fn collect_parent_binding_candidates(
    start: &Path,
    candidates: &mut BTreeMap<String, ProjectBindingCandidate>,
) {
    let mut current = Some(canonical(start));
    for depth in 0..12 {
        let Some(path) = current else { break };
        // Filesystem/account roots are session envelopes, never inferred projects.
        if path.parent().is_none() {
            break;
        }
        let markers = binding_markers(&path);
        if !markers.is_empty() {
            let marker_score: u16 = if markers.iter().any(|value| value == "focusa_marker") {
                950
            } else if markers.iter().any(|value| value == "git") {
                900
            } else {
                800
            };
            insert_binding_candidate(
                candidates,
                &path,
                "cwd_ancestor_markers",
                marker_score.saturating_sub(depth * 5),
                "ancestor_or_current",
            );
        }
        current = path.parent().map(Path::to_path_buf);
    }
}

fn collect_child_binding_candidates(
    start: &Path,
    candidates: &mut BTreeMap<String, ProjectBindingCandidate>,
) {
    let mut frontier = vec![(canonical(start), 0_u8)];
    let mut inspected = 0_usize;
    while let Some((directory, depth)) = frontier.pop() {
        if depth >= 2 || inspected >= 64 {
            continue;
        }
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            if inspected >= 64 {
                break;
            }
            inspected += 1;
            let path = entry.path();
            if !path.is_dir() || entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            let markers = binding_markers(&path);
            if !markers.is_empty() {
                let score: u16 = if markers.iter().any(|value| value == "focusa_marker") {
                    860
                } else if markers.iter().any(|value| value == "git") {
                    810
                } else {
                    720
                };
                insert_binding_candidate(
                    candidates,
                    &path,
                    "parent_directory_child_scan",
                    score.saturating_sub(u16::from(depth) * 10),
                    "bounded_child",
                );
            } else {
                frontier.push((path, depth + 1));
            }
        }
    }
}

/// Rank project/worktree authority candidates for resumed sessions and broad cwd starts.
///
/// Active Git worktrees outrank their canonical parent, explicit roots outrank
/// inferred roots, and a persisted root is promoted only when it remains marked
/// or shares the current Git common directory. Equal-ranked child projects fail
/// closed and require explicit confirmation.
pub fn resolve_project_binding_candidates(
    start: &Path,
    explicit_project_root: Option<&Path>,
    persisted_project_root: Option<&Path>,
) -> ProjectBindingDecision {
    let mut candidates = BTreeMap::new();
    collect_parent_binding_candidates(start, &mut candidates);
    collect_child_binding_candidates(start, &mut candidates);

    if let Some(explicit) = explicit_project_root {
        insert_binding_candidate(
            &mut candidates,
            explicit,
            "explicit_project_root",
            1000,
            "explicit",
        );
    }
    if let Some(persisted) = persisted_project_root {
        let marked = !binding_markers(persisted).is_empty();
        insert_binding_candidate(
            &mut candidates,
            persisted,
            "persisted_session_project_root",
            if marked { 930 } else { 620 },
            if marked {
                "persisted_marked_root"
            } else {
                "persisted_unverified_root"
            },
        );
    }

    let mut ranked = candidates.into_values().collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.project_root.cmp(&right.project_root))
    });
    let ambiguous = ranked.len() > 1
        && ranked[0].score == ranked[1].score
        && ranked[0].project_root != ranked[1].project_root;
    let selected = (!ambiguous).then(|| ranked.first()).flatten();
    let selected_missing = selected.is_none();
    let state = if ambiguous {
        "QUARANTINED"
    } else if selected.is_some() {
        "VERIFY"
    } else {
        "RECOVERING"
    };
    let selected_score = selected
        .map(|candidate| candidate.score)
        .unwrap_or_default();
    let confidence = if selected_score >= 900 {
        "high"
    } else if selected_score >= 700 {
        "medium"
    } else if selected.is_some() {
        "low"
    } else {
        "none"
    };
    let mut evidence_sources = ranked
        .iter()
        .flat_map(|candidate| candidate.sources.iter().cloned())
        .collect::<Vec<_>>();
    evidence_sources.sort();
    evidence_sources.dedup();
    let evidence_revision = stable_id(
        "project-binding-evidence",
        &[
            state.to_string(),
            selected
                .map(|candidate| candidate.project_root.clone())
                .unwrap_or_default(),
            ranked
                .iter()
                .map(|candidate| format!("{}:{}", candidate.project_root, candidate.score))
                .collect::<Vec<_>>()
                .join("|"),
        ],
    );
    ProjectBindingDecision {
        status: if ambiguous {
            "ambiguous"
        } else if selected.is_some() {
            "selected"
        } else {
            "unbound"
        }
        .to_string(),
        schema: "focusa.project_binding_decision.v1".to_string(),
        decision_id: stable_id("project-binding", std::slice::from_ref(&evidence_revision)),
        state: state.to_string(),
        selected_project_root: selected.map(|candidate| candidate.project_root.clone()),
        selected_active_worktree_root: selected
            .and_then(|candidate| candidate.active_worktree_root.clone()),
        selected_worktree_root: selected
            .and_then(|candidate| candidate.active_worktree_root.clone()),
        canonical_parent_root: selected
            .and_then(|candidate| candidate.canonical_parent_root.clone()),
        continuity_id: None,
        ambiguous,
        requires_confirmation: ambiguous
            || selected.is_none()
            || selected.is_some_and(|value| value.score < 800),
        reason: if ambiguous {
            "multiple equally ranked project roots; mutation-bound selection required"
        } else if selected.is_some() {
            "highest-ranked evidence-backed project/worktree candidate selected for verification"
        } else {
            "no marked project binding candidate found; recovery remains non-modal"
        }
        .to_string(),
        candidates: ranked,
        evidence_sources,
        evidence_revision,
        evidence_freshness: "unknown".to_string(),
        rejection_reasons: if ambiguous {
            vec!["conflicting_strong_candidates".to_string()]
        } else if selected_missing {
            vec!["no_safe_selected_project_root".to_string()]
        } else {
            vec!["canonical_project_verification_required".to_string()]
        },
        scope_safety_policy_version: "focusa.scope_safety.v1".to_string(),
        repo_fingerprint: None,
        project_fingerprint: None,
        confidence: confidence.to_string(),
        verification_status: "pending".to_string(),
        permitted_capability_tier: if ambiguous {
            "unbound_read_only"
        } else {
            "recovery_read_plan"
        }
        .to_string(),
        recovery_packet_ref: None,
        operator_decision_ref: None,
        effective_at: None,
        supersedes_decision_id: None,
        binding_receipt_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_classification_separates_parent_folder_from_branch_context() {
        let parent = Path::new("/repo/focusa");
        assert_eq!(
            classify_workspace(parent, parent, "main"),
            WorkspaceKind::Primary
        );
        assert_eq!(
            classify_workspace(parent, parent, "feature/worktree"),
            WorkspaceKind::FeatureBranch
        );
        assert_eq!(
            classify_workspace(Path::new("/repo/focusa-wt"), parent, "feature/worktree"),
            WorkspaceKind::LinkedWorktree
        );
        assert_eq!(
            classify_workspace(parent, parent, "HEAD"),
            WorkspaceKind::Detached
        );
    }

    #[test]
    fn linked_worktree_resolves_parent_git_and_shared_beads_authority() {
        let base = std::env::temp_dir().join(format!("focusa-worktree-{}", uuid::Uuid::now_v7()));
        let parent = base.join("focusa");
        let worktree = base.join("focusa-mac-agent");
        std::fs::create_dir_all(parent.join(".beads")).unwrap();
        std::fs::write(
            parent.join(".beads/issues.jsonl"),
            r#"{"id":"focusa-test","title":"test"}"#,
        )
        .unwrap();
        std::fs::write(parent.join("README.md"), "focusa").unwrap();
        git_ok(&base, &["init", "-b", "main", parent.to_str().unwrap()]);
        git_ok(&parent, &["config", "user.email", "test@focusa.local"]);
        git_ok(&parent, &["config", "user.name", "Focusa Test"]);
        git_ok(&parent, &["add", "README.md", ".beads/issues.jsonl"]);
        git_ok(&parent, &["commit", "-m", "initial"]);
        git_ok(
            &parent,
            &[
                "worktree",
                "add",
                "-b",
                "feature/mac-agent",
                worktree.to_str().unwrap(),
            ],
        );

        let context = resolve_git_working_context(&worktree).unwrap().unwrap();
        assert_eq!(
            context.canonical_parent_root,
            path_text(&canonical(&parent))
        );
        assert_eq!(
            context.active_worktree_root,
            path_text(&canonical(&worktree))
        );
        assert_eq!(
            context.working_subpath.workspace_kind,
            WorkspaceKind::LinkedWorktree
        );
        assert_eq!(
            context.working_subpath.beads_root,
            Some(path_text(&canonical(&parent).join(".beads")))
        );
        assert!(
            context
                .working_subpath
                .beads_prefix
                .as_deref()
                .is_some_and(|prefix| prefix.starts_with("focusa-wt-"))
        );

        git_ok(
            &parent,
            &["worktree", "remove", "--force", worktree.to_str().unwrap()],
        );
        assert!(matches!(
            resolve_git_working_context(&worktree),
            Err(WorkingSubpathError::MissingPath(_))
        ));
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn binding_candidates_select_one_marked_child_from_parent_directory() {
        let base =
            std::env::temp_dir().join(format!("focusa-binding-parent-{}", uuid::Uuid::now_v7()));
        let project = base.join("focusa");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(project.join(".focusa-project.json"), b"{}\n").unwrap();

        let decision = resolve_project_binding_candidates(&base, None, None);
        let expected = path_text(&canonical(&project));
        assert_eq!(decision.status, "selected");
        assert_eq!(
            decision.selected_project_root.as_deref(),
            Some(expected.as_str())
        );
        assert!(!decision.requires_confirmation);
        assert_eq!(decision.schema, "focusa.project_binding_decision.v1");
        assert_eq!(decision.state, "VERIFY");
        assert_eq!(decision.permitted_capability_tier, "recovery_read_plan");
        let verified = decision.mark_verified();
        assert_eq!(verified.state, "BOUND");
        assert_eq!(verified.permitted_capability_tier, "scoped");
        assert!(verified.binding_receipt_id.is_some());
        assert!(verified.rejection_reasons.is_empty());
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn binding_candidates_fail_closed_for_multiple_equal_children() {
        let base =
            std::env::temp_dir().join(format!("focusa-binding-ambiguous-{}", uuid::Uuid::now_v7()));
        for name in ["project-a", "project-b"] {
            let project = base.join(name);
            std::fs::create_dir_all(&project).unwrap();
            std::fs::write(project.join(".focusa-project.json"), b"{}\n").unwrap();
        }

        let decision = resolve_project_binding_candidates(&base, None, None);
        assert_eq!(decision.status, "ambiguous");
        assert!(decision.ambiguous);
        assert!(decision.selected_project_root.is_none());
        assert!(decision.requires_confirmation);
        assert_eq!(decision.state, "QUARANTINED");
        assert_eq!(decision.permitted_capability_tier, "unbound_read_only");
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn macos_volumes_user_home_is_never_promoted_as_project() {
        let base = std::env::temp_dir().join(format!(
            "focusa-binding-macos-volumes-{}",
            uuid::Uuid::now_v7()
        ));
        let user_home = base.join("Volumes/Macintosh HD/Users/vsmith");
        let project = user_home.join("Projects/focusa");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(project.join(".focusa-project.json"), b"{}\n").unwrap();

        let decision = resolve_project_binding_candidates(&user_home, None, None);
        let expected_project = path_text(&canonical(&project));
        let forbidden_home = path_text(&canonical(&user_home));
        assert_eq!(
            decision.selected_project_root.as_deref(),
            Some(expected_project.as_str())
        );
        assert!(
            decision
                .candidates
                .iter()
                .all(|candidate| candidate.project_root != forbidden_home)
        );
        assert_eq!(decision.state, "VERIFY");
        std::fs::remove_dir_all(base).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn binding_candidates_canonicalize_temp_space_unicode_and_symlink_roots() {
        use std::os::unix::fs::symlink;

        let base =
            std::env::temp_dir().join(format!("focusa binding unicode {}", uuid::Uuid::now_v7()));
        let project = base.join("π project");
        let alias = base.join("project-link");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(
            project.join(".focusa-project.json"),
            br#"{"project_id":"unicode-project"}"#,
        )
        .unwrap();
        symlink(&project, &alias).unwrap();

        let decision = resolve_project_binding_candidates(&base, Some(&alias), Some(&project));
        let expected = path_text(&canonical(&project));
        assert_eq!(
            decision.selected_project_root.as_deref(),
            Some(expected.as_str())
        );
        assert!(!decision.ambiguous);
        assert_eq!(decision.state, "VERIFY");
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn binding_candidates_prefer_explicit_active_root_over_persisted_root() {
        let base =
            std::env::temp_dir().join(format!("focusa-binding-resume-{}", uuid::Uuid::now_v7()));
        let current = base.join("current");
        let persisted = base.join("persisted");
        for project in [&current, &persisted] {
            std::fs::create_dir_all(project).unwrap();
            std::fs::write(project.join(".focusa-project.json"), b"{}\n").unwrap();
        }

        let decision = resolve_project_binding_candidates(&base, Some(&current), Some(&persisted));
        let expected = path_text(&canonical(&current));
        assert_eq!(
            decision.selected_project_root.as_deref(),
            Some(expected.as_str())
        );
        assert_eq!(decision.candidates[0].sources[0], "explicit_project_root");
        assert_eq!(
            decision.candidates[0].project_fingerprint,
            decision.candidates[1].project_fingerprint
        );
        std::fs::remove_dir_all(base).unwrap();
    }

    fn git_ok(cwd: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(cwd)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn working_context_ids_are_deterministic_and_workspace_specific() {
        let common = "git-common:abc".to_string();
        let first = stable_id(
            "working-subpath",
            &[common.clone(), "/repo/wt-a".into(), "feature/a".into()],
        );
        let repeated = stable_id(
            "working-subpath",
            &[common.clone(), "/repo/wt-a".into(), "feature/a".into()],
        );
        let second = stable_id(
            "working-subpath",
            &[common, "/repo/wt-b".into(), "feature/b".into()],
        );
        assert_eq!(first, repeated);
        assert_ne!(first, second);
    }
}
