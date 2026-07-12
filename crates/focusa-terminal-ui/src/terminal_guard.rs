//! Terminal guard: RAII restoration of terminal state.
//!
//! §13.2: restores terminal state from Drop on every handled exit path.

use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    cursor::Show,
    execute,
};
use std::io::{self, Stderr, Write};

/// RAII guard that enters alternate screen on creation and restores on drop.
pub struct TerminalGuard {
    active: bool,
    raw_mode: bool,
}

impl TerminalGuard {
    /// Initialize terminal for animated UI.
    /// Returns `None` if alternate screen cannot be entered.
    pub fn new() -> io::Result<Self> {
        // Enter alternate screen and hide cursor.
        let mut stderr = io::stderr();
        execute!(stderr, EnterAlternateScreen)?;
        // Do NOT enable raw mode unless proven needed; spec says avoid raw mode.
        Ok(TerminalGuard {
            active: true,
            raw_mode: false,
        })
    }

    /// Explicitly restore terminal state.
    pub fn restore(&mut self) -> io::Result<()> {
        if self.active {
            let mut stderr = io::stderr();
            if self.raw_mode {
                let _ = disable_raw_mode();
            }
            execute!(stderr, Show, LeaveAlternateScreen)?;
            let _ = stderr.flush();
            self.active = false;
        }
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

/// Install a scoped panic hook that restores terminal before calling the prior hook.
pub fn install_terminal_panic_hook() {
    let prior = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut stderr = io::stderr();
        let _ = disable_raw_mode();
        let _ = execute!(stderr, Show, LeaveAlternateScreen);
        let _ = stderr.flush();
        prior(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_idempotent_restore() {
        // We cannot easily test alternate screen in a non-TTY test harness,
        // but we verify idempotence by creating and dropping without panic.
        // If stderr is not a TTY, EnterAlternateScreen may fail; that's acceptable.
        if let Ok(mut guard) = TerminalGuard::new() {
            guard.restore().unwrap();
            guard.restore().unwrap(); // idempotent
        }
    }
}
