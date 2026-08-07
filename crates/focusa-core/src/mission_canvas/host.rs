//! Core-owned rich-host resolution for the Focusa Desktop Mission Canvas.
//!
//! Host selection is a read operation over an already-resolved Workstream
//! context.  The API adapter supplies the generated capability projection and
//! exact authority; this module does not inspect CWD, tabs, recent records, or
//! any other presentation-local fallback.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::workstream_context::{WorkstreamContext, WorkstreamContextError};

use super::model::MissionCanvasScope;

pub const RICH_HOST_RESOLVE_OPERATION: &str = "focusa.mission_canvas.rich_host.resolve";
/// Capability advertised by the generated Desktop host client.  The short
/// alias is retained because existing capability projections use
/// `mission_canvas` for the complete host surface.
pub const RICH_HOST_RESOLVE_CAPABILITY: &str = "mission_canvas";
pub const DESKTOP_TAURI_CAPABILITY: &str = "mission_canvas.desktop_tauri";
pub const PI_OVERLAY_COMPATIBILITY_CAPABILITY: &str = "mission_canvas.pi_overlay";
pub const DESKTOP_TAURI_RENDERER: &str = "focusa_desktop_tauri";
pub const PI_OVERLAY_RENDERER: &str = "focusa_pi_rich_window";
pub const HOST_RESOLVER_REVISION: &str = "host-resolver:v2";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostPlatform {
    MacOS,
    Windows,
    Linux,
}

impl HostPlatform {
    pub fn current() -> Self {
        #[cfg(target_os = "macos")]
        {
            return Self::MacOS;
        }
        #[cfg(target_os = "windows")]
        {
            return Self::Windows;
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Self::Linux
        }
    }

    fn contract_value(self) -> &'static str {
        match self {
            Self::MacOS => "macOS",
            Self::Windows => "Windows",
            Self::Linux => "Linux",
        }
    }
}

/// Generated-contract-shaped result with the exact Workstream authority
/// echoed for Desktop response validation.  Subordinate identity is echoed
/// when the request supplied it; it never becomes an alternate owner.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostRendererResolution {
    #[serde(flatten)]
    pub scope: MissionCanvasScope,
    pub interaction_mode: String,
    pub selected_renderer: String,
    pub platform: String,
    pub availability: String,
    pub resolution_reason: String,
    pub asset_version: Option<String>,
    pub asset_digest: Option<String>,
    pub resolver_revision: String,
    pub diagnostic_ref: Option<String>,
}

