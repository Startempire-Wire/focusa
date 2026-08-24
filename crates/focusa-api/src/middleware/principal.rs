use std::collections::BTreeSet;

use axum::http::HeaderMap;
use focusa_core::{
    capability_authorization::CapabilityPrincipal,
    silent_sessions::{AuthenticatedPrincipal, SilentSessionRole, SilentSessionRouteScope},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiPrincipalSource {
    LocalLoopback,
    AdminToken,
    PairedDevice,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiRequestPrincipal {
    pub principal: AuthenticatedPrincipal,
    pub source: ApiPrincipalSource,
    /// Daemon-resolved capability grants. Request headers never populate this set.
    pub capability_grants: BTreeSet<String>,
}
impl ApiRequestPrincipal {
    pub fn canonical_capability_principal(&self) -> CapabilityPrincipal {
        let source = match self.source {
            ApiPrincipalSource::LocalLoopback => "local_loopback",
            ApiPrincipalSource::AdminToken => "admin_token",
            ApiPrincipalSource::PairedDevice => "paired_device",
        };
        CapabilityPrincipal {
            principal_id: self.principal.principal_id.clone(),
            source: source.into(),
            authenticated: self.principal.authenticated,
            grants: self.capability_grants.clone(),
            workstream_keys: BTreeSet::new(),
        }
    }
}

pub async fn request_principal(headers: &HeaderMap) -> Option<ApiRequestPrincipal> {
    let daemon_user = std::env::var("USER").unwrap_or_else(|_| "unknown".into());
    let authorization = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let bearer = authorization.strip_prefix("Bearer ").unwrap_or_default();
    let admin_token = std::env::var("FOCUSA_AUTH_TOKEN")
        .ok()
        .filter(|token| !token.is_empty());
    if bearer.is_empty() {
        return admin_token.is_none().then(|| ApiRequestPrincipal {
            principal: principal(
                "principal:local-loopback",
                "actor:local-daemon",
                SilentSessionRole::Administrator,
                daemon_user,
                all_scopes(),
            ),
            source: ApiPrincipalSource::LocalLoopback,
            capability_grants: ["admin:*".into(), "risk:high".into()].into_iter().collect(),
        });
    }
    if admin_token
        .as_deref()
        .is_some_and(|expected| constant_time_eq(expected.as_bytes(), bearer.as_bytes()))
    {
        return Some(ApiRequestPrincipal {
            principal: principal(
                "principal:admin-token",
                "actor:admin-token",
                SilentSessionRole::Administrator,
                daemon_user,
                all_scopes(),
            ),
            source: ApiPrincipalSource::AdminToken,
            capability_grants: ["admin:*".into(), "risk:high".into()].into_iter().collect(),
        });
    }
    if let Some(device) = paired_device(bearer).await {
        let capability_grants = capability_grants_for_device(&device.scopes);
        return Some(ApiRequestPrincipal {
            principal: principal(
                format!("principal:device:{}", device.device_id),
                format!("actor:device:{}", device.device_id),
                SilentSessionRole::Operator,
                daemon_user,
                expand_device_scopes(&device.scopes),
            ),
            source: ApiPrincipalSource::PairedDevice,
            capability_grants,
        });
    }
    None
}

async fn paired_device(token: &str) -> Option<focusa_core::types::DeviceToken> {
    {
        let state = crate::routes::device_pairing::shared_state();
        let guard = state.read().await;
        if let Some(device) = guard.tokens.get(token) {
            return Some(device.clone());
        }
    }
    let state = crate::server::app_state_for_token_lookup()?;
    let stored = state.persistence.load_device_token_full(token).ok()??;
    Some(focusa_core::types::DeviceToken {
        token: String::new(),
        device_id: stored.device_id,
        scopes: stored.scopes,
        issued_at: stored.issued_at,
        expires_at: stored.expires_at,
        last_used_at: None,
        issued_to: stored.issued_to,
    })
}

fn principal(
    principal_id: impl Into<String>,
    actor: impl Into<String>,
    role: SilentSessionRole,
    os_user: String,
    scopes: BTreeSet<SilentSessionRouteScope>,
) -> AuthenticatedPrincipal {
    AuthenticatedPrincipal {
        principal_id: principal_id.into(),
        actor: actor.into(),
        role,
        os_user,
        scopes,
        authenticated: true,
    }
}

fn all_scopes() -> BTreeSet<SilentSessionRouteScope> {
    [
        SilentSessionRouteScope::Read,
        SilentSessionRouteScope::Stream,
        SilentSessionRouteScope::Create,
        SilentSessionRouteScope::Control,
        SilentSessionRouteScope::Config,
        SilentSessionRouteScope::Admin,
        SilentSessionRouteScope::Forensics,
    ]
    .into_iter()
    .collect()
}

pub fn capability_grants_for_device(scopes: &[String]) -> BTreeSet<String> {
    let mut grants = BTreeSet::new();
    for scope in scopes {
        match scope.as_str() {
            "read" | "read:*" => {
                grants.insert("read:*".into());
            }
            "write" | "write:*" => {
                grants.insert("write:*".into());
            }
            // Historic device records carrying admin remain deliberately bounded:
            // only daemon admin tokens/local loopback may activate admin:*.
            "admin" | "admin:*" => {}
            exact if exact.contains(':') => {
                grants.insert(exact.into());
            }
            _ => {}
        }
    }
    grants
}

fn expand_device_scopes(scopes: &[String]) -> BTreeSet<SilentSessionRouteScope> {
    let mut expanded = BTreeSet::new();
    for scope in scopes {
        match scope.as_str() {
            "read" | "read:*" => {
                expanded.insert(SilentSessionRouteScope::Read);
                expanded.insert(SilentSessionRouteScope::Stream);
            }
            "write" | "write:*" => {
                expanded.insert(SilentSessionRouteScope::Create);
                expanded.insert(SilentSessionRouteScope::Control);
                expanded.insert(SilentSessionRouteScope::Config);
            }
            "admin" | "admin:*" => {
                expanded.extend(all_scopes());
            }
            exact => {
                for candidate in all_scopes() {
                    if candidate.as_str() == exact {
                        expanded.insert(candidate);
                    }
                }
            }
        }
    }
    expanded
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0_u8;
    for (left, right) in left.iter().zip(right) {
        difference |= left ^ right;
    }
    difference == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_scope_aliases_expand_to_exact_spec133_scopes() {
        let scopes = expand_device_scopes(&["read".into(), "write".into()]);
        for required in [
            SilentSessionRouteScope::Read,
            SilentSessionRouteScope::Stream,
            SilentSessionRouteScope::Create,
            SilentSessionRouteScope::Control,
            SilentSessionRouteScope::Config,
        ] {
            assert!(scopes.contains(&required));
        }
        assert!(!scopes.contains(&SilentSessionRouteScope::Admin));
        assert!(!scopes.contains(&SilentSessionRouteScope::Forensics));
    }

    #[test]
    fn exact_and_admin_device_scopes_remain_bounded() {
        let exact = expand_device_scopes(&["silent_sessions:stream".into()]);
        assert_eq!(
            exact,
            [SilentSessionRouteScope::Stream].into_iter().collect()
        );
        assert_eq!(expand_device_scopes(&["admin".into()]), all_scopes());
    }

    #[test]
    fn capability_grants_are_server_derived_and_legacy_admin_is_bounded() {
        assert_eq!(
            capability_grants_for_device(&["read".into(), "write".into(), "admin:*".into()]),
            ["read:*".into(), "write:*".into()].into_iter().collect()
        );
        assert_eq!(
            capability_grants_for_device(&["silent_sessions:stream".into()]),
            ["silent_sessions:stream".into()].into_iter().collect()
        );
    }

    #[test]
    fn token_comparison_is_constant_shape_and_fail_closed() {
        assert!(constant_time_eq(b"same", b"same"));
        assert!(!constant_time_eq(b"same", b"diff"));
        assert!(!constant_time_eq(b"short", b"longer"));
    }
}
