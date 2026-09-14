use anyhow::{Result, anyhow};
use focusa_core::scope_safety::classify_project_root;
use focusa_core::working_subpath::resolve_git_working_context;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

const SCHEMA_SELECTED: &str = "focusa.cli.selected_project.v1";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SelectedProjectProfile {
    pub schema: String,
    pub selected_project_fingerprint: String,
    pub selected_at: String,
    pub selected_by: String,
    pub note: String,
}

#[derive(Debug, Clone)]
pub enum ScopeSource {
    ExplicitFlag,
    ProjectAlias,
    EnvProjectRoot,
    EnvSelectedProject,
    LegacyEnvActiveProject,
    CliSelectedProject,
    CwdMarker,
    CwdGitRoot,
    InteractiveSelection,
}

#[derive(Debug, Clone)]
pub struct ResolvedProjectScope {
    pub project_root: String,
    pub canonical_parent_root: String,
    pub active_worktree_root: String,
    pub working_subpath_id: String,
    pub continuity_id: Option<String>,
    pub project_id: Option<String>,
    pub fingerprint: Option<String>,
    pub scope_source: ScopeSource,
    pub verified: bool,
    pub authority: &'static str,
    pub project_root_authority_failure: Option<&'static str>,
}

fn home_dir() -> PathBuf {
    std::env::var("FOCUSA_TEST_HOME").map_or_else(
        |_| {
            std::env::var("HOME")
                .unwrap_or_else(|_| ".".to_string())
                .into()
        },
        PathBuf::from,
    )
}

fn focusa_config_dir() -> PathBuf {
    home_dir().join(".config").join("focusa")
}

fn selected_profile_path() -> PathBuf {
    focusa_config_dir().join("selected-project.json")
}

fn legacy_active_project_path() -> PathBuf {
    focusa_config_dir().join("active-project")
}

fn is_safe_root(root: &str) -> Option<&'static str> {
    classify_project_root(root).reason()
}

fn project_fingerprint(path: &str) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    let hash = hasher.finish();
    format!("project-fnv1a64:{hash:016x}")
}

fn write_json_file<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<T>(&text).ok())
}

fn project_root_from_path(path: &Path) -> Option<PathBuf> {
    if let Ok(Some(context)) = resolve_git_working_context(path) {
        return Some(PathBuf::from(context.canonical_parent_root));
    }
    path.join(".focusa-project.json")
        .is_file()
        .then(|| path.to_path_buf())
}

fn root_path_is_project_candidate(path: &Path) -> bool {
    project_root_from_path(path).is_some()
}

fn find_upward_root(start: &Path) -> Option<(PathBuf, ScopeSource)> {
    let mut cursor = start.to_path_buf();
    for _ in 0..8 {
        if project_root_from_path(&cursor).is_some() {
            if cursor.join(".focusa-project.json").is_file() {
                return Some((cursor, ScopeSource::CwdMarker));
            }
            return Some((cursor, ScopeSource::CwdGitRoot));
        }
        if !cursor.pop() {
            break;
        }
    }
    None
}

fn resolved_from_root(root: &str, scope_source: ScopeSource) -> ResolvedProjectScope {
    let working_context = resolve_git_working_context(Path::new(root)).ok().flatten();
    let canonical_parent_root = working_context
        .as_ref()
        .map(|context| context.canonical_parent_root.clone())
        .unwrap_or_else(|| root.to_string());
    let active_worktree_root = working_context
        .as_ref()
        .map(|context| context.active_worktree_root.clone())
        .unwrap_or_else(|| root.to_string());
    let working_subpath_id = working_context
        .as_ref()
        .map(|context| context.working_subpath.working_subpath_id.clone())
        .unwrap_or_else(|| "primary".to_string());
    ResolvedProjectScope {
        project_root: canonical_parent_root.clone(),
        canonical_parent_root: canonical_parent_root.clone(),
        active_worktree_root,
        working_subpath_id,
        continuity_id: None,
        project_id: Path::new(&canonical_parent_root)
            .file_name()
            .and_then(|value| value.to_str())
            .map(ToString::to_string),
        fingerprint: Some(project_fingerprint(&canonical_parent_root)),
        scope_source,
        verified: true,
        authority: "selected_or_verified",
        project_root_authority_failure: None,
    }
}

