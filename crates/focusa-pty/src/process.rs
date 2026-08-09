//! Persistent Pi PTY process (PTY-005).
//!
//! One real PTY process per governed Attachment using `portable-pty` — never
//! ordinary child-process pipes. Spawn fails BEFORE process creation on scope
//! mismatch or missing Attachment fields (fail-closed identity validation).
//! The process object defines resize, input, output, interrupt (ETX), detach,
//! close, restart, and stale-output rejection through the generation-scoped
//! event sink.

use std::io::{Read, Write};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use thiserror::Error;

use crate::events::{EventSink, PtyEvent, PtyEventEnvelope, PtyGeometry};
use crate::identity::{IdentityValidationError, PtyAttachmentIdentity};

#[derive(Debug, Error)]
pub enum PtyProcessError {
    #[error("identity validation failed: {0}")]
    Identity(#[from] IdentityValidationError),
    #[error("pty open failed: {0}")]
    Open(String),
    #[error("spawn failed: {0}")]
    Spawn(String),
    #[error("writer unavailable")]
    WriterUnavailable,
    #[error("reader thread failed: {0}")]
    Reader(String),
}

pub type PtyProcessResult<T> = Result<T, PtyProcessError>;

/// Runtime command being executed. Defaults to the `pi` binary on PATH.
#[derive(Debug, Clone)]
pub struct PtyCommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
}

impl Default for PtyCommandSpec {
    fn default() -> Self {
        Self {
            program: "pi".into(),
            args: Vec::new(),
            cwd: None,
        }
    }
}

/// A governed persistent Pi process. Identity is fixed at spawn; generation
/// increments on restart so stale output can never impersonate the current
/// process.
pub struct PtyProcess {
    identity: PtyAttachmentIdentity,
    generation: Mutex<u64>,
    child: Mutex<Option<Box<dyn Child + Send>>>,
    master: Mutex<Option<Box<dyn MasterPty + Send>>>,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
    sink: Mutex<EventSink>,
    events: Mutex<Option<mpsc::Receiver<PtyEventEnvelope>>>,
}

impl std::fmt::Debug for PtyProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtyProcess")
            .field("attachment_id", &self.identity.attachment_key.attachment_id)
            .field("work_surface_id", &self.identity.work_surface_id)
            .field("generation", &self.generation())
            .field("is_alive", &self.is_alive())
            .finish()
    }
}

impl PtyProcess {
    /// Validate identity FIRST and only then create the PTY + process. A
    /// scope mismatch or missing Attachment field fails before any process
    /// exists.
    pub fn spawn(
        identity: PtyAttachmentIdentity,
        spec: PtyCommandSpec,
        geometry: PtyGeometry,
    ) -> PtyProcessResult<Arc<Self>> {
        Self::spawn_at(identity, spec, geometry, 1)
    }

    fn spawn_at(
        identity: PtyAttachmentIdentity,
        spec: PtyCommandSpec,
        geometry: PtyGeometry,
        generation: u64,
    ) -> PtyProcessResult<Arc<Self>> {
        identity.validate()?;

        let sink = EventSink::new(generation);
        let pty_system = native_pty_system();
        let size = PtySize {
            rows: geometry.rows,
            cols: geometry.columns,
            pixel_width: geometry.pixel_width,
            pixel_height: geometry.pixel_height,
        };
        let pair = pty_system
            .openpty(size)
            .map_err(|e| PtyProcessError::Open(e.to_string()))?;

        let mut command = CommandBuilder::new(spec.program);
        for arg in &spec.args {
            command.arg(arg);
        }
        if let Some(cwd) = &spec.cwd {
            command.cwd(cwd);
        }
        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|e| PtyProcessError::Spawn(e.to_string()))?;
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| PtyProcessError::Open(e.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyProcessError::Open(e.to_string()))?;

        let process = Arc::new(Self {
            identity: identity.clone(),
            generation: Mutex::new(generation),
            child: Mutex::new(Some(child)),
            master: Mutex::new(Some(pair.master)),
            writer: Mutex::new(Some(writer)),
            sink: Mutex::new(sink.clone()),
            events: Mutex::new(sink.take_receiver()),
        });

