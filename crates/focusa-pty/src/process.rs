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

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GeometryError {
    #[error("zero dimension is invalid")]
    ZeroDimension,
    #[error("dimension overflow")]
    Overflow,
}

/// PTY-008: invalid zero/overflow dimensions fail closed before reaching the
/// PTY. Columns/rows are u16; zero is invalid and anything beyond a sane
/// terminal viewport (4096) is an overflow.
pub fn validate_geometry(geometry: &PtyGeometry) -> Result<(), GeometryError> {
    if geometry.columns == 0 || geometry.rows == 0 {
        return Err(GeometryError::ZeroDimension);
    }
    if geometry.columns > 4096 || geometry.rows > 4096 {
        return Err(GeometryError::Overflow);
    }
    Ok(())
}

/// PTY-007: acknowledgement for a guarded input write.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PtyWriteAck {
    Accepted { sequence: u64 },
    Rejected { reason: WriteRejectionReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteRejectionReason {
    ForeignAttachment,
    StaleGeneration,
    NotAttached,
}

/// PTY-009: acknowledgement for an interrupt (ETX) targeting one process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterruptAck {
    pub accepted: bool,
    pub generation: u64,
}

/// PTY-009: interrupt (ETX) is distinct from terminate (kill).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminateKind {
    Interrupt,
    Terminate,
}

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
        validate_geometry(&geometry)
            .map_err(|error| PtyProcessError::Open(error.to_string()))?;

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
        match self.write_input_guarded(&self.identity, self.generation(), data) {
            PtyWriteAck::Accepted { .. } => Ok(true),
            _ => Ok(false),
        }
    }

    /// PTY-007: guarded write with acknowledgement. A foreign Attachment or a
    /// stale generation input is rejected BEFORE any byte reaches the PTY.
    pub fn write_input_guarded(
        &self,
        identity: &PtyAttachmentIdentity,
        generation: u64,
        data: &str,
    ) -> PtyWriteAck {
        if identity.attachment_key.attachment_id != self.identity.attachment_key.attachment_id
            || identity.work_surface_id != self.identity.work_surface_id
        {
            return PtyWriteAck::Rejected { reason: WriteRejectionReason::ForeignAttachment };
        }
        if generation != self.generation() {
            return PtyWriteAck::Rejected { reason: WriteRejectionReason::StaleGeneration };
        }
        let mut guard = self.writer.lock().unwrap();
        let writer = match guard.as_mut() {
            Some(writer) => writer,
            None => return PtyWriteAck::Rejected { reason: WriteRejectionReason::NotAttached },
        };
        if writer.write_all(data.as_bytes()).is_err() || writer.flush().is_err() {
            return PtyWriteAck::Rejected { reason: WriteRejectionReason::NotAttached };
        }
        PtyWriteAck::Accepted { sequence: self.sink.lock().unwrap().latest_sequence() }
    }

    /// Resize the PTY viewport. Invalid zero/overflow dimensions fail before
    /// reaching the PTY (PTY-008).
    pub fn resize(&self, geometry: PtyGeometry) -> PtyProcessResult<()> {
        validate_geometry(&geometry)
            .map_err(|error| PtyProcessError::Open(error.to_string()))?;
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

    /// PTY-009: interrupt (ETX) targeting ONE process, distinct from
    /// terminate (close/kill). Returns an acknowledgement with the run
    /// generation; the interrupted event is emitted through the sink.
    pub fn interrupt(&self) -> PtyProcessResult<InterruptAck> {
        self.write_input("\u{3}")?;
        self.sink
            .lock()
            .unwrap()
            .emit(PtyEvent::Interrupted, self.identity.clone())
            .map_err(|e| PtyProcessError::Reader(e.to_string()))?;
        Ok(InterruptAck { accepted: true, generation: self.generation() })
    }

    /// Terminate (kill + reap). Distinct from interrupt.
    pub fn terminate(&self) -> PtyProcessResult<()> {
        self.close()
    }

    /// PTY-010: resync events for a reattached surface from the SAME process
    /// generation. Returns ordered envelopes after `since_sequence`.
    pub fn resync_events(&self, since_sequence: u64) -> Vec<PtyEventEnvelope> {
        self.sink.lock().unwrap().history_after(since_sequence)
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

    #[test]
    fn geometry_validation_rejects_zero_and_overflow() {
        assert_eq!(
            validate_geometry(&PtyGeometry { columns: 0, rows: 24, pixel_width: 0, pixel_height: 0 }),
            Err(GeometryError::ZeroDimension)
        );
        assert_eq!(
            validate_geometry(&PtyGeometry { columns: 80, rows: 0, pixel_width: 0, pixel_height: 0 }),
            Err(GeometryError::ZeroDimension)
        );
        assert_eq!(
            validate_geometry(&PtyGeometry { columns: 5000, rows: 24, pixel_width: 0, pixel_height: 0 }),
            Err(GeometryError::Overflow)
        );
        assert_eq!(
            validate_geometry(&PtyGeometry { columns: 120, rows: 40, pixel_width: 960, pixel_height: 640 }),
            Ok(())
        );
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn resize_rejects_invalid_dimensions_before_reaching_pty() {
        let process = PtyProcess::spawn(
            sample_identity(),
            shell_spec(),
            PtyGeometry { columns: 80, rows: 24, pixel_width: 0, pixel_height: 0 },
        )
        .expect("spawn");
        assert!(process
            .resize(PtyGeometry { columns: 0, rows: 24, pixel_width: 0, pixel_height: 0 })
            .is_err(), "zero columns must fail before reaching the PTY");
        assert!(process
            .resize(PtyGeometry { columns: 120, rows: 40, pixel_width: 0, pixel_height: 0 })
            .is_ok(), "valid latest geometry reaches the PTY");
        process.close().expect("close");
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
    fn write_ack_rejects_foreign_attachment_and_stale_generation() {
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

        // foreign attachment
        let mut foreign = sample_identity();
        foreign.attachment_key.attachment_id = "attachment:other".into();
        assert_eq!(
            process.write_input_guarded(&foreign, process.generation(), "x\n"),
            PtyWriteAck::Rejected { reason: WriteRejectionReason::ForeignAttachment }
        );
        // stale generation
        assert_eq!(
            process.write_input_guarded(&sample_identity(), process.generation() + 99, "x\n"),
            PtyWriteAck::Rejected { reason: WriteRejectionReason::StaleGeneration }
        );
        // current identity + generation accepted
        let ack = process.write_input_guarded(&sample_identity(), process.generation(), "echo ok\n");
        assert!(matches!(ack, PtyWriteAck::Accepted { .. }), "current input accepted");
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

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn interrupt_targets_one_process_and_is_distinct_from_terminate() {
        // A process that ignores SIGINT must survive interrupt() but die on
        // close/terminate: interrupt is ETX (signal), terminate is kill.
        let process = PtyProcess::spawn(
            sample_identity(),
            PtyCommandSpec {
                program: "/bin/sh".into(),
                args: vec![
                    "-c".into(),
                    "trap '' INT; while true; do sleep 1; done".into(),
                ],
                cwd: Some("/tmp".into()),
            },
            PtyGeometry { columns: 80, rows: 24, pixel_width: 0, pixel_height: 0 },
        )
        .expect("spawn");
        let ack = process.interrupt().expect("interrupt");
        assert!(ack.accepted);
        assert_eq!(ack.generation, process.generation());
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert!(process.is_alive(), "interrupt (ETX) must NOT terminate the process");
        process.terminate().expect("terminate");
        assert!(!process.is_alive(), "terminate kills the process");
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn reattach_resyncs_subsequent_output_from_same_generation() {
        let process = PtyProcess::spawn(
            sample_identity(),
            PtyCommandSpec {
                program: "/bin/sh".into(),
                args: vec![
                    "-c".into(),
                    "for i in 1 2 3 4 5; do echo tick-$i; sleep 0.2; done".into(),
                ],
                cwd: Some("/tmp".into()),
            },
            PtyGeometry { columns: 80, rows: 24, pixel_width: 0, pixel_height: 0 },
        )
        .expect("spawn");
        let generation = process.generation();
        let mut events = process.events().expect("receiver");
        // Read a first output event (attach session).
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let mut last_seq = 0u64;
        while std::time::Instant::now() < deadline {
            while let Ok(envelope) = events.try_recv() {
                if let PtyEvent::Output { .. } = &envelope.kind {
                    last_seq = envelope.sequence;
                }
            }
            if last_seq > 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(last_seq > 0, "first output observed");
        // "Detach": drop the receiver; process keeps running.
        drop(events);
        std::thread::sleep(std::time::Duration::from_millis(700));
        // "Reattach": resync from the last observed sequence.
        let subsequent = process.resync_events(last_seq);
        let outputs: Vec<&str> = subsequent
            .iter()
            .filter_map(|envelope| match &envelope.kind {
                PtyEvent::Output { data } => Some(data.as_str()),
                _ => None,
            })
            .collect();
        assert!(
            outputs.iter().any(|data| data.contains("tick-2")),
            "reattach receives subsequent output from same generation: {outputs:?}"
        );
        assert!(
            subsequent.iter().all(|envelope| envelope.generation == generation),
            "all resynced events carry the SAME process generation"
        );
        process.close().expect("close");
    }
}
