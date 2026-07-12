//! Focusa Terminal UI — shared presentation library for the installer.
//!
//! Implements Spec 132: animated terminal experience with Hybrid AC visual design.
//! This crate is a pure presentation library; it contains no installation logic,
//! no HTTP clients, no license validation, and no file mutations.

pub mod capabilities;
pub mod install;
pub mod sanitize;
pub mod terminal_guard;

pub use capabilities::{InstallRendererMode, TerminalCapabilities, detect_capabilities};
pub use install::event::{AssetProgress, InstallEvent, InstallEventSink, InstallPhase};
pub use install::completion::InstallCompletionSummary;
pub use install::presenter::{Presenter, PlainPresenter, SilentPresenter, AnimatedPresenter, MonochromeAnimatedPresenter, ReducedMotionPresenter, AnimatedPresenterState};
pub use install::state::InstallState;
pub use sanitize::sanitize;
pub use terminal_guard::TerminalGuard;
