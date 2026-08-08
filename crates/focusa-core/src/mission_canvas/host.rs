//! Core-owned rich-host resolution and lifecycle for the Focusa Desktop Mission
//! Canvas.
//!
//! Host selection and lifecycle actions operate over an already-resolved
//! Workstream context. The API adapter supplies the generated capability
//! projection and exact authority; this module does not inspect CWD, tabs,
//! recent records, or any other presentation-local fallback.

use std::collections::BTreeSet;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::workstream_context::{WorkstreamContext, WorkstreamContextError};

use super::{
    CompositionEvent, MissionCanvasScope, MissionCanvasStore, MissionCanvasStoreError,
    StoredDocument,
};

pub const RICH_HOST_RESOLVE_OPERATION: &str = "focusa.mission_canvas.rich_host.resolve";
pub const RICH_HOST_LAUNCH_OPERATION: &str = "focusa.mission_canvas.rich_host.launch";
pub const RICH_HOST_FOCUS_OPERATION: &str = "focusa.mission_canvas.rich_host.focus";
pub const RICH_HOST_HIDE_OPERATION: &str = "focusa.mission_canvas.rich_host.hide";
pub const RICH_HOST_CLOSE_OPERATION: &str = "focusa.mission_canvas.rich_host.close";
pub const RICH_HOST_PERMISSION: &str = "mission_canvas:host";
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

/// Generated-contract-shaped lifecycle state returned by a rich-host mutation.
/// The state is still a projection: Desktop owns presentation, while the
/// Workstream, renderer resolution, and durable cursor remain Core-owned.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostLifecycleState {
    #[serde(flatten)]
    pub scope: MissionCanvasScope,
    pub host_instance_id: String,
    pub renderer_resolution: HostRendererResolution,
    pub state: String,
    pub focused: bool,
    pub process_id: Option<u32>,
    pub window_id: Option<String>,
    pub pi_draft_ref: Option<String>,
    pub canvas_draft_ref: Option<String>,
    pub last_error_ref: Option<String>,
    pub durable_event_cursor: String,
    pub lifecycle_revision: u64,
    pub updated_at: String,
}

impl HostLifecycleState {
    /// Validate the complete authority packet, including the nested renderer
    /// resolution. A Workstream match alone is not enough when the request
    /// carries Attachment, runtime, or Work Surface identity.
    pub fn validate_scope(&self, expected: &MissionCanvasScope) -> Result<(), &'static str> {
        self.scope.validate()?;
        expected.validate()?;
        if self.scope != *expected {
            return Err("host_lifecycle_scope_mismatch");
        }
        if self.renderer_resolution.scope != self.scope {
            return Err("host_renderer_scope_mismatch");
        }
        if !self.host_instance_id.starts_with("rich-host:")
            || self.host_instance_id.len() <= "rich-host:".len()
            || !self.host_instance_id["rich-host:".len()..]
                .chars()
                .all(|value| {
                    value.is_ascii_lowercase() || value.is_ascii_digit() || "._:-".contains(value)
                })
        {
            return Err("host_instance_invalid");
        }
        if !matches!(
            self.state.as_str(),
            "absent"
                | "launching"
                | "visible"
                | "focused"
                | "hidden"
                | "closing"
                | "reconnecting"
                | "failed"
        ) {
            return Err("host_state_invalid");
        }
        if self.durable_event_cursor.trim().is_empty() {
            return Err("host_event_cursor_missing");
        }
        if self.updated_at.trim().is_empty() {
            return Err("host_updated_at_missing");
        }
        Ok(())
    }
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

/// Exact generated input for the launch mutation.  The API adapter supplies
/// the Workstream context and capability/permission projections; it does not
/// manufacture a host owner from a path, tab, or current process.
#[derive(Clone, Debug)]
pub struct HostLifecycleLaunchCommand {
    pub context: WorkstreamContext,
    pub scope: MissionCanvasScope,
    pub idempotency_key: String,
    pub capabilities: BTreeSet<String>,
    pub permissions: BTreeSet<String>,
}

/// Exact generated input for the focus mutation.  Focus is deliberately a
/// separate command from launch: it may only transition an already persisted
/// Desktop presentation and must never create one as a fallback.
#[derive(Clone, Debug)]
pub struct HostLifecycleFocusCommand {
    pub context: WorkstreamContext,
    pub scope: MissionCanvasScope,
    pub idempotency_key: String,
    pub capabilities: BTreeSet<String>,
    pub permissions: BTreeSet<String>,
}

