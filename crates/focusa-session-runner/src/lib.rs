//! Protected per-user execution substrate for daemon-native Silent Sessions.
//!
//! The daemon remains canonical. This crate owns only authenticated runner
//! communication and operating-system process supervision.

pub mod identity;
#[cfg(unix)]
pub mod mutation_posix;
#[cfg(unix)]
pub mod process_posix;
pub mod protocol;
#[cfg(unix)]
pub mod transport;
