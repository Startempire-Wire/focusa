//! Permission helpers for capabilities + commands (docs/25-26).

use axum::http::HeaderMap;
use serde_json::{Value, json};
use std::collections::HashSet;

const PERMISSIONS_HEADER: &str = "x-focusa-permissions";

#[derive(Debug, Clone)]
pub struct PermissionContext {
    token_enabled: bool,
    scopes: HashSet<String>,
}

impl PermissionContext {
    pub fn allows(&self, scope: &str) -> bool {
        if !self.token_enabled {
            return true;
        }
        if self.scopes.contains("admin:*") {
            return true;
        }
        if scope.ends_with(":read") && self.scopes.contains("read:*") {
            return true;
        }
        self.scopes.contains(scope)
    }

    pub fn list(&self) -> Vec<String> {
        let mut out: Vec<String> = self.scopes.iter().cloned().collect();
        out.sort();
        out
    }
}

pub fn permission_context(headers: &HeaderMap, token_enabled: bool) -> PermissionContext {
    if !token_enabled {
        return PermissionContext {
            token_enabled,
            scopes: ["read:*", "commands:submit", "admin:*"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        };
    }

    let header = headers
        .get(PERMISSIONS_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let mut scopes: HashSet<String> = header
        .split([',', ' '])
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    if scopes.is_empty() {
        scopes = [
            "state:read",
            "lineage:read",
            "references:read",
            "metrics:read",
            "intuition:read",
            "autonomy:read",
            "constitution:read",
            "gate:read",
            "cache:read",
            "events:read",
            "agents:read",
            "contribute:read",
            "export:read",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    }

    PermissionContext {
        token_enabled,
        scopes,
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
