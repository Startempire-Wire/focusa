use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
pub struct SignedInstallationEnrollment {
    pub schema: String,
    pub enrollment: InstallationEnrollment,
    pub signer_key_id: String,
    pub signature_base64: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InstallationEnrollmentProjection {
    pub enrollments: BTreeMap<String, InstallationEnrollment>,
    pub rejected_records: u64,
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
    Offline,
    Drifted,
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
pub enum ServiceHealth {
    Online,
    Offline,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallationSurfaceObservation {
    pub installation_id: String,
    pub surface: ManagedSurface,
    pub generation: u64,
    pub observed_at_unix_ms: i64,
    pub version: Option<String>,
    pub artifact_digest: Option<String>,
    pub capabilities: BTreeSet<String>,
    pub service_health: ServiceHealth,
    pub last_proof_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectedSurfaceInventory {
    pub observation: InstallationSurfaceObservation,
    pub health: SurfaceHealth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InstallationInventoryProjection {
    pub surfaces: Vec<ProjectedSurfaceInventory>,
    pub rejected_observations: u64,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedConvergencePlan {
    pub schema: String,
    pub plan_id: String,
    pub plan: ConvergencePlan,
    pub signer_key_id: String,
    pub signature_base64: String,
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
    #[error("enrollment signature is invalid")]
    InvalidEnrollmentSignature,
    #[error("enrollment signer is untrusted")]
    UntrustedEnrollmentSigner,
    #[error("enrollment owner differs from expected operator")]
    EnrollmentOwnerMismatch,
    #[error("enrollment record generation is stale")]
    StaleEnrollmentRecord,
    #[error("enrollment record schema is unsupported")]
    UnsupportedEnrollmentSchema,
    #[error("convergence plan signature is invalid")]
    InvalidPlanSignature,
    #[error("convergence plan signer is untrusted")]
    UntrustedPlanSigner,
    #[error("convergence plan schema is unsupported")]
    UnsupportedPlanSchema,
}

impl SignedInstallationEnrollment {
    pub fn canonical_payload(&self) -> Result<Vec<u8>, ConvergenceError> {
        if self.schema != "focusa.signed_installation_enrollment.v1" {
            return Err(ConvergenceError::UnsupportedEnrollmentSchema);
        }
        serde_json::to_vec(&self.enrollment)
            .map_err(|_| ConvergenceError::InvalidEnrollmentSignature)
    }
}

pub fn verify_signed_enrollment(
    record: &SignedInstallationEnrollment,
    expected_operator_id: &str,
    trusted_keys: &BTreeMap<String, [u8; 32]>,
) -> Result<(), ConvergenceError> {
    record.enrollment.validate_identity()?;
    if record.enrollment.operator_id != expected_operator_id {
        return Err(ConvergenceError::EnrollmentOwnerMismatch);
    }
    let key = trusted_keys
        .get(&record.signer_key_id)
        .ok_or(ConvergenceError::UntrustedEnrollmentSigner)?;
    let verifying_key =
        VerifyingKey::from_bytes(key).map_err(|_| ConvergenceError::UntrustedEnrollmentSigner)?;
    let signature_bytes = BASE64
        .decode(&record.signature_base64)
        .map_err(|_| ConvergenceError::InvalidEnrollmentSignature)?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| ConvergenceError::InvalidEnrollmentSignature)?;
    verifying_key
        .verify(&record.canonical_payload()?, &signature)
        .map_err(|_| ConvergenceError::InvalidEnrollmentSignature)
}

pub fn replay_signed_enrollments(
    records: impl IntoIterator<Item = SignedInstallationEnrollment>,
    expected_operator_id: &str,
    trusted_keys: &BTreeMap<String, [u8; 32]>,
) -> InstallationEnrollmentProjection {
    let mut projection = InstallationEnrollmentProjection::default();
    for record in records {
        if verify_signed_enrollment(&record, expected_operator_id, trusted_keys).is_err()
            || projection
                .enrollments
                .get(&record.enrollment.installation_id)
                .is_some_and(|current| current.generation >= record.enrollment.generation)
        {
            projection.rejected_records += 1;
            continue;
        }
        projection
            .enrollments
            .insert(record.enrollment.installation_id.clone(), record.enrollment);
    }
    projection
}

pub fn project_installation_inventory(
    observations: impl IntoIterator<Item = InstallationSurfaceObservation>,
    desired: Option<&DesiredInstallationState>,
) -> InstallationInventoryProjection {
    let mut projection = InstallationInventoryProjection::default();
    for observation in observations {
        if observation.installation_id.trim().is_empty() || observation.generation == 0 {
            projection.rejected_observations += 1;
            continue;
        }
        let existing = projection.surfaces.iter().position(|current| {
            current.observation.installation_id == observation.installation_id
                && current.observation.surface == observation.surface
        });
        if existing.is_some_and(|index| {
            projection.surfaces[index].observation.generation >= observation.generation
        }) {
            projection.rejected_observations += 1;
            continue;
        }
        let expected = desired.filter(|state| state.installation_id == observation.installation_id);
        let health = if observation.service_health == ServiceHealth::Offline {
            SurfaceHealth::Offline
        } else if observation.service_health == ServiceHealth::Unknown
            || observation.version.as_deref().is_none_or(str::is_empty)
            || observation
                .artifact_digest
                .as_deref()
                .is_none_or(str::is_empty)
            || observation
                .last_proof_ref
                .as_deref()
                .is_none_or(str::is_empty)
        {
            SurfaceHealth::Unknown
        } else if expected.is_some_and(|state| {
            observation.version.as_deref() != Some(state.version.as_str())
                || observation.artifact_digest.as_deref()
                    != Some(state.artifact_manifest_digest.as_str())
                || !state.surfaces.contains(&observation.surface)
        }) {
            SurfaceHealth::Drifted
        } else {
            SurfaceHealth::Healthy
        };
        let projected = ProjectedSurfaceInventory {
            observation,
            health,
        };
        if let Some(index) = existing {
            projection.surfaces[index] = projected;
        } else {
            projection.surfaces.push(projected);
        }
    }
    projection.surfaces.sort_by(|left, right| {
        (&left.observation.installation_id, left.observation.surface).cmp(&(
            &right.observation.installation_id,
            right.observation.surface,
        ))
    });
    projection
}

impl InstallationEnrollment {
    fn validate_identity(&self) -> Result<(), ConvergenceError> {
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
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ConvergenceError> {
        self.validate_identity()?;
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

pub fn sign_convergence_plan(
    plan: ConvergencePlan,
    signer_key_id: &str,
    signing_key: &SigningKey,
) -> Result<SignedConvergencePlan, ConvergenceError> {
    if signer_key_id.trim().is_empty() {
        return Err(ConvergenceError::UntrustedPlanSigner);
    }
    let payload = serde_json::to_vec(&plan).map_err(|_| ConvergenceError::InvalidPlanSignature)?;
    let plan_id = format!("sha256:{}", hex::encode(Sha256::digest(&payload)));
    let signature_base64 = BASE64.encode(signing_key.sign(&payload).to_bytes());
    Ok(SignedConvergencePlan {
        schema: "focusa.signed_convergence_plan.v1".into(),
        plan_id,
        plan,
        signer_key_id: signer_key_id.into(),
        signature_base64,
    })
}

pub fn verify_signed_convergence_plan(
    signed: &SignedConvergencePlan,
    trusted_keys: &BTreeMap<String, [u8; 32]>,
) -> Result<(), ConvergenceError> {
    if signed.schema != "focusa.signed_convergence_plan.v1"
        || signed.plan.schema != "focusa.installation_convergence_plan.v1"
    {
        return Err(ConvergenceError::UnsupportedPlanSchema);
    }
    let payload =
        serde_json::to_vec(&signed.plan).map_err(|_| ConvergenceError::InvalidPlanSignature)?;
    let expected_id = format!("sha256:{}", hex::encode(Sha256::digest(&payload)));
    if signed.plan_id != expected_id {
        return Err(ConvergenceError::InvalidPlanSignature);
    }
    let key = trusted_keys
        .get(&signed.signer_key_id)
        .ok_or(ConvergenceError::UntrustedPlanSigner)?;
    let verifying_key =
        VerifyingKey::from_bytes(key).map_err(|_| ConvergenceError::UntrustedPlanSigner)?;
    let signature_bytes = BASE64
        .decode(&signed.signature_base64)
        .map_err(|_| ConvergenceError::InvalidPlanSignature)?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| ConvergenceError::InvalidPlanSignature)?;
    verifying_key
        .verify(&payload, &signature)
        .map_err(|_| ConvergenceError::InvalidPlanSignature)
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

    fn signed(
        enrollment: InstallationEnrollment,
        key: &SigningKey,
    ) -> SignedInstallationEnrollment {
        let mut record = SignedInstallationEnrollment {
            schema: "focusa.signed_installation_enrollment.v1".into(),
            enrollment,
            signer_key_id: "operator-key-1".into(),
            signature_base64: String::new(),
        };
        record.signature_base64 =
            BASE64.encode(key.sign(&record.canonical_payload().unwrap()).to_bytes());
        record
    }

    #[test]
    fn signed_enrollment_is_owner_scoped_revocable_and_replayable() {
        let key = SigningKey::from_bytes(&[7; 32]);
        let trusted = BTreeMap::from([("operator-key-1".into(), key.verifying_key().to_bytes())]);
        let admitted = signed(enrollment(), &key);
        verify_signed_enrollment(&admitted, "operator-1", &trusted).unwrap();

        let mut revoked_enrollment = enrollment();
        revoked_enrollment.generation = 4;
        revoked_enrollment.revoked = true;
        let revoked = signed(revoked_enrollment, &key);
        let projection = replay_signed_enrollments(
            [admitted.clone(), revoked.clone(), admitted.clone()],
            "operator-1",
            &trusted,
        );
        assert!(projection.enrollments["install-1"].revoked);
        assert_eq!(projection.rejected_records, 1);
        let restarted: InstallationEnrollmentProjection =
            serde_json::from_slice(&serde_json::to_vec(&projection).unwrap()).unwrap();
        assert_eq!(restarted, projection);

        let mut tampered = admitted.clone();
        tampered.enrollment.channel = "foreign".into();
        assert_eq!(
            verify_signed_enrollment(&tampered, "operator-1", &trusted),
            Err(ConvergenceError::InvalidEnrollmentSignature)
        );
        assert_eq!(
            verify_signed_enrollment(&admitted, "operator-foreign", &trusted),
            Err(ConvergenceError::EnrollmentOwnerMismatch)
        );
    }

    #[test]
    fn inventory_projection_never_fabricates_health_or_convergence() {
        let observation =
            |surface,
             generation,
             service_health,
             version: Option<&str>,
             digest: Option<&str>,
             proof: Option<&str>| InstallationSurfaceObservation {
                installation_id: "install-1".into(),
                surface,
                generation,
                observed_at_unix_ms: generation as i64,
                version: version.map(str::to_string),
                artifact_digest: digest.map(str::to_string),
                capabilities: BTreeSet::from(["health".into()]),
                service_health,
                last_proof_ref: proof.map(str::to_string),
            };
        let projection = project_installation_inventory(
            [
                observation(
                    ManagedSurface::Cli,
                    2,
                    ServiceHealth::Online,
                    Some("v0.9.144"),
                    Some("sha256:manifest"),
                    Some("proof:cli"),
                ),
                observation(
                    ManagedSurface::Daemon,
                    2,
                    ServiceHealth::Offline,
                    Some("v0.9.144"),
                    Some("sha256:manifest"),
                    Some("proof:daemon"),
                ),
                observation(
                    ManagedSurface::Desktop,
                    2,
                    ServiceHealth::Unknown,
                    None,
                    None,
                    None,
                ),
                observation(
                    ManagedSurface::AgentContext,
                    2,
                    ServiceHealth::Online,
                    Some("v0.9.143"),
                    Some("sha256:old"),
                    Some("proof:context"),
                ),
                observation(
                    ManagedSurface::Cli,
                    1,
                    ServiceHealth::Online,
                    Some("v0.9.144"),
                    Some("sha256:manifest"),
                    Some("proof:stale"),
                ),
            ],
            Some(&desired()),
        );
        let health = projection
            .surfaces
            .iter()
            .map(|surface| (surface.observation.surface, surface.health))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(health[&ManagedSurface::Cli], SurfaceHealth::Healthy);
        assert_eq!(health[&ManagedSurface::Daemon], SurfaceHealth::Offline);
        assert_eq!(health[&ManagedSurface::Desktop], SurfaceHealth::Unknown);
        assert_eq!(
            health[&ManagedSurface::AgentContext],
            SurfaceHealth::Drifted
        );
        assert_eq!(projection.rejected_observations, 1);
        let restarted: InstallationInventoryProjection =
            serde_json::from_slice(&serde_json::to_vec(&projection).unwrap()).unwrap();
        assert_eq!(restarted, projection);
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
        assert_eq!(
            plan,
            plan_convergence(&enrollment(), &desired(), &current).unwrap()
        );
        let key = SigningKey::from_bytes(&[9; 32]);
        let trusted = BTreeMap::from([("planner-key-1".into(), key.verifying_key().to_bytes())]);
        let signed = sign_convergence_plan(plan.clone(), "planner-key-1", &key).unwrap();
        verify_signed_convergence_plan(&signed, &trusted).unwrap();
        let mut tampered = signed;
        tampered.plan.actions[0].to_version = "v9.9.9".into();
        assert_eq!(
            verify_signed_convergence_plan(&tampered, &trusted),
            Err(ConvergenceError::InvalidPlanSignature)
        );
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