fn migrate_active_project_alias() -> Result<()> {
    let selected = selected_profile_path();
    if selected.exists() {
        return Ok(());
    }
    let legacy = legacy_active_project_path();
    if !legacy.exists() {
        return Ok(());
    }

    let legacy_text = fs::read_to_string(&legacy)?;
    let fingerprint = legacy_text
        .trim()
        .split(':')
        .nth(1)
        .unwrap_or("")
        .trim()
        .trim_matches('}')
        .trim_matches('"')
        .to_string();

    if !fingerprint.is_empty() {
        write_json_file(
            &selected,
            &SelectedProjectProfile {
                schema: SCHEMA_SELECTED.to_string(),
                selected_project_fingerprint: fingerprint,
                selected_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                selected_by: "active-project-migration".to_string(),
                note: "migrated from active-project".to_string(),
            },
        )?;
    }

    Ok(())
}

fn selected_fingerprint_from_profile() -> Result<Option<String>> {
    migrate_active_project_alias()?;
    let selected = selected_profile_path();
    let profile = read_json_file::<SelectedProjectProfile>(&selected);
    Ok(profile
        .filter(|p| p.schema == SCHEMA_SELECTED)
        .map(|p| p.selected_project_fingerprint))
}

pub fn write_selected_project(project_root: &str, selected_by: &str) -> Result<()> {
    write_json_file(
        &selected_profile_path(),
        &SelectedProjectProfile {
            schema: SCHEMA_SELECTED.to_string(),
            selected_project_fingerprint: project_fingerprint(project_root),
            selected_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            selected_by: selected_by.to_string(),
            note: "CLI selected project".to_string(),
        },
    )
}

pub fn resolve_project_scope(
    explicit_project_root: Option<&str>,
    explicit_project_alias: Option<&str>,
    cwd: Option<&str>,
) -> Result<ResolvedProjectScope> {
    if let Some(root) = explicit_project_root.filter(|value| !value.trim().is_empty()) {
        let root = root.trim();
        if let Some(reason) = is_safe_root(root) {
            return Err(anyhow!(
                "[CLI_SCOPE_REJECT] operation=project scope-selection reason={} value={}",
                reason,
                root
            ));
        }
        if root_path_is_project_candidate(Path::new(root)) {
            return Ok(resolved_from_root(root, ScopeSource::ExplicitFlag));
        }
        if Path::new(root).exists() {
            // Explicit, structurally safe, existing path: honor it even without a
            // project marker or commits yet (empty/new repo with a configured
            // remote must be bindable). Do NOT fall through to cwd/env upward
            // walking, which can silently rewrite the root to a broad parent
            // (e.g. /root) and reject the explicit child path.
            return Ok(resolved_from_root(root, ScopeSource::ExplicitFlag));
        }
    }

    if let Some(alias) = explicit_project_alias.filter(|value| !value.trim().is_empty()) {
        let candidate = Path::new(alias.trim());
        if root_path_is_project_candidate(candidate) {
            return Ok(resolved_from_root(
                &candidate.to_string_lossy(),
                ScopeSource::ProjectAlias,
            ));
        }
    }

    if let Some(env_root) = std::env::var("FOCUSA_PROJECT_ROOT")
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
        let env_root = env_root.trim();
        if let Some(reason) = is_safe_root(env_root) {
            return Err(anyhow!(
                "[CLI_SCOPE_REJECT] operation=project scope-selection reason={} value={}",
                reason,
                env_root
            ));
        }
        if root_path_is_project_candidate(Path::new(env_root)) {
            return Ok(resolved_from_root(env_root, ScopeSource::EnvProjectRoot));
        }
    }

    if let Some(selected) = std::env::var("FOCUSA_SELECTED_PROJECT")
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
        let selected = selected.trim().to_string();
        if let Some(profile) =
            selected_fingerprint_from_profile()?.filter(|value| !value.is_empty())
        {
            if let Ok(fallback_root) = std::env::var("FOCUSA_PROJECT_ROOT") {
                if root_path_is_project_candidate(Path::new(&fallback_root)) {
                    let mut resolved =
                        resolved_from_root(&fallback_root, ScopeSource::EnvSelectedProject);
                    resolved.fingerprint = Some(profile);
                    return Ok(resolved);
                }
            }
        }
        if root_path_is_project_candidate(Path::new(&selected)) {
            return Ok(resolved_from_root(
                &selected,
                ScopeSource::EnvSelectedProject,
            ));
        }
    }

    if let Some(legacy) = std::env::var("FOCUSA_ACTIVE_PROJECT")
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
        let legacy = legacy.trim();
        if root_path_is_project_candidate(Path::new(legacy)) {
            return Ok(resolved_from_root(
                legacy,
                ScopeSource::LegacyEnvActiveProject,
            ));
        }
    }

    if let Some(profile_fingerprint) = selected_fingerprint_from_profile()? {
        if !profile_fingerprint.is_empty() {
            if let Some(root) = std::env::var("FOCUSA_PROJECT_ROOT")
                .ok()
                .filter(|value| !value.trim().is_empty())
            {
                if root_path_is_project_candidate(Path::new(root.as_str())) {
                    let mut resolved = resolved_from_root(&root, ScopeSource::CliSelectedProject);
                    resolved.fingerprint = Some(profile_fingerprint);
                    return Ok(resolved);
                }
            }
        }
    }

    if let Some(cwd) = cwd.filter(|value| !value.trim().is_empty()) {
        if let Some((root, source)) = find_upward_root(Path::new(cwd)) {
            return Ok(resolved_from_root(&root.to_string_lossy(), source));
        }
    }

    Err(anyhow!(
        r#"{{"status":"blocked","failure_class":"project_root_selection_required","next_step_hint":"Run focusa project, focusa project discover, or pass --project-root <path>."}}"#
    ))
}

