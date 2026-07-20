//! Protected per-user execution substrate for daemon-native Silent Sessions.
//!
//! The daemon remains canonical. This crate owns only authenticated runner
//! communication and operating-system process supervision.

pub mod identity;
pub mod protocol;
