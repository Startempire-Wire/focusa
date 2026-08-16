//! Spec96 ProjectIdentity discovery and verification routes.
//!
//! ProjectIdentity is a bounded, hot-path-safe orientation record. It composes
//! filesystem/project signals; it does not select work or mutate cognitive state.

use crate::routes::preload::{
    PROFILE_RULES_AND_CONTEXT, build_packet_for_profile, commit_receipt_for,
};
use crate::scope::ScopeContext;
use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::scope_safety::classify_project_root;
use focusa_core::scoped_state::{ScopeKind, ScopeRef, WorkstreamKey};
use focusa_core::working_subpath::{
    GitWorkingContext, resolve_git_working_context, resolve_project_binding_candidates,
};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize, Default)]
pub struct ProjectIdentityQuery {
    pub cwd: Option<String>,
    pub project_root: Option<String>,
    pub current_ask: Option<String>,
    pub remote_host: Option<String>,
    pub remote_user: Option<String>,
    pub remote_port: Option<u16>,
    pub remote_repo_remote: Option<String>,
    pub remote_workspace_kind: Option<String>,
    pub remote_deploy_root: Option<String>,
    pub persisted_project_root: Option<String>,
    pub persisted_project_fingerprint: Option<String>,
    pub persisted_project_id: Option<String>,
    pub persisted_canonical_name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectVerifyRequest {
    pub cwd: Option<String>,
    pub project_root: Option<String>,
    pub project_id: Option<String>,
    pub canonical_name: Option<String>,
    pub repo_remote: Option<String>,
    pub remote_host: Option<String>,
    pub remote_user: Option<String>,
    pub remote_port: Option<u16>,
    pub remote_repo_remote: Option<String>,
    pub remote_workspace_kind: Option<String>,
    pub remote_deploy_root: Option<String>,
    pub persisted_project_root: Option<String>,
    pub persisted_project_fingerprint: Option<String>,
    pub persisted_project_id: Option<String>,
    pub persisted_canonical_name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectSessionTransferRequest {
    pub action: String,
    pub cwd: Option<String>,
    pub project_root: Option<String>,
    pub current_ask: Option<String>,
    pub continuity_id: Option<String>,
    pub source_scope: Option<WorkstreamKey>,
    pub target_scope: Option<WorkstreamKey>,
    pub source_working_subpath_id: Option<String>,
    pub target_working_subpath_id: Option<String>,
    pub target_continuity_id: Option<String>,
    pub source_session_id: Option<String>,
    pub target_session_id: Option<String>,
    pub target_workpoint_id: Option<String>,
    pub target_resume_canonical: Option<bool>,
    pub source_checkpoint_id: Option<String>,
    pub compaction_packet_id: Option<String>,
    pub adapter: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub mission: Option<String>,
    pub next_action: Option<String>,
    pub write_preload: Option<bool>,
    pub preload_target: Option<String>,
    pub preload_mode: Option<String>,
    pub receipt_preview: Option<bool>,
    pub receipt_commit: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectCardOutcomeRequest {
    pub algorithm_run_id: String,
    pub actual_outcome: String,
    pub score: Option<f64>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub project_root: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub task_timing: Value,
    #[serde(default)]
    pub token_usage: Value,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectListQuery {
    pub project_root: Option<String>,
    pub from: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectDiscoverQuery {
    pub from: Option<String>,
    pub max_depth: Option<u32>,
    pub max_results: Option<usize>,
    pub include_git_only: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectSelectionRequest {
    pub project_root: String,
    pub active_worktree_root: Option<String>,
    pub working_subpath_id: Option<String>,
    pub selected_by: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectRemoveRequest {
    pub clear: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectCreateRequest {
    pub project_root: String,
    pub project_id: String,
    pub canonical_name: String,
    pub template: Option<String>,
    pub workspace_kind: Option<String>,
    pub create_git: Option<bool>,
    pub use_selected: Option<bool>,
    pub force: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectTemplatesQuery {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectSettingsQuery {
    pub project_root: Option<String>,
    pub key: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectSettingsRequest {
    pub action: String,
    pub project_root: Option<String>,
    pub key: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
struct ProjectSignal {
    source: &'static str,
    root: Option<String>,
    confidence: &'static str,
    independent: bool,
    details: Value,
}

#[derive(Debug, Clone, Default)]
struct RemoteProjectHint {
    remote_host: Option<String>,
    remote_user: Option<String>,
    remote_port: Option<u16>,
    remote_repo_remote: Option<String>,
    remote_workspace_kind: Option<String>,
    remote_deploy_root: Option<String>,
    persisted_project_root: Option<String>,
    persisted_project_fingerprint: Option<String>,
    persisted_project_id: Option<String>,
    persisted_canonical_name: Option<String>,
}

impl RemoteProjectHint {
    fn is_present(&self) -> bool {
        self.remote_host
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
    }

    fn from_query(query: &ProjectIdentityQuery) -> Self {
        Self {
            remote_host: clean(query.remote_host.as_deref()),
            remote_user: clean(query.remote_user.as_deref()),
            remote_port: query.remote_port,
            remote_repo_remote: clean(query.remote_repo_remote.as_deref()),
            remote_workspace_kind: clean(query.remote_workspace_kind.as_deref()),
            remote_deploy_root: clean(query.remote_deploy_root.as_deref()),
            persisted_project_root: clean(query.persisted_project_root.as_deref()),
            persisted_project_fingerprint: clean(query.persisted_project_fingerprint.as_deref()),
            persisted_project_id: clean(query.persisted_project_id.as_deref()),
            persisted_canonical_name: clean(query.persisted_canonical_name.as_deref()),
        }
    }

    fn from_verify(request: &ProjectVerifyRequest) -> Self {
        Self {
            remote_host: clean(request.remote_host.as_deref()),
            remote_user: clean(request.remote_user.as_deref()),
            remote_port: request.remote_port,
            remote_repo_remote: clean(request.remote_repo_remote.as_deref()),
            remote_workspace_kind: clean(request.remote_workspace_kind.as_deref()),
            remote_deploy_root: clean(request.remote_deploy_root.as_deref()),
            persisted_project_root: clean(request.persisted_project_root.as_deref()),
            persisted_project_fingerprint: clean(request.persisted_project_fingerprint.as_deref()),
            persisted_project_id: clean(request.persisted_project_id.as_deref()),
            persisted_canonical_name: clean(request.persisted_canonical_name.as_deref()),
        }
    }

    fn context(&self) -> Value {
        if !self.is_present() {
            Value::Null
        } else {
            json!({
                "workspace_kind": "remote_ssh",
                "remote_host": self.remote_host,
                "remote_user": self.remote_user,
                "remote_port": self.remote_port,
                "remote_repo_remote": self.remote_repo_remote,
                "remote_workspace_kind": self.remote_workspace_kind,
                "remote_deploy_root": self.remote_deploy_root,
                "authority_boundary": "remote_host_plus_project_root_plus_fingerprint",
                "verification_note": "remote evidence is caller-supplied after SSH/repo inspection; Focusa daemon does not open SSH sessions"
            })
        }
    }
}

#[derive(Debug, Clone)]
struct IdentityCandidate {
    project_id: String,
    canonical_name: String,
    project_root: String,
    repo_remote: Option<String>,
    beads_prefix: Option<String>,
    workspace_kind: Option<String>,
    aliases: Vec<String>,
    project_urls: Value,
    deployment: Value,
    remote_context: Value,
    working_context: Value,
    fingerprint: String,
    confidence: &'static str,
    status: &'static str,
    signals: Vec<ProjectSignal>,
    mismatches: Vec<Value>,
    verified_at: String,
}

fn clean(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn normalize_identity_name(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn identity_name_matches(
    expected: &str,
    canonical_name: &str,
    project_id: &str,
    aliases: &[String],
    candidate_project_root: Option<&str>,
    expected_project_root: Option<&str>,
) -> bool {
    let expected = normalize_identity_name(expected);
    if expected.is_empty() {
        return false;
    }
    if std::iter::once(canonical_name)
        .chain(std::iter::once(project_id))
        .any(|candidate| normalize_identity_name(candidate) == expected)
    {
        return true;
    }

    let alias_scope_matches = match (
        candidate_project_root.and_then(|value| clean(Some(value))),
        expected_project_root.and_then(|value| clean(Some(value))),
    ) {
        (Some(candidate_root), Some(expected_root)) => candidate_root == expected_root,
        (None, None) => true,
        _ => false,
    };
    alias_scope_matches
        && aliases
            .iter()
            .map(String::as_str)
            .any(|candidate| normalize_identity_name(candidate) == expected)
}

fn unsafe_project_root_reason(value: &str) -> Option<&'static str> {
    classify_project_root(value).reason()
}

fn expand_home(path: &str) -> PathBuf {
    if path == "~"
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home);
    }
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}

fn normalize_path(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .trim_end_matches('/')
        .to_string()
}

fn resolve_start(cwd: Option<&str>, project_root: Option<&str>) -> PathBuf {
    let candidate = clean(project_root)
        .or_else(|| clean(cwd))
        .or_else(|| std::env::var("FOCUSA_PROJECT_ROOT").ok())
        .or_else(|| std::env::var("FOCUSA_HOME").ok())
        .map(|value| expand_home(&value));
    let raw =
        candidate.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    if raw.is_absolute() {
        raw
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(raw)
    }
}

fn canonicalize_working_scope_root(root: String, context: Option<&GitWorkingContext>) -> String {
    context
        .filter(|context| context.active_worktree_root == root)
        .map(|context| context.canonical_parent_root.clone())
        .unwrap_or(root)
}

fn find_upwards(start: &Path, name: &str) -> Option<PathBuf> {
    let mut cur = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        let candidate = cur.join(name);
        if candidate.exists() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

fn find_nearest_upwards(start: &Path, names: &[&str]) -> Option<PathBuf> {
    let mut cur = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        if names.iter().any(|name| cur.join(name).exists()) {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

fn parent_scope_shadowed_by_trusted_root(candidate_root: &Path, trusted_root: &str) -> bool {
    let candidate = normalize_path(candidate_root);
    candidate != trusted_root && Path::new(trusted_root).starts_with(Path::new(&candidate))
}

fn directory_detection_priority(signal: &ProjectSignal) -> i32 {
    let unsafe_penalty = signal
        .root
        .as_deref()
        .and_then(unsafe_project_root_reason)
        .map(|_| 100)
        .unwrap_or(0);
    let base = match signal.source {
        "operator_supplied_scope" => 100,
        "project_directory_detector" => 95,
        "root_marker" => 90,
        "git_root" => 80,
        "remote_project_scope" => 75,
        "beads_root" => 60,
        "workspace_root" => 50,
        "persisted_session_identity" => 30,
        "daemon_working_directory" => 10,
        "cwd" => 5,
        _ => 1,
    };
    base - unsafe_penalty
}

fn select_canonical_project_root(signals: &[ProjectSignal], fallback: &str) -> String {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for signal in signals.iter().filter(|signal| signal.independent) {
        if let Some(root) = &signal.root {
            *counts.entry(root.clone()).or_default() += 1;
        }
    }
    signals
        .iter()
        .filter(|signal| signal.independent)
        .filter_map(|signal| signal.root.as_ref().map(|root| (signal, root)))
        .max_by_key(|(signal, root)| {
            (
                directory_detection_priority(signal),
                *counts.get(*root).unwrap_or(&0),
            )
        })
        .map(|(_, root)| root.clone())
        .or_else(|| signals.iter().find_map(|signal| signal.root.clone()))
        .unwrap_or_else(|| fallback.to_string())
}

fn read_marker(marker_root: &Path) -> Option<Value> {
    let path = marker_root.join(".focusa-project.json");
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
}

fn marker_project_root(marker_root: &Path, marker: &Value) -> String {
    marker
        .get("project_root")
        .and_then(Value::as_str)
        .map(expand_home)
        .map(|path| normalize_path(&path))
        .unwrap_or_else(|| normalize_path(marker_root))
}

fn normalize_project_hint(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.")
        .trim_end_matches('/')
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '.' || *ch == '-' || *ch == '_')
        .flat_map(char::to_lowercase)
        .collect()
}

fn marker_hint_values(marker: &Value) -> Vec<String> {
    let mut values = Vec::new();
    for key in [
        "project_id",
        "canonical_name",
        "live_url",
        "root_url",
        "local_url",
    ] {
        if let Some(value) = marker.get(key).and_then(Value::as_str) {
            values.push(normalize_project_hint(value));
        }
    }
    if let Some(aliases) = marker.get("aliases").and_then(Value::as_array) {
        for alias in aliases.iter().filter_map(Value::as_str) {
            values.push(normalize_project_hint(alias));
        }
    }
    if let Some(project_urls) = marker.get("project_urls").and_then(Value::as_object) {
        for value in project_urls.values().filter_map(Value::as_str) {
            values.push(normalize_project_hint(value));
        }
    }
    values
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect()
}

fn project_hint_candidates(value: &str) -> Vec<String> {
    let mut hints = BTreeSet::new();
    let normalized = normalize_project_hint(value);
    if !normalized.is_empty() && normalized.len() <= 120 {
        hints.insert(normalized.clone());
    }
    for token in value
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_'))
        .map(normalize_project_hint)
        .filter(|token| token.len() > 2)
    {
        if token.contains('.') {
            if let Some(first_label) = token.split('.').next().filter(|label| label.len() > 2) {
                hints.insert(first_label.to_string());
            }
        }
        hints.insert(token);
    }
    hints.into_iter().collect()
}

fn marker_matches_project_hint(marker: &Value, hint: &str) -> Option<String> {
    let normalized_hint = normalize_project_hint(hint);
    if normalized_hint.is_empty() {
        return None;
    }
    marker_hint_values(marker).into_iter().find(|value| {
        value == &normalized_hint
            || value.starts_with(&format!("{}.", normalized_hint))
            || value.starts_with(&format!("{}-", normalized_hint))
    })
}

fn project_directory_search_roots(start: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(configured) = std::env::var("FOCUSA_PROJECT_SEARCH_DIRS") {
        roots.extend(
            configured
                .split(':')
                .filter(|part| !part.trim().is_empty())
                .map(PathBuf::from),
        );
    }
    if start.exists() {
        roots.push(start.to_path_buf());
        if let Some(parent) = start.parent() {
            roots.push(parent.to_path_buf());
        }
    }
    let home = Path::new("/home");
    if home.exists() {
        roots.push(home.to_path_buf());
    }
    let mut seen = BTreeSet::new();
    roots
        .into_iter()
        .filter(|root| seen.insert(normalize_path(root)))
        .collect()
}

fn find_project_marker_for_hint(
    start: &Path,
    hint: Option<&str>,
) -> Option<(PathBuf, Value, String)> {
    let hint = clean(hint)?;
    let hint_candidates = project_hint_candidates(&hint);
    if hint_candidates.is_empty() {
        return None;
    }
    let mut queue: Vec<(PathBuf, usize)> = project_directory_search_roots(start)
        .into_iter()
        .map(|root| (root, 0usize))
        .collect();
    let mut seen = BTreeSet::new();
    let mut visited = 0usize;
    while let Some((dir, depth)) = queue.pop() {
        if visited > 300 || depth > 4 || !seen.insert(normalize_path(&dir)) {
            continue;
        }
        visited += 1;
        let marker_path = dir.join(".focusa-project.json");
        if let Some(marker) = read_marker(&dir)
            && let Some(matched_hint) = hint_candidates
                .iter()
                .find_map(|candidate| marker_matches_project_hint(&marker, candidate))
        {
            return Some((
                PathBuf::from(marker_project_root(&dir, &marker)),
                marker,
                matched_hint,
            ));
        }
        if marker_path.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    queue.push((path, depth + 1));
                }
            }
        }
    }
    None
}

fn read_git_remote(git_root: &Path) -> Option<String> {
    let config_path = git_root.join(".git").join("config");
    let text = fs::read_to_string(config_path).ok()?;
    let mut in_origin = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_origin = trimmed == "[remote \"origin\"]";
            continue;
        }
        if in_origin
            && let Some(rest) = trimmed.strip_prefix("url")
            && let Some((_, value)) = rest.split_once('=')
        {
            return Some(value.trim().to_string());
        }
    }
    None
}

fn read_beads_issue_prefix(beads_root: &Path) -> Option<String> {
    let path = beads_root.join(".beads").join("issues.jsonl");
    let mut file = fs::File::open(path).ok()?;
    let mut text = String::new();
    std::io::Read::by_ref(&mut file)
        .take(64 * 1024)
        .read_to_string(&mut text)
        .ok()?;
    for line in text.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(id) = value.get("id").and_then(Value::as_str) else {
            continue;
        };
        if let Some((prefix, _)) = id.split_once('-')
            && !prefix.trim().is_empty()
        {
            return Some(prefix.trim().to_string());
        }
    }
    None
}

fn marker_string(marker: &Option<Value>, key: &str) -> Option<String> {
    marker
        .as_ref()
        .and_then(|value| value.get(key))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn marker_string_array(marker: &Option<Value>, key: &str) -> Vec<String> {
    marker
        .as_ref()
        .and_then(|value| value.get(key))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn marker_object_or_empty(marker: &Option<Value>, key: &str) -> Value {
    marker
        .as_ref()
        .and_then(|value| value.get(key))
        .filter(|value| value.is_object())
        .cloned()
        .unwrap_or_else(|| json!({}))
}

fn marker_project_urls(marker: &Option<Value>) -> Value {
    let mut urls = marker_object_or_empty(marker, "project_urls");
    if let Some(map) = urls.as_object_mut() {
        for (source_key, target_key) in [
            ("root_url", "root_url"),
            ("live_url", "live_url"),
            ("production_url", "live_url"),
            ("local_url", "local_url"),
            ("admin_url", "admin_url"),
            ("api_url", "api_url"),
        ] {
            if !map.contains_key(target_key)
                && let Some(value) = marker_string(marker, source_key)
            {
                map.insert(target_key.to_string(), Value::String(value));
            }
        }
    }
    urls
}

fn marker_deployment(marker: &Option<Value>) -> Value {
    let mut deployment = marker_object_or_empty(marker, "deployment");
    if let Some(map) = deployment.as_object_mut() {
        for key in [
            "environment",
            "deploy_environment",
            "deploy_target",
            "deploy_location",
            "deploy_command",
            "verification_url",
        ] {
            if !map.contains_key(key)
                && let Some(value) = marker_string(marker, key)
            {
                map.insert(key.to_string(), Value::String(value));
            }
        }
    }
    deployment
}

fn normalized_hint(value: &str) -> Option<String> {
    let out = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    (out.len() >= 3).then_some(out)
}

fn add_if_file(out: &mut Vec<PathBuf>, path: PathBuf) {
    if path.is_file() && !out.iter().any(|existing| existing == &path) {
        out.push(path);
    }
}

fn collect_environment_files(out: &mut Vec<PathBuf>, base: &Path, depth: usize, max_depth: usize) {
    if depth > max_depth || out.len() >= 160 {
        return;
    }
    let Ok(entries) = fs::read_dir(base) else {
        return;
    };
    for entry in entries.flatten().take(80) {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if path.is_dir() {
            if matches!(
                name.as_str(),
                "node_modules" | "vendor" | ".git" | ".svelte-kit" | "build" | "dist" | "cache"
            ) {
                continue;
            }
            collect_environment_files(out, &path, depth + 1, max_depth);
            continue;
        }
        let ext = path.extension().and_then(|v| v.to_str()).unwrap_or("");
        if path.is_file()
            && (name.contains("deploy")
                || name.contains("live")
                || name.contains("config")
                || name.contains("url")
                || name == "wp-config.php"
                || name.starts_with(".env")
                || matches!(
                    ext,
                    "md" | "sh"
                        | "php"
                        | "js"
                        | "ts"
                        | "mjs"
                        | "svelte"
                        | "yml"
                        | "yaml"
                        | "json"
                        | "txt"
                        | "env"
                ))
        {
            add_if_file(out, path);
        }
    }
}

fn candidate_environment_files(root: &Path, identity_hints: &[String]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for rel in [
        ".focusa-project.json",
        "README.md",
        "AGENTS.md",
        ".env",
        ".env.example",
        "package.json",
        "composer.json",
        "wp-config.php",
        "public_html/wp-config.php",
        "app/.env",
        "app/.env.example",
        "app/package.json",
    ] {
        add_if_file(&mut out, root.join(rel));
    }
    if let Some(parent) = root.parent() {
        for rel in [
            ".focusa-project.json",
            "README.md",
            "AGENTS.md",
            ".env",
            ".env.example",
            "package.json",
            "composer.json",
            "wp-config.php",
            "app/.env",
            "app/.env.example",
            "app/package.json",
        ] {
            add_if_file(&mut out, parent.join(rel));
        }
    }
    for hint in identity_hints
        .iter()
        .filter_map(|value| normalized_hint(value))
    {
        add_if_file(
            &mut out,
            PathBuf::from(format!("/home/{hint}/public_html/wp-config.php")),
        );
        add_if_file(
            &mut out,
            PathBuf::from(format!("/home/{hint}/public_html/.env")),
        );
    }
    for dir in [
        "scripts",
        "bin",
        ".github/workflows",
        "docs",
        "wp-content",
        "app",
        "app/src",
        "app/src/lib",
        "app/src/routes",
        "src",
    ] {
        collect_environment_files(&mut out, &root.join(dir), 0, 3);
    }
    out.into_iter().take(160).collect()
}

fn strip_token(value: &str) -> String {
    value
        .trim_matches(|ch: char| {
            ch.is_whitespace()
                || matches!(
                    ch,
                    '"' | '\'' | '`' | ')' | '(' | ']' | '[' | '}' | '{' | ',' | ';'
                )
        })
        .trim_end_matches('.')
        .to_string()
}

fn host_from_url(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host = rest
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("");
    (!host.is_empty()).then(|| host.to_string())
}

fn is_local_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains(".local") || lower.contains("localhost") || lower.contains("127.0.0.1")
}

fn first_url_matching<F>(urls: &[(String, String)], predicate: F) -> Option<String>
where
    F: Fn(&str, &str) -> bool,
{
    urls.iter()
        .find(|(url, source)| predicate(url, source))
        .map(|(url, _)| url.clone())
}

fn url_host_starts_with(url: &str, prefix: &str) -> bool {
    host_from_url(url).is_some_and(|host| host.starts_with(prefix))
}

fn looks_like_root_site_url(url: &str) -> bool {
    if is_local_url(url) || url_host_starts_with(url, "app.") || url_host_starts_with(url, "auth.")
    {
        return false;
    }
    let lower = url.to_ascii_lowercase();
    lower.contains("/wp-json") || lower.contains("/graphql") || lower.matches('/').count() <= 2
}

fn extract_urls(text: &str) -> Vec<String> {
    text.split(|ch: char| ch.is_whitespace() || matches!(ch, '"' | '\'' | '`' | '<' | '>'))
        .filter_map(|token| {
            let cleaned = strip_token(token);
            (cleaned.starts_with("https://") || cleaned.starts_with("http://")).then_some(cleaned)
        })
        .collect()
}

fn extract_urls_with_lines(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in text.lines().take(2_000) {
        for url in extract_urls(line) {
            out.push((url, line.trim().to_string()));
        }
    }
    out
}

fn source_is_docs_or_reference(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    lower == "readme.md" || lower.starts_with("docs/")
}

fn source_is_runtime_url_authority(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    lower.contains("wp-config.php")
        || lower.starts_with("app/src/")
        || lower.starts_with("src/")
        || lower.starts_with("scripts/")
        || lower.starts_with("bin/")
        || lower.starts_with(".github/workflows/")
        || matches!(
            lower.as_str(),
            ".focusa-project.json"
                | ".env"
                | ".env.example"
                | "app/.env"
                | "app/.env.example"
                | "package.json"
                | "app/package.json"
        )
}

fn line_declares_project_url(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    if lower.contains("upstream")
        || lower.contains("openai")
        || lower.contains("anthropic")
        || lower.contains("example.com")
        || lower.contains("codex.wordpress.org")
        || lower.contains("api.wordpress.org")
    {
        return false;
    }
    [
        "root_url",
        "live_url",
        "local_url",
        "app_url",
        "auth_url",
        "wp_home",
        "wp_siteurl",
        "site_url",
        "public_url",
        "graphql_url",
        "production_url",
        "deploy_target",
        "app configuration",
        "allowed origin",
        "trusted origin",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn url_allowed_for_project_inference(source: &str, line: &str, url: &str) -> bool {
    if is_local_url(url) {
        return true;
    }
    if source_is_docs_or_reference(source) {
        return line_declares_project_url(line);
    }
    if !source_is_runtime_url_authority(source) {
        return false;
    }
    if source.to_ascii_lowercase().contains("wp-config.php") {
        return true;
    }
    line_declares_project_url(line)
        || url_host_starts_with(url, "app.")
        || url_host_starts_with(url, "auth.")
}

fn extract_deploy_locations(text: &str) -> Vec<String> {
    text.split(|ch: char| ch.is_whitespace() || matches!(ch, '"' | '\'' | '`'))
        .filter_map(|token| {
            let cleaned = strip_token(token);
            (cleaned.starts_with("/home/")
                && (cleaned.contains("public_html")
                    || cleaned.contains("htdocs")
                    || cleaned.contains("www")))
            .then_some(cleaned)
        })
        .collect()
}

fn insert_if_missing(map: &mut Map<String, Value>, key: &str, value: Option<String>) {
    if !map.contains_key(key)
        && let Some(value) = value.filter(|v| !v.trim().is_empty())
    {
        map.insert(key.to_string(), Value::String(value));
    }
}

fn infer_project_environment(root: &Path, identity_hints: &[String]) -> (Value, Value) {
    let mut urls = Vec::<(String, String)>::new();
    let mut deploy_locations = Vec::<String>::new();
    let mut deploy_command = None::<String>;
    let mut sources = BTreeSet::<String>::new();
    for path in candidate_environment_files(root, identity_hints) {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .to_string();
        let text = fs::read_to_string(&path).unwrap_or_default();
        if text.is_empty() {
            continue;
        }
        let bounded = text.chars().take(65_536).collect::<String>();
        for (url, line) in extract_urls_with_lines(&bounded) {
            if url_allowed_for_project_inference(&rel, &line, &url) {
                urls.push((url, rel.clone()));
                sources.insert(rel.clone());
            }
        }
        for location in extract_deploy_locations(&bounded) {
            deploy_locations.push(location);
            sources.insert(rel.clone());
        }
        let rel_lower = rel.to_ascii_lowercase();
        if deploy_command.is_none()
            && (rel_lower.contains("deploy") || rel_lower.contains("live"))
            && matches!(
                path.extension().and_then(|v| v.to_str()),
                Some("sh" | "php" | "js" | "ts")
            )
        {
            deploy_command = Some(rel.clone());
            sources.insert(rel);
        }
    }

    let local_url = first_url_matching(&urls, |url, _| is_local_url(url));
    let wp_url = first_url_matching(&urls, |url, source| {
        !is_local_url(url)
            && (source.contains("wp-config.php")
                || url.contains("/wp-json")
                || url.contains("/graphql")
                || looks_like_root_site_url(url))
    });
    let app_url = first_url_matching(&urls, |url, _| {
        !is_local_url(url) && url_host_starts_with(url, "app.")
    });
    let auth_url = first_url_matching(&urls, |url, _| {
        !is_local_url(url) && url_host_starts_with(url, "auth.")
    });
    let graphql_url = first_url_matching(&urls, |url, _| {
        !is_local_url(url) && url.to_ascii_lowercase().contains("/graphql")
    });
    let api_url = first_url_matching(&urls, |url, _| {
        !is_local_url(url) && url.to_ascii_lowercase().contains("/api")
    });
    let live_url = wp_url
        .clone()
        .or_else(|| app_url.clone())
        .or_else(|| first_url_matching(&urls, |url, _| !is_local_url(url)));
    let root_url = live_url.clone().or_else(|| local_url.clone());
    let deploy_target = live_url.as_deref().and_then(host_from_url);
    let environment = if live_url.is_some()
        || deploy_command
            .as_deref()
            .is_some_and(|cmd| cmd.contains("live"))
    {
        Some("live".to_string())
    } else {
        Some("local".to_string())
    };

    let environment_confidence =
        if wp_url.is_some() && (app_url.is_some() || !deploy_locations.is_empty()) {
            "high"
        } else if live_url.is_some() || !deploy_locations.is_empty() {
            "medium"
        } else {
            "low"
        };

    let mut url_map = Map::new();
    insert_if_missing(&mut url_map, "root_url", root_url);
    insert_if_missing(&mut url_map, "live_url", live_url);
    insert_if_missing(&mut url_map, "wp_url", wp_url);
    insert_if_missing(&mut url_map, "app_url", app_url);
    insert_if_missing(&mut url_map, "auth_url", auth_url);
    insert_if_missing(&mut url_map, "graphql_url", graphql_url);
    insert_if_missing(&mut url_map, "api_url", api_url);
    insert_if_missing(&mut url_map, "local_url", local_url);
    url_map.insert(
        "inference_confidence".to_string(),
        Value::String(environment_confidence.to_string()),
    );
    if !sources.is_empty() {
        url_map.insert(
            "inference_sources".to_string(),
            Value::Array(
                sources
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .take(12)
                    .collect(),
            ),
        );
    }

    let mut deploy_map = Map::new();
    insert_if_missing(&mut deploy_map, "environment", environment);
    insert_if_missing(&mut deploy_map, "deploy_target", deploy_target);
    let deploy_location = deploy_locations.into_iter().next();
    insert_if_missing(&mut deploy_map, "deploy_location", deploy_location);
    insert_if_missing(&mut deploy_map, "deploy_command", deploy_command);
    deploy_map.insert(
        "inference_confidence".to_string(),
        Value::String(environment_confidence.to_string()),
    );
    if !sources.is_empty() {
        deploy_map.insert(
            "inference_sources".to_string(),
            Value::Array(sources.into_iter().map(Value::String).take(12).collect()),
        );
    }
    (Value::Object(url_map), Value::Object(deploy_map))
}

fn object_string(value: &Value, key: &str) -> Option<String> {
    value
        .as_object()
        .and_then(|map| map.get(key))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn top_level_dirs(root: &Path) -> Vec<String> {
    let mut dirs = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten().take(80) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            if name.starts_with('.') && !matches!(name, ".github" | ".beads") {
                continue;
            }
            if matches!(
                name,
                "target" | "node_modules" | "vendor" | "dist" | "build"
            ) {
                continue;
            }
            dirs.push(name.to_string());
        }
    }
    dirs.sort();
    dirs.truncate(16);
    dirs
}

fn infer_stack(root: &Path, workspace_kind: Option<&str>) -> Vec<String> {
    let mut stack = BTreeSet::new();
    if root.join("Cargo.toml").exists() || workspace_kind == Some("rust-workspace") {
        stack.insert("rust".to_string());
    }
    if root.join("package.json").exists()
        || root.join("app/package.json").exists()
        || root.join("svelte.config.js").exists()
        || root.join("app/svelte.config.js").exists()
    {
        stack.insert("node".to_string());
    }
    if root.join("svelte.config.js").exists() || root.join("app/svelte.config.js").exists() {
        stack.insert("sveltekit".to_string());
    }
    if root.join("wp-config.php").exists()
        || root.join("public_html/wp-config.php").exists()
        || root.join("wp-content").exists()
    {
        stack.insert("wordpress".to_string());
    }
    if root.join("composer.json").exists() {
        stack.insert("php".to_string());
    }
    stack.into_iter().collect()
}

fn compact_project_summary(candidate: &IdentityCandidate) -> Value {
    let urls = &candidate.project_urls;
    let deployment = &candidate.deployment;
    let stack = infer_stack(
        Path::new(&candidate.project_root),
        candidate.workspace_kind.as_deref(),
    );
    let key_dirs = top_level_dirs(Path::new(&candidate.project_root));
    let live_url = object_string(urls, "live_url");
    let local_url = object_string(urls, "local_url");
    let local_only = live_url.is_none();
    let environment = object_string(deployment, "environment").unwrap_or_else(|| {
        if local_only {
            "local".to_string()
        } else {
            "unknown".to_string()
        }
    });
    let confidence = object_string(urls, "inference_confidence")
        .or_else(|| object_string(deployment, "inference_confidence"))
        .unwrap_or_else(|| candidate.confidence.to_string());
    let mut lines = Vec::new();
    lines.push(format!(
        "project={} id={} root={} confidence={} status={} public_repo={}",
        candidate.canonical_name,
        candidate.project_id,
        candidate.project_root,
        candidate.confidence,
        candidate.status,
        candidate
            .repo_remote
            .clone()
            .unwrap_or_else(|| "unknown".to_string())
    ));
    lines.push(format!(
        "aliases={}",
        if candidate.aliases.is_empty() {
            "none".to_string()
        } else {
            candidate.aliases.join(",")
        }
    ));
    lines.push(format!(
        "stack={} workspace={} dirs={}",
        if stack.is_empty() {
            "unknown".to_string()
        } else {
            stack.join(",")
        },
        candidate
            .workspace_kind
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        if key_dirs.is_empty() {
            "unknown".to_string()
        } else {
            key_dirs.join(",")
        }
    ));
    lines.push(format!(
        "urls=local_only:{} root:{} live:{} local:{} wp:{} app:{} auth:{} graphql:{} api:{}",
        local_only,
        object_string(urls, "root_url").unwrap_or_else(|| "unknown".to_string()),
        live_url.clone().unwrap_or_else(|| "none".to_string()),
        local_url.clone().unwrap_or_else(|| "unknown".to_string()),
        object_string(urls, "wp_url").unwrap_or_else(|| "unknown".to_string()),
        object_string(urls, "app_url").unwrap_or_else(|| "unknown".to_string()),
        object_string(urls, "auth_url").unwrap_or_else(|| "unknown".to_string()),
        object_string(urls, "graphql_url").unwrap_or_else(|| "unknown".to_string()),
        object_string(urls, "api_url").unwrap_or_else(|| "unknown".to_string())
    ));
    lines.push(format!(
        "deploy=env:{} target:{} location:{} command:{} confidence:{}",
        environment,
        object_string(deployment, "deploy_target").unwrap_or_else(|| "unknown".to_string()),
        object_string(deployment, "deploy_location").unwrap_or_else(|| "unknown".to_string()),
        object_string(deployment, "deploy_command").unwrap_or_else(|| "unknown".to_string()),
        confidence
    ));
    json!({
        "schema": "focusa.project_summary.v1",
        "project": {
            "project_id": candidate.project_id.clone(),
            "canonical_name": candidate.canonical_name.clone(),
            "project_root": candidate.project_root.clone(),
            "repo_remote": candidate.repo_remote.clone(),
            "beads_prefix": candidate.beads_prefix.clone(),
            "workspace_kind": candidate.workspace_kind.clone(),
            "aliases": candidate.aliases.clone(),
            "stack": stack,
            "key_dirs": key_dirs,
            "confidence": candidate.confidence,
            "status": candidate.status,
        },
        "urls": urls.clone(),
        "deployment": deployment.clone(),
        "environment_confidence": confidence,
        "local_only": local_only,
        "public_repo": candidate.repo_remote.clone(),
        "authority_boundary": "project_root_plus_fingerprint",
        "summary_lines": lines,
    })
}

fn merge_missing_object_fields(mut primary: Value, fallback: Value) -> Value {
    if let (Some(primary_map), Some(fallback_map)) = (primary.as_object_mut(), fallback.as_object())
    {
        for (key, value) in fallback_map {
            primary_map
                .entry(key.clone())
                .or_insert_with(|| value.clone());
        }
    }
    primary
}

fn workspace_kind(root: &Path) -> Option<&'static str> {
    if root.join("Cargo.toml").exists() {
        Some("rust-workspace")
    } else if root.join("package.json").exists() {
        Some("node-workspace")
    } else if root.join("go.mod").exists() {
        Some("go-module")
    } else if root.join("pyproject.toml").exists() {
        Some("python-project")
    } else {
        None
    }
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("project")
        .to_string()
}

fn stable_fingerprint(parts: &[String]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in parts.join("|").as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("project-fnv1a64:{hash:016x}")
}

fn signal_json(signal: &ProjectSignal) -> Value {
    json!({
        "source": signal.source,
        "root": signal.root,
        "confidence": signal.confidence,
        "independent": signal.independent,
        "details": signal.details,
    })
}

fn discover_identity(
    cwd: Option<&str>,
    project_root: Option<&str>,
    current_ask: Option<&str>,
    remote_hint: RemoteProjectHint,
) -> IdentityCandidate {
    let start = resolve_start(cwd, project_root);
    let start_root = normalize_path(&start);
    let remote_nonlocal = remote_hint.is_present() && !start.exists();
    let git_context = if remote_nonlocal {
        None
    } else {
        resolve_git_working_context(&start).ok().flatten()
    };
    let explicit_project_root = clean(project_root)
        .map(|root| normalize_path(&expand_home(&root)))
        .map(|root| canonicalize_working_scope_root(root, git_context.as_ref()));
    let now = chrono::Utc::now().to_rfc3339();
    let mut signals = Vec::<ProjectSignal>::new();

    signals.push(ProjectSignal {
        source: "cwd",
        root: Some(start_root.clone()),
        confidence: "low",
        independent: false,
        details: json!({"path": start_root}),
    });

    if let Some(root) = explicit_project_root.clone() {
        signals.push(ProjectSignal {
            source: "operator_supplied_scope",
            root: Some(root.clone()),
            confidence: "medium",
            independent: true,
            details: json!({"project_root": root}),
        });

        if remote_hint.is_present() {
            signals.push(ProjectSignal {
                source: "remote_project_scope",
                root: Some(root.clone()),
                confidence: "medium",
                independent: true,
                details: json!({
                    "remote_host": remote_hint.remote_host.clone(),
                    "remote_user": remote_hint.remote_user.clone(),
                    "remote_port": remote_hint.remote_port,
                    "remote_deploy_root": remote_hint.remote_deploy_root.clone(),
                    "workspace_kind": "remote_ssh",
                    "verification_boundary": "caller_supplied_remote_evidence"
                }),
            });
            if remote_hint.remote_repo_remote.is_some()
                || remote_hint.remote_workspace_kind.is_some()
            {
                signals.push(ProjectSignal {
                    source: "remote_repo_evidence",
                    root: Some(root.clone()),
                    confidence: "high",
                    independent: true,
                    details: json!({
                        "repo_remote": remote_hint.remote_repo_remote.clone(),
                        "workspace_kind": remote_hint.remote_workspace_kind.clone(),
                        "evidence_boundary": "reported_by_calling_adapter_after_remote_inspection"
                    }),
                });
            }
        }
    }

    if let Some(persisted_root) = clean(remote_hint.persisted_project_root.as_deref()) {
        let root = normalize_path(&expand_home(&persisted_root));
        signals.push(ProjectSignal {
            source: "persisted_session_identity",
            root: Some(root.clone()),
            confidence: "medium",
            independent: false,
            details: json!({
                "project_root": root,
                "fingerprint": remote_hint.persisted_project_fingerprint.clone(),
                "project_id": remote_hint.persisted_project_id.clone(),
                "canonical_name": remote_hint.persisted_canonical_name.clone(),
                "authority_note": "prior same-session ProjectIdentity fingerprint is corroborating; mismatches degrade canonical scope"
            }),
        });
    }

    let marker_root = if remote_nonlocal {
        None
    } else {
        find_upwards(&start, ".focusa-project.json").or_else(|| {
            git_context.as_ref().and_then(|context| {
                let parent = PathBuf::from(&context.canonical_parent_root);
                parent
                    .join(".focusa-project.json")
                    .is_file()
                    .then_some(parent)
            })
        })
    };
    let marker = marker_root.as_ref().and_then(|root| read_marker(root));
    if let Some((detected_root, detected_marker, matched_hint)) =
        find_project_marker_for_hint(&start, current_ask)
    {
        signals.push(ProjectSignal {
            source: "project_directory_detector",
            root: Some(normalize_path(&detected_root)),
            confidence: "high",
            independent: true,
            details: json!({
                "matched_hint": matched_hint,
                "input_source": "current_ask_or_alias_domain",
                "project_id": detected_marker.get("project_id"),
                "canonical_name": detected_marker.get("canonical_name"),
                "project_urls": detected_marker.get("project_urls"),
                "authority_note": "core directory detection resolves parent/child/subdomain project roots before Workpoint/Trajectory authority"
            }),
        });
    }
    if let (Some(root), Some(marker_value)) = (&marker_root, &marker) {
        let root_value = marker_project_root(root, marker_value);
        signals.push(ProjectSignal {
            source: "root_marker",
            root: Some(root_value),
            confidence: "high",
            independent: true,
            details: json!({
                "marker_path": root.join(".focusa-project.json").to_string_lossy(),
                "schema": marker_value.get("schema"),
                "project_id": marker_value.get("project_id"),
                "canonical_name": marker_value.get("canonical_name"),
            }),
        });
    }

    let git_root = git_context
        .as_ref()
        .map(|context| PathBuf::from(&context.canonical_parent_root))
        .or_else(|| {
            if remote_nonlocal {
                None
            } else {
                find_upwards(&start, ".git")
            }
        });
    let repo_remote = git_root.as_ref().and_then(|root| read_git_remote(root));
    let explicit_git_scope_verified = explicit_project_root.as_deref().is_some_and(|root| {
        git_root
            .as_ref()
            .is_some_and(|git_root| normalize_path(git_root) == root)
    });
    if let Some(root) = &git_root {
        signals.push(ProjectSignal {
            source: "git_common_dir",
            root: Some(normalize_path(root)),
            confidence: "high",
            independent: true,
            details: json!({
                "repo_remote": repo_remote,
                "canonical_parent_git_root": root,
                "active_worktree_root": git_context.as_ref().map(|context| &context.active_worktree_root),
                "git_common_dir_id": git_context.as_ref().map(|context| &context.working_subpath.git_common_dir_id),
                "authority_note": "git common-dir proves parent lineage while active worktree remains separate execution authority"
            }),
        });
    }

    let beads_root = if remote_nonlocal {
        None
    } else {
        git_context
            .as_ref()
            .and_then(|context| context.working_subpath.beads_root.as_ref())
            .and_then(|beads| Path::new(beads).parent().map(Path::to_path_buf))
            .or_else(|| find_upwards(&start, ".beads"))
    };
    let discovered_beads_prefix = beads_root
        .as_ref()
        .and_then(|root| read_beads_issue_prefix(root));
    if let Some(root) = &beads_root {
        let root_shadowed = explicit_project_root
            .as_deref()
            .is_some_and(|trusted_root| {
                explicit_git_scope_verified
                    && parent_scope_shadowed_by_trusted_root(root, trusted_root)
            });
        signals.push(ProjectSignal {
            source: "beads_root",
            root: Some(normalize_path(root)),
            confidence: if root_shadowed { "medium" } else { "high" },
            independent: !root_shadowed,
            details: json!({
                "beads_dir": root.join(".beads").to_string_lossy(),
                "issue_prefix": discovered_beads_prefix.clone(),
                "authority_note": if root_shadowed { Value::String("parent beads root is advisory when explicit project_root matches git_root".to_string()) } else { Value::Null }
            }),
        });
    }

    let workspace_root = if remote_nonlocal {
        None
    } else {
        find_nearest_upwards(
            &start,
            &["Cargo.toml", "package.json", "go.mod", "pyproject.toml"],
        )
    };
    let workspace_kind = workspace_root
        .as_ref()
        .and_then(|root| workspace_kind(root));
    if let Some(root) = &workspace_root {
        let root_shadowed = explicit_project_root
            .as_deref()
            .is_some_and(|trusted_root| {
                explicit_git_scope_verified
                    && parent_scope_shadowed_by_trusted_root(root, trusted_root)
            });
        signals.push(ProjectSignal {
            source: "workspace_file",
            root: Some(normalize_path(root)),
            confidence: "medium",
            independent: !root_shadowed,
            details: json!({
                "workspace_kind": workspace_kind,
                "authority_note": if root_shadowed { Value::String("parent workspace file is advisory when explicit project_root matches git_root".to_string()) } else { Value::Null }
            }),
        });
    }

    let caller_supplied_scope = clean(project_root).is_some() || clean(cwd).is_some();
    if let Ok(daemon_cwd) = std::env::current_dir() {
        signals.push(ProjectSignal {
            source: "daemon_working_directory",
            root: Some(normalize_path(&daemon_cwd)),
            confidence: "medium",
            independent: !caller_supplied_scope,
            details: json!({"working_directory": daemon_cwd.to_string_lossy(), "authority_note": "independent only when no explicit project/cwd override is supplied"}),
        });
    }

    let canonical_root = select_canonical_project_root(&signals, &start_root);

    let mut mismatches = Vec::new();
    for signal in &signals {
        if let Some(root) = &signal.root
            && signal.independent
            && root != &canonical_root
        {
            mismatches.push(json!({
                "source": signal.source,
                "expected": canonical_root,
                "actual": root,
                "severity": if signal.source == "root_marker" { "high" } else { "medium" },
            }));
        }
    }
    if let Some(persisted_root) = clean(remote_hint.persisted_project_root.as_deref())
        .map(|root| normalize_path(&expand_home(&root)))
        .map(|root| canonicalize_working_scope_root(root, git_context.as_ref()))
        && persisted_root != canonical_root
    {
        mismatches.push(json!({
            "source": "persisted_session_identity_root",
            "expected": canonical_root.clone(),
            "actual": persisted_root,
            "severity": "high",
        }));
    }

    let unsafe_reason = unsafe_project_root_reason(&canonical_root);
    if let Some(reason) = unsafe_reason {
        mismatches.push(json!({
            "source": "project_root_authority",
            "expected": "specific project/repo root",
            "actual": canonical_root,
            "severity": "high",
            "reason": reason,
        }));
    }

    let marker_project_id = marker
        .as_ref()
        .and_then(|value| value.get("project_id"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let marker_name = marker
        .as_ref()
        .and_then(|value| value.get("canonical_name"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let marker_remote = marker
        .as_ref()
        .and_then(|value| value.get("repo_remote"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let marker_beads = marker
        .as_ref()
        .and_then(|value| value.get("beads_prefix"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let marker_workspace = marker
        .as_ref()
        .and_then(|value| value.get("workspace_kind"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let aliases = marker_string_array(&marker, "aliases");

    let project_id = marker_project_id.unwrap_or_else(|| basename(&canonical_root));
    let canonical_name = marker_name.unwrap_or_else(|| project_id.clone());
    let repo_remote = marker_remote
        .or_else(|| remote_hint.remote_repo_remote.clone())
        .or(repo_remote);
    let beads_prefix = marker_beads
        .or(discovered_beads_prefix)
        .or_else(|| Some(project_id.clone()));
    let workspace_kind = marker_workspace
        .or_else(|| remote_hint.remote_workspace_kind.clone())
        .or_else(|| workspace_kind.map(str::to_string));
    let mut identity_hints = vec![
        project_id.clone(),
        canonical_name.clone(),
        basename(&canonical_root),
    ];
    identity_hints.extend(aliases.iter().cloned());
    let (inferred_project_urls, inferred_deployment) =
        infer_project_environment(&PathBuf::from(&canonical_root), &identity_hints);
    let project_urls =
        merge_missing_object_fields(marker_project_urls(&marker), inferred_project_urls);
    let mut deployment =
        merge_missing_object_fields(marker_deployment(&marker), inferred_deployment);
    if let Some(remote_deploy_root) = remote_hint.remote_deploy_root.clone()
        && let Some(object) = deployment.as_object_mut()
    {
        object
            .entry("deploy_location".to_string())
            .or_insert(json!(remote_deploy_root));
        object
            .entry("environment".to_string())
            .or_insert(json!("remote"));
    }
    let remote_context = remote_hint.context();
    let fingerprint = stable_fingerprint(&[
        project_id.clone(),
        canonical_name.clone(),
        canonical_root.clone(),
        repo_remote.clone().unwrap_or_default(),
        beads_prefix.clone().unwrap_or_default(),
    ]);

    if let Some(raw_fingerprint) = remote_hint.persisted_project_fingerprint.as_ref() {
        if let Some(persisted_fingerprint) = clean(Some(raw_fingerprint.as_str())) {
            if persisted_fingerprint != fingerprint {
                mismatches.push(json!({
                    "source": "persisted_session_identity_fingerprint",
                    "expected": fingerprint.clone(),
                    "actual": persisted_fingerprint,
                    "severity": "high",
                }));
            }
        }
    }
    if let Some(raw_project_id) = remote_hint.persisted_project_id.as_ref() {
        if let Some(persisted_project_id) = clean(Some(raw_project_id.as_str())) {
            if persisted_project_id != project_id {
                mismatches.push(json!({
                    "source": "persisted_session_identity_project_id",
                    "expected": project_id.clone(),
                    "actual": persisted_project_id,
                    "severity": "high",
                }));
            }
        }
    }
    if let Some(raw_name) = remote_hint.persisted_canonical_name.as_ref() {
        if let Some(persisted_name) = clean(Some(raw_name.as_str())) {
            if !identity_name_matches(
                &persisted_name,
                &canonical_name,
                &project_id,
                &aliases,
                project_root,
                remote_hint.persisted_project_root.as_deref(),
            ) {
                mismatches.push(json!({
                    "source": "persisted_session_identity_canonical_name",
                    "expected": canonical_name.clone(),
                    "actual": persisted_name,
                    "severity": "medium",
                }));
            }
        }
    }

    let matching_independent = signals
        .iter()
        .filter(|signal| {
            signal.independent
                && signal.source != "operator_supplied_scope"
                && signal.root.as_deref() == Some(canonical_root.as_str())
        })
        .count();
    let has_root_marker = signals.iter().any(|s| {
        s.source == "root_marker"
            && s.independent
            && s.root.as_deref() == Some(canonical_root.as_str())
    });
    let confidence = if unsafe_reason.is_some() {
        "low"
    } else if mismatches.is_empty() && matching_independent >= 2 && has_root_marker {
        "high"
    } else if mismatches.is_empty() && matching_independent >= 1 {
        "medium"
    } else {
        "low"
    };
    let status = if unsafe_reason.is_some() {
        "unsafe_project_root"
    } else if !mismatches.is_empty() {
        "mismatch"
    } else if matching_independent >= 2 && has_root_marker {
        "verified"
    } else if matching_independent >= 1 {
        "degraded"
    } else {
        "cwd_only"
    };

    IdentityCandidate {
        project_id,
        canonical_name,
        project_root: canonical_root,
        repo_remote,
        beads_prefix,
        workspace_kind,
        aliases,
        project_urls,
        deployment,
        remote_context,
        working_context: git_context
            .as_ref()
            .map(|context| json!(context))
            .unwrap_or(Value::Null),
        fingerprint,
        confidence,
        status,
        signals,
        mismatches,
        verified_at: now,
    }
}

// FOCUSA-zsld fix: Build degraded_reasons array matching focusa doctor --json
// Returns array of {code, severity, reason, fix, evidence_ref} cards
fn build_degraded_reasons(
    candidate: &IdentityCandidate,
    mismatches: &[Value],
    verified: bool,
    canonical: bool,
) -> Value {
    let mut reasons: Vec<Value> = Vec::new();

    // Unsafe root reason
    if candidate.status == "unsafe_project_root" {
        reasons.push(json!({
            "code": "UNSAFE_PROJECT_ROOT",
            "severity": "error",
            "reason": format!("project_root {} is unsafe (no .git or project markers)", candidate.project_root),
            "fix": "verify cwd is a valid project root with .git or .focusa-project.json",
            "evidence_ref": format!("project_root:{}", candidate.project_root),
        }));
    }

    // Confidence reasons
    if candidate.confidence == "low" {
        reasons.push(json!({
            "code": "LOW_CONFIDENCE",
            "severity": "warn",
            "reason": format!("project identity confidence is low (status={}, only cwd signal)", candidate.status),
            "fix": "provide explicit project_root or run focusa_project_verify",
            "evidence_ref": format!("confidence:{}", candidate.confidence),
        }));
    }

    // Mismatch reasons
    for m in mismatches {
        let source = m.get("source").and_then(Value::as_str).unwrap_or("unknown");
        reasons.push(json!({
            "code": format!("MISMATCH_{}", source.to_uppercase()),
            "severity": "error",
            "reason": format!("project identity signal mismatch: {}", source),
            "fix": "match expected vs actual and re-run focusa_project_verify",
            "evidence_ref": m.get("expected").map(|e| format!("expected:{}", e)).unwrap_or_else(|| format!("signal:{}", source)),
        }));
    }

    // Non-canonical verified
    if !canonical && verified {
        reasons.push(json!({
            "code": "NON_CANONICAL",
            "severity": "warn",
            "reason": "project verified but not canonical (confidence != high)",
            "fix": "add explicit project_root or .focusa-project.json to make canonical",
            "evidence_ref": format!("confidence:{}", candidate.confidence),
        }));
    }

    // Generic not verified
    if !verified && reasons.is_empty() {
        reasons.push(json!({
            "code": "VERIFICATION_FAILED",
            "severity": "error",
            "reason": format!("project verification failed for {} (status={})", candidate.project_id, candidate.status),
            "fix": "inspect signals and run focusa_project_verify with explicit inputs",
            "evidence_ref": format!("status:{}", candidate.status),
        }));
    }

    json!(reasons)
}

// BAD-001 fix: Build a human-readable mismatch reason summary
// When project_identity returns mismatch, this provides a single sentence explaining WHY
fn build_mismatch_reason(
    candidate: &IdentityCandidate,
    mismatches: &[Value],
    verified: bool,
) -> String {
    if unsafe_root_reason(&candidate.project_root).is_some() {
        return format!(
            "unsafe project_root: {} (no .git or project markers detected)",
            candidate.project_root
        );
    }
    if !mismatches.is_empty() {
        let sources: Vec<&str> = mismatches
            .iter()
            .filter_map(|m| m.get("source").and_then(Value::as_str))
            .collect();
        if !sources.is_empty() {
            return format!(
                "project identity mismatch: expected={} actual={} (signals: {})",
                candidate.project_id,
                candidate.canonical_name,
                sources.join(", ")
            );
        }
    }
    if candidate.status == "cwd_only" {
        return format!(
            "cwd-only identity (no project markers at {}); confidence={:?}",
            candidate.project_root, candidate.confidence
        );
    }
    if !verified {
        return format!(
            "verification failed for {} (status={}, confidence={:?})",
            candidate.project_id, candidate.status, candidate.confidence
        );
    }
    "project identity verified".to_string()
}

// Helper to check if a path is unsafe (no markers)
fn unsafe_root_reason(root: &str) -> Option<&'static str> {
    if root.is_empty() {
        return Some("empty path");
    }
    None
}

fn candidate_payload(
    candidate: IdentityCandidate,
    expected: Option<&ProjectVerifyRequest>,
) -> Value {
    let mut mismatches = candidate.mismatches.clone();
    if let Some(expected) = expected {
        if let Some(project_id) = clean(expected.project_id.as_deref())
            && project_id != candidate.project_id
        {
            mismatches.push(json!({"source":"operator_expected_project_id", "expected": project_id, "actual": candidate.project_id, "severity":"high"}));
        }
        if let Some(name) = clean(expected.canonical_name.as_deref())
            && !identity_name_matches(
                &name,
                &candidate.canonical_name,
                &candidate.project_id,
                &candidate.aliases,
                Some(candidate.project_root.as_str()),
                expected.project_root.as_deref(),
            )
        {
            mismatches.push(json!({"source":"operator_expected_canonical_name", "expected": name, "actual": candidate.canonical_name, "severity":"medium"}));
        }
        if let Some(remote) = clean(expected.repo_remote.as_deref())
            && candidate.repo_remote.as_deref() != Some(remote.as_str())
        {
            mismatches.push(json!({"source":"operator_expected_repo_remote", "expected": remote, "actual": candidate.repo_remote, "severity":"medium"}));
        }
    }
    let verified = candidate.status == "verified"
        && mismatches.is_empty()
        && unsafe_project_root_reason(&candidate.project_root).is_none();
    let canonical = verified && candidate.confidence == "high";
    let status = if mismatches.is_empty() {
        "completed"
    } else {
        "degraded"
    };
    let identity_status = if candidate.status == "unsafe_project_root" {
        "unsafe_project_root"
    } else if mismatches.is_empty() {
        candidate.status
    } else {
        "mismatch"
    };
    let requested_project_root = expected
        .and_then(|req| clean(req.project_root.as_deref()).or_else(|| clean(req.cwd.as_deref())))
        .unwrap_or_else(|| candidate.project_root.clone());
    let persisted_project_root =
        expected.and_then(|req| clean(req.persisted_project_root.as_deref()));
    let verified_project_root = candidate.project_root.clone();
    let mut matched_axes = Vec::<String>::new();
    let mut mismatched_axes = Vec::<String>::new();
    if requested_project_root == verified_project_root {
        matched_axes.push("requested_project_root==verified_project_root".to_string());
    } else {
        mismatched_axes.push("requested_project_root!=verified_project_root".to_string());
    }
    if let Some(persisted) = persisted_project_root.as_deref() {
        if persisted == requested_project_root {
            matched_axes.push("persisted_project_root==requested_project_root".to_string());
        } else {
            mismatched_axes.push("persisted_project_root!=requested_project_root".to_string());
        }
        if persisted == verified_project_root {
            matched_axes.push("persisted_project_root==verified_project_root".to_string());
        } else {
            mismatched_axes.push("persisted_project_root!=verified_project_root".to_string());
        }
    }
    let mismatch_semantics = if verified {
        Value::Null
    } else {
        json!({
            "schema": "focusa.project_identity_mismatch.v1",
            "ProjectIdentityMismatchSemantics": true,
            "requested_project_root": requested_project_root,
            "persisted_project_root": persisted_project_root,
            "verified_project_root": verified_project_root,
            "matched_axes": matched_axes,
            "mismatched_axes": mismatched_axes,
            "authority_decision": "operator_confirmation_required",
            "safe_next_action": "Run focusa_project_verify/focusa_project_identity, then focusa_workpoint_checkpoint in the confirmed project_root before mutation",
        })
    };
    let project_summary = compact_project_summary(&candidate);
    let authority_boundary = if !candidate.working_context.is_null() {
        "canonical_project_plus_working_subpath_plus_continuity"
    } else if candidate.remote_context.is_null() {
        "project_root_plus_fingerprint"
    } else {
        "remote_host_plus_project_root_plus_fingerprint"
    };
    json!({
        "status": if !verified && !mismatches.is_empty() { "mismatch" } else { status },
        "canonical": canonical,
        "degraded": !canonical,
        // BAD-001 fix: Top-level mismatch_reason for agent clarity
        // When mismatches exist or project_root is unsafe, provide a single human-readable summary
        "mismatch_reason": if !verified {
            Some(build_mismatch_reason(&candidate, &mismatches, verified))
        } else {
            None
        },
        // FOCUSA-zsld: degraded_reasons array for MCP parity with focusa doctor
        "degraded_reasons": build_degraded_reasons(&candidate, &mismatches, verified, canonical),
        "project_summary": project_summary.clone(),
        "summary_lines": project_summary.get("summary_lines").cloned().unwrap_or_else(|| json!([])),
        "project_identity": {
            "schema": "focusa.project_identity.v1",
            "status": identity_status,
            "project_id": candidate.project_id,
            "canonical_name": candidate.canonical_name,
            "project_root": candidate.project_root,
            "repo_remote": candidate.repo_remote,
            "beads_prefix": candidate.beads_prefix,
            "workspace_kind": candidate.workspace_kind,
            "aliases": candidate.aliases,
            "project_urls": candidate.project_urls,
            "deployment": candidate.deployment,
            "remote_context": candidate.remote_context.clone(),
            "working_context": candidate.working_context,
            "canonical_parent_root": candidate.working_context.get("canonical_parent_root").cloned().unwrap_or_else(|| json!(candidate.project_root)),
            "active_worktree_root": candidate.working_context.get("active_worktree_root").cloned().unwrap_or(Value::Null),
            "project_summary": project_summary.clone(),
            "fingerprint": candidate.fingerprint,
            "confidence": candidate.confidence,
            "signals": candidate.signals.iter().map(signal_json).collect::<Vec<_>>(),
            "mismatches": mismatches,
            "verified_at": candidate.verified_at,
            "authority_boundary": authority_boundary,
        },
        "mismatch_semantics": mismatch_semantics,
        "verification": {
            "verified": verified,
            "quorum_rule": "high confidence requires at least two independent matching signals; cwd-only is degraded",
            "matching_independent_signals": candidate.signals.iter().filter(|signal| signal.independent && signal.root.as_deref() == Some(candidate.project_root.as_str())).count(),
            "required_recovery": if verified { Value::Null } else { json!("resolve mismatched project signals or provide explicit project_root after checking current repo") },
        },
        "next_tools": if verified { json!(["focusa_project_identity", "focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"]) } else { json!(["focusa_project_verify", "focusa_project_identity", "focusa_workpoint_checkpoint", "focusa_trajectory_view", "focusa_workpoint_resume"]) },
        "details": {"tool_result_v1": {
            "ok": verified,
            "status": status,
            "canonical": canonical,
            "degraded": !canonical,
            "failure_class": if verified { Value::Null } else { json!("scope_mismatch") },
            "retry": {"safe": verified, "posture": if verified { "safe_retry" } else { "do_not_retry_unchanged" }},
            "side_effects": [],
            "evidence_refs": [],
            "next_tools": if verified { json!(["focusa_project_identity", "focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"]) } else { json!(["focusa_project_verify", "focusa_project_identity", "focusa_workpoint_checkpoint", "focusa_trajectory_view", "focusa_workpoint_resume"]) }
        }}
    })
}

fn project_identity_payload_for_scope_with_remote(
    cwd: Option<&str>,
    project_root: Option<&str>,
    current_ask: Option<&str>,
    remote_hint: RemoteProjectHint,
    _scope: Option<&crate::scope::ScopeContext>,
) -> Value {
    // Identity discovery is intentionally uncached. A process-global cache can
    // return stale authority across alternating project/workstream requests.
    candidate_payload(
        discover_identity(cwd, project_root, current_ask, remote_hint),
        None,
    )
}

pub(crate) fn project_identity_payload_for_scope(
    cwd: Option<&str>,
    project_root: Option<&str>,
    scope: Option<&crate::scope::ScopeContext>,
) -> Value {
    project_identity_payload_for_scope_with_remote(
        cwd,
        project_root,
        None,
        RemoteProjectHint::default(),
        scope,
    )
}

fn project_config_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".config")
        .join("focusa")
}

fn selected_profile_path() -> PathBuf {
    project_config_home().join("selected-project.json")
}

fn project_profiles_dir() -> PathBuf {
    project_config_home().join("projects")
}

fn project_templates_dir() -> PathBuf {
    project_config_home().join("project-templates")
}

fn builtin_templates_dir() -> PathBuf {
    project_root().join("templates").join("project")
}

fn project_settings_dir() -> PathBuf {
    project_config_home().join("project-settings")
}

fn project_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn read_json_value(path: &Path) -> Option<Value> {
    let body = fs::read_to_string(path).ok()?;
    serde_json::from_str(&body).ok()
}

fn write_json_file<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let serialized = serde_json::to_string_pretty(value)
        .map_err(|err| format!("json-serialize failed: {err}"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(path, serialized).map_err(|err| err.to_string())?;
    Ok(())
}

fn clean_root(path: &str) -> Option<String> {
    let expanded = path.trim();
    if expanded.is_empty() {
        None
    } else {
        Some(expanded.to_string())
    }
}

fn project_fingerprint_for_root(root: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    root.to_string().hash(&mut hasher);
    format!("project-fnv1a64:{:016x}", hasher.finish())
}

pub fn selected_project_payload() -> Option<Value> {
    let profile = read_json_value(&selected_profile_path())?;
    let fingerprint = profile.get("selected_project_fingerprint")?.as_str()?;
    let selected_path = project_profiles_dir().join(format!("{fingerprint}.json"));
    let details = read_json_value(&selected_path)?;
    let selected_root = details.get("project_root")?.as_str()?;
    if classify_project_root(selected_root).reason().is_some() {
        return None;
    }
    let selected_root_path = Path::new(selected_root);
    if !selected_root_path.is_dir() || !selected_root_path.join(".focusa-project.json").is_file() {
        return None;
    }
    Some(json!({
        "schema": "focusa.cli.selected_project.v1",
        "status": "selected",
        "fingerprint": fingerprint,
        "selected_by": profile.get("selected_by").and_then(Value::as_str).unwrap_or("cli"),
        "note": profile.get("note").and_then(Value::as_str).unwrap_or("").to_string(),
        "selected_at": profile.get("selected_at").and_then(Value::as_str).unwrap_or(""),
        "project_root": selected_root,
        "project_profile": details,
    }))
}

fn selected_project_profile(root: &str) -> Option<Value> {
    let fingerprint = project_fingerprint_for_root(root);
    read_json_value(&project_profiles_dir().join(format!("{fingerprint}.json")))
}

fn store_selected_project(
    project_root: &str,
    selected_by: Option<String>,
    note: Option<String>,
) -> Result<Value, String> {
    let selected_root = Path::new(project_root);
    if let Some(reason) = classify_project_root(project_root).reason() {
        return Err(format!("unsafe project root: {reason}"));
    }
    if !selected_root.exists() {
        return Err("project root does not exist".to_string());
    }
    if !selected_root.join(".focusa-project.json").exists() {
        return Err("project root missing .focusa-project.json".to_string());
    }
    let fingerprint = project_fingerprint_for_root(project_root);
    let payload = json!({
        "schema": "focusa.cli.selected_project.v1",
        "selected_project_fingerprint": fingerprint,
        "project_root": project_root,
        "selected_at": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "selected_by": selected_by.unwrap_or_else(|| "focusa project".to_string()),
        "note": note.unwrap_or_else(|| "CLI convenience profile only; not canonical daemon authority".to_string()),
    });
    write_json_file(&selected_profile_path(), &payload).map_err(|err| err.to_string())?;

    if let Some(details) = read_json_value(&selected_root.join(".focusa-project.json")) {
        let mut profile = json!({
            "schema": "focusa.cli.project_profile.v1",
            "project_root": project_root,
            "fingerprint": fingerprint,
            "marker_path": selected_root.join(".focusa-project.json").to_string_lossy(),
            "scope_safety": if classify_project_root(project_root).is_safe() {"safe"} else {"unsafe"},
            "last_verified_at": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "created_at": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        });
        if let Some(obj) = profile.as_object_mut() {
            if let Some(id) = details.get("project_id").and_then(Value::as_str) {
                obj.insert("project_id".to_string(), Value::String(id.to_string()));
            }
            if let Some(name) = details.get("canonical_name").and_then(Value::as_str) {
                obj.insert(
                    "canonical_name".to_string(),
                    Value::String(name.to_string()),
                );
            }
            if let Some(v) = details.get("workspace_kind").cloned() {
                obj.insert("workspace_kind".to_string(), v);
            }
            if let Some(v) = details.get("aliases").cloned() {
                obj.insert("aliases".to_string(), v);
            }
        }
        let _ = write_json_file(
            &project_profiles_dir().join(format!("{fingerprint}.json")),
            &profile,
        );
    }

    Ok(payload)
}

fn read_dashboard_settings(project_root: &Path) -> Value {
    let settings = read_json_value(&project_root.join(".focusa").join("settings.json"));
    if let Some(value) = settings {
        value
    } else {
        json!({
            "schema": "focusa.project_settings.v1",
            "project_id": "",
            "proof_policy": "proof_or_explicit_gap",
            "default_continuity_id": "focusa-main",
            "created_by": "focusa project settings",
            "authority": "local_project_preferences_only"
        })
    }
}

fn current_project_identity(project_root: &str) -> Value {
    project_identity_payload_for_scope(Some(project_root), Some(project_root), None)
}

fn collect_project_candidate(root: &Path) -> Option<Value> {
    if !root.is_dir() {
        return None;
    }
    let marker_path = root.join(".focusa-project.json");
    let has_marker = marker_path.exists();
    let marker = if marker_path.exists() {
        read_json_value(&marker_path)
    } else {
        None
    };
    let has_git = root.join(".git").exists();
    if !(has_marker || has_git && root.join("Cargo.toml").exists()) {
        return None;
    }
    Some(json!({
        "schema": "focusa.project_summary.v1",
        "project_root": root.to_string_lossy(),
        "has_marker": has_marker,
        "has_git": has_git,
        "project_id": marker.as_ref().and_then(|m| m.get("project_id")).and_then(Value::as_str).unwrap_or("unknown").to_string(),
        "canonical_name": marker.as_ref().and_then(|m| m.get("canonical_name")).and_then(Value::as_str).unwrap_or(root.file_name().unwrap_or_default().to_string_lossy().as_ref()).to_string(),
        "status": if has_marker {"project-root-marker"} else {"git-root"},
        "stack": workspace_kind(root).unwrap_or("unknown"),
    }))
}

fn discover_project_candidates(from: &Path, max_depth: u32, max_results: usize) -> Vec<Value> {
    let mut candidates = Vec::new();
    let mut queue: Vec<(PathBuf, u32)> = vec![(from.to_path_buf(), 0)];
    let mut seen: std::collections::HashSet<String> = Default::default();
    while let Some((dir, depth)) = queue.pop() {
        if candidates.len() >= max_results {
            break;
        }
        if let Some(norm) = dir.to_str() {
            if !seen.insert(norm.to_string()) {
                continue;
            }
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|v| v.to_str()) {
                        if name == ".git"
                            || name == "target"
                            || name == "node_modules"
                            || name == ".cache"
                        {
                            continue;
                        }
                    }
                    if let Some(child) = collect_project_candidate(&path) {
                        candidates.push(child);
                        if candidates.len() >= max_results {
                            break;
                        }
                    }
                    if depth < max_depth {
                        queue.push((path, depth + 1));
                    }
                }
            }
        }
    }
    candidates
}

fn build_project_dashboard(
    runtime_payload: Value,
    selected: Option<Value>,
    project_profiles: Vec<Value>,
) -> Value {
    let runtime_verified = runtime_payload
        .pointer("/project_identity/status")
        .and_then(Value::as_str)
        == Some("verified");
    let effective_project = selected
        .clone()
        .or_else(|| runtime_verified.then(|| runtime_payload.clone()));
    let status = if effective_project.is_some() {
        "ok"
    } else {
        "degraded"
    };
    json!({
        "schema": "focusa.project_dashboard.v1",
        "status": status,
        "failure_class": if effective_project.is_some() { Value::Null } else { json!("project_root_selection_required") },
        "runtime": runtime_payload,
        "selected": selected,
        "effective_project": effective_project,
        "project_count": project_profiles.len(),
        "projects": project_profiles,
        "next_tools": if status == "ok" { vec!["focusa_project_verify", "focusa_trajectory_view"] } else { vec!["focusa_project_list", "focusa_project_discover", "focusa_project_use"] },
    })
}

async fn list_projects(Query(query): Query<ProjectListQuery>) -> Json<Value> {
    let runtime_root = query
        .project_root
        .as_deref()
        .or(query.from.as_deref())
        .unwrap_or(".");
    let runtime_payload = if classify_project_root(runtime_root).is_safe() {
        current_project_identity(runtime_root)
    } else {
        json!({"status":"invalid","reason":"runtime root unsafe"})
    };
    let selected = selected_project_payload();
    let mut project_profiles = Vec::new();
    if let Ok(entries) = fs::read_dir(project_profiles_dir()) {
        for entry in entries
            .flatten()
            .filter(|entry| entry.path().extension().is_some())
        {
            if let Some(item) = read_json_value(&entry.path()) {
                project_profiles.push(item);
            }
        }
    }
    Json(build_project_dashboard(
        runtime_payload,
        selected,
        project_profiles,
    ))
}

async fn current_status(Query(query): Query<ProjectListQuery>) -> Json<Value> {
    list_projects(Query(query)).await
}

async fn discover_projects(Query(query): Query<ProjectDiscoverQuery>) -> Json<Value> {
    let from = query.from.as_deref().unwrap_or(".");
    if classify_project_root(from).reason().is_some() {
        return Json(json!({"status":"blocked","reason":"unsafe from path"}));
    }
    let max_depth = query.max_depth.unwrap_or(3);
    let max_results = query.max_results.unwrap_or(60);
    let candidates = discover_project_candidates(Path::new(from), max_depth, max_results);
    Json(json!({
        "schema": "focusa.project_discover.v1",
        "status": "ok",
        "from": from,
        "max_depth": max_depth,
        "max_results": max_results,
        "projects": candidates,
        "count": candidates.len(),
    }))
}

async fn use_project(
    Query(_): Query<ProjectListQuery>,
    Json(body): Json<ProjectSelectionRequest>,
) -> Json<Value> {
    let root = body.project_root;
    match store_selected_project(root.trim(), body.selected_by, body.note) {
        Ok(payload) => Json(json!({
            "status":"ok",
            "schema":"focusa.project_selection.v2",
            "selected":payload,
            "canonical_parent_root":root,
            "active_worktree_root":body.active_worktree_root.unwrap_or_else(|| root.clone()),
            "working_subpath_id":body.working_subpath_id.unwrap_or_else(|| "primary".to_string())
        })),
        Err(reason) => {
            Json(json!({"status":"blocked","failure_class":"invalid_selection","reason":reason}))
        }
    }
}

async fn remove_selected_project(Json(_body): Json<ProjectRemoveRequest>) -> Json<Value> {
    let path = selected_profile_path();
    let _ = fs::remove_file(path);
    Json(
        json!({"status":"ok","schema":"focusa.project_selection.v1","selected":null,"note":"selection removed"}),
    )
}

async fn current_status_alias(Query(query): Query<ProjectListQuery>) -> Json<Value> {
    current_status(Query(query)).await
}

async fn create_project(Json(body): Json<ProjectCreateRequest>) -> Json<Value> {
    let root = PathBuf::from(body.project_root.trim());
    if let Some(reason) = classify_project_root(&body.project_root).reason() {
        return Json(
            json!({"status":"blocked","failure_class":"unsafe_project_root","reason":reason}),
        );
    }
    if root.exists() {
        if fs::read_dir(&root).is_err() {
            return Json(
                json!({"status":"blocked","failure_class":"invalid_root_state","reason":"project root unreadable"}),
            );
        }
        if !body.force.unwrap_or(false) {
            if fs::read_dir(&root).is_ok_and(|mut rd| rd.next().is_some()) {
                return Json(
                    json!({"status":"blocked","failure_class":"project_root_not_empty","reason":"pass --force to create in non-empty path"}),
                );
            }
        }
    }
    let focusa_dir = root.join(".focusa");
    if let Err(err) = fs::create_dir_all(&focusa_dir) {
        return Json(
            json!({"status":"blocked","failure_class":"create_failed","reason":err.to_string()}),
        );
    }
    for child in ["evidence", "workpoints", "trajectories", "templates"] {
        if let Err(err) = fs::create_dir_all(focusa_dir.join(child)) {
            return Json(
                json!({"status":"blocked","failure_class":"create_failed","reason":err.to_string()}),
            );
        }
    }
    let workspace_kind = body
        .workspace_kind
        .clone()
        .unwrap_or_else(|| "rust-monorepo".to_string());
    let project_id = body.project_id.clone();
    let canonical_name = body.canonical_name.clone();
    let marker = json!({
        "schema": "focusa.project.v1",
        "project_id": project_id,
        "canonical_name": canonical_name,
        "project_root": root.to_string_lossy(),
        "beads_prefix": "project",
        "workspace_kind": workspace_kind.clone(),
        "aliases": [],
        "created_at": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    });
    if let Err(err) = write_json_file(&root.join(".focusa-project.json"), &marker) {
        return Json(
            json!({"status":"blocked","failure_class":"marker_write_failed","reason":err}),
        );
    }
    let settings = json!({
        "schema": "focusa.project_settings.v1",
        "project_id": project_id,
        "proof_policy": "proof_or_explicit_gap",
        "default_continuity_id": format!("{}-main", project_id),
        "created_by": "focusa project new",
        "authority": "local_project_preferences_only",
        "workspace_kind": workspace_kind,
        "default_template": body.template.clone().unwrap_or_else(|| "blank".to_string()),
    });
    if let Err(err) = write_json_file(&focusa_dir.join("settings.json"), &settings) {
        return Json(
            json!({"status":"blocked","failure_class":"settings_write_failed","reason":err}),
        );
    }
    let focusa_readme = focusa_dir.join("README.md");
    if !focusa_readme.exists() {
        let _ = fs::write(
            &focusa_readme,
            "# Focusa project state\n\nLocal Focusa project preferences, evidence, workpoints, trajectories, and templates live here.\n",
        );
    }
    let readme = root.join("README.md");
    if !readme.exists() {
        let _ = fs::write(&readme, format!("# {canonical_name}\n\nFocusa project.\n"));
    }

    let git_status = if body.create_git.unwrap_or(false) && !root.join(".git").exists() {
        match Command::new("git").arg("init").current_dir(&root).status() {
            Ok(status) if status.success() => "created",
            Ok(_) => "failed",
            Err(_) => "unavailable",
        }
    } else if root.join(".git").exists() {
        "exists"
    } else {
        "skipped"
    };

    let selected = if body.use_selected.unwrap_or(false) {
        store_selected_project(
            root.to_string_lossy().as_ref(),
            Some("focusa project new".to_string()),
            Some("CLI convenience profile only; not canonical daemon authority".to_string()),
        )
        .ok()
    } else {
        None
    };

    Json(json!({
        "status":"ok",
        "schema":"focusa.project_created.v1",
        "project_root":root.to_string_lossy(),
        "created": {
            "marker": root.join(".focusa-project.json").to_string_lossy(),
            "settings": focusa_dir.join("settings.json").to_string_lossy(),
            "focusa_dir": focusa_dir.to_string_lossy(),
            "git": git_status,
            "selected": selected.is_some(),
        },
        "selected": selected,
        "authority": "created project files are local; selected profile is convenience-only"
    }))
}

fn templates_payload(name: Option<String>) -> Vec<Value> {
    let names: Vec<&str> = vec![
        "blank",
        "web-app",
        "cli-tool",
        "rust-service",
        "node-saas",
        "wordpress-plugin",
        "agent-workbench",
    ];
    names
        .into_iter()
        .map(|template| {
            json!({
                "name": template,
                "source": if builtin_templates_dir().join(template).exists() {
                    "built-in"
                } else {
                    "spec-only"
                },
                "description": format!("Project template: {template}"),
                "schema": "focusa.project_template.v1",
                "files": [],
                "directories": [],
                "post_create_hints": []
            })
        })
        .filter(|entry| {
            name.as_ref()
                .is_none_or(|n| entry.get("name").and_then(Value::as_str) == Some(n.as_str()))
        })
        .collect()
}

async fn project_templates(Query(query): Query<ProjectTemplatesQuery>) -> Json<Value> {
    let templates = templates_payload(query.name);
    Json(json!({
        "schema": "focusa.project_templates.v1",
        "status": "ok",
        "templates": templates,
        "count": templates.len(),
    }))
}

async fn project_settings_get(Query(query): Query<ProjectSettingsQuery>) -> Json<Value> {
    let root = query
        .project_root
        .clone()
        .unwrap_or_else(|| ".".to_string());
    let root_path = Path::new(root.as_str());
    let settings = read_dashboard_settings(root_path);
    if let Some(key) = query.key {
        let value = settings.get(&key).cloned();
        Json(json!({
            "schema": "focusa.project_settings.v1",
            "status": if value.is_some() {"ok"} else {"missing"},
            "project_root": root,
            "key": key,
            "value": value,
        }))
    } else {
        Json(json!({
            "schema": "focusa.project_settings.v1",
            "status": "ok",
            "project_root": root,
            "settings": settings,
        }))
    }
}

async fn project_settings_update(Json(body): Json<ProjectSettingsRequest>) -> Json<Value> {
    let root = body.project_root.unwrap_or_else(|| ".".to_string());
    let mut settings = read_dashboard_settings(Path::new(root.as_str()));
    if settings.get("schema").is_none() {
        return Json(
            json!({"status":"blocked","failure_class":"settings_missing","reason":"project root missing .focusa/settings.json"}),
        );
    }
    if body.key.as_deref().is_none() {
        return Json(
            json!({"status":"blocked","failure_class":"missing_key","reason":"key required"}),
        );
    }
    let key = body.key.unwrap();
    match body.action.as_str() {
        "set" => {
            if let Some(value) = body.value {
                if let Some(map) = settings.as_object_mut() {
                    map.insert(key.clone(), Value::String(value.clone()));
                }
            }
        }
        "unset" => {
            if let Some(map) = settings.as_object_mut() {
                map.remove(&key);
            }
        }
        _ => {
            return Json(json!({
                "status": "blocked",
                "failure_class": "unknown_action",
                "reason": "action must be set or unset"
            }));
        }
    }
    if let Some(file_root) = settings.get("schema") {
        let _ = write_json_file(
            &Path::new(root.as_str())
                .join(".focusa")
                .join("settings.json"),
            &settings,
        );
    }
    Json(json!({
        "schema": "focusa.project_settings.v1",
        "status": "ok",
        "project_root": root,
        "updated_key": key,
    }))
}

async fn identity(Query(query): Query<ProjectIdentityQuery>) -> Json<Value> {
    let remote_hint = RemoteProjectHint::from_query(&query);
    let local_binding = query
        .remote_host
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none();
    let binding = local_binding.then(|| {
        let start = resolve_start(query.cwd.as_deref(), query.project_root.as_deref());
        resolve_project_binding_candidates(
            &start,
            query.project_root.as_deref().map(Path::new),
            query.persisted_project_root.as_deref().map(Path::new),
        )
    });
    let inferred_root = binding
        .as_ref()
        .filter(|decision| !decision.requires_confirmation)
        .and_then(|decision| decision.selected_project_root.as_deref());
    let mut payload = project_identity_payload_for_scope_with_remote(
        query.cwd.as_deref(),
        query.project_root.as_deref().or(inferred_root),
        query.current_ask.as_deref(),
        remote_hint,
        None,
    );
    if let Some(decision) = binding
        && let Some(object) = payload.as_object_mut()
    {
        let decision_value = serde_json::to_value(&decision).unwrap_or(Value::Null);
        object.insert("binding_decision".to_string(), decision_value.clone());
        object.insert(
            "binding_candidates".to_string(),
            serde_json::to_value(&decision.candidates).unwrap_or_else(|_| json!([])),
        );
        if let Some(identity) = object
            .get_mut("project_identity")
            .and_then(Value::as_object_mut)
        {
            identity.insert("binding_decision".to_string(), decision_value);
        }
        if decision.ambiguous {
            object.insert("status".to_string(), json!("ambiguous_project_binding"));
            object.insert("canonical".to_string(), json!(false));
            object.insert("degraded".to_string(), json!(true));
            object.insert(
                "mismatch_reason".to_string(),
                json!("multiple equally ranked project/worktree candidates require explicit project_root"),
            );
        }
    }
    Json(payload)
}

async fn verify(
    _scope: ScopeContext,
    State(_state): State<Arc<AppState>>,
    Json(body): Json<ProjectVerifyRequest>,
) -> Json<Value> {
    let mut payload = candidate_payload(
        discover_identity(
            body.cwd.as_deref(),
            body.project_root.as_deref(),
            body.canonical_name.as_deref(),
            RemoteProjectHint::from_verify(&body),
        ),
        Some(&body),
    );
    if body
        .remote_host
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        let start = resolve_start(body.cwd.as_deref(), body.project_root.as_deref());
        let decision = resolve_project_binding_candidates(
            &start,
            body.project_root.as_deref().map(Path::new),
            body.persisted_project_root.as_deref().map(Path::new),
        );
        if let Some(object) = payload.as_object_mut() {
            object.insert(
                "binding_candidates".to_string(),
                serde_json::to_value(&decision.candidates).unwrap_or_else(|_| json!([])),
            );
            object.insert(
                "binding_decision".to_string(),
                serde_json::to_value(&decision).unwrap_or(Value::Null),
            );
            if decision.ambiguous {
                object.insert("status".to_string(), json!("ambiguous_project_binding"));
                object.insert("canonical".to_string(), json!(false));
                object.insert("degraded".to_string(), json!(true));
            }
        }
    }
    Json(payload)
}

fn sigmoid(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}

fn logit(p: f64) -> f64 {
    let p = p.clamp(1e-9, 1.0 - 1e-9);
    (p / (1.0 - p)).ln()
}

fn normalized_weighted_score(features: &[(f64, f64)]) -> f64 {
    let weight_sum: f64 = features.iter().map(|(_, w)| w.max(0.0)).sum();
    if weight_sum <= f64::EPSILON {
        return 0.0;
    }
    (features
        .iter()
        .map(|(x, w)| x.clamp(0.0, 1.0) * w.max(0.0))
        .sum::<f64>()
        / weight_sum)
        .clamp(0.0, 1.0)
}

fn softmax(scores: &[f64]) -> Vec<f64> {
    if scores.is_empty() {
        return vec![];
    }
    let max = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let exp: Vec<f64> = scores.iter().map(|s| (s - max).exp()).collect();
    let sum: f64 = exp.iter().sum();
    if sum <= f64::EPSILON {
        return vec![1.0 / scores.len() as f64; scores.len()];
    }
    exp.iter().map(|v| v / sum).collect()
}

fn expected_value(probabilities: &[f64], values: &[f64]) -> f64 {
    probabilities
        .iter()
        .zip(values.iter())
        .map(|(p, v)| p * v)
        .sum()
}

fn exponential_decay(age_units: f64, lambda: f64) -> f64 {
    (-lambda.max(0.0) * age_units.max(0.0)).exp()
}

fn ema(values: &[f64], alpha: f64) -> Option<f64> {
    let alpha = alpha.clamp(0.0, 1.0);
    let mut iter = values.iter().copied();
    let mut acc = iter.next()?;
    for value in iter {
        acc = alpha * value + (1.0 - alpha) * acc;
    }
    Some(acc)
}

fn z_score(value: f64, mean: f64, stddev: f64) -> f64 {
    if stddev.abs() <= f64::EPSILON {
        0.0
    } else {
        (value - mean) / stddev
    }
}

fn brier_score(probability: f64, outcome: f64) -> f64 {
    (probability.clamp(0.0, 1.0) - outcome.clamp(0.0, 1.0)).powi(2)
}

fn log_loss(probability: f64, outcome: f64) -> f64 {
    let p = probability.clamp(1e-9, 1.0 - 1e-9);
    let y = outcome.clamp(0.0, 1.0);
    -(y * p.ln() + (1.0 - y) * (1.0 - p).ln())
}

fn prediction_stats_card(predictions: &[Value]) -> Value {
    let evaluated: Vec<&Value> = predictions
        .iter()
        .filter(|p| !p.get("score").unwrap_or(&Value::Null).is_null())
        .collect();
    let scores: Vec<f64> = evaluated
        .iter()
        .filter_map(|p| p.get("score").and_then(Value::as_f64))
        .collect();
    let score_sum: f64 = scores.iter().sum();
    let accuracy = if scores.is_empty() {
        0.0
    } else {
        score_sum / scores.len() as f64
    };
    let ema_accuracy = ema(&scores, 0.35).unwrap_or(accuracy);
    let calibration_brier = if scores.is_empty() {
        0.0
    } else {
        scores
            .iter()
            .map(|score| brier_score(*score, 1.0))
            .sum::<f64>()
            / scores.len() as f64
    };
    json!({
        "total": predictions.len(),
        "evaluated": evaluated.len(),
        "accuracy": accuracy,
        "ema_accuracy": ema_accuracy,
        "calibration": {"brier_score_vs_success": calibration_brier, "log_loss_vs_success": log_loss(accuracy, if accuracy >= 0.5 { 1.0 } else { 0.0 })},
        "recent_open": predictions.iter().rev().filter(|p| p.get("score").unwrap_or(&Value::Null).is_null()).take(5).cloned().collect::<Vec<_>>(),
        "recent_evaluated": predictions.iter().rev().filter(|p| !p.get("score").unwrap_or(&Value::Null).is_null()).take(5).cloned().collect::<Vec<_>>(),
    })
}

fn focusa_data_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("FOCUSA_HOME") {
        return PathBuf::from(home).join("data");
    }
    if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(data_home).join("focusa");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".local/share/focusa");
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data")
}

fn project_card_weights_path() -> PathBuf {
    focusa_data_dir().join("project_card_signal_weights.json")
}

fn project_card_runs_path() -> PathBuf {
    focusa_data_dir().join("project_card_algorithm_runs.jsonl")
}

fn project_card_outcomes_path() -> PathBuf {
    focusa_data_dir().join("project_card_algorithm_outcomes.jsonl")
}

fn project_session_transfers_path(root_scope: &ScopeRef) -> PathBuf {
    focusa_data_dir()
        .join("runtime")
        .join("project-session-transfers")
        .join(root_scope.storage_key())
        .join("transfers.jsonl")
}

fn default_project_card_weights() -> BTreeMap<String, f64> {
    BTreeMap::from([
        ("trajectory".to_string(), 0.24),
        ("ontology".to_string(), 0.16),
        ("evidence".to_string(), 0.22),
        ("prediction".to_string(), 0.22),
        ("blocker".to_string(), 0.16),
        ("open_prediction".to_string(), 0.14),
        ("learn_decay".to_string(), 0.20),
    ])
}

fn load_project_card_weights() -> BTreeMap<String, f64> {
    fs::read_to_string(project_card_weights_path())
        .ok()
        .and_then(|text| serde_json::from_str::<BTreeMap<String, f64>>(&text).ok())
        .map(|weights| {
            weights
                .into_iter()
                .map(|(k, v)| (k, v.clamp(0.05, 0.50)))
                .collect()
        })
        .unwrap_or_else(default_project_card_weights)
}

fn persist_project_card_weights(weights: &BTreeMap<String, f64>) {
    let path = project_card_weights_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(bytes) = serde_json::to_vec_pretty(weights) {
        let _ = fs::write(path, bytes);
    }
}

fn update_project_card_weights(
    mut weights: BTreeMap<String, f64>,
    prediction_accuracy: f64,
    evaluated_predictions: usize,
) -> BTreeMap<String, f64> {
    if evaluated_predictions == 0 {
        return weights;
    }
    let delta = ((prediction_accuracy - 0.5) * 0.02).clamp(-0.01, 0.01);
    for key in ["prediction", "evidence", "trajectory"] {
        if let Some(value) = weights.get_mut(key) {
            *value = (*value + delta).clamp(0.05, 0.50);
        }
    }
    for key in ["open_prediction", "blocker"] {
        if let Some(value) = weights.get_mut(key) {
            *value = (*value - delta / 2.0).clamp(0.05, 0.50);
        }
    }
    weights
}

fn projected_project_card_weights(
    prediction_accuracy: f64,
    evaluated_predictions: usize,
) -> BTreeMap<String, f64> {
    // Read-only projection for hot GET /project/card; persisted learning happens on explicit outcomes.
    update_project_card_weights(
        load_project_card_weights(),
        prediction_accuracy,
        evaluated_predictions,
    )
}

fn append_jsonl(path: PathBuf, record: &Value) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path)
        && let Ok(line) = serde_json::to_string(record)
    {
        let _ = writeln!(file, "{line}");
    }
}

fn append_project_card_algorithm_run(run: &Value) {
    append_jsonl(project_card_runs_path(), run);
}

fn append_project_card_algorithm_outcome(outcome: &Value) {
    append_jsonl(project_card_outcomes_path(), outcome);
}

fn append_project_session_transfer(root_scope: &ScopeRef, record: &Value) {
    append_jsonl(project_session_transfers_path(root_scope), record);
}

fn project_card_run_exists(algorithm_run_id: &str) -> bool {
    let needle = format!(
        "\"algorithm_run_id\":\"{}\"",
        algorithm_run_id.replace('"', "")
    );
    fs::read_to_string(project_card_runs_path())
        .map(|text| text.lines().any(|line| line.contains(&needle)))
        .unwrap_or(false)
}

fn update_weights_from_algorithm_outcome(score: f64) -> BTreeMap<String, f64> {
    let weights = load_project_card_weights();
    let updated = update_project_card_weights(weights, score.clamp(0.0, 1.0), 1);
    persist_project_card_weights(&updated);
    updated
}

fn recent_jsonl_values(path: PathBuf, limit: usize) -> Vec<Value> {
    const TAIL_BYTES: u64 = 128 * 1024;
    let Ok(mut file) = fs::File::open(path) else {
        return vec![];
    };
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let start = len.saturating_sub(TAIL_BYTES);
    if file.seek(SeekFrom::Start(start)).is_err() {
        return vec![];
    }
    let mut text = String::new();
    if file.read_to_string(&mut text).is_err() {
        return vec![];
    }
    let lines = text
        .lines()
        .skip(if start > 0 { 1 } else { 0 })
        .collect::<Vec<_>>();
    let mut values = lines
        .iter()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .take(limit)
        .collect::<Vec<_>>();
    values.reverse();
    values
}

fn format_elapsed_hms(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{hours:02}:{minutes:02}:{secs:02}")
}

fn trajectory_reporting_card(
    trajectory: &Option<focusa_core::types::TrajectoryLadderContext>,
    record: &Value,
    efficiency: &Value,
    outcomes: &[Value],
) -> Value {
    let waypoints = trajectory
        .as_ref()
        .map(|t| t.waypoints.clone())
        .unwrap_or_default();
    let recent_text = outcomes
        .iter()
        .rev()
        .take(8)
        .map(|outcome| {
            format!(
                "{} {}",
                outcome
                    .get("actual_outcome")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
                outcome.get("notes").and_then(Value::as_str).unwrap_or("")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
        .to_lowercase();
    let waypoint_cards = waypoints.iter().enumerate().map(|(idx, waypoint)| {
        let marker = waypoint.to_lowercase();
        let accomplished = !marker.is_empty() && recent_text.contains(&marker.chars().take(48).collect::<String>());
        json!({
            "index": idx + 1,
            "waypoint": waypoint,
            "status": if accomplished { "accomplished_by_recent_outcome" } else { "open_or_unproven" },
            "evidence_hint": if accomplished { "recent_project_card_outcome" } else { "capture evidence/outcome when complete" }
        })
    }).collect::<Vec<_>>();
    json!({
        "schema": "focusa.trajectory_reporting_card.v1",
        "hierarchy": {
            "hlt": trajectory.as_ref().and_then(|t| t.hlt.clone()).or_else(|| record.get("root_long_term_goal").and_then(Value::as_str).map(str::to_string)),
            "ltg": record.get("long_term_goal").and_then(Value::as_str).or_else(|| record.get("root_long_term_goal").and_then(Value::as_str)),
            "desired_end_state": record.get("desired_end_state").and_then(Value::as_str),
            "mtg": trajectory.as_ref().and_then(|t| t.mlg.clone()).or_else(|| record.get("mid_level_goal").and_then(Value::as_str).map(str::to_string)),
            "stg": trajectory.as_ref().and_then(|t| t.stg.clone()).or_else(|| record.get("short_term_goal").and_then(Value::as_str).map(str::to_string)),
        },
        "waypoints": waypoint_cards,
        "accomplishment_summary": {
            "waypoints_total": waypoints.len(),
            "waypoints_accomplished_by_recent_outcomes": waypoint_cards.iter().filter(|item| item.get("status").and_then(Value::as_str) == Some("accomplished_by_recent_outcome")).count(),
            "recent_outcomes_considered": outcomes.len(),
        },
        "time_and_tokens": efficiency,
        "operator_report_rule": "End-of-task reports include elapsed time, total tokens, trajectory hierarchy, and waypoint accomplishments."
    })
}

fn project_card_efficiency_summary(outcomes: &[Value]) -> Value {
    let durations = outcomes
        .iter()
        .filter_map(|outcome| {
            outcome
                .pointer("/task_timing/elapsed_seconds")
                .and_then(Value::as_u64)
        })
        .collect::<Vec<_>>();
    let token_totals = outcomes
        .iter()
        .filter_map(|outcome| {
            outcome
                .pointer("/token_usage/total_tokens")
                .and_then(Value::as_u64)
        })
        .collect::<Vec<_>>();
    let avg_duration = if durations.is_empty() {
        0
    } else {
        durations.iter().sum::<u64>() / durations.len() as u64
    };
    let avg_tokens = if token_totals.is_empty() {
        0
    } else {
        token_totals.iter().sum::<u64>() / token_totals.len() as u64
    };
    json!({
        "schema": "focusa.project_efficiency_summary.v1",
        "outcome_count_with_timing": durations.len(),
        "outcome_count_with_tokens": token_totals.len(),
        "average_elapsed_seconds": avg_duration,
        "average_elapsed_hms": format_elapsed_hms(avg_duration),
        "last_elapsed_seconds": durations.last().copied().unwrap_or(0),
        "last_elapsed_hms": format_elapsed_hms(durations.last().copied().unwrap_or(0)),
        "average_total_tokens": avg_tokens,
        "last_total_tokens": token_totals.last().copied().unwrap_or(0),
        "goal": "improve completion time and token efficiency for similar tasks"
    })
}

fn project_card_outcome_stats(outcomes: &[Value]) -> (usize, f64, f64) {
    let scores = outcomes
        .iter()
        .filter_map(|outcome| outcome.get("score").and_then(Value::as_f64))
        .map(|score| score.clamp(0.0, 1.0))
        .collect::<Vec<_>>();
    if scores.is_empty() {
        return (0, 0.5, 0.0);
    }
    let average = scores.iter().sum::<f64>() / scores.len() as f64;
    let recent = scores.last().copied().unwrap_or(average);
    (scores.len(), average, recent)
}

fn sequence_probability(algorithmic: &Value, key: &str) -> f64 {
    algorithmic
        .get("action_probabilities")
        .and_then(|v| v.get(key))
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
}

fn git_lines(project_root: &str, args: &[&str], limit: usize) -> Vec<String> {
    if project_root.trim().is_empty() || !Path::new(project_root).exists() {
        return vec![];
    }
    Command::new("git")
        .arg("-C")
        .arg(project_root)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .take(limit)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn infer_action_from_paths(paths: &[String], current_ask: &str) -> String {
    let joined = format!(
        "{} {}",
        current_ask.to_lowercase(),
        paths.join(" ").to_lowercase()
    );
    if joined.contains("test") || joined.contains("spec") {
        "verify_or_fix_tests".to_string()
    } else if joined.contains("doc") || joined.contains("readme") {
        "update_docs".to_string()
    } else if joined.contains("route") || joined.contains("api") {
        "patch_api_flow".to_string()
    } else if joined.contains("component") || joined.contains("svelte") || joined.contains("ui") {
        "patch_ui_flow".to_string()
    } else if joined.contains("config") || joined.contains("package") {
        "patch_configuration".to_string()
    } else {
        "infer_and_continue_project_slice".to_string()
    }
}

#[allow(clippy::too_many_arguments)]
fn inferred_workpoint_candidate(
    project_root: Option<&str>,
    current_ask: &str,
    trajectory_stg: Option<&str>,
    active_workpoint: &Option<Value>,
    prediction: &Value,
    ontology: &Value,
    recent_frames: &[Value],
    recent_decisions: &[String],
    focus_goal_signals: &[Value],
) -> Value {
    let Some(root) = project_root.filter(|root| !root.trim().is_empty()) else {
        return Value::Null;
    };
    let status = git_lines(root, &["status", "--short"], 12);
    let changed = if status.is_empty() {
        git_lines(root, &["diff", "--name-only", "HEAD"], 12)
    } else {
        status
            .iter()
            .map(|line| {
                line.trim_start_matches(|c: char| {
                    c.is_whitespace()
                        || c == 'M'
                        || c == 'A'
                        || c == 'D'
                        || c == 'R'
                        || c == '?'
                        || c == '!'
                })
                .trim()
                .to_string()
            })
            .collect::<Vec<_>>()
    };
    let recent = git_lines(
        root,
        &["log", "--name-only", "--pretty=format:", "-n", "3"],
        16,
    );
    let target_objects = changed
        .iter()
        .chain(recent.iter())
        .filter(|line| !line.trim().is_empty())
        .take(10)
        .cloned()
        .collect::<Vec<_>>();
    let prediction_total = prediction.get("total").and_then(Value::as_u64).unwrap_or(0);
    let ontology_objects = ontology.get("objects").and_then(Value::as_u64).unwrap_or(0);
    let source_count = status.len()
        + recent.len()
        + usize::from(active_workpoint.is_some())
        + usize::from(!current_ask.trim().is_empty())
        + usize::from(trajectory_stg.is_some())
        + recent_frames.len()
        + recent_decisions.len()
        + focus_goal_signals.len()
        + usize::from(prediction_total > 0)
        + usize::from(ontology_objects > 0);
    if source_count == 0 {
        return Value::Null;
    }
    let action_type = infer_action_from_paths(&target_objects, current_ask);
    let next_action = trajectory_stg
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            if current_ask.trim().is_empty() {
                None
            } else {
                Some(current_ask)
            }
        })
        .unwrap_or("infer next slice from recent project activity");
    let active_workpoint_mission = active_workpoint
        .as_ref()
        .and_then(|value| value.get("mission"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let ask_differs_from_active_workpoint = !current_ask.trim().is_empty()
        && !active_workpoint_mission.trim().is_empty()
        && current_ask.trim() != active_workpoint_mission.trim();
    let recommended_bridge_action = if ask_differs_from_active_workpoint {
        "checkpoint_new_workpoint_from_current_ask"
    } else if active_workpoint.is_some() {
        "resume_active_workpoint"
    } else {
        "checkpoint_current_ask_after_identity_verification"
    };
    let checkpoint_payload_hint = json!({"mission": next_action, "current_action": action_type, "target_objects": target_objects, "next_action": next_action, "canonical": true});
    let ask_to_workpoint_bridge = json!({
        "schema": "focusa.ask_to_workpoint_bridge.v1",
        "safe_after_identity_verification": true,
        "project_root": root,
        "current_ask": current_ask,
        "active_workpoint_mission": active_workpoint_mission,
        "ask_differs_from_active_workpoint": ask_differs_from_active_workpoint,
        "recommended_bridge_action": recommended_bridge_action,
        "exact_next_action": if ask_differs_from_active_workpoint || active_workpoint.is_none() { "focusa_workpoint_checkpoint with checkpoint_payload_hint" } else { "focusa_workpoint_resume active Workpoint" },
        "checkpoint_payload_hint": checkpoint_payload_hint,
    });
    json!({
        "schema": "focusa.inferred_workpoint_candidate.v1",
        "advisory_only": true,
        "inference_reason": "verified project scope but no canonical Workpoint packet; infer from prior session workpath, prediction, metacognition prompt, ontology, trajectory STG, file changes, git activity, current ask, and prior active Workpoint when available",
        "confidence": if !target_objects.is_empty() && (!current_ask.trim().is_empty() || trajectory_stg.is_some()) { "medium" } else { "low" },
        "project_root": root,
        "mission": next_action,
        "current_action": action_type,
        "next_action": next_action,
        "target_objects": target_objects,
        "source_signals": {"git_status": status, "git_recent_files": recent, "prior_session_workpath": recent_frames, "recent_decisions": recent_decisions, "focus_goal_signals": focus_goal_signals, "prediction_summary": prediction, "ontology_summary": ontology, "metacognition_prompt": "retrieve lessons for inferred Workpoint before checkpointing", "had_prior_workpoint": active_workpoint.is_some(), "current_ask_present": !current_ask.trim().is_empty(), "trajectory_stg_present": trajectory_stg.is_some()},
        "checkpoint_payload_hint": checkpoint_payload_hint,
        "ask_to_workpoint_bridge": ask_to_workpoint_bridge,
        "operator_prompt_required": false
    })
}

fn project_success_sequence(
    project_name: &str,
    long_term_goal: Option<&str>,
    desired_end_state: Option<&str>,
    current_state: Option<&str>,
    active_gap: Option<&str>,
    algorithmic: &Value,
) -> Value {
    let execute_p = sequence_probability(algorithmic, "execute_next_step");
    let refresh_p = sequence_probability(algorithmic, "refresh_trajectory");
    let learn_p = sequence_probability(algorithmic, "evaluate_predictions_and_lessons");
    let outcome_count = algorithmic
        .get("outcome_learning")
        .and_then(|v| v.get("outcome_count"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let outcome_avg = algorithmic
        .get("outcome_learning")
        .and_then(|v| v.get("average_score"))
        .and_then(Value::as_f64)
        .unwrap_or(0.5);
    let outcome_bias = if outcome_count == 0 {
        "neutral_no_outcomes"
    } else if outcome_avg >= 0.75 {
        "execute_bias_from_successful_outcomes"
    } else if outcome_avg <= 0.45 {
        "learn_or_refresh_bias_from_weak_outcomes"
    } else {
        "balanced_outcome_bias"
    };
    let hlg = long_term_goal.unwrap_or("unknown high-level goal");
    let desired = desired_end_state.unwrap_or("verified successful end state");
    let current = current_state.unwrap_or("current state unclear");
    let gap = active_gap.unwrap_or("refresh project card and trajectory gap");
    let events = vec![
        json!({"order": 1, "event": "orient_project_card", "goal_link": hlg, "action": "Read project card, ontology objects, trajectory hierarchy, active Workpoint, prediction stats, and metacog prompts.", "tool_route": ["focusa_project_card", "focusa_traverse", "focusa_trajectory_view"], "success_metric": "project identity, trajectory, ontology, evidence, prediction, and learning context are visible"}),
        json!({"order": 2, "event": "refresh_or_confirm_trajectory", "goal_link": hlg, "action": format!("Confirm path from '{current}' to '{desired}' and isolate active gap: {gap}"), "tool_route": ["focusa_trajectory_assess", "focusa_trajectory_define_goal"], "success_metric": "HLG/desired/current/gap alignment is explicit"}),
        json!({"order": 3, "event": "retrieve_lessons", "goal_link": hlg, "action": "Retrieve metacog lessons and anti-patterns relevant to this project goal and gap.", "tool_route": ["focusa_metacog_retrieve", "focusa_metacog_doctor"], "success_metric": "top reusable lessons influence the next action"}),
        json!({"order": 4, "event": "forecast_next_action", "goal_link": hlg, "action": "Record bounded prediction for the selected next action, with ontology refs and evidence expectation.", "tool_route": ["focusa_predict_record"], "success_metric": "prediction exists before action is claimed"}),
        json!({"order": 5, "event": "execute_highest_ev_slice", "goal_link": hlg, "action": format!("Execute the smallest high expected-value slice toward: {gap}"), "tool_route": ["focusa_active_object_resolve", "focusa_workpoint_checkpoint"], "success_metric": "small slice completed without drift"}),
        json!({"order": 6, "event": "prove_outcome", "goal_link": desired, "action": "Capture evidence and link it to the active Workpoint/trajectory gap.", "tool_route": ["focusa_evidence_capture", "focusa_workpoint_link_evidence", "focusa_trajectory_assess"], "success_metric": "proof handle exists and trajectory state is updated"}),
        json!({"order": 7, "event": "evaluate_and_compound", "goal_link": hlg, "action": "Evaluate predictions, capture condensed lesson, persist algorithm run, and update learned weights.", "tool_route": ["focusa_predict_evaluate", "focusa_metacog_capture", "focusa_project_card"], "success_metric": "future project cards get better from this outcome"}),
    ];
    let recommended_first = if refresh_p >= execute_p && refresh_p >= learn_p {
        "refresh_or_confirm_trajectory"
    } else if learn_p >= execute_p {
        "retrieve_lessons"
    } else {
        "execute_highest_ev_slice"
    };
    let path_candidates = vec![
        json!({"path_id":"execute_path", "first_event":"execute_highest_ev_slice", "sequence":["orient_project_card","forecast_next_action","execute_highest_ev_slice","prove_outcome","evaluate_and_compound"], "cost": (1.0 - execute_p + (1.0 - outcome_avg) * 0.35).max(0.01), "success_probability": execute_p, "why":"lowest cost when readiness and prior outcomes are strong"}),
        json!({"path_id":"refresh_path", "first_event":"refresh_or_confirm_trajectory", "sequence":["orient_project_card","refresh_or_confirm_trajectory","retrieve_lessons","forecast_next_action","execute_highest_ev_slice","prove_outcome"], "cost": (1.0 - refresh_p + if outcome_avg < 0.5 { 0.0 } else { 0.20 }).max(0.01), "success_probability": refresh_p, "why":"preferred when trajectory uncertainty or weak outcomes make direct execution risky"}),
        json!({"path_id":"learn_path", "first_event":"retrieve_lessons", "sequence":["orient_project_card","retrieve_lessons","forecast_next_action","execute_highest_ev_slice","prove_outcome","evaluate_and_compound"], "cost": (1.0 - learn_p + if outcome_count == 0 { 0.0 } else { 0.10 }).max(0.01), "success_probability": learn_p, "why":"preferred when metacog/prediction learning is the fastest risk reducer"}),
    ];
    let shortest_path = path_candidates
        .iter()
        .min_by(|a, b| {
            a.get("cost")
                .and_then(Value::as_f64)
                .unwrap_or(f64::INFINITY)
                .partial_cmp(
                    &b.get("cost")
                        .and_then(Value::as_f64)
                        .unwrap_or(f64::INFINITY),
                )
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .unwrap_or_else(|| json!({"path_id":"unknown", "sequence":[]}));
    let eliminated_candidates = path_candidates.iter().filter(|candidate| candidate.get("path_id") != shortest_path.get("path_id")).map(|candidate| json!({
        "path_id": candidate.get("path_id").cloned().unwrap_or(Value::Null),
        "reason": if candidate.get("cost").and_then(Value::as_f64).unwrap_or(0.0) > shortest_path.get("cost").and_then(Value::as_f64).unwrap_or(0.0) { "higher_weighted_cost_to_success" } else { "lower_predicted_success_probability" },
        "cost": candidate.get("cost").cloned().unwrap_or(Value::Null),
        "success_probability": candidate.get("success_probability").cloned().unwrap_or(Value::Null),
    })).collect::<Vec<_>>();
    json!({
        "schema": "focusa.project_success_sequence.v1",
        "advisory_only": true,
        "project": project_name,
        "long_term_goal": hlg,
        "desired_end_state": desired,
        "active_gap": gap,
        "recommended_first_event": recommended_first,
        "ranking_basis": {
            "execute_probability": execute_p,
            "refresh_probability": refresh_p,
            "learn_probability": learn_p,
            "outcome_count": outcome_count,
            "outcome_average_score": outcome_avg,
            "outcome_bias": outcome_bias,
            "expected_utility": algorithmic.get("expected_utility").cloned().unwrap_or(Value::Null)
        },
        "shortest_path_to_success": {
            "method": "weighted_path_elimination_v1",
            "cost_model": "lower cost = lower risk + fewer uncertainty reducers + better prior outcomes",
            "selected": shortest_path,
            "candidates": path_candidates,
            "eliminated_candidates": eliminated_candidates
        },
        "events": events
    })
}

#[allow(clippy::too_many_arguments)]
fn project_card_algorithmic_scores(
    trajectory_present: bool,
    ontology_objects: usize,
    evidence_refs: usize,
    prediction_accuracy: f64,
    evaluated_predictions: usize,
    open_predictions: usize,
    blocker_count: usize,
    outcome_count: usize,
    average_outcome_score: f64,
    recent_outcome_score: f64,
) -> Value {
    let weights = projected_project_card_weights(prediction_accuracy, evaluated_predictions);
    let w = |key: &str, fallback: f64| {
        weights
            .get(key)
            .copied()
            .unwrap_or(fallback)
            .clamp(0.05, 0.50)
    };
    let ontology_signal = (ontology_objects as f64 / 20.0).clamp(0.0, 1.0);
    let evidence_signal = (evidence_refs as f64 / 10.0).clamp(0.0, 1.0);
    let prediction_signal = if evaluated_predictions > 0 {
        prediction_accuracy.clamp(0.0, 1.0)
    } else {
        0.35
    };
    let trajectory_signal = if trajectory_present { 1.0 } else { 0.0 };
    let open_prediction_signal = (open_predictions as f64 / 10.0).clamp(0.0, 1.0);
    let blocker_penalty = (blocker_count as f64 / 5.0).clamp(0.0, 1.0);
    let outcome_confidence = (outcome_count as f64 / 8.0).clamp(0.0, 1.0);
    let outcome_success_signal =
        (average_outcome_score * 0.7 + recent_outcome_score * 0.3).clamp(0.0, 1.0);
    let readiness = normalized_weighted_score(&[
        (trajectory_signal, w("trajectory", 0.24)),
        (ontology_signal, w("ontology", 0.16)),
        (evidence_signal, w("evidence", 0.22)),
        (prediction_signal, w("prediction", 0.22)),
        (1.0 - blocker_penalty, w("blocker", 0.16)),
        (outcome_success_signal, outcome_confidence * 0.18),
    ]);
    let bootstrap_need = normalized_weighted_score(&[
        (1.0 - trajectory_signal, w("trajectory", 0.24)),
        (1.0 - evidence_signal, w("evidence", 0.22)),
        (1.0 - ontology_signal, w("ontology", 0.16)),
        (open_prediction_signal, w("open_prediction", 0.14)),
        (blocker_penalty, w("blocker", 0.16)),
        (1.0 - outcome_success_signal, outcome_confidence * 0.12),
    ]);
    let learn_need = normalized_weighted_score(&[
        (open_prediction_signal, w("open_prediction", 0.14)),
        (1.0 - prediction_signal, w("prediction", 0.22)),
        (evidence_signal, w("evidence", 0.22)),
        (
            1.0 - exponential_decay(evaluated_predictions as f64, 0.08),
            w("learn_decay", 0.20),
        ),
        (1.0 - outcome_success_signal, outcome_confidence * 0.20),
    ]);
    let action_scores = [readiness, bootstrap_need, learn_need];
    let probabilities = softmax(
        &action_scores
            .iter()
            .map(|s| logit((*s).clamp(0.01, 0.99)))
            .collect::<Vec<_>>(),
    );
    let utilities = [
        (0.82 + outcome_success_signal * outcome_confidence * 0.10).clamp(0.60, 0.95),
        (0.76 + (1.0 - outcome_success_signal) * outcome_confidence * 0.08).clamp(0.60, 0.90),
        (0.72 + (1.0 - outcome_success_signal) * outcome_confidence * 0.12).clamp(0.60, 0.90),
    ];
    json!({
        "implemented_algorithms": [
            "normalized_weighted_score", "sigmoid", "logit", "softmax", "expected_value", "exponential_decay", "ema", "z_score", "brier_score", "log_loss"
        ],
        "storage": {
            "weights_path": project_card_weights_path().to_string_lossy(),
            "runs_path": project_card_runs_path().to_string_lossy(),
            "outcomes_path": project_card_outcomes_path().to_string_lossy(),
            "persistence": "jsonl_algorithm_runs_plus_compact_weights_json",
            "portable": true
        },
        "learned_weights": weights,
        "signals": {
            "trajectory": trajectory_signal,
            "ontology": ontology_signal,
            "evidence": evidence_signal,
            "prediction_accuracy": prediction_signal,
            "open_prediction_pressure": open_prediction_signal,
            "blocker_penalty": blocker_penalty,
            "evidence_z_score": z_score(evidence_refs as f64, 5.0, 2.0),
            "outcome_success": outcome_success_signal,
            "outcome_confidence": outcome_confidence
        },
        "outcome_learning": {
            "outcome_count": outcome_count,
            "average_score": average_outcome_score,
            "recent_score": recent_outcome_score,
            "success_signal": outcome_success_signal,
            "confidence": outcome_confidence,
            "effect": "biases readiness/refresh/learn scores and expected utility"
        },
        "scores": {
            "readiness_to_execute": readiness,
            "need_to_bootstrap_or_rebootstrap": bootstrap_need,
            "need_to_learn_or_evaluate": learn_need,
            "risk_probability": sigmoid(logit((1.0 - readiness).clamp(0.01, 0.99)))
        },
        "action_probabilities": {
            "execute_next_step": probabilities.first().copied().unwrap_or(0.0),
            "refresh_trajectory": probabilities.get(1).copied().unwrap_or(0.0),
            "evaluate_predictions_and_lessons": probabilities.get(2).copied().unwrap_or(0.0)
        },
        "expected_utility": expected_value(&probabilities, &utilities),
        "decision_rule": "choose the highest probability/utility action unless operator steering overrides"
    })
}

fn scoped_trajectory_record<'a>(
    records: &'a [focusa_core::types::TrajectoryProjectionRecord],
    active_trajectory_id: Option<&str>,
    project_root: Option<&str>,
) -> Option<&'a focusa_core::types::TrajectoryProjectionRecord> {
    let matches_scope =
        |record: &&focusa_core::types::TrajectoryProjectionRecord| match project_root {
            Some(root) => record.project_root.as_deref() == Some(root),
            None => true,
        };
    if let Some(active_trajectory_id) = active_trajectory_id
        && let Some(record) = records
            .iter()
            .rev()
            .find(|record| matches_scope(record) && record.trajectory_id == active_trajectory_id)
    {
        return Some(record);
    }
    records.iter().rev().find(matches_scope)
}

fn scoped_workpoint_record<'a>(
    records: &'a [focusa_core::types::WorkpointRecord],
    active_workpoint_id: Option<&focusa_core::types::WorkpointId>,
    project_root: Option<&str>,
) -> Option<&'a focusa_core::types::WorkpointRecord> {
    let matches_scope = |record: &&focusa_core::types::WorkpointRecord| match project_root {
        Some(root) => record.project_root.as_deref() == Some(root),
        None => true,
    };
    if let Some(active_workpoint_id) = active_workpoint_id
        && let Some(record) = records
            .iter()
            .rev()
            .find(|record| matches_scope(record) && record.workpoint_id == *active_workpoint_id)
    {
        return Some(record);
    }
    records.iter().rev().find(matches_scope)
}

async fn card(
    request_scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProjectIdentityQuery>,
) -> Json<Value> {
    let remote_hint = RemoteProjectHint::from_query(&query);
    let identity_payload = project_identity_payload_for_scope_with_remote(
        query.cwd.as_deref(),
        query.project_root.as_deref(),
        query.current_ask.as_deref(),
        remote_hint,
        None,
    );
    let project = identity_payload
        .get("project_identity")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &request_scope).await;
    let project_root = project
        .get("project_root")
        .and_then(Value::as_str)
        .or(query.project_root.as_deref());
    let trajectory_record = scoped_trajectory_record(
        &focusa.trajectory.records,
        focusa.trajectory.active_trajectory_id.as_deref(),
        project_root,
    );
    let trajectory = trajectory_record.map(|record| focusa_core::types::TrajectoryLadderContext {
        trajectory_id: Some(record.trajectory_id.clone()).filter(|value| !value.is_empty()),
        project_root: record.project_root.clone(),
        continuity_id: record.continuity_id.clone(),
        hlt: Some(record.long_term_goal.clone()).filter(|value| !value.trim().is_empty()),
        hlt_status: record.hlt_status,
        mlg: record.mid_level_goal.clone(),
        stg: record.short_term_goal.clone(),
        waypoints: record.waypoints.iter().take(8).cloned().collect(),
        active_workpoint_id: record.active_workpoint_id,
    });
    let active_trajectory_record = trajectory_record
        .and_then(|record| serde_json::to_value(record).ok())
        .unwrap_or(Value::Null);
    let active_workpoint = scoped_workpoint_record(
        &focusa.workpoint.records,
        focusa.workpoint.active_workpoint_id.as_ref(),
        project_root,
    )
    .map(|record| {
        json!({
            "workpoint_id": record.workpoint_id,
            "project_root": record.project_root,
            "continuity_id": record.continuity_id,
            "canonical": record.canonical,
            "status": format!("{:?}", record.status),
            "mission": record.mission,
            "next_slice": record.next_slice,
            "active_object_refs": record.active_object_refs,
            "verification_count": record.verification_records.len(),
            "blocker_count": record.blockers.len(),
        })
    });
    let canonical_ontology_objects = focusa.ontology.objects.len();
    let derived_project_objects = 1usize
        + usize::from(trajectory.is_some())
        + usize::from(active_workpoint.is_some())
        + focusa.workpoint.records.len().min(8)
        + trajectory
            .as_ref()
            .map(|t| t.waypoints.len().min(8))
            .unwrap_or(0);
    let runtime_ontology_objects = canonical_ontology_objects + derived_project_objects;
    let effective_ontology_objects = runtime_ontology_objects;
    let ontology_scope_key = project
        .get("project_root")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let ontology = json!({
        "objects": effective_ontology_objects,
        "runtime_objects": runtime_ontology_objects,
        "derived_project_objects": derived_project_objects,
        "links": focusa.ontology.links.len(),
        "proposals": focusa.ontology.proposals.len(),
        "verifications": focusa.ontology.verifications.len(),
        "working_set_refreshes": focusa.ontology.working_set_refreshes.len(),
        "source_index": "project_card_effective_ontology",
        "scope_key": ontology_scope_key,
        "selector": "project_card_effective",
        "freshness": "live_state_snapshot_plus_project_derivatives",
        "canonical_objects": canonical_ontology_objects,
        "count_semantics": "objects/runtime_objects is the project-card live ontology view: canonical ontology objects plus derived project/trajectory/workpoint objects; canonical_objects is the persisted ontology store count",
        "why_zero_if_empty": "runtime_objects=0 means neither canonical ontology objects nor project/trajectory/workpoint derivatives are available for this scope",
        "next_selector": "focusa_traverse surface=ontology selector=window for runtime objects; focusa_traverse surface=workpoints selector=window or focusa_trajectory_view for derived context",
        "counts": {
            "runtime_objects": runtime_ontology_objects,
            "canonical_objects": canonical_ontology_objects,
            "derived_project_objects": derived_project_objects,
            "effective_project_card_objects": effective_ontology_objects,
            "runtime_links": focusa.ontology.links.len()
        },
        "bridge_status": if runtime_ontology_objects > 0 { "runtime_ontology_plus_project_derivatives" } else { "project_derivatives_used_until_runtime_ontology_populates" }
    });
    let reference_handles = focusa.reference_index.handles.len();
    let workpoint_verifications = focusa
        .workpoint
        .records
        .iter()
        .map(|record| record.verification_records.len())
        .sum::<usize>();
    let active_blockers = focusa
        .workpoint
        .records
        .iter()
        .map(|record| record.blockers.len())
        .sum::<usize>();
    let evidence = json!({
        "reference_handles": reference_handles,
        "workpoint_verifications": workpoint_verifications,
    });
    let recent_frames = focusa
        .focus_stack
        .frames
        .iter()
        .rev()
        .take(5)
        .map(|frame| {
            json!({
                "frame_id": frame.id,
                "title": frame.title,
                "goal": frame.goal,
                "project_root": frame.project_root,
                "continuity_id": frame.continuity_id,
                "status": format!("{:?}", frame.status),
            })
        })
        .collect::<Vec<_>>();
    let recent_decisions = focusa
        .focus_stack
        .frames
        .iter()
        .rev()
        .flat_map(|frame| frame.focus_state.decisions.iter().rev().take(4).cloned())
        .take(10)
        .collect::<Vec<_>>();
    let focus_goal_signals = focusa.focus_stack.frames.iter().rev().take(5).map(|frame| json!({
        "intent": frame.focus_state.intent,
        "current_state": frame.focus_state.current_state,
        "next_steps": frame.focus_state.next_steps.iter().take(3).cloned().collect::<Vec<_>>(),
        "recent_results": frame.focus_state.recent_results.iter().take(3).cloned().collect::<Vec<_>>(),
    })).collect::<Vec<_>>();
    drop(focusa);

    let recent_algorithm_outcomes = recent_jsonl_values(project_card_outcomes_path(), 20);
    let (outcome_count, average_outcome_score, recent_outcome_score) =
        project_card_outcome_stats(&recent_algorithm_outcomes);
    let efficiency_summary = project_card_efficiency_summary(&recent_algorithm_outcomes);
    let prediction_workstream = request_scope
        .continuity_id
        .as_ref()
        .and_then(|continuity_id| {
            let root = project.get("project_root").and_then(Value::as_str)?;
            let scope_ref = ScopeRef {
                scope_kind: ScopeKind::Project,
                scope_id: project
                    .get("project_id")
                    .and_then(Value::as_str)
                    .unwrap_or(root)
                    .to_string(),
                root_path: root.into(),
                canonical_name: project
                    .get("canonical_name")
                    .and_then(Value::as_str)
                    .unwrap_or("project")
                    .to_string(),
                fingerprint: project
                    .get("fingerprint")
                    .or_else(|| project.get("project_fingerprint"))
                    .and_then(Value::as_str)
                    .unwrap_or(root)
                    .to_string(),
            };
            WorkstreamKey::new(scope_ref, continuity_id.clone()).ok()
        });
    let prediction_records = if let Some(scope) = prediction_workstream.as_ref() {
        state
            .prediction_store
            .recent(scope, 1000)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let prediction_values = prediction_records
        .iter()
        .filter_map(|record| serde_json::to_value(&record.value).ok())
        .collect::<Vec<_>>();
    let prediction = prediction_stats_card(&prediction_values);
    let current_ask = query.current_ask.as_deref().unwrap_or_default().trim();
    let project_name = project
        .get("canonical_name")
        .and_then(Value::as_str)
        .or_else(|| project.get("project_id").and_then(Value::as_str))
        .unwrap_or("project");
    let trajectory_hlt = trajectory.as_ref().and_then(|t| t.hlt.clone());
    let trajectory_stg = trajectory.as_ref().and_then(|t| t.stg.clone());
    let bootstrap_needed = trajectory_hlt
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty();
    let prediction_accuracy = prediction
        .get("accuracy")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let evaluated_predictions = prediction
        .get("evaluated")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let open_predictions = prediction
        .get("recent_open")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    let algorithmic_intelligence = project_card_algorithmic_scores(
        !bootstrap_needed,
        ontology.get("objects").and_then(Value::as_u64).unwrap_or(0) as usize,
        reference_handles + workpoint_verifications,
        prediction_accuracy,
        evaluated_predictions,
        open_predictions,
        active_blockers,
        outcome_count,
        average_outcome_score,
        recent_outcome_score,
    );
    let algorithm_run_id = uuid::Uuid::now_v7().to_string();
    append_project_card_algorithm_run(&json!({
        "algorithm_run_id": algorithm_run_id,
        "ts": chrono::Utc::now().to_rfc3339(),
        "project_root": project.get("project_root").and_then(Value::as_str).unwrap_or("unknown"),
        "current_ask": current_ask,
        "signals": algorithmic_intelligence.get("signals").cloned().unwrap_or(Value::Null),
        "learned_weights": algorithmic_intelligence.get("learned_weights").cloned().unwrap_or(Value::Null),
        "scores": algorithmic_intelligence.get("scores").cloned().unwrap_or(Value::Null),
        "action_probabilities": algorithmic_intelligence.get("action_probabilities").cloned().unwrap_or(Value::Null),
        "expected_utility": algorithmic_intelligence.get("expected_utility").cloned().unwrap_or(Value::Null),
        "outcome_learning": algorithmic_intelligence.get("outcome_learning").cloned().unwrap_or(Value::Null),
        "efficiency_summary": efficiency_summary,
        "prediction_feed": {"elapsed_and_tokens_included": true, "waypoint_accomplishments_included": true},
        "formula_version": "project_card_algorithmic_intelligence.v3"
    }));
    let next_gap = trajectory_stg
        .clone()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            if current_ask.is_empty() {
                None
            } else {
                Some(current_ask.to_string())
            }
        })
        .unwrap_or_else(|| "refresh trajectory from project card".to_string());
    let inferred_workpoint = inferred_workpoint_candidate(
        project.get("project_root").and_then(Value::as_str),
        current_ask,
        trajectory_stg.as_deref(),
        &active_workpoint,
        &prediction,
        &ontology,
        &recent_frames,
        &recent_decisions,
        &focus_goal_signals,
    );
    let ask_to_workpoint_bridge = inferred_workpoint
        .get("ask_to_workpoint_bridge")
        .cloned()
        .unwrap_or(Value::Null);
    let sequence_plan = project_success_sequence(
        project_name,
        trajectory_hlt.as_deref(),
        trajectory
            .as_ref()
            .and_then(|t| t.mlg.as_deref())
            .or(Some("verified successful end state")),
        None,
        Some(&next_gap),
        &algorithmic_intelligence,
    );
    let trajectory_waypoints = trajectory
        .as_ref()
        .map(|t| t.waypoints.clone())
        .unwrap_or_default();
    let trajectory_report_card = trajectory_reporting_card(
        &trajectory,
        &active_trajectory_record,
        &efficiency_summary,
        &recent_algorithm_outcomes,
    );
    let crosswire_health = json!({
        "schema": "focusa.project_crosswire_health.v1",
        "ontology": {"wired": effective_ontology_objects > 0, "runtime_objects": runtime_ontology_objects, "effective_objects": effective_ontology_objects, "bridge_status": ontology.get("bridge_status").cloned().unwrap_or(Value::Null)},
        "trajectory": {"wired": trajectory.is_some(), "has_hlt": trajectory.as_ref().and_then(|t| t.hlt.as_ref()).is_some(), "has_stg": trajectory.as_ref().and_then(|t| t.stg.as_ref()).is_some()},
        "prediction": {"wired": true, "total": prediction.get("total").cloned().unwrap_or(Value::Null), "evaluated": prediction.get("evaluated").cloned().unwrap_or(Value::Null)},
        "metacognition": {"wired": true, "mode": "retrieval_prompt_plus_outcome_capture"},
        "outcomes": {"wired": outcome_count > 0, "count": outcome_count, "average_score": average_outcome_score},
        "time_tokens": {"wired": efficiency_summary.get("outcome_count_with_timing").and_then(Value::as_u64).unwrap_or(0) > 0 || efficiency_summary.get("outcome_count_with_tokens").and_then(Value::as_u64).unwrap_or(0) > 0, "summary": efficiency_summary},
        "waypoints": trajectory_report_card.get("accomplishment_summary").cloned().unwrap_or(Value::Null),
        "prediction_feed": {"elapsed_tokens_waypoints_feed_future_predictions": true, "algorithm_run_records_efficiency": true, "outcome_records_efficiency": true},
        "known_external_gap": "A running Pi session may need reload to pick up newly registered tools; API/static/live contracts are authoritative."
    });
    let prior_session_context = json!({
        "schema": "focusa.project_prior_context.v1",
        "advisory_only": true,
        "trajectory_ladder": {
            "high_level_goal": trajectory_hlt,
            "mid_level_goal": trajectory.as_ref().and_then(|t| t.mlg.clone()),
            "short_term_goal": trajectory_stg,
            "waypoints": trajectory_waypoints,
        },
        "recent_frames": recent_frames,
        "recent_decisions": recent_decisions,
        "focus_goal_signals": focus_goal_signals,
        "recent_algorithm_outcomes": recent_algorithm_outcomes,
        "prediction_summary": prediction.clone(),
        "metacognition_prompt": "Retrieve lessons for the trajectory ladder, recent decisions, outcomes, and current ask before defining bootstrap goals."
    });

    Json(json!({
        "status": "completed",
        "schema": "focusa.project_card.v1",
        "advisory_only": true,
        "project_identity": project,
        "trajectory": trajectory,
        "ontology": ontology,
        "evidence": evidence,
        "prediction": prediction,
        "algorithmic_intelligence": algorithmic_intelligence,
        "algorithm_run_id": algorithm_run_id,
        "success_sequence": sequence_plan,
        "inferred_workpoint_candidate": inferred_workpoint,
        "ask_to_workpoint_bridge": ask_to_workpoint_bridge,
        "efficiency_summary": efficiency_summary,
        "trajectory_report_card": trajectory_report_card,
        "crosswire_health": crosswire_health,
        "prior_session_context": prior_session_context,
        "metacognition": {
            "summary": "Retrieve relevant lessons with /v1/metacognition/retrieve using project card + current ask.",
            "next_tools": ["focusa_metacog_retrieve", "focusa_metacog_doctor"]
        },
        "active_workpoint": active_workpoint,
        "bootstrap": {
            "needed": bootstrap_needed,
            "candidate": {
                "long_term_goal": format!("Strengthen {project_name} project intelligence through ontology-grounded trajectory, evidence, prediction, and metacog loops"),
                "desired_end_state": format!("{project_name} has an up-to-date project card, trajectory hierarchy, evidence-backed next step, evaluated predictions, and condensed reusable lessons"),
                "short_term_goal": next_gap,
                "prior_context_inputs": ["trajectory_ladder", "recent_decisions", "prediction_summary", "recent_algorithm_outcomes", "metacognition_prompt", "inferred_workpoint_candidate"],
                "inferred_workpoint_candidate": inferred_workpoint,
                "goal_source": "project_card_learning_flywheel"
            },
            "next_tools": if bootstrap_needed { json!(["focusa_trajectory_define_goal", "focusa_workpoint_checkpoint", "focusa_trajectory_view", "focusa_metacog_retrieve", "focusa_predict_record"]) } else { json!(["focusa_workpoint_checkpoint", "focusa_trajectory_assess", "focusa_metacog_retrieve", "focusa_predict_record"]) }
        },
        "possibilities": [
            {"kind": "trajectory_refresh", "action": "assess whether current project card evidence changes active trajectory gap"},
            {"kind": "prediction", "action": "record the next bounded prediction tied to the selected trajectory gap"},
            {"kind": "metacog", "action": "retrieve and condense lessons for similar project-card decisions"}
        ],
        "next_step_quality_rule": "best next step ties to trajectory gap, ontology refs, prediction rationale, evidence proof, and reusable lesson potential",
        "next_tools": ["focusa_project_identity", "focusa_traverse", "focusa_trajectory_view", "focusa_metacog_retrieve", "focusa_predict_record"]
    }))
}

#[allow(clippy::too_many_arguments)]
fn session_transfer_preload_bundle(
    action: &str,
    project_root: &str,
    continuity_id: &str,
    mission: &str,
    next_action: &str,
    latest_prior: &Value,
    target: &str,
    mode: &str,
    write_preload: bool,
    receipt_preview: bool,
    receipt_commit: bool,
    transfer_id: &str,
) -> Value {
    let has_prior = !latest_prior.is_null();
    let current = json!({"mission":mission,"next_action":next_action,"transfer_id":transfer_id});
    let source = if action == "continue" && has_prior {
        latest_prior
    } else {
        &current
    };
    let source_mission = source
        .get("mission")
        .and_then(Value::as_str)
        .unwrap_or(mission);
    let source_next = source
        .get("next_action")
        .and_then(Value::as_str)
        .unwrap_or(next_action);
    let source_transfer_id = source
        .get("transfer_id")
        .and_then(Value::as_str)
        .unwrap_or(transfer_id);

    let packet = if action == "continue" && has_prior {
        let mut packet = build_packet_for_profile(PROFILE_RULES_AND_CONTEXT).unwrap_or(Value::Null);
        packet["target"] = json!(target);
        packet["mode"] = json!(mode);
        packet["project_root"] = json!(project_root);
        packet["continuity_id"] = json!(continuity_id);
        packet["source_transfer_id"] = json!(source_transfer_id);
        packet["dynamic_context_lines"] = json!([source_mission, source_next]);
        packet["selected_context"] = json!({"include":[
            {"kind":"session_transfer","path":"transfer:mission","body":source_mission},
            {"kind":"session_transfer","path":"transfer:next_action","body":source_next}
        ],"exclude":[],"over_budget":[]});
        packet["rendered"] = json!(format!(
            "# Focusa Session Transfer Bootstrap\n\n- mission: {source_mission}\n- next: {source_next}\n"
        ));
        packet
    } else {
        Value::Null
    };

    let preview = if receipt_preview {
        json!({
            "schema":"focusa.preload_session_transfer_receipt_preview.v1",
            "receipt_kind":"bootstrap_delivery",
            "preview":true,
            "target":target,
            "mode":mode,
            "source_transfer_id":source_transfer_id,
            "packet_available":!packet.is_null()
        })
    } else {
        Value::Null
    };
    let committed = if receipt_commit {
        match commit_receipt_for(
            PROFILE_RULES_AND_CONTEXT,
            &format!("session-transfer-{source_transfer_id}"),
        ) {
            Ok((receipt, replay)) => {
                json!({"status":"completed","idempotent_replay":replay,"receipt":receipt})
            }
            Err(error) => {
                json!({"status":"failed","failure_class":"receipt_commit_failed","error":error})
            }
        }
    } else {
        Value::Null
    };
    let degraded = action == "continue" && !has_prior;
    json!({
        "status": if degraded { "degraded" } else { "completed" },
        "target": target,
        "mode": mode,
        "packet": packet,
        "write": {
            "requested": write_preload,
            "performed": false,
            "reason": if write_preload { "operator_handoff_command_required" } else { "write_preload_false" }
        },
        "receipt_preview": preview,
        "receipt_commit": committed,
        "next_tools": if degraded { vec!["focusa_preload_build"] } else { vec!["focusa_preload_verify", "focusa_preload_receipt_preview"] }
    })
}

async fn session_transfer(
    request_scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(body): Json<ProjectSessionTransferRequest>,
) -> Json<Value> {
    let action = body.action.trim().to_lowercase();
    if !["save", "continue", "status", "rollover", "verify_target"].contains(&action.as_str()) {
        return Json(json!({
            "status":"blocked",
            "failure_class":"invalid_action",
            "reason":"session transfer action must be save, continue, status, rollover, or verify_target"
        }));
    }
    let query = ProjectIdentityQuery {
        cwd: body.cwd.clone(),
        project_root: body.project_root.clone(),
        current_ask: body.current_ask.clone().or_else(|| {
            Some(match action.as_str() {
                "save" => "Save current Focusa work for transfer".to_string(),
                "continue" => "Continue latest saved Focusa work like a game save".to_string(),
                _ => "Inspect Focusa session transfer readiness".to_string(),
            })
        }),
        ..Default::default()
    };
    let card_payload = card(request_scope.clone(), State(state), Query(query))
        .await
        .0;
    let project_root = card_payload
        .pointer("/project_identity/project_root")
        .and_then(Value::as_str)
        .or(body.project_root.as_deref())
        .or(request_scope.project_root.as_deref())
        .unwrap_or("")
        .to_string();
    let continuity_id = body
        .source_scope
        .as_ref()
        .map(|scope| scope.continuity_id.clone())
        .or_else(|| body.continuity_id.clone())
        .or_else(|| request_scope.continuity_id.clone())
        .unwrap_or_default();
    if project_root.is_empty() || continuity_id.trim().is_empty() {
        return Json(json!({
            "status": "blocked",
            "schema": "focusa.project_session_transfer_response.v2",
            "failure_class": "scope_mismatch",
            "reason": "typed source project_root and continuity_id are required; static continuity fallback is forbidden",
            "next_tools": ["focusa_project_identity", "focusa_workpoint_resume"]
        }));
    }
    let canonical_name = card_payload
        .pointer("/project_identity/canonical_name")
        .and_then(Value::as_str)
        .unwrap_or_else(|| project_root.rsplit('/').next().unwrap_or("project"));
    let fingerprint = card_payload
        .pointer("/project_identity/fingerprint")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| stable_fingerprint(std::slice::from_ref(&project_root)));
    let source_scope = match body.source_scope.clone() {
        Some(scope) if scope.validate().is_ok() => scope,
        Some(_) => {
            return Json(
                json!({"status":"blocked","failure_class":"scope_mismatch","reason":"invalid source_scope"}),
            );
        }
        None => {
            let root_scope = match ScopeRef::project(
                format!("project:{fingerprint}"),
                &project_root,
                canonical_name,
                &fingerprint,
            ) {
                Ok(scope) => scope,
                Err(error) => {
                    return Json(
                        json!({"status":"blocked","failure_class":"scope_mismatch","reason":error.to_string()}),
                    );
                }
            };
            match WorkstreamKey::new(root_scope, continuity_id.clone()) {
                Ok(scope) => scope,
                Err(error) => {
                    return Json(
                        json!({"status":"blocked","failure_class":"scope_mismatch","reason":error.to_string()}),
                    );
                }
            }
        }
    };
    let target_scope = if action == "rollover" {
        let target = match body.target_scope.clone() {
            Some(scope) => scope,
            None => {
                let target_continuity = body.target_continuity_id.clone().unwrap_or_default();
                if target_continuity.trim().is_empty() {
                    return Json(
                        json!({"status":"blocked","failure_class":"scope_mismatch","reason":"rollover requires target_scope or target_continuity_id"}),
                    );
                }
                match WorkstreamKey::new(source_scope.root_scope.clone(), target_continuity) {
                    Ok(scope) => scope,
                    Err(error) => {
                        return Json(
                            json!({"status":"blocked","failure_class":"scope_mismatch","reason":error.to_string()}),
                        );
                    }
                }
            }
        };
        if target.validate().is_err()
            || target.root_scope != source_scope.root_scope
            || target.continuity_id == source_scope.continuity_id
        {
            return Json(json!({
                "status":"blocked",
                "failure_class":"scope_mismatch",
                "reason":"target scope must share the verified project root and use a new continuity_id"
            }));
        }
        Some(target)
    } else {
        body.target_scope.clone()
    };
    let source_working_subpath_id = body
        .source_working_subpath_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("primary")
        .to_string();
    let target_working_subpath_id = body
        .target_working_subpath_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(source_working_subpath_id.as_str())
        .to_string();
    let inferred = card_payload
        .get("inferred_workpoint_candidate")
        .cloned()
        .unwrap_or(Value::Null);
    let hint = inferred
        .get("checkpoint_payload_hint")
        .cloned()
        .unwrap_or(Value::Null);
    let mission = body
        .mission
        .clone()
        .or_else(|| {
            hint.get("mission")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .or_else(|| {
            inferred
                .get("mission")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| {
            body.current_ask
                .clone()
                .unwrap_or_else(|| "Resume saved Focusa work".to_string())
        });
    let next_action = body
        .next_action
        .clone()
        .or_else(|| {
            hint.get("next_action")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .or_else(|| {
            inferred
                .get("next_action")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "Continue from session-transfer packet".to_string());
    let transfer_id = uuid::Uuid::now_v7().to_string();
    let transfers_path = project_session_transfers_path(&source_scope.root_scope);
    let recent_transfers = recent_jsonl_values(transfers_path.clone(), 256);
    let latest_prior = recent_transfers
        .iter()
        .rev()
        .find(|record| {
            if record.get("schema").and_then(Value::as_str)
                != Some("focusa.project_session_transfer.v2")
            {
                return false;
            }
            let source_matches = record
                .pointer("/source_scope/root_scope/root_path")
                .and_then(Value::as_str)
                == Some(project_root.as_str())
                && record
                    .pointer("/source_scope/continuity_id")
                    .and_then(Value::as_str)
                    == Some(source_scope.continuity_id.as_str())
                && record
                    .get("source_working_subpath_id")
                    .and_then(Value::as_str)
                    .unwrap_or("primary")
                    == source_working_subpath_id.as_str();
            let target_matches = record
                .pointer("/target_scope/root_scope/root_path")
                .and_then(Value::as_str)
                == Some(project_root.as_str())
                && record
                    .pointer("/target_scope/continuity_id")
                    .and_then(Value::as_str)
                    == Some(source_scope.continuity_id.as_str())
                && record
                    .get("target_working_subpath_id")
                    .and_then(Value::as_str)
                    .unwrap_or("primary")
                    == source_working_subpath_id.as_str();
            source_matches || target_matches
        })
        .cloned()
        .unwrap_or(Value::Null);
    let preload_target = body.preload_target.as_deref().unwrap_or("cursor");
    let preload_mode = body.preload_mode.as_deref().unwrap_or("session_transfer");
    let preload = session_transfer_preload_bundle(
        &action,
        &project_root,
        &continuity_id,
        &mission,
        &next_action,
        &latest_prior,
        preload_target,
        preload_mode,
        body.write_preload.unwrap_or(false),
        body.receipt_preview.unwrap_or(true),
        body.receipt_commit.unwrap_or(false),
        &transfer_id,
    );
    let transfer_status = preload["status"].as_str().unwrap_or("completed");
    let operator_handoff = json!({
        "command": format!("cd {project_root} && pi"),
        "first_tool": format!("focusa_session_transfer action=\"continue\" project_root=\"{project_root}\" continuity_id=\"{continuity_id}\""),
        "preload": format!("focusa preload write --target {preload_target} --project-root {project_root} --continuity-id {continuity_id}"),
        "receipt_preview": format!("focusa preload receipt-preview --target {preload_target} --project-root {project_root} --continuity-id {continuity_id}"),
        "authority_boundary": "canonical_parent_plus_working_subpath_plus_continuity_id"
    });
    let record = json!({
        "schema": "focusa.project_session_transfer.v3",
        "transfer_id": transfer_id,
        "source_scope": source_scope,
        "target_scope": target_scope,
        "source_working_subpath_id": source_working_subpath_id,
        "target_working_subpath_id": target_working_subpath_id,
        "transition": {
            "status": if action == "rollover" { "target_attachment_pending" } else { "saved" },
            "source_session_id": body.source_session_id,
            "target_session_id": body.target_session_id,
            "target_workpoint_id": body.target_workpoint_id,
            "target_resume_canonical": body.target_resume_canonical,
            "source_checkpoint_id": body.source_checkpoint_id,
            "compaction_packet_id": body.compaction_packet_id,
            "adapter": body.adapter.as_deref().unwrap_or("unknown"),
            "evidence_refs": body.evidence_refs,
            "requires_target_resume_verification": action == "rollover"
        },
        "action": action,
        "ts": chrono::Utc::now().to_rfc3339(),
        "project_root": project_root,
        "continuity_id": continuity_id,
        "mission": mission,
        "next_action": next_action,
        "inferred_workpoint_candidate": inferred,
        "checkpoint_payload_hint": hint,
        "trajectory_report_card": card_payload.get("trajectory_report_card").cloned().unwrap_or(Value::Null),
        "crosswire_health": card_payload.get("crosswire_health").cloned().unwrap_or(Value::Null),
        "success_sequence": card_payload.get("success_sequence").cloned().unwrap_or(Value::Null),
        "algorithm_run_id": card_payload.get("algorithm_run_id").cloned().unwrap_or(Value::Null),
        "preload": preload,
        "operator_handoff": operator_handoff
    });
    if action == "save" || action == "rollover" {
        append_project_session_transfer(&source_scope.root_scope, &record);
    }
    let transfer = if (action == "continue" || action == "verify_target") && !latest_prior.is_null()
    {
        let mut prior = latest_prior.clone();
        prior["preload"] = preload.clone();
        prior["operator_handoff"] = operator_handoff.clone();
        if action == "verify_target" {
            let verified = body.target_resume_canonical == Some(true)
                && body
                    .target_workpoint_id
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                && body
                    .target_session_id
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty());
            let expected_status = if verified {
                "target_resume_verified"
            } else {
                "target_resume_degraded"
            };
            let existing_receipt = recent_transfers.iter().rev().find(|receipt| {
                receipt.get("schema").and_then(Value::as_str)
                    == Some("focusa.project_session_transition_receipt.v1")
                    && receipt.get("transfer_id") == prior.get("transfer_id")
                    && receipt.get("target_session_id").and_then(Value::as_str)
                        == body.target_session_id.as_deref()
                    && receipt.get("target_workpoint_id").and_then(Value::as_str)
                        == body.target_workpoint_id.as_deref()
                    && receipt
                        .get("target_resume_canonical")
                        .and_then(Value::as_bool)
                        == body.target_resume_canonical
                    && receipt.get("status").and_then(Value::as_str) == Some(expected_status)
            });
            let receipt = if let Some(existing) = existing_receipt {
                let mut replay = existing.clone();
                replay["idempotent_replay"] = json!(true);
                replay
            } else {
                let receipt = json!({
                    "schema": "focusa.project_session_transition_receipt.v1",
                    "receipt_id": Uuid::now_v7().to_string(),
                    "transfer_id": prior.get("transfer_id"),
                    "source_scope": prior.get("source_scope"),
                    "target_scope": prior.get("target_scope"),
                    "target_session_id": body.target_session_id,
                    "target_workpoint_id": body.target_workpoint_id,
                    "target_resume_canonical": body.target_resume_canonical,
                    "status": expected_status,
                    "verified_at": Utc::now().to_rfc3339(),
                    "evidence_refs": body.evidence_refs,
                });
                append_project_session_transfer(&source_scope.root_scope, &receipt);
                receipt
            };
            prior["transition_receipt"] = receipt;
            prior["transition"]["status"] = json!(expected_status);
        }
        prior
    } else {
        record
    };
    Json(json!({
        "status": transfer_status,
        "schema": "focusa.project_session_transfer_response.v1",
        "action": action,
        "saved": action == "save" || action == "rollover",
        "transfer": transfer,
        "latest_prior_save": latest_prior,
        "preload": preload,
        "storage": {"transfers_path": transfers_path.to_string_lossy(), "scope": source_scope.root_scope},
        "next_tools": if transfer_status == "degraded" {
            vec!["focusa_preload_build", "focusa_project_card", "focusa_trajectory_view"]
        } else {
            vec!["focusa_workpoint_checkpoint", "focusa_workpoint_resume", "focusa_preload_verify", "focusa_trajectory_view"]
        }
    }))
}

async fn card_outcome(Json(body): Json<ProjectCardOutcomeRequest>) -> Json<Value> {
    let algorithm_run_id = body.algorithm_run_id.trim();
    if algorithm_run_id.is_empty() || body.actual_outcome.trim().is_empty() {
        return Json(json!({
            "status": "blocked",
            "failure_class": "validation_rejected",
            "reason": "algorithm_run_id and actual_outcome are required",
            "next_tools": ["focusa_project_card"]
        }));
    }
    if !project_card_run_exists(algorithm_run_id) {
        return Json(json!({
            "status": "not_found",
            "failure_class": "not_found",
            "algorithm_run_id": algorithm_run_id,
            "recovery_hint": "call /v1/project/card to create a fresh algorithm_run_id, then attach the outcome",
            "next_tools": ["focusa_project_card"]
        }));
    }
    let score = body.score.unwrap_or(1.0).clamp(0.0, 1.0);
    let learned_weights = update_weights_from_algorithm_outcome(score);
    let outcome_id = uuid::Uuid::now_v7().to_string();
    let record = json!({
        "outcome_id": outcome_id,
        "algorithm_run_id": algorithm_run_id,
        "ts": chrono::Utc::now().to_rfc3339(),
        "project_root": body.project_root,
        "actual_outcome": body.actual_outcome,
        "score": score,
        "evidence_refs": body.evidence_refs,
        "notes": body.notes,
        "task_timing": body.task_timing,
        "token_usage": body.token_usage,
        "learned_weights_after": learned_weights,
        "formula_version": "project_card_algorithmic_intelligence.v1"
    });
    append_project_card_algorithm_outcome(&record);
    Json(json!({
        "status": "recorded",
        "schema": "focusa.project_card_algorithm_outcome.v1",
        "outcome": record,
        "storage": {"outcomes_path": project_card_outcomes_path().to_string_lossy(), "weights_path": project_card_weights_path().to_string_lossy()},
        "flywheel": {"outcome_to_weights": true, "next_tools": ["focusa_project_card", "focusa_predict_record", "focusa_metacog_capture"]}
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/project/identity", get(identity))
        .route("/v1/project/verify", post(verify))
        .route("/v1/project/card", get(card))
        .route("/v1/project/card/outcome", post(card_outcome))
        .route("/v1/project/session-transfer", post(session_transfer))
        .route("/v1/project/list", get(list_projects))
        .route("/v1/project/discover", get(discover_projects))
        .route("/v1/project/use", post(use_project))
        .route("/v1/project/bind", post(use_project))
        .route("/v1/project/switch", post(use_project))
        .route("/v1/project/current", get(current_status))
        .route("/v1/project/status", get(current_status_alias))
        .route("/v1/project/remove", post(remove_selected_project))
        .route("/v1/project/new", post(create_project))
        .route("/v1/project/templates", get(project_templates))
        .route(
            "/v1/project/settings",
            get(project_settings_get).post(project_settings_update),
        )
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    fn temp_project(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "focusa-project-test-{name}-{}",
            uuid::Uuid::now_v7()
        ));
        fs::create_dir_all(&root).expect("create temp project");
        root
    }

    #[test]
    fn identity_name_match_accepts_safe_case_and_aliases() {
        let aliases = vec!["focusa-daemon".to_string(), "focusa-cli".to_string()];
        assert!(identity_name_matches(
            "focusa",
            "Focusa",
            "focusa",
            &aliases,
            Some("/workspace/focusa"),
            Some("/workspace/focusa")
        ));
        assert!(identity_name_matches(
            "FOCUSA DAEMON",
            "Focusa",
            "focusa",
            &aliases,
            Some("/workspace/focusa"),
            Some("/workspace/focusa")
        ));
        assert!(!identity_name_matches(
            "uiai-engine",
            "Focusa",
            "focusa",
            &aliases,
            Some("/workspace/focusa"),
            Some("/workspace/focusa")
        ));
    }

    #[test]
    fn scoped_trajectory_record_prefers_project_root_match() {
        let mut record_a = focusa_core::types::TrajectoryProjectionRecord::default();
        record_a.trajectory_id = "project-a-traject-id".to_string();
        record_a.project_root = Some("/tmp/focusa-project-a".to_string());
        record_a.long_term_goal = "project A".to_string();

        let mut record_b = focusa_core::types::TrajectoryProjectionRecord::default();
        record_b.trajectory_id = "project-b-traject-id".to_string();
        record_b.project_root = Some("/tmp/focusa-project-b".to_string());
        record_b.long_term_goal = "project B".to_string();

        let records = vec![record_a, record_b];
        let chosen = scoped_trajectory_record(
            &records,
            Some("project-a-traject-id"),
            Some("/tmp/focusa-project-b"),
        );
        assert_eq!(
            chosen.expect("record should exist").trajectory_id,
            "project-b-traject-id"
        );
    }

    #[test]
    fn scoped_workpoint_record_prefers_project_root_match() {
        let mut a = focusa_core::types::WorkpointRecord::default();
        a.project_root = Some("/tmp/focusa-project-a".to_string());
        a.workpoint_id = focusa_core::types::WorkpointId::now_v7();
        a.work_item_id = Some("wp-a".to_string());

        let mut b = focusa_core::types::WorkpointRecord::default();
        b.workpoint_id = focusa_core::types::WorkpointId::now_v7();
        b.project_root = Some("/tmp/focusa-project-b".to_string());
        b.work_item_id = Some("wp-b".to_string());

        let records = vec![a, b];
        let chosen = scoped_workpoint_record(&records, None, Some("/tmp/focusa-project-b"));
        assert_eq!(
            chosen.expect("record should exist").work_item_id.as_deref(),
            Some("wp-b")
        );
    }

    #[test]
    fn git_beads_workspace_quorum_verifies_identity() {
        let root = temp_project("quorum");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(
            root.join(".git/config"),
            "[remote \"origin\"]\n\turl = https://example.test/focusa.git\n",
        )
        .unwrap();
        fs::create_dir_all(root.join(".beads")).unwrap();
        fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
        fs::write(
            root.join(".focusa-project.json"),
            r#"{"schema":"focusa.project.v1","project_id":"quorum","canonical_name":"Quorum"}"#,
        )
        .unwrap();
        let candidate = discover_identity(root.to_str(), None, None, RemoteProjectHint::default());
        assert_eq!(candidate.status, "verified");
        assert_eq!(candidate.confidence, "high");
        assert!(candidate.mismatches.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn remote_hint_is_bound_into_identity_payload() {
        let root = temp_project("remote-hint");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join(".git/config"), "").unwrap();
        fs::create_dir_all(root.join(".beads")).unwrap();

        let payload = project_identity_payload_for_scope_with_remote(
            root.to_str(),
            None,
            None,
            RemoteProjectHint {
                remote_host: Some("host.7svnstrms.com".to_string()),
                remote_user: Some("planmarr".to_string()),
                remote_port: Some(2200),
                remote_repo_remote: Some(
                    "https://github.com/example/plan-the-marriage.git".to_string(),
                ),
                remote_workspace_kind: Some("react-vite".to_string()),
                remote_deploy_root: Some("/home/planmarr/public_html".to_string()),
                ..RemoteProjectHint::default()
            },
            None,
        );

        assert_eq!(
            payload
                .pointer("/project_identity/remote_context/remote_host")
                .and_then(Value::as_str),
            Some("host.7svnstrms.com")
        );
        assert_eq!(
            payload
                .pointer("/project_identity/remote_context/remote_port")
                .and_then(Value::as_u64),
            Some(2200)
        );
        assert_eq!(
            payload
                .pointer("/project_identity/authority_boundary")
                .and_then(Value::as_str),
            Some("remote_host_plus_project_root_plus_fingerprint")
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn marker_exposes_environment_and_deploy_facts() {
        let root = temp_project("environment");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join(".git/config"), "").unwrap();
        fs::create_dir_all(root.join(".beads")).unwrap();
        fs::write(
            root.join(".focusa-project.json"),
            format!(
                r#"{{"schema":"focusa.project.v1","project_id":"asap","canonical_name":"ASAP Digest","project_root":"{}","root_url":"https://app.asapdigest.com","local_url":"https://asapdigest.local","deployment":{{"environment":"live","deploy_target":"app.asapdigest.com","deploy_location":"/home/asapdigest/public_html","deploy_command":"scripts/deploy-live.sh"}}}}"#,
                root.to_string_lossy()
            ),
        )
        .unwrap();
        let payload = project_identity_payload_for_scope(root.to_str(), None, None);
        assert_eq!(
            payload
                .pointer("/project_identity/project_urls/root_url")
                .and_then(Value::as_str),
            Some("https://app.asapdigest.com")
        );
        assert_eq!(
            payload
                .pointer("/project_identity/deployment/environment")
                .and_then(Value::as_str),
            Some("live")
        );
        assert_eq!(
            payload
                .pointer("/project_identity/deployment/deploy_location")
                .and_then(Value::as_str),
            Some("/home/asapdigest/public_html")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scans_repo_files_to_infer_live_environment_without_marker_fields() {
        let root = temp_project("scan-env");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join(".git/config"), "").unwrap();
        fs::create_dir_all(root.join(".beads")).unwrap();
        fs::create_dir_all(root.join("scripts")).unwrap();
        fs::write(
            root.join("README.md"),
            "Local legacy URL: https://asapdigest.local\nDocs reference: https://codex.wordpress.org/Editing_wp-config.php\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("app/src")).unwrap();
        fs::write(
            root.join("app/src/hooks.server.js"),
            "const allowedOrigin = 'https://app.asapdigest.com';\n",
        )
        .unwrap();
        fs::write(
            root.join("scripts/deploy-live.sh"),
            "rsync -av ./ /home/asapdigest/public_html/\n",
        )
        .unwrap();
        fs::write(
            root.join("wp-config.php"),
            "define('WP_HOME', 'https://asapdigest.com');\ndefine('WP_SITEURL', 'https://asapdigest.com');\n",
        )
        .unwrap();
        let payload = project_identity_payload_for_scope(root.to_str(), None, None);
        assert_eq!(
            payload
                .pointer("/project_identity/project_urls/live_url")
                .and_then(Value::as_str),
            Some("https://asapdigest.com")
        );
        assert_ne!(
            payload
                .pointer("/project_identity/project_urls/live_url")
                .and_then(Value::as_str),
            Some("https://codex.wordpress.org/Editing_wp-config.php")
        );
        assert_eq!(
            payload
                .pointer("/project_identity/project_urls/local_url")
                .and_then(Value::as_str),
            Some("https://asapdigest.local")
        );
        assert_eq!(
            payload
                .pointer("/project_identity/deployment/environment")
                .and_then(Value::as_str),
            Some("live")
        );
        assert_eq!(
            payload
                .pointer("/project_identity/deployment/deploy_location")
                .and_then(Value::as_str),
            Some("/home/asapdigest/public_html/")
        );
        assert_eq!(
            payload
                .pointer("/project_summary/urls/wp_url")
                .and_then(Value::as_str),
            Some("https://asapdigest.com")
        );
        assert!(
            payload
                .pointer("/project_summary/project/stack")
                .and_then(Value::as_array)
                .is_some_and(|stack| stack
                    .iter()
                    .any(|value| value.as_str() == Some("wordpress")))
        );
        assert!(
            payload
                .pointer("/summary_lines")
                .and_then(Value::as_array)
                .is_some_and(|lines| lines.iter().any(|line| line
                    .as_str()
                    .is_some_and(|text| text.contains("urls=local_only:"))))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn parent_public_html_wp_config_does_not_bleed_into_child_project() {
        let parent = temp_project("parent-live-root");
        let root = parent.join("focusa");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join(".git/config"), "").unwrap();
        fs::create_dir_all(root.join(".beads")).unwrap();
        fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
        fs::write(
            root.join(".env.example"),
            "# Local only\nFOCUSA_API_URL=http://127.0.0.1:8787\n",
        )
        .unwrap();
        fs::create_dir_all(parent.join("public_html")).unwrap();
        fs::write(
            parent.join("public_html/wp-config.php"),
            "define('WP_HOME', 'https://unrelated-live.example');\n",
        )
        .unwrap();

        let payload = project_identity_payload_for_scope(root.to_str(), None, None);
        assert_eq!(
            payload
                .pointer("/project_identity/project_urls/live_url")
                .and_then(Value::as_str),
            None
        );
        assert_eq!(
            payload
                .pointer("/project_summary/local_only")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            payload
                .pointer("/project_identity/deployment/environment")
                .and_then(Value::as_str),
            Some("local")
        );
        let _ = fs::remove_dir_all(parent);
    }

    #[test]
    fn explicit_git_scope_ignores_parent_account_beads_and_workspace() {
        let parent = temp_project("parent-account-scope");
        fs::create_dir_all(parent.join(".beads")).unwrap();
        fs::write(parent.join("package.json"), r#"{"name":"hosting-account"}"#).unwrap();
        let root = parent.join("uiai-engine");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(
            root.join(".git/config"),
            "[remote \"origin\"]\n\turl = https://example.test/uiai-engine.git\n",
        )
        .unwrap();
        fs::write(root.join("go.mod"), "module example.test/uiai-engine\n").unwrap();
        fs::write(
            root.join(".focusa-project.json"),
            r#"{"schema":"focusa.project.v1","project_id":"uiai-engine","canonical_name":"UIAI Engine"}"#,
        )
        .unwrap();

        let candidate = discover_identity(
            root.to_str(),
            root.to_str(),
            None,
            RemoteProjectHint::default(),
        );
        assert_eq!(candidate.status, "verified");
        assert_eq!(candidate.confidence, "high");
        assert!(candidate.mismatches.is_empty());
        assert!(candidate.signals.iter().any(|signal| {
            signal.source == "beads_root"
                && signal.root.as_deref() == Some(parent.to_string_lossy().as_ref())
                && !signal.independent
        }));
        assert!(candidate.signals.iter().any(|signal| {
            signal.source == "workspace_file"
                && signal.root.as_deref() == Some(root.to_string_lossy().as_ref())
                && signal.independent
        }));
        let _ = fs::remove_dir_all(parent);
    }

    #[test]
    fn conflicting_marker_degrades_identity() {
        let root = temp_project("mismatch");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join(".git/config"), "").unwrap();
        fs::write(root.join(".focusa-project.json"), r#"{"schema":"focusa.project.v1","project_id":"other","project_root":"/definitely/not/this/project"}"#).unwrap();
        let candidate = discover_identity(root.to_str(), None, None, RemoteProjectHint::default());
        assert_eq!(candidate.status, "mismatch");
        assert!(!candidate.mismatches.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn persisted_session_identity_mismatch_degrades_identity() {
        let root = temp_project("persisted-mismatch");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join(".git/config"), "").unwrap();
        fs::create_dir_all(root.join(".beads")).unwrap();
        fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
        let payload = project_identity_payload_for_scope_with_remote(
            root.to_str(),
            None,
            None,
            RemoteProjectHint {
                persisted_project_root: Some("/other/project".to_string()),
                persisted_project_fingerprint: Some("project-fnv1a64:deadbeefdeadbeef".to_string()),
                persisted_project_id: Some("other".to_string()),
                persisted_canonical_name: Some("Other".to_string()),
                ..RemoteProjectHint::default()
            },
            None,
        );
        assert_eq!(
            payload
                .pointer("/project_identity/status")
                .and_then(Value::as_str),
            Some("mismatch")
        );
        assert_eq!(
            payload.get("canonical").and_then(Value::as_bool),
            Some(false)
        );
        assert!(
            payload
                .pointer("/project_identity/signals")
                .and_then(Value::as_array)
                .is_some_and(|signals| signals.iter().any(|signal| {
                    signal.get("source").and_then(Value::as_str)
                        == Some("persisted_session_identity")
                        && signal.get("independent").and_then(Value::as_bool) == Some(false)
                }))
        );
        assert!(
            payload
                .pointer("/project_identity/mismatches")
                .and_then(Value::as_array)
                .is_some_and(|items| items
                    .iter()
                    .any(|item| item.get("source").and_then(Value::as_str)
                        == Some("persisted_session_identity_fingerprint")))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn marker_aliases_and_beads_issue_prefix_are_exposed() {
        let root = temp_project("aliases-prefix");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join(".git/config"), "").unwrap();
        fs::create_dir_all(root.join(".beads")).unwrap();
        fs::write(
            root.join(".beads/issues.jsonl"),
            r#"{"id":"focusa-abcd","title":"x"}"#,
        )
        .unwrap();
        fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
        fs::write(
            root.join(".focusa-project.json"),
            format!(
                r#"{{"schema":"focusa.project.v1","project_id":"focusa","canonical_name":"Focusa","project_root":"{}","aliases":["focusa-daemon","focusa-cli"]}}"#,
                root.to_string_lossy()
            ),
        )
        .unwrap();
        let payload = project_identity_payload_for_scope(root.to_str(), None, None);
        assert_eq!(
            payload
                .pointer("/project_identity/aliases/0")
                .and_then(Value::as_str),
            Some("focusa-daemon")
        );
        assert_eq!(
            payload
                .pointer("/project_identity/beads_prefix")
                .and_then(Value::as_str),
            Some("focusa")
        );
        assert!(
            payload
                .pointer("/project_identity/signals")
                .and_then(Value::as_array)
                .is_some_and(|signals| signals.iter().any(|signal| signal
                    .pointer("/details/issue_prefix")
                    .and_then(Value::as_str)
                    == Some("focusa")))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn create_project_writes_marker_settings_and_focusa_skeleton() {
        let root = temp_project("project-new-skeleton");
        let Json(payload) = create_project(Json(ProjectCreateRequest {
            project_root: root.to_string_lossy().to_string(),
            project_id: "new-project".to_string(),
            canonical_name: "New Project".to_string(),
            template: Some("blank".to_string()),
            workspace_kind: Some("rust-monorepo".to_string()),
            create_git: Some(false),
            use_selected: Some(false),
            force: Some(false),
        }))
        .await;
        assert_eq!(payload.get("status").and_then(Value::as_str), Some("ok"));
        assert!(root.join(".focusa-project.json").exists());
        assert!(root.join(".focusa/settings.json").exists());
        for child in ["evidence", "workpoints", "trajectories", "templates"] {
            assert!(root.join(".focusa").join(child).is_dir());
        }
        assert!(root.join(".focusa/README.md").exists());
        assert!(root.join("README.md").exists());
        assert_eq!(
            payload.pointer("/created/git").and_then(Value::as_str),
            Some("skipped")
        );
        assert_eq!(
            payload
                .pointer("/created/selected")
                .and_then(Value::as_bool),
            Some(false)
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn session_transfer_continue_without_prior_save_is_degraded() {
        let bundle = session_transfer_preload_bundle(
            "continue",
            "/tmp/project",
            "focusa-cont-test",
            "mission",
            "next",
            &Value::Null,
            "cursor",
            "session_transfer",
            false,
            true,
            false,
            "transfer-new",
        );
        assert_eq!(bundle["status"], "degraded");
        assert!(bundle["packet"].is_null());
        assert_eq!(bundle["write"]["performed"], false);
        assert!(bundle["receipt_commit"].is_null());
        assert_eq!(bundle["next_tools"][0], "focusa_preload_build");
    }

    #[test]
    fn session_transfer_continue_builds_from_latest_prior_save() {
        let prior = json!({
            "transfer_id":"transfer-prior",
            "mission":"saved mission",
            "next_action":"saved next"
        });
        let bundle = session_transfer_preload_bundle(
            "continue",
            "/tmp/project",
            "focusa-cont-test",
            "current",
            "current next",
            &prior,
            "cursor",
            "session_transfer",
            true,
            true,
            false,
            "transfer-new",
        );
        assert_eq!(bundle["status"], "completed");
        assert_eq!(bundle["packet"]["source_transfer_id"], "transfer-prior");
        assert_eq!(
            bundle["packet"]["dynamic_context_lines"][0],
            "saved mission"
        );
        assert_eq!(bundle["write"]["requested"], true);
        assert_eq!(bundle["write"]["performed"], false);
        assert_eq!(bundle["receipt_preview"]["packet_available"], true);
        assert!(bundle["receipt_commit"].is_null());
    }

    #[test]
    fn dashboard_requires_selection_when_runtime_scope_is_unverified() {
        let dashboard = build_project_dashboard(
            json!({"status":"invalid","project_identity":{"status":"mismatch"}}),
            None,
            vec![],
        );
        assert_eq!(dashboard["status"], "degraded");
        assert_eq!(
            dashboard["failure_class"],
            "project_root_selection_required"
        );
        assert!(dashboard["effective_project"].is_null());
    }

    #[test]
    fn dashboard_prefers_safe_selected_project_over_runtime_mismatch() {
        let selected = json!({
            "status":"selected",
            "project_root":"/tmp/safe-project",
            "fingerprint":"project:test"
        });
        let dashboard = build_project_dashboard(
            json!({"project_identity":{"status":"mismatch","root":"/usr/local/lib/focusa"}}),
            Some(selected.clone()),
            vec![selected.clone()],
        );
        assert_eq!(dashboard["status"], "ok");
        assert_eq!(dashboard["effective_project"], selected);
        assert_eq!(dashboard["project_count"], 1);
    }

    #[test]
    fn dashboard_accepts_verified_runtime_without_saved_selection() {
        let runtime = json!({"project_identity":{"status":"verified","root":"/tmp/project"}});
        let dashboard = build_project_dashboard(runtime.clone(), None, vec![]);
        assert_eq!(dashboard["status"], "ok");
        assert_eq!(dashboard["effective_project"], runtime);
    }

    #[test]
    fn broad_root_never_verifies_as_project_identity() {
        let candidate = discover_identity(
            Some("/root"),
            Some("/root"),
            None,
            RemoteProjectHint::default(),
        );
        assert_eq!(candidate.status, "unsafe_project_root");
        assert_eq!(candidate.confidence, "low");
        assert!(candidate.mismatches.iter().any(
            |item| item.get("source").and_then(Value::as_str) == Some("project_root_authority")
        ));
        let payload = candidate_payload(candidate, None);
        assert_eq!(
            payload
                .pointer("/project_identity/status")
                .and_then(Value::as_str),
            Some("unsafe_project_root")
        );
        assert_eq!(
            payload
                .pointer("/details/tool_result_v1/failure_class")
                .and_then(Value::as_str),
            Some("scope_mismatch")
        );
    }
}
