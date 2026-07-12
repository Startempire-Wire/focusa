//! Presenter implementations.
//!
//! §4.3: AnimatedPresenter, MonochromeAnimatedPresenter, ReducedMotionPresenter,
//!       PlainPresenter, SilentPresenter.

use super::event::{AssetProgress, InstallEvent, InstallPhase};
use super::state::{InstallState, PhaseStatus};
use super::completion::InstallCompletionSummary;
use crate::sanitize::sanitize;
use std::sync::{Arc, Mutex};

/// Trait for all presenters.
pub trait Presenter: Send {
    /// Consume an event and update presentation state.
    fn handle_event(&mut self, event: &InstallEvent);
    /// Returns true if this presenter uses the animated terminal UI.
    fn is_animated(&self) -> bool;
    /// Render the final durable human summary.
    fn render_final_summary(&self, summary: &InstallCompletionSummary);
    /// Render a durable error after terminal restoration.
    fn render_error(&self, phase: InstallPhase, message: &str, recovery_hint: Option<&str>);
}

/// Plain text presenter: prints phase lines directly to stderr/stdout.
pub struct PlainPresenter {
    state: InstallState,
    quiet: bool,
}

impl PlainPresenter {
    pub fn new(quiet: bool) -> Self {
        PlainPresenter {
            state: InstallState::default(),
            quiet,
        }
    }

    fn print(&self, line: &str) {
        if !self.quiet {
            eprintln!("{}", sanitize(line));
        }
    }
}

impl Presenter for PlainPresenter {
    fn handle_event(&mut self, event: &InstallEvent) {
        match event {
            InstallEvent::PhaseStarted { phase, message } => {
                self.state.set_active(*phase);
                self.print(&format!("{} {}", InstallState::status_symbol(PhaseStatus::Active), phase.label()));
                if !message.is_empty() {
                    self.print(message);
                }
            }
            InstallEvent::PhaseSucceeded { phase, detail } => {
                self.state.set_succeeded(*phase);
                let symbol = InstallState::status_symbol(PhaseStatus::Succeeded);
                if let Some(d) = detail {
                    self.print(&format!("{} {} — {}", symbol, phase.label(), sanitize(d)));
                } else {
                    self.print(&format!("{} {}", symbol, phase.label()));
                }
            }
            InstallEvent::PhaseSkipped { phase, reason } => {
                self.state.set_skipped(*phase);
                self.print(&format!("{} {} — skipped: {}", InstallState::status_symbol(PhaseStatus::Skipped), phase.label(), sanitize(reason)));
            }
            InstallEvent::PhaseWarning { phase, message, recovery_hint } => {
                self.state.set_warning(*phase, message.clone());
                self.print(&format!("! {} — {}", phase.label(), sanitize(message)));
                if let Some(h) = recovery_hint {
                    self.print(&format!("  recovery: {}", sanitize(h)));
                }
            }
            InstallEvent::PhaseFailed { phase, message, recovery_hint } => {
                self.state.set_failed(*phase, message.clone(), recovery_hint.clone());
                self.print(&format!("✗ {} — {}", phase.label(), sanitize(message)));
                if let Some(h) = recovery_hint {
                    self.print(&format!("  recovery: {}", sanitize(h)));
                }
            }
            InstallEvent::AssetStarted { asset, total_bytes } => {
                if let Some(total) = total_bytes {
                    self.print(&format!("  → {} ({} bytes)", sanitize(asset), total));
                } else {
                    self.print(&format!("  → {}", sanitize(asset)));
                }
            }
            InstallEvent::AssetProgress { asset, downloaded_bytes, total_bytes } => {
                if let Some(total) = total_bytes {
                    let pct = *downloaded_bytes as f64 / *total as f64 * 100.0;
                    self.print(&format!("  → {} {:.1}%", sanitize(asset), pct));
                } else {
                    self.print(&format!("  → {} {} bytes", sanitize(asset), downloaded_bytes));
                }
            }
            InstallEvent::RollbackStarted { reason } => {
                self.state.start_rollback(reason.clone());
                self.print(&format!("↶ Rolling back safely: {}", sanitize(reason)));
            }
            InstallEvent::RollbackSucceeded => {
                self.state.rollback_succeeded();
                self.print("↶ Rollback completed");
            }
            InstallEvent::RollbackFailed { message, recovery_hint } => {
                self.print(&format!("✗ Rollback failed: {}", sanitize(message)));
                self.print(&format!("  recovery: {}", sanitize(recovery_hint)));
            }
            InstallEvent::InstallFinished { summary } => {
                if !self.quiet {
                    println!("{}", summary.render_human());
                }
            }
            _ => {}
        }
    }

    fn is_animated(&self) -> bool { false }

    fn render_final_summary(&self, summary: &InstallCompletionSummary) {
        if !self.quiet {
            println!("{}", summary.render_human());
        }
    }

    fn render_error(&self, _phase: InstallPhase, message: &str, recovery_hint: Option<&str>) {
        eprintln!("Install failed: {}", sanitize(message));
        if let Some(h) = recovery_hint {
            eprintln!("Recovery: {}", sanitize(h));
        }
    }
}

