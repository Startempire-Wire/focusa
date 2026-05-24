//! Spec96 ProjectIdentity discovery and verification routes.
//!
//! ProjectIdentity is a bounded, hot-path-safe orientation record. It composes
//! filesystem/project signals; it does not select work or mutate cognitive state.

use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Deserialize, Default)]
pub struct ProjectIdentityQuery {
    pub cwd: Option<String>,
    pub project_root: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectVerifyRequest {
    pub cwd: Option<String>,
    pub project_root: Option<String>,
    pub project_id: Option<String>,
    pub canonical_name: Option<String>,
    pub repo_remote: Option<String>,
}

#[derive(Debug, Clone)]
struct ProjectSignal {
    source: &'static str,
    root: Option<String>,
    confidence: &'static str,
    independent: bool,
    details: Value,
}

#[derive(Debug, Clone)]
struct IdentityCandidate {
    project_id: String,
    canonical_name: String,
    project_root: String,
    repo_remote: Option<String>,
    beads_prefix: Option<String>,
    workspace_kind: Option<String>,
    project_urls: Value,
    deployment: Value,
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

fn unsafe_project_root_reason(value: &str) -> Option<&'static str> {
    let root = value.trim().trim_end_matches('/');
    if root.is_empty() {
        return Some("missing_project_root");
    }
    match root {
        "/" | "/root" | "/home" | "/tmp" | "/var" | "/usr" | "/opt" => {
            Some("unsafe_broad_project_root")
        }
        _ if root
            .strip_prefix("/home/")
            .is_some_and(|rest| !rest.contains('/')) =>
        {
            Some("unsafe_user_home_project_root")
        }
        _ => None,
    }
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

fn marker_string(marker: &Option<Value>, key: &str) -> Option<String> {
    marker
        .as_ref()
        .and_then(|value| value.get(key))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
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
    for base in [
        Some(root.to_path_buf()),
        root.parent().map(Path::to_path_buf),
    ]
    .into_iter()
    .flatten()
    {
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
            add_if_file(&mut out, base.join(rel));
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
        for url in extract_urls(&bounded) {
            urls.push((url, rel.clone()));
            sources.insert(rel.clone());
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
    } else if local_url.is_some() {
        Some("local".to_string())
    } else {
        None
    };

    let environment_confidence =
        if wp_url.is_some() && (app_url.is_some() || deploy_locations.first().is_some()) {
            "high"
        } else if live_url.is_some() || deploy_locations.first().is_some() {
            "medium"
        } else if local_url.is_some() {
            "low"
        } else {
            "unknown"
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

fn discover_identity(cwd: Option<&str>, project_root: Option<&str>) -> IdentityCandidate {
    let start = resolve_start(cwd, project_root);
    let start_root = normalize_path(&start);
    let now = chrono::Utc::now().to_rfc3339();
    let mut signals = Vec::<ProjectSignal>::new();

    signals.push(ProjectSignal {
        source: "cwd",
        root: Some(start_root.clone()),
        confidence: "low",
        independent: false,
        details: json!({"path": start_root}),
    });

    if let Some(operator_root) = clean(project_root) {
        let root = normalize_path(&expand_home(&operator_root));
        signals.push(ProjectSignal {
            source: "operator_supplied_scope",
            root: Some(root.clone()),
            confidence: "medium",
            independent: true,
            details: json!({"project_root": root}),
        });
    }

    let marker_root = find_upwards(&start, ".focusa-project.json");
    let marker = marker_root.as_ref().and_then(|root| read_marker(root));
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

    let git_root = find_upwards(&start, ".git");
    let repo_remote = git_root.as_ref().and_then(|root| read_git_remote(root));
    if let Some(root) = &git_root {
        signals.push(ProjectSignal {
            source: "git_root",
            root: Some(normalize_path(root)),
            confidence: "high",
            independent: true,
            details: json!({"repo_remote": repo_remote}),
        });
    }

    let beads_root = find_upwards(&start, ".beads");
    if let Some(root) = &beads_root {
        signals.push(ProjectSignal {
            source: "beads_root",
            root: Some(normalize_path(root)),
            confidence: "high",
            independent: true,
            details: json!({"beads_dir": root.join(".beads").to_string_lossy()}),
        });
    }

    let workspace_root = ["Cargo.toml", "package.json", "go.mod", "pyproject.toml"]
        .iter()
        .find_map(|name| find_upwards(&start, name));
    let workspace_kind = workspace_root
        .as_ref()
        .and_then(|root| workspace_kind(root));
    if let Some(root) = &workspace_root {
        signals.push(ProjectSignal {
            source: "workspace_file",
            root: Some(normalize_path(root)),
            confidence: "medium",
            independent: true,
            details: json!({"workspace_kind": workspace_kind}),
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

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for signal in signals.iter().filter(|signal| signal.independent) {
        if let Some(root) = &signal.root {
            *counts.entry(root.clone()).or_default() += 1;
        }
    }
    let canonical_root = counts
        .iter()
        .max_by_key(|(_, count)| **count)
        .map(|(root, _)| root.clone())
        .or_else(|| signals.iter().find_map(|signal| signal.root.clone()))
        .unwrap_or_else(|| start_root.clone());

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

    let matching_independent = signals
        .iter()
        .filter(|signal| {
            signal.independent && signal.root.as_deref() == Some(canonical_root.as_str())
        })
        .count();
    let confidence = if unsafe_reason.is_some() {
        "low"
    } else if mismatches.is_empty() && matching_independent >= 2 {
        "high"
    } else if mismatches.is_empty() && matching_independent == 1 {
        "medium"
    } else {
        "low"
    };
    let status = if unsafe_reason.is_some() {
        "unsafe_project_root"
    } else if !mismatches.is_empty() {
        "mismatch"
    } else if matching_independent >= 2 {
        "verified"
    } else if matching_independent == 1 {
        "degraded"
    } else {
        "cwd_only"
    };

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

    let project_id = marker_project_id.unwrap_or_else(|| basename(&canonical_root));
    let canonical_name = marker_name.unwrap_or_else(|| project_id.clone());
    let repo_remote = marker_remote.or(repo_remote);
    let beads_prefix = marker_beads.or_else(|| Some(project_id.clone()));
    let workspace_kind = marker_workspace.or_else(|| workspace_kind.map(str::to_string));
    let identity_hints = [
        project_id.clone(),
        canonical_name.clone(),
        basename(&canonical_root),
    ];
    let (inferred_project_urls, inferred_deployment) =
        infer_project_environment(&PathBuf::from(&canonical_root), &identity_hints);
    let project_urls =
        merge_missing_object_fields(marker_project_urls(&marker), inferred_project_urls);
    let deployment = merge_missing_object_fields(marker_deployment(&marker), inferred_deployment);
    let fingerprint = stable_fingerprint(&[
        project_id.clone(),
        canonical_name.clone(),
        canonical_root.clone(),
        repo_remote.clone().unwrap_or_default(),
        beads_prefix.clone().unwrap_or_default(),
    ]);

    IdentityCandidate {
        project_id,
        canonical_name,
        project_root: canonical_root,
        repo_remote,
        beads_prefix,
        workspace_kind,
        project_urls,
        deployment,
        fingerprint,
        confidence,
        status,
        signals,
        mismatches,
        verified_at: now,
    }
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
            && name != candidate.canonical_name
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
    json!({
        "status": status,
        "canonical": canonical,
        "degraded": !canonical,
        "project_identity": {
            "schema": "focusa.project_identity.v1",
            "status": identity_status,
            "project_id": candidate.project_id,
            "canonical_name": candidate.canonical_name,
            "project_root": candidate.project_root,
            "repo_remote": candidate.repo_remote,
            "beads_prefix": candidate.beads_prefix,
            "workspace_kind": candidate.workspace_kind,
            "project_urls": candidate.project_urls,
            "deployment": candidate.deployment,
            "fingerprint": candidate.fingerprint,
            "confidence": candidate.confidence,
            "signals": candidate.signals.iter().map(signal_json).collect::<Vec<_>>(),
            "mismatches": mismatches,
            "verified_at": candidate.verified_at,
            "authority_boundary": "project_root_plus_fingerprint",
        },
        "verification": {
            "verified": verified,
            "quorum_rule": "high confidence requires at least two independent matching signals; cwd-only is degraded",
            "matching_independent_signals": candidate.signals.iter().filter(|signal| signal.independent && signal.root.as_deref() == Some(candidate.project_root.as_str())).count(),
            "required_recovery": if verified { Value::Null } else { json!("resolve mismatched project signals or provide explicit project_root after checking current repo") },
        },
        "next_tools": ["focusa_project_identity", "focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"],
        "details": {"tool_result_v1": {
            "ok": verified,
            "status": status,
            "canonical": canonical,
            "degraded": !canonical,
            "failure_class": if verified { Value::Null } else { json!("scope_mismatch") },
            "retry": {"safe": verified, "posture": if verified { "safe_retry" } else { "do_not_retry_unchanged" }},
            "side_effects": [],
            "evidence_refs": [],
            "next_tools": ["focusa_project_identity", "focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"]
        }}
    })
}

pub(crate) fn project_identity_payload_for_scope(
    cwd: Option<&str>,
    project_root: Option<&str>,
) -> Value {
    candidate_payload(discover_identity(cwd, project_root), None)
}

async fn identity(Query(query): Query<ProjectIdentityQuery>) -> Json<Value> {
    Json(project_identity_payload_for_scope(
        query.cwd.as_deref(),
        query.project_root.as_deref(),
    ))
}

async fn verify(
    State(_state): State<Arc<AppState>>,
    Json(body): Json<ProjectVerifyRequest>,
) -> Json<Value> {
    Json(candidate_payload(
        discover_identity(body.cwd.as_deref(), body.project_root.as_deref()),
        Some(&body),
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/project/identity", get(identity))
        .route("/v1/project/verify", post(verify))
}

#[cfg(test)]
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
        let candidate = discover_identity(root.to_str(), None);
        assert_eq!(candidate.status, "verified");
        assert_eq!(candidate.confidence, "high");
        assert!(candidate.mismatches.is_empty());
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
        let payload = project_identity_payload_for_scope(root.to_str(), None);
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
            "Local legacy URL: https://asapdigest.local\nLive app: https://app.asapdigest.com\n",
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
        let payload = project_identity_payload_for_scope(root.to_str(), None);
        assert_eq!(
            payload
                .pointer("/project_identity/project_urls/live_url")
                .and_then(Value::as_str),
            Some("https://asapdigest.com")
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
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn conflicting_marker_degrades_identity() {
        let root = temp_project("mismatch");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join(".git/config"), "").unwrap();
        fs::write(root.join(".focusa-project.json"), r#"{"schema":"focusa.project.v1","project_id":"other","project_root":"/definitely/not/this/project"}"#).unwrap();
        let candidate = discover_identity(root.to_str(), None);
        assert_eq!(candidate.status, "mismatch");
        assert!(!candidate.mismatches.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn broad_root_never_verifies_as_project_identity() {
        let candidate = discover_identity(Some("/root"), Some("/root"));
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
