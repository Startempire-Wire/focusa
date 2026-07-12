//! Terminal lifecycle and cancellation primitives.
//!
//! §13: alternate-screen restoration is RAII, panic hooks are scoped, and
//! cancellation is explicit rather than swallowed by presentation.

use crossterm::{
    cursor::Show,
    execute,
    terminal::{disable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen},
};
use signal_hook::consts::SIGINT;
#[cfg(unix)]
use signal_hook::consts::SIGTERM;
use std::io::{self, stderr, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// RAII guard that enters alternate screen and always restores it on drop.
pub struct TerminalGuard {
    active: bool,
    raw_mode: bool,
}

impl TerminalGuard {
    pub fn new() -> io::Result<Self> {
        let mut output = stderr();
        execute!(output, EnterAlternateScreen)?;
        Ok(Self {
            active: true,
            raw_mode: false,
        })
    }

    pub fn restore(&mut self) -> io::Result<()> {
        if self.active {
            let mut output = stderr();
            if self.raw_mode {
                let _ = disable_raw_mode();
            }
            execute!(output, Show, LeaveAlternateScreen)?;
            let _ = output.flush();
            self.active = false;
        }
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Read the current terminal size for resize-aware render loops.
    pub fn size(&self) -> io::Result<(u16, u16)> {
        size()
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

/// Scoped panic hook. Dropping it restores the hook that was installed before it.
pub struct PanicHookGuard {
    prior: Option<Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Send + Sync + 'static>>,
    shared: Arc<Mutex<Option<Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Send + Sync + 'static>>>>,
}

pub fn install_terminal_panic_hook() -> PanicHookGuard {
    let prior = std::panic::take_hook();
    let shared = Arc::new(Mutex::new(Some(prior)));
    let for_hook = Arc::clone(&shared);
    std::panic::set_hook(Box::new(move |info| {
        let mut output = stderr();
        let _ = disable_raw_mode();
        let _ = execute!(output, Show, LeaveAlternateScreen);
        let _ = output.flush();
        if let Ok(prior) = for_hook.lock() {
            if let Some(hook) = prior.as_ref() {
                hook(info);
            }
        }
    }));
    let prior = shared.lock().ok().and_then(|mut value| value.take());
    PanicHookGuard { prior, shared }
}

impl Drop for PanicHookGuard {
    fn drop(&mut self) {
        let _ = std::panic::take_hook();
        if let Some(prior) = self.prior.take() {
            std::panic::set_hook(prior);
        }
        if let Ok(mut shared) = self.shared.lock() {
            shared.take();
        }
    }
}

/// Signal registrations are scoped and removed when the presenter exits.
pub struct SignalGuard {
    ids: Vec<signal_hook::SigId>,
}

pub fn install_signal_handlers(token: &CancellationToken) -> io::Result<SignalGuard> {
    let mut ids = Vec::new();
    ids.push(signal_hook::flag::register(SIGINT, token.0.clone()).map_err(io::Error::other)?);
    #[cfg(unix)]
    ids.push(signal_hook::flag::register(SIGTERM, token.0.clone()).map_err(io::Error::other)?);
    Ok(SignalGuard { ids })
}

impl Drop for SignalGuard {
    fn drop(&mut self) {
        for id in self.ids.drain(..) {
            signal_hook::low_level::unregister(id);
        }
    }
}

/// Shared cancellation state used by the installer and renderer.
#[derive(Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_idempotent_restore() {
        if let Ok(mut guard) = TerminalGuard::new() {
            assert!(guard.is_active());
            guard.restore().unwrap();
            guard.restore().unwrap();
            assert!(!guard.is_active());
        }
    }

    #[test]
    fn cancellation_is_shared() {
        let token = CancellationToken::new();
        let copy = token.clone();
        assert!(!copy.is_cancelled());
        token.cancel();
        assert!(copy.is_cancelled());
    }

    #[test]
    fn signal_registration_is_scoped() {
        let token = CancellationToken::new();
        let guard = install_signal_handlers(&token).expect("signal handlers register");
        assert!(!token.is_cancelled());
        drop(guard);
        assert!(!token.is_cancelled());
    }
}
