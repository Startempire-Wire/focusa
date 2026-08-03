use crate::installation_convergence::{
    ConvergenceActionKind, ConvergencePlan, InstallationPlatform, ManagedSurface,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceManager {
    Launchd,
    SystemdUser,
    WindowsService,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlatformConvergenceOperation {
    VerifyManifest {
        digest: String,
    },
    DownloadAndVerify {
        surface: ManagedSurface,
        target_name: String,
    },
    SnapshotCurrent {
        surface: ManagedSurface,
    },
    StopService {
        manager: ServiceManager,
    },
    AtomicReplace {
        surface: ManagedSurface,
        target_name: String,
    },
    StartService {
        manager: ServiceManager,
    },
    HealthProbe {
        surface: ManagedSurface,
    },
    PreserveNoOp {
        surface: ManagedSurface,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformConvergencePlan {
    pub schema: String,
    pub installation_id: String,
    pub platform: InstallationPlatform,
    pub desired_version: String,
    pub operations: Vec<PlatformConvergenceOperation>,
    pub rollback_order: Vec<ManagedSurface>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlatformConvergenceError {
    #[error("convergence plan contains no actions")]
    EmptyPlan,
    #[error("surface is unsupported on platform: {0:?}")]
    UnsupportedSurface(ManagedSurface),
    #[error("surface target name is unavailable")]
    MissingTarget,
}

pub fn build_platform_convergence_plan(
    plan: &ConvergencePlan,
) -> Result<PlatformConvergencePlan, PlatformConvergenceError> {
    if plan.actions.is_empty() {
        return Err(PlatformConvergenceError::EmptyPlan);
    }
    let desired_version = plan
        .actions
        .first()
        .map(|action| action.to_version.clone())
        .ok_or(PlatformConvergenceError::EmptyPlan)?;
    let mut operations = vec![PlatformConvergenceOperation::VerifyManifest {
        digest: plan.artifact_manifest_digest.clone(),
    }];
    let mut rollback_order = Vec::new();
    for action in &plan.actions {
        validate_surface(plan.platform, action.surface)?;
        if action.action == ConvergenceActionKind::NoOp {
            operations.push(PlatformConvergenceOperation::PreserveNoOp {
                surface: action.surface,
            });
            continue;
        }
        let target_name = target_name(plan.platform, action.surface)?;
        operations.push(PlatformConvergenceOperation::DownloadAndVerify {
            surface: action.surface,
            target_name: target_name.clone(),
        });
        operations.push(PlatformConvergenceOperation::SnapshotCurrent {
            surface: action.surface,
        });
        if action.surface == ManagedSurface::Daemon {
            operations.push(PlatformConvergenceOperation::StopService {
                manager: service_manager(plan.platform),
            });
        }
        operations.push(PlatformConvergenceOperation::AtomicReplace {
            surface: action.surface,
            target_name,
        });
        if action.surface == ManagedSurface::Daemon {
            operations.push(PlatformConvergenceOperation::StartService {
                manager: service_manager(plan.platform),
            });
        }
        operations.push(PlatformConvergenceOperation::HealthProbe {
            surface: action.surface,
        });
        rollback_order.push(action.surface);
    }
    rollback_order.reverse();
    Ok(PlatformConvergencePlan {
        schema: "focusa.platform_convergence_plan.v1".into(),
        installation_id: plan.installation_id.clone(),
        platform: plan.platform,
        desired_version,
        operations,
        rollback_order,
    })
}

fn validate_surface(
    platform: InstallationPlatform,
    surface: ManagedSurface,
) -> Result<(), PlatformConvergenceError> {
    if surface == ManagedSurface::Desktop
        && matches!(
            platform,
            InstallationPlatform::LinuxGnuX64 | InstallationPlatform::LinuxMuslX64
        )
    {
        return Err(PlatformConvergenceError::UnsupportedSurface(surface));
    }
    Ok(())
}

fn service_manager(platform: InstallationPlatform) -> ServiceManager {
    match platform {
        InstallationPlatform::MacosArm64 | InstallationPlatform::MacosX64 => {
            ServiceManager::Launchd
        }
        InstallationPlatform::LinuxGnuX64 | InstallationPlatform::LinuxMuslX64 => {
            ServiceManager::SystemdUser
        }
        InstallationPlatform::WindowsX64 | InstallationPlatform::WindowsArm64 => {
            ServiceManager::WindowsService
        }
    }
}

fn target_name(
    platform: InstallationPlatform,
    surface: ManagedSurface,
) -> Result<String, PlatformConvergenceError> {
    let executable_suffix = matches!(
        platform,
        InstallationPlatform::WindowsX64 | InstallationPlatform::WindowsArm64
    )
    .then_some(".exe")
    .unwrap_or("");
    let target = match surface {
        ManagedSurface::Cli => format!("focusa{executable_suffix}"),
        ManagedSurface::Daemon => format!("focusa-daemon{executable_suffix}"),
        ManagedSurface::Tui => format!("focusa-tui{executable_suffix}"),
        ManagedSurface::Desktop => match platform {
            InstallationPlatform::MacosArm64 | InstallationPlatform::MacosX64 => {
                "Focusa.app.zip".into()
            }
            InstallationPlatform::WindowsX64 | InstallationPlatform::WindowsArm64 => {
                "Focusa-setup.exe".into()
            }
            _ => return Err(PlatformConvergenceError::MissingTarget),
        },
        ManagedSurface::PiExtension => "focusa-pi-extension.tar.gz".into(),
        ManagedSurface::AgentContext => "focusa-agent-context.tar.gz".into(),
    };
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::installation_convergence::SurfaceConvergenceAction;

    fn plan(platform: InstallationPlatform, surfaces: &[ManagedSurface]) -> ConvergencePlan {
        ConvergencePlan {
            schema: "focusa.installation_convergence_plan.v1".into(),
            installation_id: "install-1".into(),
            enrollment_generation: 1,
            desired_generation: 2,
            platform,
            channel: "stable".into(),
            artifact_manifest_digest: "sha256:manifest".into(),
            actions: surfaces
                .iter()
                .map(|surface| SurfaceConvergenceAction {
                    surface: *surface,
                    action: ConvergenceActionKind::Update,
                    from_version: Some("0.9.143".into()),
                    to_version: "0.9.144".into(),
                    reason: "version_drift".into(),
                })
                .collect(),
            operator_approval_ref: "approval:1".into(),
        }
    }

    #[test]
    fn windows_plan_uses_executables_service_and_reverse_rollback_order() {
        let plan = build_platform_convergence_plan(&plan(
            InstallationPlatform::WindowsArm64,
            &[
                ManagedSurface::Cli,
                ManagedSurface::Daemon,
                ManagedSurface::Desktop,
            ],
        ))
        .unwrap();
        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            PlatformConvergenceOperation::AtomicReplace { target_name, .. }
                if target_name == "focusa.exe"
        )));
        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            PlatformConvergenceOperation::StopService {
                manager: ServiceManager::WindowsService
            }
        )));
        assert_eq!(
            plan.rollback_order,
            vec![
                ManagedSurface::Desktop,
                ManagedSurface::Daemon,
                ManagedSurface::Cli
            ]
        );
    }

    #[test]
    fn mac_linux_and_windows_surface_boundaries_are_explicit() {
        assert!(
            build_platform_convergence_plan(&plan(
                InstallationPlatform::MacosArm64,
                &[ManagedSurface::Desktop, ManagedSurface::Daemon],
            ))
            .is_ok()
        );
        assert!(
            build_platform_convergence_plan(&plan(
                InstallationPlatform::LinuxMuslX64,
                &[
                    ManagedSurface::Cli,
                    ManagedSurface::Daemon,
                    ManagedSurface::PiExtension
                ],
            ))
            .is_ok()
        );
        assert_eq!(
            build_platform_convergence_plan(&plan(
                InstallationPlatform::LinuxGnuX64,
                &[ManagedSurface::Desktop],
            )),
            Err(PlatformConvergenceError::UnsupportedSurface(
                ManagedSurface::Desktop
            ))
        );
    }
}
