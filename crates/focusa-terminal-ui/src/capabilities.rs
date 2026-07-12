//! Terminal capability detection and renderer mode selection.
//!
//! §12: stderr, TERM, CI, size, env controls.

use std::io::IsTerminal;

/// Selected renderer mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallRendererMode {
    TrueColorAnimated,
    Ansi256Animated,
    MonochromeAnimated,
    ReducedMotion,
    Plain,
    Silent,
}

impl InstallRendererMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            InstallRendererMode::TrueColorAnimated => "truecolor_animated",
            InstallRendererMode::Ansi256Animated => "ansi256_animated",
            InstallRendererMode::MonochromeAnimated => "monochrome_animated",
            InstallRendererMode::ReducedMotion => "reduced_motion",
            InstallRendererMode::Plain => "plain",
            InstallRendererMode::Silent => "silent",
        }
    }

    pub const fn is_animated(self) -> bool {
        matches!(
            self,
            InstallRendererMode::TrueColorAnimated
                | InstallRendererMode::Ansi256Animated
                | InstallRendererMode::MonochromeAnimated
                | InstallRendererMode::ReducedMotion
        )
    }

    pub const fn is_silent(self) -> bool {
        matches!(self, InstallRendererMode::Silent)
    }
}

/// Detected terminal capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalCapabilities {
    pub stderr_is_terminal: bool,
    pub term: String,
    pub ci: bool,
    pub no_color: bool,
    pub reduced_motion_env: bool,
    pub size_columns: u16,
    pub size_rows: u16,
    pub color_depth: ColorDepth,
    pub mode: InstallRendererMode,
    pub minimum_size_met: bool,
    /// Deterministic diagnostic seed; never serialized into install JSON.
    pub animation_seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorDepth {
    TrueColor,
    Ansi256,
    Monochrome,
}

/// Validate installer UI controls before any installer mutation.
pub fn validate_environment() -> Result<(), String> {
    if let Ok(value) = std::env::var("FOCUSA_INSTALL_UI") {
        if !matches!(
            value.as_str(),
            "auto" | "full" | "mono" | "reduced" | "plain"
        ) {
            return Err(format!(
                "invalid FOCUSA_INSTALL_UI={value:?}; use auto|full|mono|reduced|plain"
            ));
        }
    }
    if let Ok(value) = std::env::var("FOCUSA_INSTALL_SEED") {
        value
            .parse::<u64>()
            .map_err(|_| "FOCUSA_INSTALL_SEED must be an unsigned 64-bit integer".to_string())?;
    }
    if let Ok(value) = std::env::var("FOCUSA_REDUCE_MOTION") {
        if !matches!(value.as_str(), "0" | "1") {
            return Err("FOCUSA_REDUCE_MOTION must be 0 or 1".to_string());
        }
    }
    Ok(())
}

/// Detect capabilities from the current environment.
pub fn detect_capabilities(no_animation: bool, json: bool, quiet: bool) -> TerminalCapabilities {
    let stderr_term = std::io::stderr().is_terminal();
    let term = std::env::var("TERM").unwrap_or_default();
    let ci = std::env::var_os("CI").is_some();
    let no_color = std::env::var_os("NO_COLOR").is_some()
        || std::env::var("CLICOLOR")
            .map(|value| value == "0")
            .unwrap_or(false);
    let reduced_motion_env = std::env::var("FOCUSA_REDUCE_MOTION")
        .map(|v| v == "1")
        .unwrap_or(false);

    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let minimum_size_met = cols >= 70 && rows >= 22;

    let color_depth = detect_color_depth(&term, no_color);

    let mode = select_mode(
        json,
        quiet,
        no_animation,
        ci,
        stderr_term,
        &term,
        minimum_size_met,
        no_color,
        reduced_motion_env,
        color_depth,
    );

    TerminalCapabilities {
        stderr_is_terminal: stderr_term,
        term,
        ci,
        no_color,
        reduced_motion_env,
        size_columns: cols,
        size_rows: rows,
        color_depth,
        mode,
        minimum_size_met,
        animation_seed: animation_seed(),
    }
}

pub fn animation_seed() -> u64 {
    std::env::var("FOCUSA_INSTALL_SEED")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0xF0C0_5A_u64)
}

fn detect_color_depth(term: &str, no_color: bool) -> ColorDepth {
    if no_color {
        return ColorDepth::Monochrome;
    }
    if std::env::var("COLORTERM")
        .map(|v| v.eq_ignore_ascii_case("truecolor") || v.eq_ignore_ascii_case("24bit"))
        .unwrap_or(false)
        || term.contains("256color")
            && std::env::var("TERM_PROGRAM")
                .map(|v| v == "iTerm.app" || v == "WezTerm" || v == "Ghostty")
                .unwrap_or(false)
    {
        ColorDepth::TrueColor
    } else if term.contains("256color") || term.contains("-256") {
        ColorDepth::Ansi256
    } else {
        ColorDepth::Monochrome
    }
}

#[allow(clippy::too_many_arguments)]
fn select_mode(
    json: bool,
    quiet: bool,
    no_animation: bool,
    ci: bool,
    stderr_term: bool,
    term: &str,
    min_size_met: bool,
    no_color: bool,
    reduced_motion_env: bool,
    color_depth: ColorDepth,
) -> InstallRendererMode {
    if json || quiet {
        return InstallRendererMode::Silent;
    }
    if no_animation || ci || !stderr_term || term == "dumb" || term.is_empty() || !min_size_met {
        return InstallRendererMode::Plain;
    }

    // Environment override
    if let Ok(ui) = std::env::var("FOCUSA_INSTALL_UI") {
        match ui.as_str() {
            "plain" => return InstallRendererMode::Plain,
            "reduced" => {
                return if reduced_motion_env || no_color {
                    InstallRendererMode::ReducedMotion
                } else {
                    InstallRendererMode::ReducedMotion
                }
            }
            "mono" => return InstallRendererMode::MonochromeAnimated,
            "full" => (),
            _ => (),
        }
    }

    if reduced_motion_env {
        InstallRendererMode::ReducedMotion
    } else if no_color {
        InstallRendererMode::MonochromeAnimated
    } else {
        match color_depth {
            ColorDepth::TrueColor => InstallRendererMode::TrueColorAnimated,
            ColorDepth::Ansi256 => InstallRendererMode::Ansi256Animated,
            ColorDepth::Monochrome => InstallRendererMode::MonochromeAnimated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_forces_silent() {
        let cap = select_mode(
            true,
            false,
            false,
            false,
            true,
            "xterm-256color",
            true,
            false,
            false,
            ColorDepth::TrueColor,
        );
        assert_eq!(cap, InstallRendererMode::Silent);
    }

    #[test]
    fn ci_forces_plain() {
        let cap = select_mode(
            false,
            false,
            false,
            true,
            true,
            "xterm-256color",
            true,
            false,
            false,
            ColorDepth::TrueColor,
        );
        assert_eq!(cap, InstallRendererMode::Plain);
    }

    #[test]
    fn dumb_term_forces_plain() {
        let cap = select_mode(
            false,
            false,
            false,
            false,
            true,
            "dumb",
            true,
            false,
            false,
            ColorDepth::TrueColor,
        );
        assert_eq!(cap, InstallRendererMode::Plain);
    }

    #[test]
    fn no_color_uses_monochrome_not_plain() {
        let cap = select_mode(
            false,
            false,
            false,
            false,
            true,
            "xterm-256color",
            true,
            true,
            false,
            ColorDepth::TrueColor,
        );
        assert_eq!(cap, InstallRendererMode::MonochromeAnimated);
    }
}
