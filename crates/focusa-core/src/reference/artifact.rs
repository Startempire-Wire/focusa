//! Artifact types and prompt representation.

use crate::types::{HandleKind, HandleRef};

/// Format a handle for prompt inclusion.
///
/// Prompt representation: `[HANDLE:<kind>:<id> "<label>"]`
/// Kind uses snake_case to match serde serialization.
pub fn to_prompt_ref(handle: &HandleRef) -> String {
    let kind_str = handle_kind_str(handle.kind);
    let escaped_label = handle.label.replace('\\', "\\\\").replace('"', "\\\"");
    format!("[HANDLE:{}:{} \"{}\"]", kind_str, handle.id, escaped_label)
}

pub fn handle_kind_str(kind: HandleKind) -> &'static str {
    match kind {
        HandleKind::Log => "log",
        HandleKind::Diff => "diff",
        HandleKind::Text => "text",
        HandleKind::Json => "json",
        HandleKind::Url => "url",
        HandleKind::FileSnapshot => "file_snapshot",
        HandleKind::Other => "other",
    }
}
