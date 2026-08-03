use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallationPlatform {
    MacosArm64,
    MacosX64,
    LinuxGnuX64,
    LinuxMuslX64,
    WindowsX64,
    WindowsArm64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedSurface {
    Cli,
    Daemon,
    Tui,
    Desktop,
    PiExtension,
    AgentContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallationEnrollment {
    pub installation_id: String,
    pub operator_id: String,
    pub host_id: String,
    pub platform: InstallationPlatform,
    pub channel: String,
    pub enrolled_surfaces: BTreeSet<ManagedSurface>,
    pub authority_signature_ref: String,
    pub generation: u64,
    pub revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesiredInstallationState {
    pub installation_id: String,
    pub generation: u64,
    pub version: String,
    pub channel: String,
    pub surfaces: BTreeSet<ManagedSurface>,
    pub artifact_manifest_digest: String,
    pub operator_approval_ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceHealth {
    Healthy,
    Degraded,
    Missing,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledSurfaceState {
    pub version: String,
    pub artifact_digest: String,
    pub health: SurfaceHealth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentInstallationState {
    pub installation_id: String,
    pub generation: u64,
    pub channel: String,
    pub surfaces: BTreeMap<ManagedSurface, InstalledSurfaceState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConvergenceActionKind {
    Install,
    Update,
    Repair,
    NoOp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceConvergenceAction {
    pub surface: ManagedSurface,
    pub action: ConvergenceActionKind,
    pub from_version: Option<String>,
    pub to_version: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergencePlan {
    pub schema: String,
    pub installation_id: String,
    pub enrollment_generation: u64,
    pub desired_generation: u64,
    pub platform: InstallationPlatform,
    pub channel: String,
    pub artifact_manifest_digest: String,
    pub actions: Vec<SurfaceConvergenceAction>,
    pub operator_approval_ref: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConvergenceError {
    #[error("installation identity is missing: {0}")]
    MissingIdentity(&'static str),
    #[error("enrollment has been revoked")]
    EnrollmentRevoked,
    #[error("installation ids do not match")]
    InstallationMismatch,
    #[error("desired channel differs from enrolled channel")]
    ChannelChangeNotAuthorized,
    #[error("desired surface is not enrolled")]
    SurfaceNotEnrolled,
    #[error("desired state generation is stale")]
    StaleDesiredState,
    #[error("version is invalid: {0}")]
    InvalidVersion(String),
    #[error("automatic downgrade is forbidden")]
    DowngradeForbidden,
}

impl InstallationEnrollment {
    pub fn validate(&self) -> Result<(), ConvergenceError> {
        for (value, field) in [
            (&self.installation_id, "installation_id"),
            (&self.operator_id, "operator_id"),
            (&self.host_id, "host_id"),
            (&self.channel, "channel"),
            (&self.authority_signature_ref, "authority_signature_ref"),
        ] {
            if value.trim().is_empty() {
                return Err(ConvergenceError::MissingIdentity(field));
            }
        }
        if self.revoked {
            return Err(ConvergenceError::EnrollmentRevoked);
        }
        Ok(())
    }
}

pub fn plan_convergence(
    enrollment: &InstallationEnrollment,
    desired: &DesiredInstallationState,
    current: &CurrentInstallationState,
) -> Result<ConvergencePlan, ConvergenceError> {
    enrollment.validate()?;
    for (value, field) in [
        (&desired.version, "desired.version"),
        (
            &desired.artifact_manifest_digest,
            "artifact_manifest_digest",
        ),
        (&desired.operator_approval_ref, "operator_approval_ref"),
    ] {
        if value.trim().is_empty() {
            return Err(ConvergenceError::MissingIdentity(field));
        }
    }
    if desired.installation_id != enrollment.installation_id
        || current.installation_id != enrollment.installation_id
    {
        return Err(ConvergenceError::InstallationMismatch);
    }
    if desired.channel != enrollment.channel || current.channel != enrollment.channel {
        return Err(ConvergenceError::ChannelChangeNotAuthorized);
    }
    if !desired.surfaces.is_subset(&enrollment.enrolled_surfaces) {
        return Err(ConvergenceError::SurfaceNotEnrolled);
    }
    if desired.generation <= current.generation || desired.generation < enrollment.generation {
        return Err(ConvergenceError::StaleDesiredState);
    }
    let desired_version = parse_version(&desired.version)?;
    let mut actions = Vec::with_capacity(desired.surfaces.len());
    for surface in &desired.surfaces {
        let (action, from_version, reason) = match current.surfaces.get(surface) {
            None => (
                ConvergenceActionKind::Install,
                None,
                "surface_missing".into(),
            ),
            Some(installed) => {
                let current_version = parse_version(&installed.version)?;
                if current_version > desired_version {
                    return Err(ConvergenceError::DowngradeForbidden);
                }
                if current_version < desired_version {
                    (
                        ConvergenceActionKind::Update,
                        Some(installed.version.clone()),
                        "version_drift".into(),
                    )
                } else if installed.health != SurfaceHealth::Healthy
                    || installed.artifact_digest.trim().is_empty()
                {
                    (
                        ConvergenceActionKind::Repair,
                        Some(installed.version.clone()),
                        "health_or_digest_drift".into(),
                    )
                } else {
                    (
                        ConvergenceActionKind::NoOp,
                        Some(installed.version.clone()),
                        "already_converged".into(),
                    )
                }
            }
        };
        actions.push(SurfaceConvergenceAction {
            surface: *surface,
            action,
            from_version,
            to_version: desired.version.clone(),
            reason,
        });
    }
    Ok(ConvergencePlan {
        schema: "focusa.installation_convergence_plan.v1".into(),
        installation_id: enrollment.installation_id.clone(),
        enrollment_generation: enrollment.generation,
        desired_generation: desired.generation,
        platform: enrollment.platform,
        channel: enrollment.channel.clone(),
        artifact_manifest_digest: desired.artifact_manifest_digest.clone(),
        actions,
        operator_approval_ref: desired.operator_approval_ref.clone(),
    })
}

fn parse_version(value: &str) -> Result<(u64, u64, u64), ConvergenceError> {
    let normalized = value.trim().trim_start_matches('v');
    let core = normalized.split(['-', '+']).next().unwrap_or("");
    let parts = core
        .split('.')
        .map(|part| part.parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ConvergenceError::InvalidVersion(value.into()))?;
    match parts.as_slice() {
        [major, minor, patch] => Ok((*major, *minor, *patch)),
        _ => Err(ConvergenceError::InvalidVersion(value.into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enrollment() -> InstallationEnrollment {
        InstallationEnrollment {
            installation_id: "install-1".into(),
            operator_id: "operator-1".into(),
            host_id: "host-1".into(),
            platform: InstallationPlatform::WindowsArm64,
            channel: "stable".into(),
            enrolled_surfaces: BTreeSet::from([
                ManagedSurface::Cli,
                ManagedSurface::Daemon,
                ManagedSurface::Desktop,
            ]),
            authority_signature_ref: "signature:enrollment".into(),
            generation: 3,
            revoked: false,
        }
    }

    fn desired() -> DesiredInstallationState {
        DesiredInstallationState {
            installation_id: "install-1".into(),
            generation: 5,
            version: "v0.9.144".into(),
            channel: "stable".into(),
            surfaces: BTreeSet::from([
                ManagedSurface::Cli,
                ManagedSurface::Daemon,
                ManagedSurface::Desktop,
            ]),
            artifact_manifest_digest: "sha256:manifest".into(),
            operator_approval_ref: "approval:1".into(),
        }
    }

    #[test]
    fn planner_emits_install_update_repair_without_implicit_scope_change() {
        let current = CurrentInstallationState {
            installation_id: "install-1".into(),
            generation: 4,
            channel: "stable".into(),
            surfaces: BTreeMap::from([
                (
                    ManagedSurface::Cli,
                    InstalledSurfaceState {
                        version: "0.9.143".into(),
                        artifact_digest: "sha256:old".into(),
                        health: SurfaceHealth::Healthy,
                    },
                ),
                (
                    ManagedSurface::Daemon,
                    InstalledSurfaceState {
                        version: "0.9.144".into(),
                        artifact_digest: "".into(),
                        health: SurfaceHealth::Degraded,
                    },
                ),
            ]),
        };
        let plan = plan_convergence(&enrollment(), &desired(), &current).unwrap();
        assert_eq!(plan.actions[0].action, ConvergenceActionKind::Update);
        assert_eq!(plan.actions[1].action, ConvergenceActionKind::Repair);
        assert_eq!(plan.actions[2].action, ConvergenceActionKind::Install);
    }

    #[test]
    fn planner_rejects_downgrade_channel_change_and_unenrolled_surface() {
        let current = CurrentInstallationState {
            installation_id: "install-1".into(),
            generation: 4,
            channel: "stable".into(),
            surfaces: BTreeMap::from([(
                ManagedSurface::Cli,
                InstalledSurfaceState {
                    version: "0.10.0".into(),
                    artifact_digest: "sha256:newer".into(),
                    health: SurfaceHealth::Healthy,
                },
            )]),
        };
        assert_eq!(
            plan_convergence(&enrollment(), &desired(), &current),
            Err(ConvergenceError::DowngradeForbidden)
        );
        let mut changed = desired();
        changed.channel = "preview".into();
        assert_eq!(
            plan_convergence(&enrollment(), &changed, &current),
            Err(ConvergenceError::ChannelChangeNotAuthorized)
        );
        let mut surface = desired();
        surface.surfaces.insert(ManagedSurface::Tui);
        assert_eq!(
            plan_convergence(&enrollment(), &surface, &current),
            Err(ConvergenceError::SurfaceNotEnrolled)
        );
    }
}
