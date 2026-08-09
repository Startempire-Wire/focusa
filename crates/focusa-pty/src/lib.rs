//! PTY-004/005/006 — governed persistent Pi PTY runtime.
//!
//! One persistent Pi process per governed Attachment, backed by a real PTY
//! library (`portable-pty`). Ordinary child-process pipes are never used.
//! Every event carries the exact AttachmentKey, WorkSurfaceId, run
//! generation, and a monotonic sequence; output from a stale generation is
//! rejected.

pub mod events;
pub mod identity;
pub mod process;
pub mod registry;
