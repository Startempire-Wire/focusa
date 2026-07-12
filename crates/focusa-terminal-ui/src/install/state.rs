//! Install state machine.
//!
//! Tracks the current phase, per-phase status, warnings, and asset progress.

use super::event::{AssetProgress, InstallEvent, InstallPhase, VerificationScanOutcome};
use crate::sanitize::sanitize;
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

/// Most recent real verification cycle; never inferred from download state.
#[derive(Debug, Clone, PartialEq)]
pub struct VerificationScanState {
    pub asset: String,
    pub outcome: VerificationScanOutcome,
}

/// Aggregated install state used by renderers.
#[derive(Debug, Clone, PartialEq)]
pub struct InstallState {
    /// Phase statuses in canonical order.
    pub phases: Vec<(InstallPhase, PhaseStatus)>,
    /// Active asset downloads.
    pub assets: HashMap<String, AssetProgress>,
    /// Most recent real verification cycle.
    pub verification_scan: Option<VerificationScanState>,
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
            verification_scan: None,
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
    /// Apply one presentation event while preserving truthful state rules.
    pub fn apply_event(&mut self, event: &InstallEvent) {
        match event {
            InstallEvent::PhaseStarted { phase, message } => {
                self.set_active(*phase);
                self.current_message = sanitize(message).into_owned();
            }
            InstallEvent::PhaseMessage { phase, message } => {
                self.set_active(*phase);
                self.current_message = sanitize(message).into_owned();
            }
            InstallEvent::PhaseSucceeded { phase, .. } => {
                self.set_succeeded(*phase);
            }
            InstallEvent::PhaseSkipped { phase, reason } => {
                self.set_skipped(*phase);
                self.warnings.push(sanitize(reason).into_owned());
            }
            InstallEvent::PhaseWarning { phase, message, .. } => {
                self.set_warning(*phase, sanitize(message).into_owned());
            }
            InstallEvent::PhaseFailed {
                phase,
                message,
                recovery_hint,
            } => {
                self.set_failed(
                    *phase,
                    sanitize(message).into_owned(),
                    recovery_hint
                        .as_deref()
                        .map(|hint| sanitize(hint).into_owned()),
                );
            }
            InstallEvent::AssetStarted { asset, total_bytes } => {
                let asset = sanitize(asset).into_owned();
                self.update_asset(
                    asset.clone(),
                    AssetProgress {
                        asset,
                        downloaded_bytes: 0,
                        total_bytes: *total_bytes,
                    },
                );
            }
            InstallEvent::AssetProgress {
                asset,
                downloaded_bytes,
                total_bytes,
            } => {
                let asset = sanitize(asset).into_owned();
                self.update_asset(
                    asset.clone(),
                    AssetProgress {
                        asset,
                        downloaded_bytes: *downloaded_bytes,
                        total_bytes: *total_bytes,
                    },
                );
            }
            InstallEvent::VerificationScan { asset, outcome } => {
                self.verification_scan = Some(VerificationScanState {
                    asset: sanitize(asset).into_owned(),
                    outcome: *outcome,
                });
            }
            InstallEvent::RollbackStarted { reason } => {
                self.start_rollback(sanitize(reason).into_owned())
            }
            InstallEvent::RollbackSucceeded => self.rollback_succeeded(),
            InstallEvent::RollbackFailed {
                message,
                recovery_hint,
            } => {
                self.failure = Some(sanitize(message).into_owned());
                self.recovery_hint = Some(sanitize(recovery_hint).into_owned());
            }
            InstallEvent::AssetFinished {
                asset,
                downloaded_bytes,
            } => {
                let asset = sanitize(asset).into_owned();
                if let Some(progress) = self.assets.get_mut(&asset) {
                    progress.downloaded_bytes = *downloaded_bytes;
                }
            }
            InstallEvent::InstallFinished { .. } => {}
        }
    }

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
    /// Record byte progress without allowing visual regressions or total changes.
    pub fn update_asset(&mut self, asset: String, progress: AssetProgress) -> bool {
        if let Some(previous) = self.assets.get(&asset) {
            if progress.downloaded_bytes < previous.downloaded_bytes {
                return false;
            }
            if previous.total_bytes.is_some()
                && progress.total_bytes.is_some()
                && previous.total_bytes != progress.total_bytes
            {
                return false;
            }
        }
        self.assets.insert(asset, progress);
        true
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
        let mut s = InstallState::default();
        s.set_active(InstallPhase::DownloadAssets);
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

    #[test]
    fn verification_scan_retains_real_asset_outcome() {
        let mut state = InstallState::default();
        for outcome in [
            VerificationScanOutcome::Active,
            VerificationScanOutcome::Succeeded,
            VerificationScanOutcome::Warning,
            VerificationScanOutcome::Failed,
        ] {
            state.apply_event(&InstallEvent::VerificationScan {
                asset: "focusa-daemon".into(),
                outcome,
            });
            let scan = state.verification_scan.as_ref().unwrap();
            assert_eq!(scan.asset, "focusa-daemon");
            assert_eq!(scan.outcome, outcome);
        }
    }

    #[test]
    fn asset_progress_is_monotonic() {
        let mut state = InstallState::default();
        assert!(state.update_asset(
            "cli".into(),
            AssetProgress {
                asset: "cli".into(),
                downloaded_bytes: 10,
                total_bytes: Some(20)
            }
        ));
        assert!(!state.update_asset(
            "cli".into(),
            AssetProgress {
                asset: "cli".into(),
                downloaded_bytes: 9,
                total_bytes: Some(20)
            }
        ));
        assert!(state.update_asset(
            "cli".into(),
            AssetProgress {
                asset: "cli".into(),
                downloaded_bytes: 20,
                total_bytes: Some(20)
            }
        ));
    }
}
