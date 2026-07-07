//! ASCII intros and interactive prompt helpers for the focusa CLI.
//!
//! Operator directive (2026-07-05):
//! - Beautiful intros over flat menus (Focusa wordmark + ambient banner).
//! - Interactive selectors over plain `--scope project|host` flags where the
//!   surrounding TTY allows; falls back to safe defaults otherwise.
//! - Calm colors that survive non-UTF8 terminals (all bytes are valid ANSI).
//!
//! This module avoids pulling extra dependencies; it stays portable across
//! the CI runners and the Mac/VPS operator shells.

use std::time::Duration;

pub const FOCUSA_BOLD: &str = "\x1b[1m";
pub const FOCUSA_RESET: &str = "\x1b[0m";
pub const FOCUSA_PRIMARY: &str = "\x1b[38;5;39m";
pub const FOCUSA_ACCENT: &str = "\x1b[38;5;213m";
pub const FOCUSA_DIM: &str = "\x1b[38;5;245m";

// 5-line ASCII wordmark rendering the name "FOCUSA" with block characters.
pub const FOCUSA_WORDMARK: &str = "\
    \x20\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\x20\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\x20\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\x20\u{2588}\u{2588}\x20\x20\x20\u{2588}\u{2588}\x20\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\x20\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\x20\x20\n\
    \x20\u{2588}\u{2588}\x20\x20\x20\x20\x20\x20\u{2588}\u{2588}\x20\x20\x20\u{2588}\u{2588}\x20\u{2588}\u{2588}\x20\x20\x20\x20\x20\x20\u{2588}\u{2588}\x20\x20\x20\u{2588}\u{2588}\x20\u{2588}\u{2588}\x20\x20\x20\x20\x20\x20\u{2588}\u{2588}\x20\x20\x20\u{2588}\u{2588}\n\
    \x20\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\x20\x20\x20\u{2588}\u{2588}\x20\x20\x20\u{2588}\u{2588}\x20\u{2588}\u{2588}\x20\x20\x20\x20\x20\x20\u{2588}\u{2588}\x20\x20\x20\u{2588}\u{2588}\x20\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\x20\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\n\
    \x20\u{2588}\u{2588}\x20\x20\x20\x20\x20\x20\u{2588}\u{2588}\x20\x20\x20\u{2588}\u{2588}\x20\u{2588}\u{2588}\x20\x20\x20\x20\x20\x20\u{2588}\u{2588}\x20\x20\x20\u{2588}\u{2588}\x20\x20\x20\x20\x20\x20\u{2588}\u{2588}\x20\u{2588}\u{2588}\x20\x20\x20\u{2588}\u{2588}\n\
    \x20\u{2588}\u{2588}\x20\x20\x20\x20\x20\x20\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\x20\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\x20\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\x20\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\x20\u{2588}\u{2588}\x20\x20\x20\u{2588}\u{2588}\n\
    \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\
";

pub const FOCUSA_TAGLINE: &str = "cognitive governance runtime";

pub fn render_wordmark() -> String {
    format!(
        "{bold}{primary}{word}{reset}\n{dim}{tagline}{reset}\n",
        bold = FOCUSA_BOLD,
        primary = FOCUSA_PRIMARY,
        word = FOCUSA_WORDMARK,
        reset = FOCUSA_RESET,
        tagline = FOCUSA_TAGLINE,
        dim = FOCUSA_DIM,
    )
}

pub fn render_help_banner() -> String {
    let mut out = String::new();
    out.push_str(&render_wordmark());
    out.push_str(&format!(
        "{dim}---------- 60-second quickstart ----------{reset}\n  \
         bash scripts/install-daemon.sh /usr/local\n  \
         focusa start && sleep 2\n  \
         focusa init --quickstart\n\n\
         {dim}---------- canonical subcommands ----------{reset}\n",
        dim = FOCUSA_DIM,
        reset = FOCUSA_RESET,
    ));
    out
}

pub fn render_about_banner(version: &str, owner: Option<&str>, repo: Option<&str>) -> String {
    let mut out = String::new();
    out.push_str(&render_wordmark());
    out.push_str(&format!(
        "{accent}one-line:{reset} Focusa turns long AI chat into long-running AI project work.\n",
        accent = FOCUSA_ACCENT,
        reset = FOCUSA_RESET,
    ));
    out.push_str(&format!(
        "{dim}version{reset} {ver}  {dot}  {dim}repo{reset} {repo}\n",
        dim = FOCUSA_DIM,
        reset = FOCUSA_RESET,
        ver = version,
        dot = "•",
        repo = repo.unwrap_or("Startempire-Wire/focusa"),
    ));
    if let Some(label) = owner {
        out.push_str(&format!(
            "{dim}owner:{reset} {label}\n",
            label = label,
            dim = FOCUSA_DIM,
            reset = FOCUSA_RESET
        ));
    }
    out
}

pub fn render_onboard_banner(project_root_label: &str, scope_label: &str) -> String {
    let mut out = String::new();
    out.push_str(&render_wordmark());
    out.push_str(&format!(
        "{primary}Focusa operator preview onboarding{reset}\n",
        primary = FOCUSA_PRIMARY,
        reset = FOCUSA_RESET,
    ));
    out.push_str(&format!(
        "{dim}project_root:{reset} {root}\n{dim}scope:{reset} {scope}\n",
        dim = FOCUSA_DIM,
        reset = FOCUSA_RESET,
        root = project_root_label,
        scope = scope_label,
    ));
    out.push_str(&format!(
        "{dim}step:{reset} {accent}bind{reset}  →  {accent}check{reset}  →  {accent}orient{reset}\n",
        dim = FOCUSA_DIM,
        accent = FOCUSA_ACCENT,
        reset = FOCUSA_RESET,
    ));
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptIntent {
    Interactive,
    NonInteractive,
}

pub fn detect_prompt_intent() -> PromptIntent {
    match std::env::var("FOCUSA_FORCE_INTERACTIVE").ok().as_deref() {
        Some("0") => return PromptIntent::NonInteractive,
        Some("1") => return PromptIntent::Interactive,
        _ => {}
    }
    let stdin_tty =
        matches!(std::env::var("FOCUSA_STDIN_TTY").ok().as_deref(), Some("1")) || atty_stdin();
    let stdout_tty = matches!(
        std::env::var("FOCUSA_STDOUT_TTY").ok().as_deref(),
        Some("1")
    ) || atty_stdout();
    if stdin_tty && stdout_tty {
        PromptIntent::Interactive
    } else {
        PromptIntent::NonInteractive
    }
}

fn atty_stdin() -> bool {
    let term = std::env::var("TERM").unwrap_or_default();
    !term.is_empty() && term != "dumb"
}

fn atty_stdout() -> bool {
    atty_stdin()
}

pub const SCOPE_CHOICES: [&str; 2] = ["project", "host"];

/// Interactive scope picker. Returns the chosen index. Falls back to the
/// default (project = 0) when the terminal is non-interactive or no TTY
/// detail is supplied.
pub fn pick_scope_intent<F>(intent: PromptIntent, choose: F) -> usize
where
    F: FnOnce(&[&str]) -> usize,
{
    if intent == PromptIntent::Interactive {
        return choose(&SCOPE_CHOICES);
    }
    0
}

pub fn ease_in() -> Duration {
    Duration::from_millis(120)
}
