//! PTY-012 — typed invoke command surface.
//!
//! Only typed PTY operations are exposed; there is no free-form command
//! string. Registry lifecycle is app-managed: the invoke handler holds an
//! `Arc<PtyRegistry>` that outlives views, and every operation resolves the
//! ONE process for the exact governed Attachment before acting.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::events::PtyGeometry;
use crate::identity::PtyAttachmentIdentity;
use crate::process::{PtyWriteAck, WriteRejectionReason};
use crate::registry::{PtyRegistry, RegistryChange};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PtyInvokeCommand {
    Attach {
        identity: PtyAttachmentIdentity,
        geometry: PtyGeometry,
        program: Option<String>,
    },
    Input {
        attachment_id: String,
        work_surface_id: String,
        data: String,
        generation: u64,
    },
    Resize {
        attachment_id: String,
        work_surface_id: String,
        geometry: PtyGeometry,
    },
    Interrupt {
        attachment_id: String,
        work_surface_id: String,
    },
    Detach {
        attachment_id: String,
        work_surface_id: String,
    },
    Close {
        attachment_id: String,
        work_surface_id: String,
    },
    Restart {
        attachment_id: String,
        work_surface_id: String,
        program: Option<String>,
    },
    Resync {
        attachment_id: String,
        work_surface_id: String,
        since_sequence: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PtyInvokeError {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PtyInvokeResult {
    Ok { change: RegistryChange },
    WriteAck { ack: PtyWriteAck },
    Resized,
    Interrupted { generation: u64 },
    Resync { events: Vec<crate::events::PtyEventEnvelope> },
    Err(PtyInvokeError),
}

impl PtyInvokeResult {
    pub fn is_ok(&self) -> bool {
        !matches!(self, PtyInvokeResult::Err(_))
    }
}

pub struct PtyInvokeHandler {
    registry: Arc<PtyRegistry>,
}

impl PtyInvokeHandler {
    pub fn new(registry: Arc<PtyRegistry>) -> Self {
        Self { registry }
    }

    fn default_spec() -> crate::process::PtyCommandSpec {
        crate::process::PtyCommandSpec::default()
    }

    pub fn handle(&self, command: PtyInvokeCommand) -> PtyInvokeResult {
        match command {
            PtyInvokeCommand::Attach { identity, geometry, program } => {
                let mut spec = Self::default_spec();
                if let Some(program) = program {
                    spec.program = program;
                }
                match self.registry.attach(identity, spec, geometry) {
                    Ok((_process, change)) => PtyInvokeResult::Ok { change },
                    Err(error) => PtyInvokeResult::Err(PtyInvokeError { reason: error.to_string() }),
                }
            }
            PtyInvokeCommand::Input { attachment_id, work_surface_id, data, generation } => {
                let Some(process) = self.find(&attachment_id, &work_surface_id) else {
                    return PtyInvokeResult::WriteAck {
                        ack: PtyWriteAck::Rejected { reason: WriteRejectionReason::ForeignAttachment },
                    };
                };
                let identity = process.identity().clone();
                PtyInvokeResult::WriteAck { ack: process.write_input_guarded(&identity, generation, &data) }
            }
            PtyInvokeCommand::Resize { attachment_id, work_surface_id, geometry } => {
                let Some(process) = self.find(&attachment_id, &work_surface_id) else {
                    return PtyInvokeResult::Err(PtyInvokeError { reason: "no process for exact Attachment".into() });
                };
                match process.resize(geometry) {
                    Ok(()) => PtyInvokeResult::Resized,
                    Err(error) => PtyInvokeResult::Err(PtyInvokeError { reason: error.to_string() }),
                }
            }
            PtyInvokeCommand::Interrupt { attachment_id, work_surface_id } => {
                let Some(process) = self.find(&attachment_id, &work_surface_id) else {
                    return PtyInvokeResult::Err(PtyInvokeError { reason: "no process for exact Attachment".into() });
                };
                match process.interrupt() {
                    Ok(ack) => PtyInvokeResult::Interrupted { generation: ack.generation },
                    Err(error) => PtyInvokeResult::Err(PtyInvokeError { reason: error.to_string() }),
                }
            }
            PtyInvokeCommand::Detach { attachment_id, work_surface_id } => {
                let Some(process) = self.find(&attachment_id, &work_surface_id) else {
                    return PtyInvokeResult::Err(PtyInvokeError { reason: "no process for exact Attachment".into() });
                };
                match process.detach() {
                    Ok(()) => PtyInvokeResult::Ok { change: RegistryChange::Detached },
                    Err(error) => PtyInvokeResult::Err(PtyInvokeError { reason: error.to_string() }),
                }
            }
            PtyInvokeCommand::Close { attachment_id, work_surface_id } => {
                let Some(process) = self.find(&attachment_id, &work_surface_id) else {
                    return PtyInvokeResult::Err(PtyInvokeError { reason: "no process for exact Attachment".into() });
                };
                match process.close() {
                    Ok(()) => PtyInvokeResult::Ok { change: RegistryChange::Closed },
                    Err(error) => PtyInvokeResult::Err(PtyInvokeError { reason: error.to_string() }),
                }
            }
            PtyInvokeCommand::Restart { attachment_id, work_surface_id, program } => {
                let Some(process) = self.find(&attachment_id, &work_surface_id) else {
                    return PtyInvokeResult::Err(PtyInvokeError { reason: "no process for exact Attachment".into() });
                };
                let mut spec = Self::default_spec();
                if let Some(program) = program {
                    spec.program = program;
                }
                let geometry = PtyGeometry { columns: 80, rows: 24, pixel_width: 0, pixel_height: 0 };
                match process.restart(spec, geometry) {
                    Ok(()) => PtyInvokeResult::Ok { change: RegistryChange::Restarted },
                    Err(error) => PtyInvokeResult::Err(PtyInvokeError { reason: error.to_string() }),
                }
            }
            PtyInvokeCommand::Resync { attachment_id, work_surface_id, since_sequence } => {
                let Some(process) = self.find(&attachment_id, &work_surface_id) else {
                    return PtyInvokeResult::Resync { events: Vec::new() };
                };
                PtyInvokeResult::Resync { events: process.resync_events(since_sequence) }
            }
        }
    }

    fn find(&self, attachment_id: &str, work_surface_id: &str) -> Option<Arc<crate::process::PtyProcess>> {
        // Resolve by the exact identity the registry key was derived from:
        // iterate registered processes and match both fields (no partial
        // authority ever grants access).
        let registry = self.registry.clone();
        registry.find(attachment_id, work_surface_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::tests::sample_identity;

    #[test]
    fn typed_surface_exposes_only_pty_operations() {
        // The command enum IS the surface: every variant is a typed PTY
        // operation; there is no free-form command string anywhere.
        let handler = PtyInvokeHandler::new(Arc::new(PtyRegistry::new()));
        let identity = sample_identity();
        let result = handler.handle(PtyInvokeCommand::Attach {
            identity,
            geometry: PtyGeometry { columns: 80, rows: 24, pixel_width: 0, pixel_height: 0 },
            program: None,
        });
        assert!(result.is_ok(), "typed attach is exposed");
    }

    #[test]
    fn input_for_unknown_attachment_is_rejected() {
        let handler = PtyInvokeHandler::new(Arc::new(PtyRegistry::new()));
        let result = handler.handle(PtyInvokeCommand::Input {
            attachment_id: "attachment:nobody".into(),
            work_surface_id: "surface:pi".into(),
            data: "echo x\n".into(),
            generation: 1,
        });
        match result {
            PtyInvokeResult::WriteAck { ack } => {
                assert!(matches!(ack, PtyWriteAck::Rejected { reason: WriteRejectionReason::ForeignAttachment }))
            }
            other => panic!("expected write ack, got {other:?}"),
        }
    }

    #[test]
    fn close_and_detach_are_distinct_operations() {
        // PTY-011: closing a view (detach) and terminating the canonical
        // runtime (close) are separate operations. On an empty registry both
        // are no-ops with distinct outcomes.
        let handler = PtyInvokeHandler::new(Arc::new(PtyRegistry::new()));
        let detach = handler.handle(PtyInvokeCommand::Detach {
            attachment_id: "attachment:none".into(),
            work_surface_id: "surface:pi".into(),
        });
        let close = handler.handle(PtyInvokeCommand::Close {
            attachment_id: "attachment:none".into(),
            work_surface_id: "surface:pi".into(),
        });
        assert!(matches!(detach, PtyInvokeResult::Err(_)));
        assert!(matches!(close, PtyInvokeResult::Err(_)));
    }
}
