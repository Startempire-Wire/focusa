//! Core service for the governed Mission Canvas domain-pack install operation.
//!
//! The API route is only an adapter.  This service owns the operation's
//! confirmation, capability, permission, exact WorkstreamContext, pack
//! validation, request digest, and hand-off to the atomic Mission Canvas
//! persistence boundary.

use std::collections::BTreeSet;

use chrono::Utc;
use serde_json::to_vec;
use thiserror::Error;

use crate::workstream_context::{WorkstreamContext, WorkstreamContextError};

use super::{
    CompositionRegistry, DomainPack, DomainPackInstallReceipt, MissionCanvasScope,
    MissionCanvasStore, MissionCanvasStoreError,
};

pub const DOMAIN_PACK_INSTALL_OPERATION: &str = "focusa.mission_canvas.domain_pack.install";
pub const DOMAIN_PACK_INSTALL_CAPABILITY: &str = "mission_canvas";
pub const DOMAIN_PACK_INSTALL_PERMISSION: &str = "mission_canvas:write";
pub const DOMAIN_PACK_INSTALL_CONFIRMATION: &str = "confirm";

/// Typed command accepted by the core domain-pack service.
///
/// `scope` retains the complete Mission Canvas presentation/runtime authority
/// chain for persistence. `context` is the canonical actor/authority envelope
/// extracted from the same Workstream and is never reconstructed from CWD,
/// session recency, or a path string.
#[derive(Clone, Debug)]
pub struct DomainPackInstallCommand {
    pub context: WorkstreamContext,
    pub scope: MissionCanvasScope,
    pub pack: DomainPack,
    pub idempotency_key: String,
    pub confirmation: Option<String>,
    pub capabilities: BTreeSet<String>,
    pub permissions: BTreeSet<String>,
}

