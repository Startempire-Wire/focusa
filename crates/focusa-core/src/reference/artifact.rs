//! Artifact types and prompt representation.

use crate::types::HandleRef;

/// Format a handle for prompt inclusion.
///
/// Prompt representation: `[HANDLE:<kind>:<id> "<label>"]`
pub fn to_prompt_ref(handle: &HandleRef) -> String {
    format!(
        "[HANDLE:{:?}:{} \"{}\"]",
        handle.kind, handle.id, handle.label
    )
}
