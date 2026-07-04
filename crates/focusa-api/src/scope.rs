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
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical request scope parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeContext {
    /// The resolved project root path (e.g., `/workspace/focusa-project`).
    pub project_root: Option<String>,
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

    /// Check if this scope matches another scope.
    pub fn matches(&self, other: &ScopeContext) -> bool {
        match (&self.project_root, &other.project_root) {
            (Some(a), Some(b)) => a == b,
            _ => true, // empty scopes match everything (lenient)
        }
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
        // Try headers first
        let project_root = header_or_query(&parts.headers, parts.uri.query().unwrap_or(""), "x-scope-project-root", "project_root");
        let continuity_id = header_or_query(&parts.headers, parts.uri.query().unwrap_or(""), "x-scope-continuity-id", "continuity_id");
        let session_id = header_or_query(&parts.headers, parts.uri.query().unwrap_or(""), "x-scope-session-id", "session_id");

        let source = if parts.headers.contains_key("x-scope-project-root") {
            ScopeSource::Header
        } else if project_root.is_some() || continuity_id.is_some() {
            ScopeSource::Query
        } else {
            ScopeSource::Default
        };

        Ok(ScopeContext {
            project_root: project_root.filter(|s| !s.is_empty()),
            continuity_id: continuity_id.filter(|s| !s.is_empty()),
            session_id: session_id.filter(|s| !s.is_empty()),
            source,
        })
    }
}

fn header_or_query(headers: &HeaderMap, query_str: &str, header_name: &str, query_key: &str) -> Option<String> {
    if let Some(val) = headers.get(header_name) {
        if let Ok(s) = val.to_str() {
            return Some(s.to_string());
        }
    }
    // Parse query parameters
    for pair in query_str.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            if k == query_key {
                return Some(v.to_string());  // simple ASCII-safe query values
            }
        }
    }
    None
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
