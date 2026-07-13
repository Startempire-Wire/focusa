//! Focusa Terminal UI — shared presentation library for the installer.
//!
//! Implements Spec 132: animated terminal experience with Hybrid AC visual design.
//! This crate is a pure presentation library; it contains no installation logic,
//! no HTTP clients, no license validation, and no file mutations.

pub mod capabilities;
pub mod install;
pub mod sanitize;
pub mod terminal_guard;

pub use capabilities::{
    InstallRendererMode, TerminalCapabilities, animation_seed, detect_capabilities,
    validate_environment,
};
pub use install::completion::InstallCompletionSummary;
pub use install::event::{
    AssetProgress, InstallEvent, InstallEventSink, InstallPhase, VerificationScanOutcome,
};
pub use install::presenter::{
    AnimatedPresenter, AnimatedPresenterState, MonochromeAnimatedPresenter, PlainPresenter,
    Presenter, ReducedMotionPresenter, SilentPresenter, presenter_for_mode,
};
pub use install::renderer::{AnimatedRenderLoop, HybridRenderer};
pub use install::state::InstallState;
pub use sanitize::sanitize;
pub use terminal_guard::{
    CancellationToken, PanicHookGuard, SignalGuard, TerminalGuard, install_signal_handlers,
    install_terminal_panic_hook,
};
