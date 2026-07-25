/// Request-local scope context extracted from API request headers/query parameters.
///
/// API-01: Daemon app runtime request-local scope.
/// Every request carries an optional scope identifying the project root + continuity
/// that the request operates on. Scope-unaware endpoints use the default (empty) scope.
///
/// Extracted via axum's `FromRequestParts`:
/// ```rust,ignore
/// async fn handler(ScopeContext { project_root, continuity_id, .. }: ScopeContext) { ... }
/// ```
use axum::{
    extract::{FromRequestParts, Query},
    http::{HeaderMap, StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use focusa_core::scoped_state::{ScopeRef, WorkstreamKey};
use focusa_core::working_subpath::resolve_git_working_context;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fmt};

/// Canonical request scope parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeContext {
    /// The resolved project root path (e.g., `/workspace/focusa-project`).
    pub project_root: Option<String>,
    /// Active checkout/worktree root before canonical-parent normalization.
    pub active_worktree_root: Option<String>,
    /// Stable working context identity within the canonical project.
    pub working_subpath_id: Option<String>,
    /// The continuity/workstream identifier (e.g., `focusa-cont-root-...`).
    pub continuity_id: Option<String>,
    /// The Pi session identifier (e.g., `pi-3850319-...`).
    pub session_id: Option<String>,
    /// The source that provided the scope (header, query, default).
    pub source: ScopeSource,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ScopeSource {
    #[default]
    Default,
    Header,
    Query,
}

impl ScopeContext {
    /// Whether this request has an explicit scope.
    pub fn is_scoped(&self) -> bool {
        self.project_root.is_some() || self.continuity_id.is_some()
    }

    /// Short identifier for logging.
    pub fn short(&self) -> String {
        match (&self.project_root, &self.continuity_id) {
            (Some(r), _) => r.split('/').next_back().unwrap_or(r).to_string(),
            (_, Some(c)) => c.chars().take(12).collect(),
            (None, None) => "default".to_string(),
        }
    }

    /// Check exact project/workstream authority equality. Missing scope never matches.
    pub fn matches(&self, other: &ScopeContext) -> bool {
        matches!(
            (
                self.project_root.as_deref(),
                self.continuity_id.as_deref(),
                other.project_root.as_deref(),
                other.continuity_id.as_deref(),
            ),
            (Some(left_root), Some(left_continuity), Some(right_root), Some(right_continuity))
                if left_root == right_root && left_continuity == right_continuity
        )
    }

    /// Build a typed project/workstream key for request-local state.
    ///
    /// Spec104 API-01: request-local runtime state must not fall back to a
    /// daemon-global singleton. Endpoints that own scoped mutable state call
    /// this and reject unscoped requests rather than borrowing global authority.
    pub fn require_workstream_key(&self) -> Result<WorkstreamKey, String> {
        let project_root = self
            .project_root
            .as_deref()
            .ok_or_else(|| "x-scope-project-root or project_root is required".to_string())?;
        let continuity_id = self
            .continuity_id
            .as_deref()
            .ok_or_else(|| "x-scope-continuity-id or continuity_id is required".to_string())?;
        let canonical_name = project_root
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or("project");
        let fingerprint = format!(
            "sha256:{}",
            hex::encode(Sha256::digest(
                project_root.trim_end_matches('/').as_bytes()
            ))
        );
        let scope = ScopeRef::project(
            format!("project:{fingerprint}"),
            project_root,
            canonical_name,
            fingerprint,
        )
        .map_err(|error| error.to_string())?;
        WorkstreamKey::new(scope, continuity_id).map_err(|error| error.to_string())
    }
}

impl fmt::Display for ScopeContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scope({})", self.short())
    }
}

// ---- axum extractor ----

