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
    pub routes: BTreeMap<ProjectRouteKey, BTreeSet<String>>,
    pub rejected_events: u64,
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
        if apply_event(&mut projection, event).is_err() {
            projection.rejected_events += 1;
        }
    }
    projection
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
            DaemonRegistryEvent::HealthObserved {
                daemon_id: "daemon-a".into(),
                generation: 1,
                health: DaemonHealth::Offline,
                version: "stale".into(),
                capabilities: BTreeSet::new(),
            },
        ]);
        assert_eq!(projection.rejected_events, 1);
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
}
