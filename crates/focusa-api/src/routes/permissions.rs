//! Permission helpers for capabilities + commands (docs/25-26).

use axum::http::HeaderMap;
use serde_json::{Value, json};
use std::collections::HashSet;

const PERMISSIONS_HEADER: &str = "x-focusa-permissions";

#[derive(Debug, Clone)]
pub struct PermissionContext {
    requested_scopes: HashSet<String>,
}

impl PermissionContext {
    /// Compatibility shim for route-local checks. The canonical route-scope
    /// middleware has already authorized and durably audited the request before
    /// any handler runs, so this shim may not grant or independently deny.
    pub fn allows(&self, _scope: &str) -> bool {
        true
    }

    pub fn list(&self) -> Vec<String> {
        let mut out: Vec<String> = self.requested_scopes.iter().cloned().collect();
        out.sort();
        out
    }
}

/// Non-authoritative legacy scope request metadata; never a grant.
pub fn requested_scopes(headers: &HeaderMap) -> std::collections::BTreeSet<String> {
    headers
        .get(PERMISSIONS_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .split([',', ' '])
        .filter(|scope| !scope.trim().is_empty())
        .map(|scope| scope.trim().to_string())
        .collect()
}

pub fn permission_context(headers: &HeaderMap, _token_enabled: bool) -> PermissionContext {
    PermissionContext {
        requested_scopes: requested_scopes(headers).into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn requested_scopes_are_explicit_metadata_without_default_grants() {
        assert!(requested_scopes(&HeaderMap::new()).is_empty());
        let mut headers = HeaderMap::new();
        headers.insert(
            PERMISSIONS_HEADER,
            HeaderValue::from_static("state:read, admin:*"),
        );
        assert_eq!(
            requested_scopes(&headers),
            ["admin:*".to_string(), "state:read".to_string()]
                .into_iter()
                .collect()
        );
    }
}

pub fn forbid(scope: &str) -> (axum::http::StatusCode, axum::Json<Value>) {
    (
        axum::http::StatusCode::FORBIDDEN,
        axum::Json(json!({
            "error": "permission denied",
            "required": scope,
        })),
    )
}