/// Exact generated input for the hide mutation.  Hiding transitions a persisted
/// presentation into the background without closing process ownership or drafts.
#[derive(Clone, Debug)]
pub struct HostLifecycleHideCommand {
    pub context: WorkstreamContext,
    pub scope: MissionCanvasScope,
    pub idempotency_key: String,
    pub capabilities: BTreeSet<String>,
    pub permissions: BTreeSet<String>,
}

/// Exact generated input for the close mutation.  Closing is a lifecycle end
/// transition that avoids destruction: the projection document remains owned by
/// the exact Workstream and only the presentation state changes.
#[derive(Clone, Debug)]
pub struct HostLifecycleCloseCommand {
    pub context: WorkstreamContext,
    pub scope: MissionCanvasScope,
    pub idempotency_key: String,
    pub capabilities: BTreeSet<String>,
    pub permissions: BTreeSet<String>,
}

#[derive(Debug, Error)]
pub enum HostLifecycleError {
    #[error("rich-host Workstream context is invalid: {0}")]
    Context(#[from] WorkstreamContextError),
    #[error("rich-host scope is invalid: {0}")]
    Scope(&'static str),
    #[error("rich-host capability is unavailable: {0}")]
    CapabilityUnavailable(String),
    #[error("rich-host permission is unavailable: {0}")]
    PermissionDenied(String),
    #[error("rich-host lifecycle mutation requires a non-empty idempotency_key")]
    IdempotencyKeyRequired,
    #[error("rich-host presentation does not exist for the exact Workstream")]
    PresentationNotFound,
    #[error("rich-host presentation cannot be focused: {0}")]
    PresentationUnavailable(String),
    #[error("rich-host focus requires the existing Desktop renderer: {0}")]
    RendererUnavailable(String),
    #[error("rich-host idempotency key conflicts with an existing lifecycle action")]
    IdempotencyConflict,
    #[error("rich-host resolution failed: {0}")]
    Resolution(#[from] HostRendererResolutionError),
    #[error("rich-host lifecycle document is invalid: {0}")]
    InvalidDocument(String),
    #[error("rich-host serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("rich-host persistence failed: {0}")]
    Store(#[from] MissionCanvasStoreError),
}

/// Core-owned launch service.  It records one attachment/workstream-scoped
/// lifecycle projection and returns the generated HostLifecycleState.  It does
/// not spawn Pi, fork a model stream, choose layout, or infer contributions.
#[derive(Clone, Copy, Debug, Default)]
pub struct HostLifecycleService;

impl HostLifecycleService {
    pub fn validate(
        &self,
        command: &HostLifecycleLaunchCommand,
        platform: HostPlatform,
    ) -> Result<HostRendererResolution, HostLifecycleError> {
        self.validate_command(
            &command.context,
            &command.scope,
            &command.idempotency_key,
            &command.capabilities,
            &command.permissions,
            platform,
        )
    }

    pub fn validate_focus(
        &self,
        command: &HostLifecycleFocusCommand,
        platform: HostPlatform,
    ) -> Result<HostRendererResolution, HostLifecycleError> {
        let resolution = self.validate_command(
            &command.context,
            &command.scope,
            &command.idempotency_key,
            &command.capabilities,
            &command.permissions,
            platform,
        )?;
        if resolution.selected_renderer != DESKTOP_TAURI_RENDERER {
            return Err(HostLifecycleError::RendererUnavailable(
                "the existing presentation is not Focusa Desktop Tauri".into(),
            ));
        }
        Ok(resolution)
    }

    pub fn hide(
        &self,
        store: &MissionCanvasStore,
        command: &HostLifecycleHideCommand,
        platform: HostPlatform,
    ) -> Result<HostLifecycleState, HostLifecycleError> {
        let _resolution = self.validate_command(
            &command.context,
            &command.scope,
            &command.idempotency_key,
            &command.capabilities,
            &command.permissions,
            platform,
        )?;
        let host_instance_id = stable_host_instance_id(&command.scope)?;
        let document_id = format!("host:{host_instance_id}");
        let existing = store.get_document(
            "mission_canvas_host_lifecycle",
            &command.scope,
            &document_id,
        )?;
        let Some(document) = existing else {
            return Err(HostLifecycleError::PresentationNotFound);
        };
        let (previous_state, previous_idempotency_key) =
            lifecycle_document(&document, &command.scope)?;
        if previous_state.lifecycle_revision != document.revision {
            return Err(HostLifecycleError::InvalidDocument(
                "document revision does not match lifecycle revision".into(),
            ));
        }
        if previous_state.host_instance_id != host_instance_id {
            return Err(HostLifecycleError::InvalidDocument(
                "host instance is not derived from the exact Workstream scope".into(),
            ));
        }
        if matches!(
            previous_state.state.as_str(),
            "absent" | "closing" | "failed"
        ) {
            return Err(HostLifecycleError::PresentationUnavailable(
                previous_state.state.clone(),
            ));
        }
        if previous_idempotency_key == command.idempotency_key {
            if previous_state.state == "hidden" && !previous_state.focused {
                return Ok(previous_state);
            }
            return Err(HostLifecycleError::IdempotencyConflict);
        }

        let lifecycle_revision = previous_state
            .lifecycle_revision
            .checked_add(1)
            .ok_or_else(|| {
                HostLifecycleError::InvalidDocument("lifecycle revision overflow".into())
            })?;
        let now = Utc::now().to_rfc3339();
        let state = HostLifecycleState {
            scope: command.scope.clone(),
            host_instance_id: host_instance_id.clone(),
            renderer_resolution: previous_state.renderer_resolution.clone(),
            state: "hidden".into(),
            focused: false,
            process_id: previous_state.process_id,
            window_id: previous_state.window_id.clone(),
            pi_draft_ref: previous_state.pi_draft_ref.clone(),
            canvas_draft_ref: previous_state.canvas_draft_ref.clone(),
            last_error_ref: previous_state.last_error_ref.clone(),
            durable_event_cursor: "event:pending".into(),
            lifecycle_revision,
            updated_at: now.clone(),
        };
        state
            .validate_scope(&command.scope)
            .map_err(HostLifecycleError::Scope)?;
        let event = CompositionEvent {
            event_id: format!(
                "projection-event:rich-host-hide:{}:{}",
                host_instance_id.replace(':', "-"),
                digest_fragment(&command.idempotency_key),
            ),
            event_kind: "host_hidden".into(),
            scope: command.scope.clone(),
            // Hiding changes lifecycle ownership only; canonical composition
            // revision is unchanged.
            projection_revision: 0,
            layout_revision: 0,
            causation_id: Some(command.idempotency_key.clone()),
            correlation_id: Some(command.context.authority.authority_ref.clone()),
            occurred_at: now.clone(),
            payload: json!({
                "operation_id": RICH_HOST_HIDE_OPERATION,
                "host_instance_id": host_instance_id,
                "renderer": state.renderer_resolution.selected_renderer.clone(),
                "lifecycle_revision": lifecycle_revision,
            }),
            evidence_refs: vec![],
            receipt_refs: vec![format!("receipt:rich-host-hide:{lifecycle_revision}")],
        };
        let document = StoredDocument {
            document_id,
            scope: command.scope.clone(),
            revision: lifecycle_revision,
            payload: json!({
                "idempotency_key": command.idempotency_key,
                "state": state,
            }),
            updated_at: now,
        };
        let persisted =
            store.put_idempotent_lifecycle_document(&document, &command.idempotency_key, &event)?;
        let (persisted_state, _) = lifecycle_document(&persisted, &command.scope)?;
        Ok(persisted_state)
    }

    pub fn close(
        &self,
        store: &MissionCanvasStore,
        command: &HostLifecycleCloseCommand,
        platform: HostPlatform,
    ) -> Result<HostLifecycleState, HostLifecycleError> {
        let _resolution = self.validate_command(
            &command.context,
            &command.scope,
            &command.idempotency_key,
            &command.capabilities,
            &command.permissions,
            platform,
        )?;
        let host_instance_id = stable_host_instance_id(&command.scope)?;
        let document_id = format!("host:{host_instance_id}");
        let existing = store.get_document(
            "mission_canvas_host_lifecycle",
            &command.scope,
            &document_id,
        )?;
        let Some(document) = existing else {
            return Err(HostLifecycleError::PresentationNotFound);
        };
        let (previous_state, previous_idempotency_key) =
            lifecycle_document(&document, &command.scope)?;
        if previous_state.lifecycle_revision != document.revision {
            return Err(HostLifecycleError::InvalidDocument(
                "document revision does not match lifecycle revision".into(),
            ));
        }
        if previous_state.host_instance_id != host_instance_id {
            return Err(HostLifecycleError::InvalidDocument(
                "host instance is not derived from the exact Workstream scope".into(),
            ));
        }
        if matches!(previous_state.state.as_str(), "absent" | "failed") {
            return Err(HostLifecycleError::PresentationUnavailable(
                previous_state.state.clone(),
            ));
        }
        if previous_state.state == "closing" {
            if previous_idempotency_key == command.idempotency_key && !previous_state.focused {
                return Ok(previous_state);
            }
            return Err(HostLifecycleError::PresentationUnavailable(
                previous_state.state.clone(),
            ));
        }
        if previous_idempotency_key == command.idempotency_key {
            return Err(HostLifecycleError::IdempotencyConflict);
        }

        let lifecycle_revision = previous_state
            .lifecycle_revision
            .checked_add(1)
            .ok_or_else(|| {
                HostLifecycleError::InvalidDocument("lifecycle revision overflow".into())
            })?;
        let now = Utc::now().to_rfc3339();
        let state = HostLifecycleState {
            scope: command.scope.clone(),
            host_instance_id: host_instance_id.clone(),
            renderer_resolution: previous_state.renderer_resolution.clone(),
            state: "closing".into(),
            focused: false,
            process_id: previous_state.process_id,
            window_id: previous_state.window_id.clone(),
            pi_draft_ref: previous_state.pi_draft_ref.clone(),
            canvas_draft_ref: previous_state.canvas_draft_ref.clone(),
            last_error_ref: previous_state.last_error_ref.clone(),
            durable_event_cursor: "event:pending".into(),
            lifecycle_revision,
            updated_at: now.clone(),
        };
        state
            .validate_scope(&command.scope)
            .map_err(HostLifecycleError::Scope)?;
        let event = CompositionEvent {
            event_id: format!(
                "projection-event:rich-host-close:{}:{}",
                host_instance_id.replace(':', "-"),
                digest_fragment(&command.idempotency_key),
            ),
            event_kind: "host_closed".into(),
            scope: command.scope.clone(),
            // Closing changes lifecycle visibility only; canonical composition
            // revision is unchanged.
            projection_revision: 0,
            layout_revision: 0,
            causation_id: Some(command.idempotency_key.clone()),
            correlation_id: Some(command.context.authority.authority_ref.clone()),
            occurred_at: now.clone(),
            payload: json!({
                "operation_id": RICH_HOST_CLOSE_OPERATION,
                "host_instance_id": host_instance_id,
                "renderer": state.renderer_resolution.selected_renderer.clone(),
                "lifecycle_revision": lifecycle_revision,
            }),
            evidence_refs: vec![],
            receipt_refs: vec![format!("receipt:rich-host-close:{lifecycle_revision}")],
        };
        let document = StoredDocument {
            document_id,
            scope: command.scope.clone(),
            revision: lifecycle_revision,
            payload: json!({
                "idempotency_key": command.idempotency_key,
                "state": state,
            }),
            updated_at: now,
        };
        let persisted =
            store.put_idempotent_lifecycle_document(&document, &command.idempotency_key, &event)?;
        let (persisted_state, _) = lifecycle_document(&persisted, &command.scope)?;
        Ok(persisted_state)
    }

    fn validate_command(
        &self,
        context: &WorkstreamContext,
        scope: &MissionCanvasScope,
        idempotency_key: &str,
        capabilities: &BTreeSet<String>,
        permissions: &BTreeSet<String>,
        platform: HostPlatform,
    ) -> Result<HostRendererResolution, HostLifecycleError> {
        context.validate()?;
        scope.validate().map_err(HostLifecycleError::Scope)?;
        validate_context_scope(context, scope).map_err(HostLifecycleError::Resolution)?;
        if !has_permission(permissions, RICH_HOST_PERMISSION) {
            return Err(HostLifecycleError::PermissionDenied(
                RICH_HOST_PERMISSION.to_owned(),
            ));
        }
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 200 {
            return Err(HostLifecycleError::IdempotencyKeyRequired);
        }
        HostRendererResolutionService
            .resolve(context, scope, capabilities, platform)
            .map_err(Into::into)
    }

    pub fn launch(
        &self,
        store: &MissionCanvasStore,
        command: &HostLifecycleLaunchCommand,
        platform: HostPlatform,
    ) -> Result<HostLifecycleState, HostLifecycleError> {
        let resolution = self.validate(command, platform)?;
        let host_instance_id = stable_host_instance_id(&command.scope)?;
        let document_id = format!("host:{host_instance_id}");
        let existing = store.get_document(
            "mission_canvas_host_lifecycle",
            &command.scope,
            &document_id,
        )?;
        let (previous_state, previous_idempotency_key) = match existing {
            Some(document) => {
                let (state, key) = lifecycle_document(&document, &command.scope)?;
                if state.lifecycle_revision != document.revision {
                    return Err(HostLifecycleError::InvalidDocument(
                        "document revision does not match lifecycle revision".into(),
                    ));
                }
                if state.host_instance_id != host_instance_id {
                    return Err(HostLifecycleError::InvalidDocument(
                        "host instance is not derived from the exact Workstream scope".into(),
                    ));
                }
                (Some(state), Some(key))
            }
            None => (None, None),
        };

        if previous_idempotency_key.as_deref() == Some(command.idempotency_key.as_str()) {
            return Ok(previous_state.expect("idempotent lifecycle document has state"));
        }

        let lifecycle_revision = previous_state
            .as_ref()
            .map(|state| {
                state.lifecycle_revision.checked_add(1).ok_or_else(|| {
                    HostLifecycleError::InvalidDocument("lifecycle revision overflow".into())
                })
            })
            .transpose()?
            .unwrap_or(1);
        let now = Utc::now().to_rfc3339();
        let state = HostLifecycleState {
            scope: command.scope.clone(),
            host_instance_id: host_instance_id.clone(),
            renderer_resolution: resolution,
            state: "visible".into(),
            focused: true,
            // The Desktop host owns its own process/window lifecycle.  The
            // launch operation presents Desktop without forking Pi or claiming a Pi process id.
            process_id: None,
            window_id: Some(format!("window:{host_instance_id}")),
            pi_draft_ref: None,
            canvas_draft_ref: None,
            last_error_ref: None,
            durable_event_cursor: "event:pending".into(),
            lifecycle_revision,
            updated_at: now.clone(),
        };
        state
            .validate_scope(&command.scope)
            .map_err(HostLifecycleError::Scope)?;
        let event = CompositionEvent {
            event_id: format!(
                "projection-event:rich-host-launch:{}:{}",
                host_instance_id.replace(':', "-"),
                digest_fragment(&command.idempotency_key),
            ),
            event_kind: "host_launched".into(),
            scope: command.scope.clone(),
            projection_revision: 0,
            layout_revision: 0,
            causation_id: Some(command.idempotency_key.clone()),
            correlation_id: Some(command.context.authority.authority_ref.clone()),
            occurred_at: now.clone(),
            payload: json!({
                "operation_id": RICH_HOST_LAUNCH_OPERATION,
                "host_instance_id": host_instance_id,
                "renderer": state.renderer_resolution.selected_renderer.clone(),
                "lifecycle_revision": lifecycle_revision,
            }),
            evidence_refs: vec![],
            receipt_refs: vec![format!("receipt:rich-host-launch:{lifecycle_revision}")],
        };
        let document = StoredDocument {
            document_id,
            scope: command.scope.clone(),
            revision: lifecycle_revision,
            payload: json!({
                "idempotency_key": command.idempotency_key,
                "state": state,
            }),
            updated_at: now,
        };
        let persisted =
            store.put_idempotent_lifecycle_document(&document, &command.idempotency_key, &event)?;
        let (persisted_state, _) = lifecycle_document(&persisted, &command.scope)?;
        Ok(persisted_state)
    }

    /// Focus an already persisted Desktop presentation.  This operation only
    /// mutates the host lifecycle projection without changing canonical activity;
    /// it never resolves composition, creates a Work Surface, or launches a replacement.
    pub fn focus(
        &self,
        store: &MissionCanvasStore,
        command: &HostLifecycleFocusCommand,
        platform: HostPlatform,
    ) -> Result<HostLifecycleState, HostLifecycleError> {
        let _resolution = self.validate_focus(command, platform)?;
        let host_instance_id = stable_host_instance_id(&command.scope)?;
        let document_id = format!("host:{host_instance_id}");
        let existing = store.get_document(
            "mission_canvas_host_lifecycle",
            &command.scope,
            &document_id,
        )?;
        let Some(document) = existing else {
            return Err(HostLifecycleError::PresentationNotFound);
        };
        let (previous_state, previous_idempotency_key) =
            lifecycle_document(&document, &command.scope)?;
        if previous_state.lifecycle_revision != document.revision {
            return Err(HostLifecycleError::InvalidDocument(
                "document revision does not match lifecycle revision".into(),
            ));
        }
        if previous_state.host_instance_id != host_instance_id {
            return Err(HostLifecycleError::InvalidDocument(
                "host instance is not derived from the exact Workstream scope".into(),
            ));
        }
        if previous_state.renderer_resolution.selected_renderer != DESKTOP_TAURI_RENDERER {
            return Err(HostLifecycleError::RendererUnavailable(format!(
                "persisted renderer is {}",
                previous_state.renderer_resolution.selected_renderer
            )));
        }
        if matches!(
            previous_state.state.as_str(),
            "absent" | "closing" | "failed"
        ) {
            return Err(HostLifecycleError::PresentationUnavailable(
                previous_state.state.clone(),
            ));
        }
        if previous_idempotency_key == command.idempotency_key {
            if previous_state.state == "focused" && previous_state.focused {
                return Ok(previous_state);
            }
            return Err(HostLifecycleError::IdempotencyConflict);
        }

        let lifecycle_revision = previous_state
            .lifecycle_revision
            .checked_add(1)
            .ok_or_else(|| {
                HostLifecycleError::InvalidDocument("lifecycle revision overflow".into())
            })?;
        let now = Utc::now().to_rfc3339();
        let state = HostLifecycleState {
            scope: command.scope.clone(),
            host_instance_id: host_instance_id.clone(),
            renderer_resolution: previous_state.renderer_resolution.clone(),
            state: "focused".into(),
            focused: true,
            process_id: previous_state.process_id,
            window_id: previous_state.window_id.clone(),
            pi_draft_ref: previous_state.pi_draft_ref.clone(),
            canvas_draft_ref: previous_state.canvas_draft_ref.clone(),
            last_error_ref: previous_state.last_error_ref.clone(),
            durable_event_cursor: "event:pending".into(),
            lifecycle_revision,
            updated_at: now.clone(),
        };
        state
            .validate_scope(&command.scope)
            .map_err(HostLifecycleError::Scope)?;
        let event = CompositionEvent {
            event_id: format!(
                "projection-event:rich-host-focus:{}:{}",
                host_instance_id.replace(':', "-"),
                digest_fragment(&command.idempotency_key),
            ),
            event_kind: "host_focused".into(),
            scope: command.scope.clone(),
            // Focusing a presentation must not advance canonical composition
            // or activity revisions.
            projection_revision: 0,
            layout_revision: 0,
            causation_id: Some(command.idempotency_key.clone()),
            correlation_id: Some(command.context.authority.authority_ref.clone()),
            occurred_at: now.clone(),
            payload: json!({
                "operation_id": RICH_HOST_FOCUS_OPERATION,
                "host_instance_id": host_instance_id,
                "renderer": state.renderer_resolution.selected_renderer.clone(),
                "lifecycle_revision": lifecycle_revision,
                "canonical_activity_changed": false,
            }),
            evidence_refs: vec![],
            receipt_refs: vec![format!("receipt:rich-host-focus:{lifecycle_revision}")],
        };
        let document = StoredDocument {
            document_id,
            scope: command.scope.clone(),
            revision: lifecycle_revision,
            payload: json!({
                "idempotency_key": command.idempotency_key,
                "state": state,
            }),
            updated_at: now,
        };
        let persisted =
            store.put_idempotent_lifecycle_document(&document, &command.idempotency_key, &event)?;
        let (persisted_state, _) = lifecycle_document(&persisted, &command.scope)?;
        Ok(persisted_state)
    }
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

fn lifecycle_document(
    document: &StoredDocument,
    expected_scope: &MissionCanvasScope,
) -> Result<(HostLifecycleState, String), HostLifecycleError> {
    let object = document.payload.as_object().ok_or_else(|| {
        HostLifecycleError::InvalidDocument("lifecycle envelope is not an object".into())
    })?;
    let idempotency_key = object
        .get("idempotency_key")
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            HostLifecycleError::InvalidDocument("lifecycle idempotency key is missing".into())
        })?
        .to_owned();
    let state_value = object
        .get("state")
        .cloned()
        .ok_or_else(|| HostLifecycleError::InvalidDocument("lifecycle state is missing".into()))?;
    let state: HostLifecycleState = serde_json::from_value(state_value)?;
    state
        .validate_scope(expected_scope)
        .map_err(HostLifecycleError::Scope)?;
    Ok((state, idempotency_key))
}

fn stable_host_instance_id(scope: &MissionCanvasScope) -> Result<String, HostLifecycleError> {
    let bytes = serde_json::to_vec(scope)?;
    let digest = Sha256::digest(bytes);
    Ok(format!("rich-host:desktop:{}", hex::encode(&digest[..16])))
}

fn digest_fragment(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    hex::encode(&digest[..16])
}

fn has_permission(permissions: &BTreeSet<String>, required: &str) -> bool {
    permissions.contains(required)
        || permissions.contains("mission_canvas:*")
        || permissions.contains("admin:*")
        || permissions.contains("*")
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

    #[test]
    fn launch_validation_requires_permission_capability_and_idempotency() {
        let owner = workstream("ws:launch");
        let context = request(owner.clone());
        let scope = MissionCanvasScope::new(owner.clone(), None).unwrap();
        let service = HostLifecycleService;
        let mut command = HostLifecycleLaunchCommand {
            context: context.clone(),
            scope: scope.clone(),
            idempotency_key: "launch:1".into(),
            capabilities: [RICH_HOST_RESOLVE_CAPABILITY.into()].into_iter().collect(),
            permissions: BTreeSet::new(),
        };

        assert!(matches!(
            service.validate(&command, HostPlatform::Linux),
            Err(HostLifecycleError::PermissionDenied(_))
        ));
        command.permissions.insert(RICH_HOST_PERMISSION.into());
        command.capabilities.clear();
        assert!(matches!(
            service.validate(&command, HostPlatform::Linux),
            Err(HostLifecycleError::Resolution(
                HostRendererResolutionError::CapabilityUnavailable(_)
            ))
        ));
        command
            .capabilities
            .insert(RICH_HOST_RESOLVE_CAPABILITY.into());
        command.idempotency_key.clear();
        assert!(matches!(
            service.validate(&command, HostPlatform::Linux),
            Err(HostLifecycleError::IdempotencyKeyRequired)
        ));

        command.idempotency_key = "launch:foreign".into();
        command.scope = MissionCanvasScope::new(workstream("ws:foreign"), None).unwrap();
        assert!(matches!(
            service.validate(&command, HostPlatform::Linux),
            Err(HostLifecycleError::Resolution(
                HostRendererResolutionError::Context(WorkstreamContextError::WorkstreamMismatch)
            ))
        ));
        assert_eq!(context.workstream, owner);
    }

    #[test]
    fn focus_requires_existing_desktop_and_preserves_canonical_activity() {
        let owner = workstream("ws:focus");
        let scope = MissionCanvasScope::new(owner.clone(), None).unwrap();
        let context = request(owner);
        let service = HostLifecycleService;
        let store = MissionCanvasStore::open_in_memory().unwrap();
        let focus_command = HostLifecycleFocusCommand {
            context: context.clone(),
            scope: scope.clone(),
            idempotency_key: "focus:1".into(),
            capabilities: [DESKTOP_TAURI_CAPABILITY.into()].into_iter().collect(),
            permissions: [RICH_HOST_PERMISSION.into()].into_iter().collect(),
        };

        assert!(matches!(
            service.focus(&store, &focus_command, HostPlatform::MacOS),
            Err(HostLifecycleError::PresentationNotFound)
        ));

        let launched = service
            .launch(
                &store,
                &HostLifecycleLaunchCommand {
                    context: context.clone(),
                    scope: scope.clone(),
                    idempotency_key: "launch:focus".into(),
                    capabilities: [DESKTOP_TAURI_CAPABILITY.into()].into_iter().collect(),
                    permissions: [RICH_HOST_PERMISSION.into()].into_iter().collect(),
                },
                HostPlatform::MacOS,
            )
            .unwrap();
        let focused = service
            .focus(&store, &focus_command, HostPlatform::MacOS)
            .unwrap();

        assert_eq!(focused.state, "focused");
        assert!(focused.focused);
        assert_eq!(focused.lifecycle_revision, 2);
        assert_eq!(focused.host_instance_id, launched.host_instance_id);
        assert_eq!(focused.window_id, launched.window_id);
        assert_eq!(focused.renderer_resolution, launched.renderer_resolution);
        assert!(store.load_projection(&scope).unwrap().is_none());

        let events = store.events_after(&scope, 0, 10).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].1.event_kind, "host_focused");
        assert_eq!(events[1].1.projection_revision, 0);
        assert_eq!(events[1].1.layout_revision, 0);
        assert_eq!(events[1].1.payload["canonical_activity_changed"], false);

        let retry = service
            .focus(&store, &focus_command, HostPlatform::MacOS)
            .unwrap();
        assert_eq!(retry, focused);
        assert_eq!(store.events_after(&scope, 0, 10).unwrap().len(), 2);
    }

    #[test]
    fn focus_rejects_fallback_renderer_missing_capability_and_foreign_scope() {
        let owner = workstream("ws:focus-validation");
        let scope = MissionCanvasScope::new(owner.clone(), None).unwrap();
        let context = request(owner.clone());
        let service = HostLifecycleService;
        let mut command = HostLifecycleFocusCommand {
            context: context.clone(),
            scope: scope.clone(),
            idempotency_key: "focus:validation".into(),
            capabilities: BTreeSet::new(),
            permissions: [RICH_HOST_PERMISSION.into()].into_iter().collect(),
        };

        assert!(matches!(
            service.validate_focus(&command, HostPlatform::Linux),
            Err(HostLifecycleError::Resolution(
                HostRendererResolutionError::CapabilityUnavailable(_)
            ))
        ));
        command.capabilities = [PI_OVERLAY_COMPATIBILITY_CAPABILITY.into()]
            .into_iter()
            .collect();
        assert!(matches!(
            service.validate_focus(&command, HostPlatform::Linux),
            Err(HostLifecycleError::RendererUnavailable(_))
        ));
        command.capabilities = [DESKTOP_TAURI_CAPABILITY.into()].into_iter().collect();
        command.scope = MissionCanvasScope::new(workstream("ws:foreign-focus"), None).unwrap();
        assert!(matches!(
            service.validate_focus(&command, HostPlatform::Linux),
            Err(HostLifecycleError::Resolution(
                HostRendererResolutionError::Context(WorkstreamContextError::WorkstreamMismatch)
            ))
        ));
    }

    #[test]
    fn close_requires_existing_presentation_and_preserves_lifecycle_context() {
        let owner = workstream("ws:close");
        let scope = MissionCanvasScope::new(owner.clone(), None).unwrap();
        let context = request(owner);
        let service = HostLifecycleService;
        let store = MissionCanvasStore::open_in_memory().unwrap();
        let close_command = HostLifecycleCloseCommand {
            context: context.clone(),
            scope: scope.clone(),
            idempotency_key: "close:initial".into(),
            capabilities: [DESKTOP_TAURI_CAPABILITY.into()].into_iter().collect(),
            permissions: [RICH_HOST_PERMISSION.into()].into_iter().collect(),
        };

        assert!(matches!(
            service.close(&store, &close_command, HostPlatform::MacOS),
            Err(HostLifecycleError::PresentationNotFound)
        ));

        let launched = service
            .launch(
                &store,
                &HostLifecycleLaunchCommand {
                    context: context.clone(),
                    scope: scope.clone(),
                    idempotency_key: "launch:close".into(),
                    capabilities: [DESKTOP_TAURI_CAPABILITY.into()].into_iter().collect(),
                    permissions: [RICH_HOST_PERMISSION.into()].into_iter().collect(),
                },
                HostPlatform::MacOS,
            )
            .unwrap();
        let closed = service
            .close(&store, &close_command, HostPlatform::MacOS)
            .unwrap();

        assert_eq!(closed.state, "closing");
        assert_eq!(closed.focused, false);
        assert_eq!(closed.lifecycle_revision, 2);
        assert_eq!(closed.host_instance_id, launched.host_instance_id);
        assert_eq!(closed.window_id, launched.window_id);
        assert_eq!(closed.renderer_resolution, launched.renderer_resolution);

        let events = store.events_after(&scope, 0, 10).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].1.event_kind, "host_closed");
        assert_eq!(events[1].1.projection_revision, 0);
        assert_eq!(events[1].1.layout_revision, 0);
        assert_eq!(
            events[1].1.payload["operation_id"],
            RICH_HOST_CLOSE_OPERATION
        );

