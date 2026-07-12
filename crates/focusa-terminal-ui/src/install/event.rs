//! Install event contract.
//!
//! §5: typed events emitted by the installer and consumed by presenters.

use serde::Serialize;
use std::fmt;

/// Stable phase identifiers for installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum InstallPhase {
    InitializeEnvironment,
    DetectSystem,
    ValidateLicense,
    ResolveRelease,
    DownloadAssets,
    VerifyIntegrity,
    InstallBinaries,
    IntegratePi,
    RegisterService,
    PersistPath,
    RunHealthChecks,
    Finalize,
    Complete,
    Rollback,
}

impl InstallPhase {
    /// Human-readable label required by the spec.
    pub const fn label(self) -> &'static str {
        match self {
            InstallPhase::InitializeEnvironment => "Initialize environment",
            InstallPhase::DetectSystem => "Detect system",
            InstallPhase::ValidateLicense => "Validate license",
            InstallPhase::ResolveRelease => "Resolve release",
            InstallPhase::DownloadAssets => "Download assets",
            InstallPhase::VerifyIntegrity => "Verify checksums and trust",
            InstallPhase::InstallBinaries => "Install binaries",
            InstallPhase::IntegratePi => "Integrate Pi",
            InstallPhase::RegisterService => "Register service",
            InstallPhase::PersistPath => "Persist PATH",
            InstallPhase::RunHealthChecks => "Run health checks",
            InstallPhase::Finalize => "Finalize",
            InstallPhase::Complete => "Complete",
            InstallPhase::Rollback => "Roll back safely",
        }
    }

    /// Whether this phase is part of the normal forward flow.
    pub const fn is_forward(self) -> bool {
        !matches!(self, InstallPhase::Rollback)
    }
}

impl fmt::Display for InstallPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Per-asset download progress.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AssetProgress {
    pub asset: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

/// Events emitted by the installer orchestrator.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum InstallEvent {
    PhaseStarted {
        phase: InstallPhase,
        message: String,
    },
    PhaseMessage {
        phase: InstallPhase,
        message: String,
    },
    AssetStarted {
        asset: String,
        total_bytes: Option<u64>,
    },
    AssetProgress {
        asset: String,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
    },
    AssetFinished {
        asset: String,
        downloaded_bytes: u64,
    },
    PhaseSucceeded {
        phase: InstallPhase,
        detail: Option<String>,
    },
    PhaseSkipped {
        phase: InstallPhase,
        reason: String,
    },
    PhaseWarning {
        phase: InstallPhase,
        message: String,
        recovery_hint: Option<String>,
    },
    PhaseFailed {
        phase: InstallPhase,
        message: String,
        recovery_hint: Option<String>,
    },
    RollbackStarted {
        reason: String,
    },
    RollbackSucceeded,
    RollbackFailed {
        message: String,
        recovery_hint: String,
    },
    InstallFinished {
        summary: crate::install::completion::InstallCompletionSummary,
    },
}

impl InstallEvent {
    /// The phase this event relates to, if any.
    pub fn phase(&self) -> Option<InstallPhase> {
        match self {
            InstallEvent::PhaseStarted { phase, .. } => Some(*phase),
            InstallEvent::PhaseMessage { phase, .. } => Some(*phase),
            InstallEvent::PhaseSucceeded { phase, .. } => Some(*phase),
            InstallEvent::PhaseSkipped { phase, .. } => Some(*phase),
            InstallEvent::PhaseWarning { phase, .. } => Some(*phase),
            InstallEvent::PhaseFailed { phase, .. } => Some(*phase),
            _ => None,
        }
    }
}

/// Sink trait for install events.
///
/// §4.3: The installer emits domain-neutral presentation events through this interface.
pub trait InstallEventSink: Send + Sync {
    fn emit(&self, event: InstallEvent);
}

/// A simple channel-based event sink for bridging between installer and renderer.
pub struct ChannelEventSink {
    pub sender: std::sync::mpsc::Sender<InstallEvent>,
}

impl InstallEventSink for ChannelEventSink {
    fn emit(&self, event: InstallEvent) {
        let _ = self.sender.send(event);
    }
}

/// A no-op sink used when the presenter is silent.
pub struct NullEventSink;

impl InstallEventSink for NullEventSink {
    fn emit(&self, _event: InstallEvent) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_labels_are_stable() {
        assert_eq!(InstallPhase::DetectSystem.label(), "Detect system");
        assert_eq!(InstallPhase::DownloadAssets.label(), "Download assets");
        assert_eq!(
            InstallPhase::VerifyIntegrity.label(),
            "Verify checksums and trust"
        );
        assert_eq!(InstallPhase::Rollback.label(), "Roll back safely");
    }

    #[test]
    fn event_phase_extraction() {
        let ev = InstallEvent::PhaseStarted {
            phase: InstallPhase::DetectSystem,
            message: " detecting...".into(),
        };
        assert_eq!(ev.phase(), Some(InstallPhase::DetectSystem));
    }
}
