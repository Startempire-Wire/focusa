use crate::daemon_multiplex::{DaemonRegistryError, DaemonRegistryProjection, ProjectRouteKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonRequestIdentity {
    pub request_id: String,
    pub daemon_id: String,
    pub controller_id: String,
    pub presented_auth_fingerprint: String,
    pub presented_endpoint: String,
    pub native_session_id: String,
    pub route: ProjectRouteKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonAuthorizationReceipt {
    pub schema: String,
    pub request_id: String,
    pub daemon_id: String,
    pub controller_id: String,
    pub project_root: String,
    pub continuity_id: String,
    pub working_subpath_id: String,
    pub native_session_id: String,
    pub auth_fingerprint: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DaemonAuthorizationError {
    #[error("daemon request identity is missing: {0}")]
    MissingIdentity(&'static str),
    #[error("daemon route authorization failed: {0}")]
    Route(#[from] DaemonRegistryError),
    #[error("request targeted a daemon that does not own the exact route")]
    DaemonMismatch,
    #[error("controller identity does not match daemon enrollment")]
    ControllerMismatch,
    #[error("presented authentication fingerprint does not match enrollment")]
    AuthenticationMismatch,
    #[error("request endpoint does not match daemon enrollment")]
    EndpointMismatch,
    #[error("native session is not admitted by daemon enrollment")]
    SessionMismatch,
    #[error("request id has already been authorized")]
    ReplayDetected,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonReplayGuard {
    authorized_request_ids: BTreeSet<String>,
}

impl DaemonReplayGuard {
    pub fn authorize(
        &mut self,
        registry: &DaemonRegistryProjection,
        request: &DaemonRequestIdentity,
    ) -> Result<DaemonAuthorizationReceipt, DaemonAuthorizationError> {
        for (value, field) in [
            (&request.request_id, "request_id"),
            (&request.daemon_id, "daemon_id"),
            (&request.controller_id, "controller_id"),
            (
                &request.presented_auth_fingerprint,
                "presented_auth_fingerprint",
            ),
            (&request.presented_endpoint, "presented_endpoint"),
            (&request.native_session_id, "native_session_id"),
        ] {
            if value.trim().is_empty() {
                return Err(DaemonAuthorizationError::MissingIdentity(field));
            }
        }
        if self.authorized_request_ids.contains(&request.request_id) {
            return Err(DaemonAuthorizationError::ReplayDetected);
        }
        let registration = registry.resolve(&request.route)?;
        if registration.daemon_id != request.daemon_id {
            return Err(DaemonAuthorizationError::DaemonMismatch);
        }
        if registration.controller_id != request.controller_id {
            return Err(DaemonAuthorizationError::ControllerMismatch);
        }
        if registration.auth_fingerprint != request.presented_auth_fingerprint {
            return Err(DaemonAuthorizationError::AuthenticationMismatch);
        }
        if registration.endpoint != request.presented_endpoint {
            return Err(DaemonAuthorizationError::EndpointMismatch);
        }
        if !registration
            .allowed_native_sessions
            .contains(&request.native_session_id)
        {
            return Err(DaemonAuthorizationError::SessionMismatch);
        }
        self.authorized_request_ids
            .insert(request.request_id.clone());
        Ok(DaemonAuthorizationReceipt {
            schema: "focusa.daemon_request_authorization.v1".into(),
            request_id: request.request_id.clone(),
            daemon_id: request.daemon_id.clone(),
            controller_id: request.controller_id.clone(),
            project_root: request.route.project_root.clone(),
            continuity_id: request.route.continuity_id.clone(),
            working_subpath_id: request.route.working_subpath_id.clone(),
            native_session_id: request.native_session_id.clone(),
            auth_fingerprint: request.presented_auth_fingerprint.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daemon_multiplex::{
        DaemonHealth, DaemonRegistration, DaemonRegistryEvent, reduce_daemon_registry,
    };

    fn route() -> ProjectRouteKey {
        ProjectRouteKey {
            project_root: "/srv/focusa".into(),
            continuity_id: "continuity".into(),
            working_subpath_id: "working-subpath:main".into(),
        }
    }

    fn registry() -> DaemonRegistryProjection {
        reduce_daemon_registry([
            DaemonRegistryEvent::Enrolled {
                registration: DaemonRegistration {
                    daemon_id: "daemon-1".into(),
                    controller_id: "controller-1".into(),
                    endpoint: "https://daemon.example.test".into(),
                    auth_fingerprint: "sha256:peer-cert".into(),
                    version: "0.9.143".into(),
                    capabilities: BTreeSet::from(["workpoint".into()]),
                    allowed_native_sessions: BTreeSet::from(["session-1".into()]),
                    health: DaemonHealth::Healthy,
                    generation: 1,
                },
            },
            DaemonRegistryEvent::ScopeAssigned {
                daemon_id: "daemon-1".into(),
                generation: 1,
                route: route(),
            },
        ])
    }

    fn request() -> DaemonRequestIdentity {
        DaemonRequestIdentity {
            request_id: "request-1".into(),
            daemon_id: "daemon-1".into(),
            controller_id: "controller-1".into(),
            presented_auth_fingerprint: "sha256:peer-cert".into(),
            presented_endpoint: "https://daemon.example.test".into(),
            native_session_id: "session-1".into(),
            route: route(),
        }
    }

    #[test]
    fn exact_authenticated_request_is_authorized_once() {
        let mut guard = DaemonReplayGuard::default();
        let receipt = guard.authorize(&registry(), &request()).unwrap();
        assert_eq!(receipt.daemon_id, "daemon-1");
        assert_eq!(
            guard.authorize(&registry(), &request()),
            Err(DaemonAuthorizationError::ReplayDetected)
        );
    }

    #[test]
    fn foreign_controller_peer_and_route_fail_closed() {
        let mut foreign = request();
        foreign.controller_id = "controller-foreign".into();
        assert_eq!(
            DaemonReplayGuard::default().authorize(&registry(), &foreign),
            Err(DaemonAuthorizationError::ControllerMismatch)
        );
        foreign = request();
        foreign.presented_auth_fingerprint = "sha256:foreign".into();
        assert_eq!(
            DaemonReplayGuard::default().authorize(&registry(), &foreign),
            Err(DaemonAuthorizationError::AuthenticationMismatch)
        );
        foreign = request();
        foreign.presented_endpoint = "https://foreign.example.test".into();
        assert_eq!(
            DaemonReplayGuard::default().authorize(&registry(), &foreign),
            Err(DaemonAuthorizationError::EndpointMismatch)
        );
        foreign = request();
        foreign.native_session_id = "session-foreign".into();
        assert_eq!(
            DaemonReplayGuard::default().authorize(&registry(), &foreign),
            Err(DaemonAuthorizationError::SessionMismatch)
        );
        foreign = request();
        foreign.route.continuity_id = "other".into();
        assert!(matches!(
            DaemonReplayGuard::default().authorize(&registry(), &foreign),
            Err(DaemonAuthorizationError::Route(
                DaemonRegistryError::NoRoute
            ))
        ));
        for error in [
            DaemonAuthorizationError::AuthenticationMismatch,
            DaemonAuthorizationError::EndpointMismatch,
            DaemonAuthorizationError::SessionMismatch,
        ] {
            let rendered = error.to_string();
            assert!(!rendered.contains("peer-cert"));
            assert!(!rendered.contains("session-1"));
            assert!(!rendered.contains("daemon.example"));
        }
    }

    #[test]
    fn replay_guard_survives_restart_without_persisting_raw_bearer_material() {
        let mut guard = DaemonReplayGuard::default();
        guard.authorize(&registry(), &request()).unwrap();
        let bytes = serde_json::to_vec(&guard).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("peer-cert"));
        let mut restarted: DaemonReplayGuard = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            restarted.authorize(&registry(), &request()),
            Err(DaemonAuthorizationError::ReplayDetected)
        );
    }
}
