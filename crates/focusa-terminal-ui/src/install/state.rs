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
            InstallPhase::InitializeEnvironment,
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
            InstallPhase::Complete,
        ];
        InstallState {
            phases: phases
                .into_iter()
                .map(|p| (p, PhaseStatus::Pending))
                .collect(),
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
    fn transition(&mut self, phase: InstallPhase, next: PhaseStatus) -> bool {
        let Some((_, current)) = self.phases.iter_mut().find(|(p, _)| *p == phase) else {
            return false;
        };
        let legal = match (*current, next) {
            (PhaseStatus::Pending, PhaseStatus::Active | PhaseStatus::Skipped) => true,
            (
                PhaseStatus::Active,
                PhaseStatus::Succeeded | PhaseStatus::Warning | PhaseStatus::Failed,
            ) => true,
            (PhaseStatus::Warning | PhaseStatus::Failed, PhaseStatus::Active) => true,
            (a, b) if a == b => true,
            _ => false,
        };
        if legal {
            *current = next;
            self.recompute_completion();
        }
        legal
    }

    /// Mark a phase active. Returns false when the transition is illegal.
    pub fn set_active(&mut self, phase: InstallPhase) -> bool {
        let changed = self.transition(phase, PhaseStatus::Active);
        if changed {
            self.current_message = phase.label().to_string();
        }
        changed
    }

    pub fn set_succeeded(&mut self, phase: InstallPhase) -> bool {
        self.transition(phase, PhaseStatus::Succeeded)
    }

    pub fn set_skipped(&mut self, phase: InstallPhase) -> bool {
        self.transition(phase, PhaseStatus::Skipped)
    }

    pub fn set_warning(&mut self, phase: InstallPhase, message: String) -> bool {
        let changed = self.transition(phase, PhaseStatus::Warning);
        if changed {
            self.warnings.push(message);
        }
        changed
    }

    pub fn set_failed(
        &mut self,
        phase: InstallPhase,
        message: String,
        hint: Option<String>,
    ) -> bool {
        let changed = self.transition(phase, PhaseStatus::Failed);
        if changed {
            self.failure = Some(message);
            self.recovery_hint = hint;
        }
        changed
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
        s.set_active(InstallPhase::DetectSystem);
        s.set_succeeded(InstallPhase::DetectSystem);
        let c2 = s.phase_completion;
        assert!(c2 > c1);
        s.set_active(InstallPhase::ValidateLicense);
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
        assert!(!s.set_succeeded(InstallPhase::DownloadAssets));
        assert!(s.set_active(InstallPhase::DownloadAssets));
        assert!(s.set_succeeded(InstallPhase::DownloadAssets));
        let (_, st) = s
            .phases
            .iter()
            .find(|(p, _)| *p == InstallPhase::DownloadAssets)
            .unwrap();
        assert_eq!(*st, PhaseStatus::Succeeded);
    }
}