#[derive(Debug, Error)]
pub enum DomainPackInstallError {
    #[error("domain-pack Workstream context is invalid: {0}")]
    Context(#[from] WorkstreamContextError),
    #[error("domain-pack scope is invalid: {0}")]
    Scope(&'static str),
    #[error("domain-pack operation is unavailable without capability: {0}")]
    CapabilityUnavailable(String),
    #[error("domain-pack operation is unavailable without permission: {0}")]
    PermissionDenied(String),
    #[error("domain-pack installation requires confirmation=confirm")]
    ConfirmationRequired,
    #[error("domain-pack installation requires a non-empty idempotency_key")]
    IdempotencyKeyRequired,
    #[error("domain-pack is invalid: {0}")]
    InvalidPack(String),
    #[error("domain-pack request serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("domain-pack persistence failed: {0}")]
    Store(#[from] MissionCanvasStoreError),
}

/// Stateless core operation service.  Durable idempotency and receipt state
/// live in [`MissionCanvasStore`], so retries remain correct across daemon
/// restarts and multiple API connections.
#[derive(Clone, Copy, Debug, Default)]
pub struct DomainPackInstallService;

impl DomainPackInstallService {
    pub fn validate(
        &self,
        command: &DomainPackInstallCommand,
    ) -> Result<(), DomainPackInstallError> {
        command.context.validate()?;
        command
            .scope
            .validate()
            .map_err(DomainPackInstallError::Scope)?;
        validate_context_scope(&command.context, &command.scope)?;

        if !has_capability(&command.capabilities, DOMAIN_PACK_INSTALL_CAPABILITY) {
            return Err(DomainPackInstallError::CapabilityUnavailable(
                DOMAIN_PACK_INSTALL_CAPABILITY.to_owned(),
            ));
        }
        if !has_permission(&command.permissions, DOMAIN_PACK_INSTALL_PERMISSION) {
            return Err(DomainPackInstallError::PermissionDenied(
                DOMAIN_PACK_INSTALL_PERMISSION.to_owned(),
            ));
        }
        if command.confirmation.as_deref() != Some(DOMAIN_PACK_INSTALL_CONFIRMATION) {
            return Err(DomainPackInstallError::ConfirmationRequired);
        }
        if command.idempotency_key.trim().is_empty() {
            return Err(DomainPackInstallError::IdempotencyKeyRequired);
        }

        let mut registry = CompositionRegistry::builtin();
        registry
            .install_domain_pack(command.pack.clone())
            .map_err(|error| DomainPackInstallError::InvalidPack(error.to_string()))?;
        for entry in &command.pack.registry_entries {
            for required in &entry.required_capabilities {
                if !has_capability(&command.capabilities, required) {
                    return Err(DomainPackInstallError::CapabilityUnavailable(
                        required.clone(),
                    ));
                }
            }
            for required in &entry.required_permissions {
                if !has_permission(&command.permissions, required) {
                    return Err(DomainPackInstallError::PermissionDenied(required.clone()));
                }
            }
        }
        Ok(())
    }

    pub fn install(
        &self,
        store: &MissionCanvasStore,
        command: &DomainPackInstallCommand,
    ) -> Result<DomainPackInstallReceipt, DomainPackInstallError> {
        self.validate(command)?;
        let request_digest = request_digest(command)?;
        let issued_at = Utc::now().to_rfc3339();
        store
            .install_domain_pack(
                &command.scope,
                &command.pack,
                &command.idempotency_key,
                &request_digest,
                &command.context.authority.authority_ref,
                &issued_at,
            )
            .map_err(Into::into)
    }
}

fn validate_context_scope(
    context: &WorkstreamContext,
    scope: &MissionCanvasScope,
) -> Result<(), DomainPackInstallError> {
    if context.workstream != scope.workstream {
        return Err(WorkstreamContextError::WorkstreamMismatch.into());
    }
    let scope_continuity = scope.continuity_id.clone().or_else(|| {
        scope
            .attachment
            .as_ref()
            .and_then(|attachment| attachment.continuity_id.clone())
    });
    if context.continuity_id != scope_continuity {
        return Err(WorkstreamContextError::ContinuityMismatch.into());
    }
    if context.attachment != scope.attachment {
        return Err(WorkstreamContextError::WorkstreamMismatch.into());
    }
    let scope_binding = scope.workspace_binding_id.clone().or_else(|| {
        scope
            .attachment
            .as_ref()
            .map(|attachment| attachment.workspace_binding_id.clone())
    });
    if context.workspace_binding_id != scope_binding {
        return Err(WorkstreamContextError::WorkspaceBindingMismatch.into());
    }
    Ok(())
}

fn has_capability(capabilities: &BTreeSet<String>, required: &str) -> bool {
    capabilities.contains(required)
        || capabilities.contains("mission_canvas:*")
        || capabilities.contains("*")
}

fn has_permission(permissions: &BTreeSet<String>, required: &str) -> bool {
    permissions.contains(required)
        || permissions.contains("mission_canvas:*")
        || permissions.contains("admin:*")
}

fn request_digest(command: &DomainPackInstallCommand) -> Result<String, serde_json::Error> {
    use sha2::{Digest, Sha256};

    let bytes = to_vec(&(
        &command.scope,
        &command.pack,
        &command.idempotency_key,
        &command.context.authority.authority_ref,
    ))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoped_state::ScopeRef as LegacyScopeRef;
    use crate::workstream_context::{
        ActorRef, ActorType, AuthorityContext, WorkstreamRequestEnvelope,
    };
    use crate::workstream_identity::{ScopeRef, WorkstreamId, WorkstreamKey};
    use serde_json::json;

    fn scope() -> MissionCanvasScope {
        let legacy = LegacyScopeRef::project(
            "project:focusa",
            "/workspace/focusa",
            "Focusa",
            "host-a:worktree-main",
        )
        .unwrap();
        let workstream = WorkstreamKey::new(
            ScopeRef::project(legacy).unwrap(),
            WorkstreamId::parse("ws:domain-pack").unwrap(),
        );
        MissionCanvasScope {
            workstream,
            continuity_id: None,
            attachment: None,
            workspace_binding_id: None,
            runtime_object: None,
            work_surface_id: None,
        }
    }

    fn pack() -> DomainPack {
        DomainPack {
            pack_id: "domain.healthcare".into(),
            version: "1.0.0".into(),
            profile: super::super::profiles::WorkspaceProfileDefinition {
                profile_id: "domain.healthcare.clinical".into(),
                revision: 1,
                display_name: "Clinical".into(),
                candidate_contribution_ids: vec!["contribution:clinical-record".into()],
                density: "standard".into(),
                terminology_registry_ref: "registry:terminology:healthcare".into(),
                renderer_registry_ref: "registry:renderer:builtin".into(),
                domain_semantic_binding_registry_ref: None,
                viability_rule_revision: "profile-viability:v1".into(),
                installed: true,
            },
            activities: vec![super::super::profiles::ActivityModeDefinition {
                activity_mode_id: "domain.healthcare.review".into(),
                revision: 1,
                display_name: "Review".into(),
                candidate_contribution_ids: vec!["contribution:clinical-record".into()],
                terminology_overrides_ref: None,
                viability_rule_revision: "activity-viability:v1".into(),
            }],
            registry_entries: vec![super::super::profiles::RegistryDefinition {
                registry_kind: "DomainSemanticBindingRegistry".into(),
                entry_id: "semantics:clinical".into(),
                revision: 1,
                schema_ref: "registry-entry.schema.json".into(),
                payload_ref: "registry-entry:semantics:clinical@1".into(),
                required_capabilities: vec![],
                required_permissions: vec![],
                enabled: true,
                payload: json!({"document":"clinical_record"}),
            }],
        }
    }

    fn command() -> DomainPackInstallCommand {
        let scope = scope();
        let context = WorkstreamContext::extract(WorkstreamRequestEnvelope::for_workstream(
            scope.workstream.clone(),
            ActorRef::new(ActorType::Desktop, "actor:desktop").unwrap(),
            AuthorityContext::canonical("authority:desktop", "desktop supplied exact Workstream"),
        ))
        .unwrap();
        DomainPackInstallCommand {
            context,
            scope,
            pack: pack(),
            idempotency_key: "idempotency:domain-pack:1".into(),
            confirmation: Some("confirm".into()),
            capabilities: [DOMAIN_PACK_INSTALL_CAPABILITY.into()]
                .into_iter()
                .collect(),
            permissions: [DOMAIN_PACK_INSTALL_PERMISSION.into()]
                .into_iter()
                .collect(),
        }
    }

    #[test]
    fn service_requires_exact_controls_before_persistence() {
        let service = DomainPackInstallService;
        let mut request = command();
        request.confirmation = None;
        assert!(matches!(
            service.validate(&request),
            Err(DomainPackInstallError::ConfirmationRequired)
        ));

        let mut request = command();
        request.capabilities.clear();
        assert!(matches!(
            service.validate(&request),
            Err(DomainPackInstallError::CapabilityUnavailable(_))
        ));

        let mut request = command();
        request.idempotency_key.clear();
        assert!(matches!(
            service.validate(&request),
            Err(DomainPackInstallError::IdempotencyKeyRequired)
        ));
    }

    #[test]
    fn service_rejects_foreign_context_before_pack_install() {
        let mut command = command();
        let other = WorkstreamKey::new(
            command.scope.workstream.scope.clone(),
            WorkstreamId::parse("ws:foreign").unwrap(),
        );
        command.context.workstream = other;
        assert!(matches!(
            DomainPackInstallService.validate(&command),
            Err(DomainPackInstallError::Context(
                WorkstreamContextError::WorkstreamMismatch
            ))
        ));
    }
}
