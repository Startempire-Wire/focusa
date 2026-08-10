//! Focusa Desktop native shell.
//!
//! Presentation remains in Svelte. The only native domain surface exposed here
//! is the governed portable-pty runtime required by the Agent TUI Work Surface.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use focusa_pty::{
    events::PtyGeometry,
    identity::PtyAttachmentIdentity,
    invoke::{PtyInvokeCommand, PtyInvokeHandler, PtyInvokeResult},
    registry::PtyRegistry,
};

struct DesktopPtyState {
    handler: PtyInvokeHandler,
}

impl DesktopPtyState {
    fn new() -> Self {
        Self {
            handler: PtyInvokeHandler::new(Arc::new(PtyRegistry::new())),
        }
    }

    fn handle(&self, command: PtyInvokeCommand) -> PtyInvokeResult {
        self.handler.handle(command)
    }
}

#[tauri::command]
fn focusa_pty_attach(
    state: tauri::State<'_, DesktopPtyState>,
    identity: PtyAttachmentIdentity,
    geometry: PtyGeometry,
) -> PtyInvokeResult {
    state.handle(PtyInvokeCommand::Attach {
        identity,
        geometry,
        program: None,
    })
}

#[tauri::command(rename_all = "snake_case")]
fn focusa_pty_input(
    state: tauri::State<'_, DesktopPtyState>,
    attachment_id: String,
    work_surface_id: String,
    data: String,
    generation: u64,
) -> PtyInvokeResult {
    state.handle(PtyInvokeCommand::Input {
        attachment_id,
        work_surface_id,
        data,
        generation,
    })
}

#[tauri::command(rename_all = "snake_case")]
fn focusa_pty_resize(
    state: tauri::State<'_, DesktopPtyState>,
    attachment_id: String,
    work_surface_id: String,
    geometry: PtyGeometry,
) -> PtyInvokeResult {
    state.handle(PtyInvokeCommand::Resize {
        attachment_id,
        work_surface_id,
        geometry,
    })
}

#[tauri::command(rename_all = "snake_case")]
fn focusa_pty_interrupt(
    state: tauri::State<'_, DesktopPtyState>,
    attachment_id: String,
    work_surface_id: String,
) -> PtyInvokeResult {
    state.handle(PtyInvokeCommand::Interrupt {
        attachment_id,
        work_surface_id,
    })
}

#[tauri::command(rename_all = "snake_case")]
fn focusa_pty_detach(
    state: tauri::State<'_, DesktopPtyState>,
    attachment_id: String,
    work_surface_id: String,
) -> PtyInvokeResult {
    state.handle(PtyInvokeCommand::Detach {
        attachment_id,
        work_surface_id,
    })
}

#[tauri::command(rename_all = "snake_case")]
fn focusa_pty_close(
    state: tauri::State<'_, DesktopPtyState>,
    attachment_id: String,
    work_surface_id: String,
) -> PtyInvokeResult {
    state.handle(PtyInvokeCommand::Close {
        attachment_id,
        work_surface_id,
    })
}

#[tauri::command(rename_all = "snake_case")]
fn focusa_pty_restart(
    state: tauri::State<'_, DesktopPtyState>,
    attachment_id: String,
    work_surface_id: String,
) -> PtyInvokeResult {
    state.handle(PtyInvokeCommand::Restart {
        attachment_id,
        work_surface_id,
        program: None,
    })
}

#[tauri::command(rename_all = "snake_case")]
fn focusa_pty_resync(
    state: tauri::State<'_, DesktopPtyState>,
    attachment_id: String,
    work_surface_id: String,
    since_sequence: u64,
) -> PtyInvokeResult {
    state.handle(PtyInvokeCommand::Resync {
        attachment_id,
        work_surface_id,
        since_sequence,
    })
}

fn main() {
    tauri::Builder::default()
        .manage(DesktopPtyState::new())
        .invoke_handler(tauri::generate_handler![
            focusa_pty_attach,
            focusa_pty_input,
            focusa_pty_resize,
            focusa_pty_interrupt,
            focusa_pty_detach,
            focusa_pty_close,
            focusa_pty_restart,
            focusa_pty_resync,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Focusa Desktop");
}
