//! Bounded Spec133 Silent Session export rendering.

use std::path::PathBuf;

use focusa_core::silent_sessions::{
    OutputChannel, SecureStreamStore, SilentSessionId, SilentSessionRunId,
};
use serde_json::{Value, json};

use crate::server::AppState;

pub(super) fn export_output(
    state: &AppState,
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
) -> anyhow::Result<Value> {
    let configured = PathBuf::from(&state.config.data_dir);
    let data_root = if configured.is_absolute() {
        configured
    } else {
        std::env::current_dir()?.join(configured)
    };
    let store =
        SecureStreamStore::new(data_root.join("silent-sessions"), state.persistence.clone())?;
    let channels = [
        OutputChannel::Stdout,
        OutputChannel::Stderr,
        OutputChannel::StructuredHarnessEvents,
        OutputChannel::AssistantText,
        OutputChannel::ThinkingText,
        OutputChannel::ToolCalls,
        OutputChannel::ToolOutput,
        OutputChannel::FocusaControlEvents,
        OutputChannel::OperatorInput,
        OutputChannel::SystemDiagnostics,
    ];
    let mut output = serde_json::Map::new();
    for channel in channels {
        let mut cursor = None;
        let mut records = Vec::new();
        for _ in 0..10_000 {
            let (page, next_cursor) =
                store.read_after(session_id, run_id, channel, cursor.as_deref(), 1_000)?;
            let page_empty = page.is_empty();
            records.extend(page);
            if page_empty || next_cursor.is_none() || next_cursor == cursor {
                cursor = next_cursor;
                break;
            }
            cursor = next_cursor;
        }
        output.insert(
            channel.as_str().to_string(),
            json!({"records": records, "next_cursor": cursor}),
        );
    }
    Ok(Value::Object(output))
}

pub(super) fn export_as_jsonl(bundle: &Value) -> anyhow::Result<String> {
    let mut keys = bundle
        .as_object()
        .map(|object| object.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    keys.sort();
    let mut lines = Vec::with_capacity(keys.len());
    for key in keys {
        let value = &bundle[&key];
        lines.push(serde_json::to_string(&json!({
            "kind": key,
            "data": value,
        }))?);
    }
    Ok(lines.join("\n"))
}
