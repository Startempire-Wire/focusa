//! API client for TUI — fetches Focusa state via REST.
//!
//! Spec104 TUI-01: typed scope display. All requests flow through the
//! typed `TypedScope` envelope which preserves project_root + continuity_id
//! + session_id. The TUI does not mutate canonical scope state.

use crate::app::*;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Typed scope envelope (Spec104 TUI-01 / WL-02).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypedScope {
    pub project_root: String,
    pub continuity_id: String,
    pub session_id: Option<String>,
    pub scope_status: Option<String>,
    pub scope_source: Option<String>,
    pub canonical_scope: Option<bool>,
}

impl TypedScope {
    /// Render for TUI display: `[status] project_root (continuity_id)`.
    pub fn display(&self) -> String {
        let status = self.scope_status.as_deref().unwrap_or("unknown");
        if self.canonical_scope == Some(false) {
            return format!(
                "[advisory] {} ({}) [{}]",
                self.project_root, self.continuity_id, status
            );
        }
        format!(
            "{} ({}) [{}]",
            self.project_root, self.continuity_id, status
        )
    }

    /// Encode as query-string suffix for HTTP requests.
    pub fn as_query_suffix(&self) -> String {
        let mut params = url_query_stringify::UrlQueryParams::new();
        if !self.project_root.is_empty() {
            params.push("project_root", &self.project_root);
        }
        if !self.continuity_id.is_empty() {
            params.push("continuity_id", &self.continuity_id);
        }
        params.finish()
    }
}

mod url_query_stringify {
    pub struct UrlQueryParams(String);
    impl UrlQueryParams {
        pub fn new() -> Self {
            Self(String::new())
        }
        pub fn push(&mut self, k: &str, v: &str) {
            if !self.0.is_empty() {
                self.0.push('&');
            }
            self.0.push_str(&format!("{}={}", k, urlencoded(k, v)));
        }
        pub fn finish(self) -> String {
            if self.0.is_empty() {
                String::new()
            } else {
                format!("?{}", self.0)
            }
        }
    }
    fn urlencoded(_k: &str, v: &str) -> String {
        // Minimal URL-encoding for scope params: only handle & = ? in values.
        v.replace('&', "%26")
            .replace('=', "%3D")
            .replace('?', "%3F")
            .replace(' ', "%20")
    }
}

pub struct ApiClient {
    base_url: String,
    client: Client,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Fetch arbitrary JSON from an endpoint.
    pub async fn fetch_json(&self, path: &str) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client.get(&url).send().await?.json().await?;
        Ok(resp)
    }

    /// Spec104 TUI-01: Fetch with typed ScopeContext. The scope query
    /// suffix preserves project_root + continuity_id + session_id end-to-end.
    pub async fn fetch_with_scope(
        &self,
        path: &str,
        scope: &TypedScope,
    ) -> Result<serde_json::Value> {
        let suffix = scope.as_query_suffix();
        let url = format!("{}{}{}", self.base_url, path, suffix);
        let resp = self
            .client
            .get(&url)
            .header("x-scope-project-root", &scope.project_root)
            .header("x-scope-continuity-id", &scope.continuity_id)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    /// Spec104 TUI-01: Fetch raw state with typed scope.
    pub async fn fetch_state_with_scope(&self, scope: &TypedScope) -> Result<StateSnapshot> {
        let resp = self.fetch_with_scope("/v1/state/dump", scope).await?;
        Ok(serde_json::from_value(resp)?)
    }

    /// Fetch full state snapshot from the daemon.
    pub async fn fetch_state(&self) -> Result<StateSnapshot> {
        let url = format!("{}/v1/state/dump", self.base_url);
        let resp: Value = self.client.get(&url).send().await?.json().await?;

        // Parse the full state into our snapshot model.
        let session = resp.get("session").and_then(|s| {
            Some(SessionInfo {
                session_id: s.get("session_id")?.as_str()?.to_string(),
            })
        });

        let focus_stack = parse_stack(&resp);
        let focus_state = parse_focus_state(&resp);
        let candidates = parse_candidates(&resp);
        let version = resp.get("version").and_then(|v| v.as_u64()).unwrap_or(0);

        // Fetch recent events separately.
        let events = self.fetch_events().await.unwrap_or_default();

        Ok(StateSnapshot {
            session,
            focus_stack,
            focus_state,
            candidates,
            events,
            version,
        })
    }

    async fn fetch_events(&self) -> Result<Vec<EventInfo>> {
        let url = format!("{}/v1/events/recent?limit=50", self.base_url);
        let resp: Value = self.client.get(&url).send().await?.json().await?;

        let entries = resp.get("events").and_then(|e| e.as_array());
        let mut events = Vec::new();
        if let Some(arr) = entries {
            for entry in arr {
                events.push(EventInfo {
                    timestamp: entry
                        .get("timestamp")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    event_type: entry
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    event_id: entry
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .chars()
                        .take(8)
                        .collect(),
                });
            }
        }
        Ok(events)
    }
}

fn parse_stack(resp: &Value) -> StackInfo {
    let stack = match resp.get("focus_stack") {
        Some(s) => s,
        None => return StackInfo::default(),
    };

    let active_id = stack
        .get("active_id")
        .and_then(|v| v.as_str())
        .map(String::from);

    let frames = stack
        .get("frames")
        .and_then(|f| f.as_array())
        .map(|arr| {
            arr.iter()
                .map(|f| FrameInfo {
                    frame_id: f
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    beads_id: f
                        .get("beads_issue_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    intent: f
                        .get("focus_state")
                        .and_then(|fs| fs.get("intent"))
                        .and_then(|v| v.as_str())
                        .or_else(|| f.get("title").and_then(|v| v.as_str()))
                        .unwrap_or("")
                        .to_string(),
                    status: f
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    depth: 0, // Computed from stack_path_cache position, not stored.
                })
                .collect()
        })
        .unwrap_or_default();

    let stack_path = stack
        .get("stack_path_cache")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    StackInfo {
        active_id,
        frames,
        stack_path,
    }
}

fn parse_focus_state(resp: &Value) -> Option<FocusStateInfo> {
    // Focus state lives inside the active frame's focus_state field.
    let stack = resp.get("focus_stack")?;
    let active_id = stack.get("active_id")?.as_str()?;
    let frames = stack.get("frames")?.as_array()?;
    let active = frames
        .iter()
        .find(|f| f.get("id").and_then(|v| v.as_str()) == Some(active_id))?;

    let fs = active.get("focus_state")?;

    Some(FocusStateInfo {
        intent: fs
            .get("intent")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        constraints: parse_string_array(fs.get("constraints")),
        decisions: parse_string_array(fs.get("decisions")),
        next_steps: parse_string_array(fs.get("next_steps")),
        current_state: fs
            .get("current_state")
            .and_then(|v| v.as_str())
            .map(String::from),
    })
}

fn parse_candidates(resp: &Value) -> Vec<CandidateInfo> {
    let gate = match resp.get("focus_gate") {
        Some(g) => g,
        None => return Vec::new(),
    };

    gate.get("candidates")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .map(|c| CandidateInfo {
                    id: c
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    kind: c
                        .get("kind")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    label: c
                        .get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    pressure: c.get("pressure").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    pinned: c.get("pinned").and_then(|v| v.as_bool()).unwrap_or(false),
                    status: c
                        .get("state")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_string_array(val: Option<&Value>) -> Vec<String> {
    val.and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}
