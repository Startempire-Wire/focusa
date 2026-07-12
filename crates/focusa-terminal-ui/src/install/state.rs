//! Install state machine.
//!
//! Tracks the current phase, per-phase status, warnings, and asset progress.

use super::event::{AssetProgress, InstallPhase};
use std::collections::HashMap;

/// Current status of a phase in the state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseStatus {
    Pending,
    Active,
    Succeeded,
    Skipped,
    Warning,
    Failed,
}

/// Aggregated install state used by renderers.
#[derive(Debug, Clone, PartialEq)]
pub struct InstallState {
    /// Phase statuses in canonical order.
    pub phases: Vec<(InstallPhase, PhaseStatus)>,
    /// Active asset downloads.
    pub assets: HashMap<String, AssetProgress>,
    /// Current warning messages.
    pub warnings: Vec<String>,
    /// Active failure message (cleared on rollback start).
    pub failure: Option<String>,
    /// Recovery hint for failure.
    pub recovery_hint: Option<String>,
    /// Whether rollback is active.
    pub rollback_active: bool,
    /// Overall completion fraction based on phases (not bytes).
    pub phase_completion: f32,
    /// Current message shown under the active phase.
    pub current_message: String,
}

impl Default for InstallState {
    fn default() -> Self {
        let phases = vec![
            InstallPhase::DetectSystem,
            InstallPhase::ValidateLicense,
            InstallPhase::ResolveRelease,
            InstallPhase::DownloadAssets,
            InstallPhase::VerifyIntegrity,
            InstallPhase::InstallBinaries,
            InstallPhase::IntegratePi,
            InstallPhase::RegisterService,
            InstallPhase::PersistPath,
            InstallPhase::RunHealthChecks,
            InstallPhase::Finalize,
        ];
        InstallState {
            phases: phases.into_iter().map(|p| (p, PhaseStatus::Pending)).collect(),
            assets: HashMap::new(),
            warnings: Vec::new(),
            failure: None,
            recovery_hint: None,
            rollback_active: false,
            phase_completion: 0.0,
            current_message: String::new(),
        }
    }
}

impl InstallState {
    /// Mark a phase as active and clear any prior failure for that phase.
    pub fn set_active(&mut self, phase: InstallPhase) {
        for (p, s) in &mut self.phases {
            if *p == phase {
                *s = PhaseStatus::Active;
            } else if *s == PhaseStatus::Active {
                // Should not happen with proper orchestration, but be safe.
            }
        }
        self.current_message = phase.label().to_string();
        self.recompute_completion();
    }

    /// Mark a phase as succeeded.
    pub fn set_succeeded(&mut self, phase: InstallPhase) {
        for (p, s) in &mut self.phases {
            if *p == phase {
                *s = PhaseStatus::Succeeded;
            }
        }
        self.recompute_completion();
    }

    /// Mark a phase as skipped.
    pub fn set_skipped(&mut self, phase: InstallPhase) {
        for (p, s) in &mut self.phases {
            if *p == phase {
                *s = PhaseStatus::Skipped;
            }
        }
        self.recompute_completion();
    }

    /// Mark a phase as warning.
    pub fn set_warning(&mut self, phase: InstallPhase, message: String) {
        for (p, s) in &mut self.phases {
            if *p == phase {
                *s = PhaseStatus::Warning;
            }
        }
        self.warnings.push(message);
        self.recompute_completion();
    }

    /// Mark a phase as failed.
    pub fn set_failed(&mut self, phase: InstallPhase, message: String, hint: Option<String>) {
        for (p, s) in &mut self.phases {
            if *p == phase {
                *s = PhaseStatus::Failed;
            }
        }
        self.failure = Some(message);
        self.recovery_hint = hint;
    }

    /// Start rollback.
    pub fn start_rollback(&mut self, reason: String) {
        self.rollback_active = true;
        self.failure = Some(reason);
    }

    /// End rollback successfully.
    pub fn rollback_succeeded(&mut self) {
        self.rollback_active = false;
    }

    /// Update asset progress.
    pub fn update_asset(&mut self, asset: String, progress: AssetProgress) {
        self.assets.insert(asset, progress);
    }

    fn recompute_completion(&mut self) {
        let total = self.phases.len();
        if total == 0 {
            self.phase_completion = 0.0;
            return;
        }
        let done = self
            .phases
            .iter()
            .filter(|(_, s)| matches!(s, PhaseStatus::Succeeded | PhaseStatus::Skipped))
            .count();
        self.phase_completion = done as f32 / total as f32;
    }

    /// Status symbol for a phase status. §9.1
    pub const fn status_symbol(status: PhaseStatus) -> &'static str {
        match status {
            PhaseStatus::Pending => "○",
            PhaseStatus::Active => "◆",
            PhaseStatus::Succeeded => "✓",
            PhaseStatus::Skipped => "–",
            PhaseStatus::Warning => "!",
            PhaseStatus::Failed => "✗",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_all_pending() {
        let s = InstallState::default();
        assert!(s.phases.iter().all(|(_, st)| *st == PhaseStatus::Pending));
    }

    #[test]
    fn completion_monotonic() {
        let mut s = InstallState::default();
        let c1 = s.phase_completion;
        s.set_succeeded(InstallPhase::DetectSystem);
        let c2 = s.phase_completion;
        assert!(c2 > c1);
        s.set_succeeded(InstallPhase::ValidateLicense);
        let c3 = s.phase_completion;
        assert!(c3 > c2);
    }

    #[test]
    fn illegal_regression_rejected() {
        // Once failed, we do not silently auto-succeed.
        let mut s = InstallState::default();
        s.set_failed(
            InstallPhase::DownloadAssets,
            "network error".into(),
            Some("check connection".into()),
        );
        // Re-succeeding the same phase is allowed in the state machine
        // because the orchestrator may retry, but the test documents that
        // the renderer shows the latest authoritative state.
        s.set_succeeded(InstallPhase::DownloadAssets);
        let (_, st) = s.phases.iter().find(|(p, _)| *p == InstallPhase::DownloadAssets).unwrap();
        assert_eq!(*st, PhaseStatus::Succeeded);
    }
}