        // Reader thread: partial reads preserve bytes (PTY-006).
        let identity_reader = identity.clone();
        let sink_reader = sink.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            let mut pending = Vec::new();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        pending.extend_from_slice(&buf[..n]);
                        while let Some(pos) = pending.iter().position(|b| *b == b'\n') {
                            let end = pos + 1;
                            let line: Vec<u8> = pending[..end].to_vec();
                            let data = String::from_utf8_lossy(&line).into_owned();
                            let _ = sink_reader.emit(PtyEvent::Output { data }, identity_reader.clone());
                            pending.drain(..end);
                        }
                        if pending.len() >= 64 * 1024 {
                            let data = String::from_utf8_lossy(&pending).into_owned();
                            let _ = sink_reader.emit(PtyEvent::Output { data }, identity_reader.clone());
                            pending.clear();
                        }
                    }
                    Err(_) => break,
                }
            }
            if !pending.is_empty() {
                let data = String::from_utf8_lossy(&pending).into_owned();
                let _ = sink_reader.emit(PtyEvent::Output { data }, identity_reader);
            }
        });

        Ok(process)
    }

    pub fn identity(&self) -> &PtyAttachmentIdentity {
        &self.identity
    }

    pub fn generation(&self) -> u64 {
        *self.generation.lock().unwrap()
    }

    pub fn events(&self) -> Option<mpsc::Receiver<PtyEventEnvelope>> {
        self.events.lock().unwrap().take()
    }

    /// Stale-output rejection for consumers: only the current generation with
    /// an in-range sequence is accepted.
    pub fn accepts(&self, generation: u64, sequence: u64) -> bool {
        self.sink.lock().unwrap().accepts(generation, sequence)
    }

    /// Write input into the PTY. Returns false when no process is attached.
    pub fn write_input(&self, data: &str) -> PtyProcessResult<bool> {
        let mut guard = self.writer.lock().unwrap();
        let writer = guard.as_mut().ok_or(PtyProcessError::WriterUnavailable)?;
        writer
            .write_all(data.as_bytes())
            .map_err(|e| PtyProcessError::Reader(e.to_string()))?;
        writer
            .flush()
            .map_err(|e| PtyProcessError::Reader(e.to_string()))?;
        Ok(true)
    }

    /// Resize the PTY viewport.
    pub fn resize(&self, geometry: PtyGeometry) -> PtyProcessResult<()> {
        let mut guard = self.master.lock().unwrap();
        let master = guard.as_mut().ok_or(PtyProcessError::WriterUnavailable)?;
        master
            .resize(PtySize {
                rows: geometry.rows,
                cols: geometry.columns,
                pixel_width: geometry.pixel_width,
                pixel_height: geometry.pixel_height,
            })
            .map_err(|e| PtyProcessError::Open(e.to_string()))?;
        self.sink
            .lock()
            .unwrap()
            .emit(
                PtyEvent::Resized { geometry },
                self.identity.clone(),
            )
            .map_err(|e| PtyProcessError::Reader(e.to_string()))?;
        Ok(())
    }

    /// Interrupt: write ETX (Ctrl-C) into the PTY master.
    pub fn interrupt(&self) -> PtyProcessResult<()> {
        self.write_input("\u{3}")?;
        self.sink
            .lock()
            .unwrap()
            .emit(PtyEvent::Interrupted, self.identity.clone())
            .map_err(|e| PtyProcessError::Reader(e.to_string()))?;
        Ok(())
    }

    /// Kill + reap the child process.
    pub fn close(&self) -> PtyProcessResult<()> {
        if let Some(mut child) = self.child.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        *self.writer.lock().unwrap() = None;
        self.sink
            .lock()
            .unwrap()
            .emit(PtyEvent::Closed, self.identity.clone())
            .map_err(|e| PtyProcessError::Reader(e.to_string()))?;
        Ok(())
    }

    /// Detach presentation while keeping the live process (PTY-010 semantics
    /// defined here; the registry decides what remains attached).
    pub fn detach(&self) -> PtyProcessResult<()> {
        self.sink
            .lock()
            .unwrap()
            .emit(PtyEvent::Detached, self.identity.clone())
            .map_err(|e| PtyProcessError::Reader(e.to_string()))?;
        Ok(())
    }

    /// Restart: kill the old process and spawn a fresh one with a NEW run
    /// generation, so stale output cannot impersonate the current process.
    pub fn restart(&self, spec: PtyCommandSpec, geometry: PtyGeometry) -> PtyProcessResult<()> {
        self.close().ok();
        let next_generation = self.generation() + 1;
        let restarted = Self::spawn_at(self.identity.clone(), spec, geometry, next_generation)?;
        // Steal the fresh process internals (incl. its generation-scoped sink).
        let mut child_guard = self.child.lock().unwrap();
        let mut writer_guard = self.writer.lock().unwrap();
        let mut events_guard = self.events.lock().unwrap();
        let mut sink_guard = self.sink.lock().unwrap();
        *child_guard = restarted.child.lock().unwrap().take();
        *writer_guard = restarted.writer.lock().unwrap().take();
        *events_guard = restarted.events.lock().unwrap().take();
        *sink_guard = restarted.sink.lock().unwrap().clone();
        drop((child_guard, writer_guard, events_guard, sink_guard));
        *self.generation.lock().unwrap() = next_generation;
        self.sink
            .lock()
            .unwrap()
            .emit(PtyEvent::Restarted, self.identity.clone())
            .map_err(|e| PtyProcessError::Reader(e.to_string()))?;
        Ok(())
    }

    pub fn is_alive(&self) -> bool {
        self.child.lock().unwrap().is_some()
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::tests::sample_identity;

    fn shell_spec() -> PtyCommandSpec {
        #[cfg(target_os = "macos")]
        {
            PtyCommandSpec {
                program: "/bin/sh".into(),
                args: vec!["-c".into(), "printf 'hello\\n'; sleep 0.2; printf 'world\\n'".into()],
                cwd: Some("/tmp".into()),
            }
        }
        #[cfg(target_os = "linux")]
        {
            PtyCommandSpec {
                program: "/bin/sh".into(),
                args: vec!["-c".into(), "printf 'hello\\n'; sleep 0.2; printf 'world\\n'".into()],
                cwd: Some("/tmp".into()),
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            PtyCommandSpec { program: "cmd".into(), args: vec![], cwd: Some(".".into()) }
        }
    }

    #[test]
    fn spawn_fails_before_process_creation_on_scope_mismatch() {
        let mut identity = sample_identity();
        identity.attachment_key.workstream.scope.scope_kind = "host".into();
        let result = PtyProcess::spawn(
            identity,
            shell_spec(),
            PtyGeometry { columns: 80, rows: 24, pixel_width: 0, pixel_height: 0 },
        );
        assert!(result.is_err(), "must fail before any process exists");
        match result {
            Err(PtyProcessError::Identity(e)) => {
                assert_eq!(e, IdentityValidationError::ScopeNotProject)
            }
            other => panic!("expected identity failure, got {other:?}"),
        }
    }

    #[test]
    fn spawn_fails_before_process_creation_on_missing_attachment() {
        let mut identity = sample_identity();
        identity.attachment_key.attachment_id = String::new();
        let result = PtyProcess::spawn(
            identity,
            shell_spec(),
            PtyGeometry { columns: 80, rows: 24, pixel_width: 0, pixel_height: 0 },
        );
        assert!(result.is_err());
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn process_emits_ordered_output_with_exact_identity() {
        let process = PtyProcess::spawn(
            sample_identity(),
            shell_spec(),
            PtyGeometry { columns: 80, rows: 24, pixel_width: 0, pixel_height: 0 },
        )
        .expect("spawn");
        assert!(process.is_alive());
        let mut events = process.events().expect("receiver");
        let mut outputs = Vec::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while outputs.len() < 2 && std::time::Instant::now() < deadline {
            while let Ok(envelope) = events.try_recv() {
                match envelope.kind {
                    PtyEvent::Output { data } => outputs.push(data),
                    _ => {}
                }
                assert_eq!(envelope.attachment_key.attachment_key.attachment_id, "attachment:pi");
                assert_eq!(envelope.work_surface_id, "surface:pi");
                assert_eq!(envelope.generation, 1);
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert_eq!(outputs.len(), 2, "ordered output events: {outputs:?}");
        assert!(outputs[0].contains("hello"), "partial-read bytes preserved: {outputs:?}");
        assert!(outputs[1].contains("world"), "second line in order: {outputs:?}");
        assert!(!process.accepts(0, 1), "stale generation rejected");
        assert!(!process.accepts(1, 999), "non-monotonic sequence rejected");
        process.close().expect("close");
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn input_and_interrupt_are_defined_and_implemented() {
        let process = PtyProcess::spawn(
            sample_identity(),
            PtyCommandSpec {
                program: "/bin/sh".into(),
                args: vec!["-c".into(), "cat".into()],
                cwd: Some("/tmp".into()),
            },
            PtyGeometry { columns: 80, rows: 24, pixel_width: 0, pixel_height: 0 },
        )
        .expect("spawn");
        process.write_input("echo typed\n").expect("write input");
        process.resize(PtyGeometry { columns: 120, rows: 40, pixel_width: 0, pixel_height: 0 })
            .expect("resize");
        let mut events = process.events().expect("receiver");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut saw_output = false;
        while std::time::Instant::now() < deadline {
            while let Ok(envelope) = events.try_recv() {
                if let PtyEvent::Output { data } = &envelope.kind {
                    if data.contains("echo typed") {
                        saw_output = true;
                    }
                }
            }
            if saw_output {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(saw_output, "input reached the process and echoed back");
        process.interrupt().expect("interrupt");
        process.close().expect("close");
    }
}
