//! Visual theme — calm, organic, non-demanding.
//!
//! Per docs/11-menubar-ui-spec.md:
//! - Background: off-white / dark
//! - Primary: charcoal / grayscale
//! - Accent: light navy
//! - Focused: mid-gray
//! - Nothing demands attention

use ratatui::style::{Color, Modifier, Style};

// Charcoal/grayscale palette for dark terminal.
pub const _BG: Color = Color::Reset;
pub const FG: Color = Color::Gray;
pub const FG_DIM: Color = Color::DarkGray;
pub const ACCENT: Color = Color::Rgb(100, 130, 180); // light navy
pub const ACTIVE: Color = Color::Rgb(180, 200, 230);  // bright focus
pub const WARN: Color = Color::Rgb(200, 160, 80);     // warm amber
pub const ERROR: Color = Color::Rgb(180, 80, 80);     // soft red
pub const SUCCESS: Color = Color::Rgb(80, 160, 100);  // calm green
pub const BORDER: Color = Color::DarkGray;

pub fn title() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn tab_active() -> Style {
    Style::default().fg(ACTIVE).add_modifier(Modifier::BOLD)
}

pub fn tab_inactive() -> Style {
    Style::default().fg(FG_DIM)
}

pub fn label() -> Style {
    Style::default().fg(FG_DIM)
}

pub fn value() -> Style {
    Style::default().fg(FG)
}

pub fn highlight() -> Style {
    Style::default().fg(ACCENT)
}

pub fn status_ok() -> Style {
    Style::default().fg(SUCCESS)
}

pub fn status_err() -> Style {
    Style::default().fg(ERROR)
}

pub fn status_warn() -> Style {
    Style::default().fg(WARN)
}

pub fn border() -> Style {
    Style::default().fg(BORDER)
}
