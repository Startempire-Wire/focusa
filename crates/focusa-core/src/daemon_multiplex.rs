use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProjectRouteKey {
    pub project_root: String,
    pub continuity_id: String,
    pub working_subpath_id: String,
}

impl ProjectRouteKey {
    pub fn validate(&self) -> Result<(), DaemonRegistryError> {
        if !self.project_root.starts_with('/') || self.project_root == "/" {
            return Err(DaemonRegistryError::UnsafeProjectRoot);
        }
        if self.continuity_id.trim().is_empty() {
            return Err(DaemonRegistryError::MissingIdentity("continuity_id"));
        }
        if self.working_subpath_id.trim().is_empty() {
            return Err(DaemonRegistryError::MissingIdentity("working_subpath_id"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaemonHealth {
    Healthy,
    Degraded,
    Offline,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonRegistration {
    pub daemon_id: String,
    pub controller_id: String,
    pub endpoint: String,
    pub auth_fingerprint: String,
    pub version: String,
    pub capabilities: BTreeSet<String>,
    pub allowed_native_sessions: BTreeSet<String>,
    pub health: DaemonHealth,
    pub generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DaemonRegistryEvent {
    Enrolled {
        registration: DaemonRegistration,
    },
    HealthObserved {
        daemon_id: String,
        generation: u64,
        health: DaemonHealth,
        version: String,
        capabilities: BTreeSet<String>,
    },
    ScopeAssigned {
        daemon_id: String,
        generation: u64,
        route: ProjectRouteKey,
    },
    Revoked {
        daemon_id: String,
        generation: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DaemonRegistryProjection {
    pub registrations: BTreeMap<String, DaemonRegistration>,
    #[serde(with = "route_map_serde")]
    pub routes: BTreeMap<ProjectRouteKey, BTreeSet<String>>,
    pub quarantined_daemons: BTreeMap<String, String>,
    pub rejected_events: u64,
}

mod route_map_serde {
    use super::ProjectRouteKey;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::{BTreeMap, BTreeSet};

    pub fn serialize<S>(
        value: &BTreeMap<ProjectRouteKey, BTreeSet<String>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.iter().collect::<Vec<_>>().serialize(serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<BTreeMap<ProjectRouteKey, BTreeSet<String>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries = Vec::<(ProjectRouteKey, BTreeSet<String>)>::deserialize(deserializer)?;
        Ok(entries.into_iter().collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonRoutingAuthority {
    pub schema: String,
    pub route: ProjectRouteKey,
    pub native_session_id: String,
    pub status: String,
    pub selected_daemon_id: Option<String>,
    pub selected_endpoint: Option<String>,
    pub health: Option<DaemonHealth>,
    pub capabilities: BTreeSet<String>,
    pub recovery_required: bool,
    pub failure_class: Option<String>,
}

pub fn project_routing_authority(
    registry: &DaemonRegistryProjection,
    route: &ProjectRouteKey,
    native_session_id: &str,
) -> DaemonRoutingAuthority {
    let resolved = if native_session_id.trim().is_empty() {
        Err(DaemonRegistryError::MissingIdentity("native_session_id"))
    } else {
        registry.resolve(route)
    };
    match resolved {
        Ok(registration)
            if registration
                .allowed_native_sessions
                .contains(native_session_id) =>
        {
            DaemonRoutingAuthority {
                schema: "focusa.daemon_routing_authority.v1".into(),
                route: route.clone(),
                native_session_id: native_session_id.into(),
                status: "resolved".into(),
                selected_daemon_id: Some(registration.daemon_id.clone()),
                selected_endpoint: Some(registration.endpoint.clone()),
                health: Some(registration.health),
                capabilities: registration.capabilities.clone(),
                recovery_required: false,
                failure_class: None,
            }
        }
        Ok(_) => unresolved_authority(route, native_session_id, "session_not_admitted"),
        Err(DaemonRegistryError::AmbiguousRoute) => {
            unresolved_authority(route, native_session_id, "ambiguous_route")
        }
        Err(DaemonRegistryError::NoRoute) => {
            unresolved_authority(route, native_session_id, "no_exact_route")
        }
        Err(_) => unresolved_authority(route, native_session_id, "invalid_scope"),
    }
}

fn unresolved_authority(
    route: &ProjectRouteKey,
    native_session_id: &str,
    failure_class: &str,
) -> DaemonRoutingAuthority {
    DaemonRoutingAuthority {
        schema: "focusa.daemon_routing_authority.v1".into(),
        route: route.clone(),
        native_session_id: native_session_id.into(),
        status: "unresolved".into(),
        selected_daemon_id: None,
        selected_endpoint: None,
        health: None,
        capabilities: BTreeSet::new(),
        recovery_required: true,
        failure_class: Some(failure_class.into()),
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DaemonRegistryError {
    #[error("daemon identity is missing: {0}")]
    MissingIdentity(&'static str),
    #[error("daemon endpoint is not an admitted HTTPS or loopback HTTP endpoint")]
    UnsafeEndpoint,
    #[error("project root is unsafe")]
    UnsafeProjectRoot,
    #[error("daemon registration is unknown")]
    UnknownDaemon,
    #[error("registry event generation is stale")]
    StaleGeneration,
    #[error("no healthy daemon owns the exact project route")]
    NoRoute,
    #[error("multiple healthy daemons own the exact project route")]
    AmbiguousRoute,
    #[error("daemon authentication fingerprint is missing")]
    MissingAuthentication,
}

impl DaemonRegistration {
    pub fn validate(&self) -> Result<(), DaemonRegistryError> {
        for (value, field) in [
            (&self.daemon_id, "daemon_id"),
            (&self.controller_id, "controller_id"),
            (&self.version, "version"),
        ] {
            if value.trim().is_empty() {
                return Err(DaemonRegistryError::MissingIdentity(field));
            }
        }
        if self.auth_fingerprint.trim().is_empty() {
            return Err(DaemonRegistryError::MissingAuthentication);
        }
        let endpoint = self.endpoint.to_ascii_lowercase();
        let safe = endpoint.starts_with("https://")
            || endpoint.starts_with("http://127.0.0.1:")
            || endpoint.starts_with("http://localhost:")
            || endpoint.starts_with("http://[::1]:");
        if !safe || self.endpoint.contains('@') {
            return Err(DaemonRegistryError::UnsafeEndpoint);
        }
        Ok(())
    }
}

pub fn reduce_daemon_registry(
    events: impl IntoIterator<Item = DaemonRegistryEvent>,
) -> DaemonRegistryProjection {
    let mut projection = DaemonRegistryProjection::default();
    for event in events {
        let daemon_id = event_daemon_id(&event).to_string();
        match apply_event(&mut projection, event) {
            Ok(()) => {
                projection.quarantined_daemons.remove(&daemon_id);
            }
            Err(error) => {
                projection.rejected_events += 1;
                if !daemon_id.is_empty() {
                    projection
                        .quarantined_daemons
                        .insert(daemon_id, error.to_string());
                }
            }
        }
    }
    projection
}

fn event_daemon_id(event: &DaemonRegistryEvent) -> &str {
    match event {
        DaemonRegistryEvent::Enrolled { registration } => &registration.daemon_id,
        DaemonRegistryEvent::HealthObserved { daemon_id, .. }
        | DaemonRegistryEvent::ScopeAssigned { daemon_id, .. }
        | DaemonRegistryEvent::Revoked { daemon_id, .. } => daemon_id,
    }
}

fn apply_event(
    projection: &mut DaemonRegistryProjection,
    event: DaemonRegistryEvent,
) -> Result<(), DaemonRegistryError> {
    match event {
        DaemonRegistryEvent::Enrolled { registration } => {
            registration.validate()?;
            if projection
                .registrations
                .get(&registration.daemon_id)
                .is_some_and(|current| current.generation >= registration.generation)
            {
                return Err(DaemonRegistryError::StaleGeneration);
            }
            projection
                .registrations
                .insert(registration.daemon_id.clone(), registration);
        }
        DaemonRegistryEvent::HealthObserved {
            daemon_id,
            generation,
            health,
            version,
            capabilities,
        } => {
            let registration = projection
                .registrations
                .get_mut(&daemon_id)
                .ok_or(DaemonRegistryError::UnknownDaemon)?;
            if generation <= registration.generation {
                return Err(DaemonRegistryError::StaleGeneration);
            }
            registration.generation = generation;
            registration.health = health;
            registration.version = version;
            registration.capabilities = capabilities;
        }
        DaemonRegistryEvent::ScopeAssigned {
            daemon_id,
            generation,
            route,
        } => {
            route.validate()?;
            let registration = projection
                .registrations
                .get(&daemon_id)
                .ok_or(DaemonRegistryError::UnknownDaemon)?;
            if generation != registration.generation || registration.health == DaemonHealth::Revoked
            {
                return Err(DaemonRegistryError::StaleGeneration);
            }
            projection
                .routes
                .entry(route)
                .or_default()
                .insert(daemon_id);
        }
        DaemonRegistryEvent::Revoked {
            daemon_id,
            generation,
        } => {
            let registration = projection
                .registrations
                .get_mut(&daemon_id)
                .ok_or(DaemonRegistryError::UnknownDaemon)?;
            if generation <= registration.generation {
                return Err(DaemonRegistryError::StaleGeneration);
            }
            registration.generation = generation;
            registration.health = DaemonHealth::Revoked;
            for owners in projection.routes.values_mut() {
                owners.remove(&daemon_id);
            }
        }
    }
    Ok(())
}

impl DaemonRegistryProjection {
    pub fn resolve(
        &self,
        route: &ProjectRouteKey,
    ) -> Result<&DaemonRegistration, DaemonRegistryError> {
        route.validate()?;
        let owners = self.routes.get(route).ok_or(DaemonRegistryError::NoRoute)?;
        let healthy = owners
            .iter()
            .filter(|daemon_id| !self.quarantined_daemons.contains_key(*daemon_id))
            .filter_map(|daemon_id| self.registrations.get(daemon_id))
            .filter(|registration| registration.health == DaemonHealth::Healthy)
            .collect::<Vec<_>>();
        match healthy.as_slice() {
            [registration] => Ok(registration),
            [] => Err(DaemonRegistryError::NoRoute),
            _ => Err(DaemonRegistryError::AmbiguousRoute),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registration(id: &str, generation: u64) -> DaemonRegistration {
        DaemonRegistration {
            daemon_id: id.into(),
            controller_id: "controller-1".into(),
            endpoint: format!("https://{id}.example.test"),
            auth_fingerprint: format!("sha256:{id}"),
            version: "0.9.143".into(),
            capabilities: BTreeSet::from(["workpoint".into()]),
            allowed_native_sessions: BTreeSet::from(["session-1".into()]),
            health: DaemonHealth::Healthy,
            generation,
        }
    }

    fn route() -> ProjectRouteKey {
        ProjectRouteKey {
            project_root: "/srv/focusa".into(),
            continuity_id: "continuity-1".into(),
            working_subpath_id: "working-subpath:main".into(),
        }
    }

    #[test]
    fn canonical_surface_projection_never_infers_foreign_daemon_or_session() {
        let projection = reduce_daemon_registry([
            DaemonRegistryEvent::Enrolled {
                registration: registration("daemon-a", 1),
            },
            DaemonRegistryEvent::ScopeAssigned {
                daemon_id: "daemon-a".into(),
                generation: 1,
                route: route(),
            },
        ]);
        let resolved = project_routing_authority(&projection, &route(), "session-1");
        assert_eq!(resolved.status, "resolved");
        assert_eq!(resolved.selected_daemon_id.as_deref(), Some("daemon-a"));
        let foreign = project_routing_authority(&projection, &route(), "session-foreign");
        assert_eq!(foreign.status, "unresolved");
        assert_eq!(foreign.selected_daemon_id, None);
        assert_eq!(
            foreign.failure_class.as_deref(),
            Some("session_not_admitted")
        );
        let mut foreign_route = route();
        foreign_route.project_root = "/srv/foreign".into();
        let missing = project_routing_authority(&projection, &foreign_route, "session-1");
        assert_eq!(missing.selected_daemon_id, None);
        assert_eq!(missing.failure_class.as_deref(), Some("no_exact_route"));
    }

    #[test]
    fn exact_route_resolves_one_authenticated_healthy_daemon() {
        let projection = reduce_daemon_registry([
            DaemonRegistryEvent::Enrolled {
                registration: registration("daemon-a", 1),
            },
            DaemonRegistryEvent::ScopeAssigned {
                daemon_id: "daemon-a".into(),
                generation: 1,
                route: route(),
            },
        ]);
        assert_eq!(projection.resolve(&route()).unwrap().daemon_id, "daemon-a");
    }

    #[test]
    fn ambiguous_stale_and_revoked_routes_fail_closed() {
        let projection = reduce_daemon_registry([
            DaemonRegistryEvent::Enrolled {
                registration: registration("daemon-a", 1),
            },
            DaemonRegistryEvent::Enrolled {
                registration: registration("daemon-b", 1),
            },
            DaemonRegistryEvent::ScopeAssigned {
                daemon_id: "daemon-a".into(),
                generation: 1,
                route: route(),
            },
            DaemonRegistryEvent::ScopeAssigned {
                daemon_id: "daemon-b".into(),
                generation: 1,
                route: route(),
            },
        ]);
        assert_eq!(projection.rejected_events, 0);
        assert_eq!(
            projection.resolve(&route()),
            Err(DaemonRegistryError::AmbiguousRoute)
        );

        let revoked = reduce_daemon_registry([
            DaemonRegistryEvent::Enrolled {
                registration: registration("daemon-a", 1),
            },
            DaemonRegistryEvent::ScopeAssigned {
                daemon_id: "daemon-a".into(),
                generation: 1,
                route: route(),
            },
            DaemonRegistryEvent::Revoked {
                daemon_id: "daemon-a".into(),
                generation: 2,
            },
        ]);
        assert_eq!(revoked.resolve(&route()), Err(DaemonRegistryError::NoRoute));
    }

    #[test]
    fn replay_survives_restart_and_quarantines_stale_duplicate_and_untrusted_daemons() {
        let mut unsafe_registration = registration("daemon-unsafe", 1);
        unsafe_registration.endpoint = "http://remote.example.test".into();
        let events = vec![
            DaemonRegistryEvent::Enrolled {
                registration: registration("daemon-a", 1),
            },
            DaemonRegistryEvent::ScopeAssigned {
                daemon_id: "daemon-a".into(),
                generation: 1,
                route: route(),
            },
            DaemonRegistryEvent::Enrolled {
                registration: registration("daemon-a", 1),
            },
            DaemonRegistryEvent::Enrolled {
                registration: unsafe_registration,
            },
        ];
        let serialized = serde_json::to_vec(&events).unwrap();
        let replayed_events: Vec<DaemonRegistryEvent> =
            serde_json::from_slice(&serialized).unwrap();
        let first = reduce_daemon_registry(events);
        let after_restart = reduce_daemon_registry(replayed_events);
        assert_eq!(first, after_restart);
        assert_eq!(first.rejected_events, 2);
        assert!(first.quarantined_daemons.contains_key("daemon-a"));
        assert!(first.quarantined_daemons.contains_key("daemon-unsafe"));
        assert_eq!(first.resolve(&route()), Err(DaemonRegistryError::NoRoute));
        assert_eq!(
            serde_json::to_vec(&first).unwrap(),
            serde_json::to_vec(&after_restart).unwrap()
        );
    }
}
