//! Letta adapter — wraps Letta CLI as a subprocess.
//!
//! Mode A — CLI subprocess wrapping.
//!
//! Focusa starts the Letta CLI, mediates I/O:
//!   1. Read user prompt
//!   2. Assemble Focusa-enhanced prompt via Expression Engine
//!   3. Send to Letta stdin
//!   4. Stream Letta stdout back to user
//!   5. Emit turn events to daemon
//!
//! Letta adapter declares limited capabilities because
//! tool output capture is best-effort (depends on Letta's output format).

use crate::adapters::openai::AdapterCapabilities;

/// Letta adapter capability declaration.
pub fn capabilities() -> AdapterCapabilities {
    AdapterCapabilities {
        streaming: true,           // Letta streams output
        tool_output_capture: false, // Best-effort only
        structured_messages: false, // CLI mode uses plain text
    }
}