pub fn resolve_active_workstream_scope(cwd: Option<&str>) -> Result<ResolvedProjectScope> {
    let mut resolved = resolve_project_scope(None, None, cwd)?;
    if resolved.continuity_id.is_none() {
        resolved.continuity_id = std::env::var("FOCUSA_CONTINUITY_ID")
            .ok()
            .filter(|value| !value.trim().is_empty());
    }
    if resolved.continuity_id.is_none() {
        return Err(anyhow!(
            r#"{{"status":"not_configured","failure_class":"continuity_scope_required","next_step_hint":"Set FOCUSA_CONTINUITY_ID or select/resume a project Workpoint before querying work-loop state."}}"#
        ));
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_scope_without_candidates() {
        let err =
            resolve_project_scope(Some("/tmp/focusa-does-not-exist"), None, None).unwrap_err();
        assert!(err.to_string().contains("project_root_selection_required"));
    }

    #[test]
    fn explicit_existing_safe_root_is_honored_without_marker_or_commits() {
        // Regression for #300: an explicit, safe, existing project root (e.g. an
        // empty/new git repo with only a remote) must be honored, not rewritten
        // to a broad parent (e.g. /root) by cwd/env upward walking.
        let root = std::env::temp_dir().join("focusa-cli-explicit-plain-dir");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        let resolved = resolve_project_scope(Some(root.to_str().unwrap()), None, None).unwrap();
        assert_eq!(resolved.project_root, root.to_string_lossy());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn explicit_broad_root_still_rejected() {
        let err = resolve_project_scope(Some("/root"), None, None).unwrap_err();
        assert!(err.to_string().contains("unsafe_broad_project_root"));
    }

    #[test]
    fn reads_selected_profile_schema() {
        let root = std::env::temp_dir().join("focusa-cli-selected-test");
        let path = root
            .join(".config")
            .join("focusa")
            .join("selected-project.json");
        let _ = fs::remove_dir_all(&root);

        let profile = SelectedProjectProfile {
            schema: SCHEMA_SELECTED.to_string(),
            selected_project_fingerprint: "project-fnv1a64:example".to_string(),
            selected_at: "2026-07-01T00:00:00Z".to_string(),
            selected_by: "cli".to_string(),
            note: "test".to_string(),
        };
        write_json_file(&path, &profile).unwrap();

        let decoded = read_json_file::<SelectedProjectProfile>(&path).unwrap();
        assert_eq!(
            decoded.selected_project_fingerprint,
            "project-fnv1a64:example"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