/// Silent presenter: emits nothing except errors.
pub struct SilentPresenter;

impl Presenter for SilentPresenter {
    fn handle_event(&mut self, _event: &InstallEvent) {}
    fn is_animated(&self) -> bool { false }
    fn render_final_summary(&self, _summary: &InstallCompletionSummary) {}
    fn render_error(&self, _phase: InstallPhase, message: &str, recovery_hint: Option<&str>) {
        eprintln!("Install failed: {}", sanitize(message));
        if let Some(h) = recovery_hint {
            eprintln!("Recovery: {}", sanitize(h));
        }
    }
}

/// Shared state for animated presenters (truecolor, 256, mono, reduced).
pub struct AnimatedPresenterState {
    pub state: Arc<Mutex<InstallState>>,
    pub should_exit: Arc<Mutex<bool>>,
}

impl AnimatedPresenterState {
    pub fn new() -> Self {
        AnimatedPresenterState {
            state: Arc::new(Mutex::new(InstallState::default())),
            should_exit: Arc::new(Mutex::new(false)),
        }
    }
}

/// TrueColor animated presenter with full Hybrid AC.
pub struct AnimatedPresenter {
    shared: Arc<AnimatedPresenterState>,
}

impl AnimatedPresenter {
    pub fn new(shared: Arc<AnimatedPresenterState>) -> Self {
        AnimatedPresenter { shared }
    }
}

impl Presenter for AnimatedPresenter {
    fn handle_event(&mut self, event: &InstallEvent) {
        if let Ok(mut st) = self.shared.state.lock() {
            match event {
                InstallEvent::PhaseStarted { phase, .. } => st.set_active(*phase),
                InstallEvent::PhaseSucceeded { phase, .. } => st.set_succeeded(*phase),
                InstallEvent::PhaseSkipped { phase, .. } => st.set_skipped(*phase),
                InstallEvent::PhaseWarning { phase, message, .. } => st.set_warning(*phase, message.clone()),
                InstallEvent::PhaseFailed { phase, message, recovery_hint } => st.set_failed(*phase, message.clone(), recovery_hint.clone()),
                InstallEvent::AssetStarted { asset, total_bytes } => {
                    st.update_asset(asset.clone(), AssetProgress {
                        asset: asset.clone(),
                        downloaded_bytes: 0,
                        total_bytes: *total_bytes,
                    });
                }
                InstallEvent::AssetProgress { asset, downloaded_bytes, total_bytes } => {
                    st.update_asset(asset.clone(), AssetProgress {
                        asset: asset.clone(),
                        downloaded_bytes: *downloaded_bytes,
                        total_bytes: *total_bytes,
                    });
                }
                InstallEvent::RollbackStarted { .. } => st.rollback_active = true,
                InstallEvent::RollbackSucceeded => st.rollback_active = false,
                _ => {}
            }
        }
    }

    fn is_animated(&self) -> bool { true }

    fn render_final_summary(&self, summary: &InstallCompletionSummary) {
        println!("{}", summary.render_human());
    }

    fn render_error(&self, _phase: InstallPhase, message: &str, recovery_hint: Option<&str>) {
        eprintln!("Install failed: {}", sanitize(message));
        if let Some(h) = recovery_hint {
            eprintln!("Recovery: {}", sanitize(h));
        }
    }
}

/// Monochrome animated presenter (no color but still animated).
pub struct MonochromeAnimatedPresenter {
    inner: AnimatedPresenter,
}

impl MonochromeAnimatedPresenter {
    pub fn new(shared: Arc<AnimatedPresenterState>) -> Self {
        MonochromeAnimatedPresenter {
            inner: AnimatedPresenter::new(shared),
        }
    }
}

impl Presenter for MonochromeAnimatedPresenter {
    fn handle_event(&mut self, event: &InstallEvent) {
        self.inner.handle_event(event);
    }
    fn is_animated(&self) -> bool { true }
    fn render_final_summary(&self, summary: &InstallCompletionSummary) {
        self.inner.render_final_summary(summary);
    }
    fn render_error(&self, phase: InstallPhase, message: &str, recovery_hint: Option<&str>) {
        self.inner.render_error(phase, message, recovery_hint);
    }
}

/// Reduced motion presenter (static frame updates on phase changes).
pub struct ReducedMotionPresenter {
    inner: AnimatedPresenter,
}

impl ReducedMotionPresenter {
    pub fn new(shared: Arc<AnimatedPresenterState>) -> Self {
        ReducedMotionPresenter {
            inner: AnimatedPresenter::new(shared),
        }
    }
}

impl Presenter for ReducedMotionPresenter {
    fn handle_event(&mut self, event: &InstallEvent) {
        self.inner.handle_event(event);
    }
    fn is_animated(&self) -> bool { true }
    fn render_final_summary(&self, summary: &InstallCompletionSummary) {
        self.inner.render_final_summary(summary);
    }
    fn render_error(&self, phase: InstallPhase, message: &str, recovery_hint: Option<&str>) {
        self.inner.render_error(phase, message, recovery_hint);
    }
}