/// Extract scope context from request headers or query parameters.
///
/// Header priority: `X-Scope-Project-Root`, `X-Scope-Continuity-Id`, `X-Scope-Session-Id`.
/// Query fallback: `project_root`, `continuity_id`, `session_id`.
impl<S> FromRequestParts<S> for ScopeContext
where
    S: Send + Sync,
{
    type Rejection = ScopeRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Query(query) = Query::<HashMap<String, String>>::try_from_uri(&parts.uri)
            .map_err(|error| ScopeRejection::Internal(format!("invalid scope query: {error}")))?;

        // Try headers first
        let requested_project_root = header_or_query(
            &parts.headers,
            &query,
            "x-scope-project-root",
            "project_root",
        );
        let working_context = if let Some(root) = requested_project_root.clone() {
            tokio::task::spawn_blocking(move || {
                resolve_git_working_context(std::path::Path::new(&root))
                    .ok()
                    .flatten()
            })
            .await
            .ok()
            .flatten()
        } else {
            None
        };
        let project_root = working_context
            .as_ref()
            .map(|context| context.canonical_parent_root.clone())
            .or(requested_project_root.clone());
        let continuity_id = header_or_query(
            &parts.headers,
            &query,
            "x-scope-continuity-id",
            "continuity_id",
        );
        let session_id =
            header_or_query(&parts.headers, &query, "x-scope-session-id", "session_id");

        let source = if parts.headers.contains_key("x-scope-project-root") {
            ScopeSource::Header
        } else if project_root.is_some() || continuity_id.is_some() {
            ScopeSource::Query
        } else {
            ScopeSource::Default
        };

        Ok(ScopeContext {
            project_root: project_root.filter(|s| !s.is_empty()),
            active_worktree_root: working_context
                .as_ref()
                .map(|context| context.active_worktree_root.clone())
                .or(requested_project_root.filter(|s| !s.is_empty())),
            working_subpath_id: working_context
                .as_ref()
                .map(|context| context.working_subpath.working_subpath_id.clone()),
            continuity_id: continuity_id.filter(|s| !s.is_empty()),
            session_id: session_id.filter(|s| !s.is_empty()),
            source,
        })
    }
}

fn header_or_query(
    headers: &HeaderMap,
    query: &HashMap<String, String>,
    header_name: &str,
    query_key: &str,
) -> Option<String> {
    if let Some(val) = headers.get(header_name) {
        if let Ok(s) = val.to_str() {
            return Some(s.to_string());
        }
    }
    query.get(query_key).cloned()
}

#[derive(Debug)]
pub enum ScopeRejection {
    Internal(String),
}

impl IntoResponse for ScopeRejection {
    fn into_response(self) -> Response {
        let ScopeRejection::Internal(body) = self;
        (StatusCode::BAD_REQUEST, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_scope_values_are_form_decoded() {
        let uri = "/v1/work-loop/status?project_root=%2FVolumes%2FMacintosh+HD%2Ffocusa&continuity_id=workloop-completion"
            .parse()
            .expect("valid URI");
        let Query(query) =
            Query::<HashMap<String, String>>::try_from_uri(&uri).expect("valid form query");
        let headers = HeaderMap::new();
        assert_eq!(
            header_or_query(&headers, &query, "x-scope-project-root", "project_root").as_deref(),
            Some("/Volumes/Macintosh HD/focusa")
        );
        assert_eq!(
            header_or_query(&headers, &query, "x-scope-continuity-id", "continuity_id").as_deref(),
            Some("workloop-completion")
        );
    }

    #[test]
    fn missing_or_cross_workstream_scope_never_matches() {
        let empty = ScopeContext::default();
        let left = ScopeContext {
            project_root: Some("/workspace/a".into()),
            continuity_id: Some("cont-a".into()),
            ..Default::default()
        };
        let other_continuity = ScopeContext {
            project_root: Some("/workspace/a".into()),
            continuity_id: Some("cont-b".into()),
            ..Default::default()
        };
        assert!(!empty.matches(&left));
        assert!(!left.matches(&empty));
        assert!(!left.matches(&other_continuity));
        assert!(left.matches(&left));
    }
}
