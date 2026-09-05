//! Permission helpers for capabilities + commands (docs/25-26).

use axum::http::HeaderMap;
use serde_json::{Value, json};
use std::collections::HashSet;

const PERMISSIONS_HEADER: &str = "x-focusa-permissions";

// Legacy pairing `write` is bounded to these ordinary capabilities, never
// administrative scopes or future capabilities implicitly. Exact stored scopes
// retain their original meaning.
const DEVICE_WRITE_SCOPES: &[&str] = &[
    "state:write",
    "workpoint:write",
    "trajectory:write",
    "metacog:write",
    "prediction:write",
    "work_loop:control",
    "silent_sessions:create",
    "silent_sessions:control",
    "silent_sessions:config",
    "telemetry:write",
    "attachments:write",
    "commands:submit",
];

#[derive(Debug, Clone)]
pub struct PermissionContext {
    token_enabled: bool,
    scopes: HashSet<String>,
}

impl PermissionContext {
    /// Only call with scopes obtained from a verified token record.
    pub(crate) fn from_device_grants(grants: &[String]) -> Self {
        let mut scopes = HashSet::new();
        for grant in grants {
            match grant.as_str() {
                "read" => {
                    scopes.insert("read:*".to_string());
                    scopes.insert("silent_sessions:stream".to_string());
                }
                "write" => scopes.extend(DEVICE_WRITE_SCOPES.iter().map(|s| s.to_string())),
                _ => {
                    scopes.insert(grant.clone());
                }
            }
        }
        Self {
            token_enabled: true,
            scopes,
        }
    }

    /// Bind the existing downstream permission surface to verified grants.
    /// Explicit over-requests fail; absent headers retain only granted legacy
    /// defaults. An explicit empty header represents no permissions.
    pub(crate) fn bind_device_request(
        &self,
        headers: &mut HeaderMap,
    ) -> Result<(), axum::http::StatusCode> {
        use axum::http::{HeaderValue, StatusCode};
        if headers.get_all(PERMISSIONS_HEADER).iter().count() > 1
            || headers
                .get(PERMISSIONS_HEADER)
                .is_some_and(|v| v.to_str().is_err())
        {
            return Err(StatusCode::FORBIDDEN);
        }
        let explicit = headers.contains_key(PERMISSIONS_HEADER);
        let requested = permission_context(headers, true);
        let mut effective = Vec::new();
        for scope in requested.list() {
            if self.allows(&scope) {
                effective.push(scope);
            } else if explicit {
                return Err(StatusCode::FORBIDDEN);
            }
        }
        let value =
            HeaderValue::from_str(&effective.join(",")).map_err(|_| StatusCode::FORBIDDEN)?;
        headers.insert(PERMISSIONS_HEADER, value);
        Ok(())
    }

    pub fn allows(&self, scope: &str) -> bool {
        if !self.token_enabled {
            return true;
        }
        if self.scopes.contains("admin:*") {
            return true;
        }
        if scope.starts_with("public:") {
            // Pre-auth public routes (health, pairing) are always allowed.
            // The auth_layer decides whether to require an admin token for
            // public:pairing; route_scope_layer only enforces scope, not
            // identity.
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

    if !headers.contains_key(PERMISSIONS_HEADER) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderValue, StatusCode};

    #[test]
    fn device_grants_bound_requests_and_never_supply_missing_grants() {
        for (grants, requested, allowed) in [
            (vec!["read"], "state:read", true),
            (vec!["read"], "read:*", true),
            (vec!["read"], "silent_sessions:stream", true),
            (vec!["read"], "state:write", false),
            (vec!["read"], "state:read,admin:*", false),
            (vec!["write"], "state:write", true),
            (vec!["write"], "state:read", false),
            (vec!["write"], "admin:*", false),
            (vec!["write"], "sync:admin", false),
            (vec!["write"], "future:write", false),
            (vec!["read", "write"], "state:read,state:write", true),
            (vec!["state:read"], "read:*", false),
            (vec!["custom:read"], "custom:read", true),
            (vec![], "state:read", false),
        ] {
            let grants = grants.into_iter().map(str::to_string).collect::<Vec<_>>();
            let verified = PermissionContext::from_device_grants(&grants);
            let mut headers = HeaderMap::new();
            headers.insert(
                PERMISSIONS_HEADER,
                HeaderValue::from_str(requested).unwrap(),
            );
            let result = verified.bind_device_request(&mut headers);
            assert_eq!(result.is_ok(), allowed, "{grants:?}: {requested}");
            if result.is_ok() {
                for scope in permission_context(&headers, true).list() {
                    assert!(verified.allows(&scope));
                }
            }
        }
    }

    #[test]
    fn absent_empty_and_invalid_permission_headers_remain_distinct() {
        let read = PermissionContext::from_device_grants(&["read".to_string()]);
        let mut headers = HeaderMap::new();
        read.bind_device_request(&mut headers).unwrap();
        assert!(permission_context(&headers, true).allows("state:read"));
        assert!(!permission_context(&headers, true).allows("admin:service"));

        headers.insert(PERMISSIONS_HEADER, HeaderValue::from_static(""));
        read.bind_device_request(&mut headers).unwrap();
        assert!(permission_context(&headers, true).list().is_empty());

        let mut absent = HeaderMap::new();
        PermissionContext::from_device_grants(&[])
            .bind_device_request(&mut absent)
            .unwrap();
        assert!(permission_context(&absent, true).list().is_empty());

        headers.insert(PERMISSIONS_HEADER, HeaderValue::from_static("state:read"));
        headers.append(PERMISSIONS_HEADER, HeaderValue::from_static("admin:*"));
        assert_eq!(
            read.bind_device_request(&mut headers),
            Err(StatusCode::FORBIDDEN)
        );
        headers.remove(PERMISSIONS_HEADER);
        headers.insert(
            PERMISSIONS_HEADER,
            HeaderValue::from_bytes(&[0xff]).unwrap(),
        );
        assert_eq!(
            read.bind_device_request(&mut headers),
            Err(StatusCode::FORBIDDEN)
        );
    }
}