#[derive(Debug, Error)]
pub enum HostRendererResolutionError {
    #[error("rich-host Workstream context is invalid: {0}")]
    Context(#[from] WorkstreamContextError),
    #[error("rich-host scope is invalid: {0}")]
    Scope(&'static str),
    #[error("rich-host resolution is unavailable without capability: {0}")]
    CapabilityUnavailable(String),
}

/// Stateless core service for `focusa.mission_canvas.rich_host.resolve`.
#[derive(Clone, Copy, Debug, Default)]
pub struct HostRendererResolutionService;

impl HostRendererResolutionService {
    pub fn resolve(
        &self,
        context: &WorkstreamContext,
        scope: &MissionCanvasScope,
        capabilities: &BTreeSet<String>,
        platform: HostPlatform,
    ) -> Result<HostRendererResolution, HostRendererResolutionError> {
        context.validate()?;
        scope
            .validate()
            .map_err(HostRendererResolutionError::Scope)?;
        validate_context_scope(context, scope)?;

        let (interaction_mode, selected_renderer, availability, resolution_reason) =
            if has_desktop_capability(capabilities) {
                (
                    "canvas-guided",
                    DESKTOP_TAURI_RENDERER,
                    "available",
                    "Focusa Desktop Tauri is the primary Mission Canvas host; Pi overlay is compatibility-only",
                )
            } else if has_pi_compatibility_capability(capabilities) {
                (
                    "canvas-guided",
                    PI_OVERLAY_RENDERER,
                    "fallback",
                    "Focusa Desktop Tauri is unavailable; Pi overlay is a compatibility-only fallback",
                )
            } else {
                return Err(HostRendererResolutionError::CapabilityUnavailable(
                    RICH_HOST_RESOLVE_CAPABILITY.to_owned(),
                ));
            };

        Ok(HostRendererResolution {
            scope: scope.clone(),
            interaction_mode: interaction_mode.into(),
            selected_renderer: selected_renderer.into(),
            platform: platform.contract_value().into(),
            availability: availability.into(),
            resolution_reason: resolution_reason.into(),
            asset_version: Some(env!("CARGO_PKG_VERSION").into()),
            asset_digest: None,
            resolver_revision: HOST_RESOLVER_REVISION.into(),
            diagnostic_ref: None,
        })
    }
}

fn has_desktop_capability(capabilities: &BTreeSet<String>) -> bool {
    capabilities.contains(RICH_HOST_RESOLVE_CAPABILITY)
        || capabilities.contains(DESKTOP_TAURI_CAPABILITY)
        || capabilities.contains("mission_canvas:*")
        || capabilities.contains("*")
}

fn has_pi_compatibility_capability(capabilities: &BTreeSet<String>) -> bool {
    capabilities.contains(PI_OVERLAY_COMPATIBILITY_CAPABILITY)
        || capabilities.contains(PI_OVERLAY_RENDERER)
        || capabilities.contains("pi:mission_canvas")
}

fn validate_context_scope(
    context: &WorkstreamContext,
    scope: &MissionCanvasScope,
) -> Result<(), HostRendererResolutionError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoped_state::ScopeRef as LegacyScopeRef;
    use crate::workstream_context::{
        ActorRef, ActorType, AuthorityContext, WorkstreamRequestEnvelope,
    };
    use crate::workstream_identity::{ScopeRef, WorkstreamId, WorkstreamKey};

    fn workstream(id: &str) -> WorkstreamKey {
        let legacy = LegacyScopeRef::project(
            "project:focusa",
            "/workspace/focusa",
            "Focusa",
            "host-a:worktree-main",
        )
        .unwrap();
        WorkstreamKey::new(
            ScopeRef::project(legacy).unwrap(),
            WorkstreamId::parse(id).unwrap(),
        )
    }

    fn request(owner: WorkstreamKey) -> WorkstreamContext {
        WorkstreamContext::extract(WorkstreamRequestEnvelope::for_workstream(
            owner,
            ActorRef::new(ActorType::Desktop, "actor:desktop").unwrap(),
            AuthorityContext::canonical("authority:desktop", "exact Desktop Workstream authority"),
        ))
        .unwrap()
    }

    #[test]
    fn desktop_is_primary_and_pi_is_only_compatibility_fallback() {
        let owner = workstream("ws:host");
        let context = request(owner.clone());
        let desktop = HostRendererResolutionService
            .resolve(
                &context,
                &MissionCanvasScope::new(owner.clone(), None).unwrap(),
                &[RICH_HOST_RESOLVE_CAPABILITY.into()].into_iter().collect(),
                HostPlatform::MacOS,
            )
            .unwrap();
        assert_eq!(desktop.selected_renderer, DESKTOP_TAURI_RENDERER);
        assert_eq!(desktop.interaction_mode, "canvas-guided");

        let fallback = HostRendererResolutionService
            .resolve(
                &context,
                &MissionCanvasScope::new(owner, None).unwrap(),
                &[PI_OVERLAY_COMPATIBILITY_CAPABILITY.into()]
                    .into_iter()
                    .collect(),
                HostPlatform::MacOS,
            )
            .unwrap();
        assert_eq!(fallback.selected_renderer, PI_OVERLAY_RENDERER);
        assert_eq!(fallback.interaction_mode, "canvas-guided");
        assert_eq!(fallback.availability, "fallback");
    }

    #[test]
    fn missing_capability_and_foreign_workstream_fail_closed() {
        let owner = workstream("ws:host");
        let context = request(owner.clone());
        let scope = MissionCanvasScope::new(owner.clone(), None).unwrap();
        assert!(matches!(
            HostRendererResolutionService.resolve(
                &context,
                &scope,
                &BTreeSet::new(),
                HostPlatform::Linux,
            ),
            Err(HostRendererResolutionError::CapabilityUnavailable(_))
        ));

        let foreign = MissionCanvasScope::new(workstream("ws:foreign"), None).unwrap();
        assert!(matches!(
            HostRendererResolutionService.resolve(
                &context,
                &foreign,
                &[RICH_HOST_RESOLVE_CAPABILITY.into()].into_iter().collect(),
                HostPlatform::Linux,
            ),
            Err(HostRendererResolutionError::Context(
                WorkstreamContextError::WorkstreamMismatch
            ))
        ));
    }
}
