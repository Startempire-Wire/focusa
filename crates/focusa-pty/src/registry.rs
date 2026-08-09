//! One persistent Pi process per governed Attachment (PTY-004).
//!
//! The registry is a long-lived runtime object that OUTLIVES views: hiding or
//! switching views never kills a process. Duplicate attach for the same exact
//! identity is idempotent (returns the existing process). detach keeps the
//! live process (presentation detaches); close kills it; restart respawns it
//! under a fresh run generation.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::events::PtyGeometry;
use crate::identity::PtyAttachmentIdentity;
#[cfg(test)]
use crate::process::PtyProcessError;
use crate::process::{InterruptAck, PtyCommandSpec, PtyProcess, PtyProcessResult};
#[allow(unused_imports)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryChange {
    Spawned,
    Reattached,
    Detached,
    Closed,
    Restarted,
}

pub struct PtyRegistry {
    processes: Mutex<HashMap<String, Arc<PtyProcess>>>,
}

impl Default for PtyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PtyRegistry {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
        }
    }

    fn key(identity: &PtyAttachmentIdentity) -> String {
        identity.registry_key().unwrap_or_default()
    }

    /// Attach a process for the exact Attachment. Idempotent: attaching the
    /// same identity twice returns the SAME live process (never a second
    /// spawn). A different identity spawns its own process.
    pub fn attach(
        &self,
        identity: PtyAttachmentIdentity,
        spec: PtyCommandSpec,
        geometry: PtyGeometry,
    ) -> PtyProcessResult<(Arc<PtyProcess>, RegistryChange)> {
        let key = Self::key(&identity);
        let mut guard = self.processes.lock().unwrap();
        if let Some(existing) = guard.get(&key) {
            if existing.is_alive() {
                return Ok((Arc::clone(existing), RegistryChange::Reattached));
            }
            // Process died (close was called but the entry remained for
            // restart): respawn in place.
            let process = PtyProcess::spawn(identity, spec, geometry)?;
            guard.insert(key, Arc::clone(&process));
            return Ok((process, RegistryChange::Restarted));
        }
        let process = PtyProcess::spawn(identity, spec, geometry)?;
        guard.insert(key, Arc::clone(&process));
        Ok((process, RegistryChange::Spawned))
    }

    /// The live process for an identity, if any.
    pub fn get(&self, identity: &PtyAttachmentIdentity) -> Option<Arc<PtyProcess>> {
        self.processes
            .lock()
            .unwrap()
            .get(&Self::key(identity))
            .cloned()
    }

    /// Presentation detaches; the live process stays (view switches and
    /// reconnect reuse the SAME process).
    pub fn detach(&self, identity: &PtyAttachmentIdentity) -> PtyProcessResult<RegistryChange> {
        if let Some(process) = self.get(identity) {
            process.detach()?;
        }
        Ok(RegistryChange::Detached)
    }

    /// Kill + remove the process for an identity.
    pub fn close(&self, identity: &PtyAttachmentIdentity) -> PtyProcessResult<RegistryChange> {
        let key = Self::key(identity);
        let removed = self.processes.lock().unwrap().remove(&key);
        if let Some(process) = removed {
            process.close()?;
        }
        Ok(RegistryChange::Closed)
    }

    /// PTY-009: interrupt targets the ONE registered process for the exact
    /// Attachment (never a broadcast, never a terminate). Returns the ack.
    pub fn interrupt(&self, identity: &PtyAttachmentIdentity) -> PtyProcessResult<Option<InterruptAck>> {
        match self.get(identity) {
            Some(process) => process.interrupt().map(Some),
            None => Ok(None),
        }
    }

    /// PTY-010: reattached presentation resyncs subsequent output from the
    /// SAME process generation via the process event history.
    pub fn resync(&self, identity: &PtyAttachmentIdentity, since_sequence: u64) -> Vec<crate::events::PtyEventEnvelope> {
        match self.get(identity) {
            Some(process) => process.resync_events(since_sequence),
            None => Vec::new(),
        }
    }

    /// Kill + respawn under a fresh generation; the entry stays registered.
    pub fn restart(
        &self,
        identity: &PtyAttachmentIdentity,
        spec: PtyCommandSpec,
        geometry: PtyGeometry,
    ) -> PtyProcessResult<(Arc<PtyProcess>, RegistryChange)> {
        let key = Self::key(identity);
        let process = match self.get(identity) {
            Some(process) => {
                process.restart(spec, geometry)?;
                process
            }
            None => PtyProcess::spawn(identity.clone(), spec, geometry)?,
        };
        self.processes.lock().unwrap().insert(key, Arc::clone(&process));
        Ok((process, RegistryChange::Restarted))
    }

    /// Number of registered live processes.
    pub fn len(&self) -> usize {
        self.processes.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::tests::sample_identity;

    fn shell_spec() -> PtyCommandSpec {
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            PtyCommandSpec {
                program: "/bin/sh".into(),
                args: vec!["-c".into(), "cat".into()],
                cwd: Some("/tmp".into()),
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            PtyCommandSpec { program: "cmd".into(), args: vec![], cwd: Some(".".into()) }
        }
    }

    fn geometry() -> PtyGeometry {
        PtyGeometry { columns: 80, rows: 24, pixel_width: 0, pixel_height: 0 }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn duplicate_attach_is_idempotent() {
        let registry = PtyRegistry::new();
        let (first, change) = registry
            .attach(sample_identity(), shell_spec(), geometry())
            .expect("first attach");
        assert_eq!(change, RegistryChange::Spawned);
        let (second, change) = registry
            .attach(sample_identity(), shell_spec(), geometry())
            .expect("second attach");
        assert_eq!(change, RegistryChange::Reattached, "duplicate attach reuses the process");
        assert!(Arc::ptr_eq(&first, &second), "same live process returned");
        assert_eq!(registry.len(), 1, "one process per governed Attachment");
        registry.close(&sample_identity()).expect("close");
        assert_eq!(registry.len(), 0);
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn detach_keeps_the_live_process() {
        let registry = PtyRegistry::new();
        let (process, _) = registry
            .attach(sample_identity(), shell_spec(), geometry())
            .expect("attach");
        registry.detach(&sample_identity()).expect("detach");
        assert!(process.is_alive(), "detach must NOT kill the process");
        // Re-attach returns the SAME process (same Pi session).
        let (reattached, change) = registry
            .attach(sample_identity(), shell_spec(), geometry())
            .expect("reattach");
        assert_eq!(change, RegistryChange::Reattached);
        assert!(Arc::ptr_eq(&process, &reattached));
        registry.close(&sample_identity()).expect("close");
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn distinct_attachments_get_distinct_processes() {
        let registry = PtyRegistry::new();
        let mut other = sample_identity();
        other.work_surface_id = "surface:inspector".into();
        let (a, _) = registry.attach(sample_identity(), shell_spec(), geometry()).expect("a");
        let (b, _) = registry.attach(other.clone(), shell_spec(), geometry()).expect("b");
        assert!(!Arc::ptr_eq(&a, &b));
        assert_eq!(registry.len(), 2);
        registry.close(&sample_identity()).expect("close a");
        registry.close(&other).expect("close b");
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn restart_respawns_under_fresh_generation() {
        let registry = PtyRegistry::new();
        let (process, _) = registry
            .attach(sample_identity(), shell_spec(), geometry())
            .expect("attach");
        let generation_before = process.generation();
        let (restarted, change) = registry
            .restart(&sample_identity(), shell_spec(), geometry())
            .expect("restart");
        assert_eq!(change, RegistryChange::Restarted);
        assert_eq!(restarted.generation(), generation_before + 1, "fresh run generation");
        assert!(restarted.is_alive());
        registry.close(&sample_identity()).expect("close");
    }

    #[test]
    fn invalid_identity_never_registers() {
        let registry = PtyRegistry::new();
        let mut identity = sample_identity();
        identity.attachment_key.workstream.scope.scope_kind = "host".into();
        let result = registry.attach(identity, shell_spec(), geometry());
        assert!(matches!(
            result,
            Err(PtyProcessError::Identity(_))
        ));
        assert_eq!(registry.len(), 0, "fail closed before registration");
    }
}
