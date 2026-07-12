//! Installer presentation modules.

pub mod canvas;
pub mod completion;
pub mod continuity_core;
pub mod event;
pub mod glow_base;
pub mod layout;
pub mod matrix_rain;
pub mod palette;
pub mod presenter;
pub mod renderer;
pub mod state;

pub use canvas::BlockCanvas;
pub use completion::InstallCompletionSummary;
pub use continuity_core::ContinuityCore;
pub use event::{AssetProgress, InstallEvent, InstallPhase, VerificationScanOutcome};
pub use glow_base::GlowBase;
pub use layout::{Layout, LayoutKind};
pub use matrix_rain::MatrixRain;
pub use palette::{Ansi256Palette, MonochromePalette, TrueColorPalette};
pub use presenter::{
    presenter_for_mode, AnimatedPresenter, MonochromeAnimatedPresenter, PlainPresenter, Presenter,
    ReducedMotionPresenter, SilentPresenter,
};
pub use renderer::{AnimatedRenderLoop, HybridRenderer};
pub use state::InstallState;