        let retry = service
            .close(&store, &close_command, HostPlatform::MacOS)
            .unwrap();
        assert_eq!(retry, closed);
        assert_eq!(store.events_after(&scope, 0, 10).unwrap().len(), 2);
    }

    #[test]
    fn close_is_unavailable_once_closing_or_failed_state() {
        let owner = workstream("ws:close-unavailable");
        let scope = MissionCanvasScope::new(owner.clone(), None).unwrap();
        let context = request(owner);
        let service = HostLifecycleService;
        let store = MissionCanvasStore::open_in_memory().unwrap();
        service
            .launch(
                &store,
                &HostLifecycleLaunchCommand {
                    context: context.clone(),
                    scope: scope.clone(),
                    idempotency_key: "launch:close-unavailable".into(),
                    capabilities: [DESKTOP_TAURI_CAPABILITY.into()].into_iter().collect(),
                    permissions: [RICH_HOST_PERMISSION.into()].into_iter().collect(),
                },
                HostPlatform::MacOS,
            )
            .unwrap();
        let command = HostLifecycleCloseCommand {
            context: context.clone(),
            scope: scope.clone(),
            idempotency_key: "close:blocking".into(),
            capabilities: [DESKTOP_TAURI_CAPABILITY.into()].into_iter().collect(),
            permissions: [RICH_HOST_PERMISSION.into()].into_iter().collect(),
        };
        let first = service
            .close(&store, &command, HostPlatform::MacOS)
            .unwrap();
        assert_eq!(first.state, "closing");

        let blocking = HostLifecycleCloseCommand {
            idempotency_key: "close:different".into(),
            ..command
        };
        assert!(matches!(
            service.close(&store, &blocking, HostPlatform::MacOS),
            Err(HostLifecycleError::PresentationUnavailable(_))
        ));
    }
}
