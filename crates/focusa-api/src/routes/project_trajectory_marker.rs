//! Derived per-workstream marker projections; canonical ledgers remain authoritative.

use super::{ProjectTrajectoryBindingRecord, TrajectoryIntegrityGuardRecord};
use serde_json::{Value, json};

const FIELD: &str = "workstream_trajectory_bindings";
const SCHEMA: &str = "focusa.project_workstream_bindings.v1";

/// Select only the requested canonical workstream's cache, retaining legacy
/// compatibility when that workstream has not yet acquired a scoped projection.
pub(super) fn projection<'a>(
    marker: &'a Value,
    project_root: &str,
    continuity_id: Option<&str>,
) -> Result<&'a Value, String> {
    let Some(store) = marker.get(FIELD) else {
        return Ok(marker);
    };
    if store.get("schema").and_then(Value::as_str) != Some(SCHEMA) {
        return Err("unsupported workstream marker projection schema".to_string());
    }
    let entries = store
        .get("entries")
        .and_then(Value::as_object)
        .ok_or_else(|| "workstream marker entries must be an object".to_string())?;
    let continuity = continuity_id
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "canonical workstream continuity is required".to_string())?;
    let Some(entry) = entries.get(continuity) else {
        return Ok(marker);
    };
    if entry.get("project_root").and_then(Value::as_str) != Some(project_root)
        || entry.get("continuity_id").and_then(Value::as_str) != Some(continuity)
    {
        return Err("workstream marker projection scope mismatch".to_string());
    }
    Ok(entry)
}

/// Update only one derived projection. Never replace another workstream's legacy
/// singleton, entries, shared project identity or extension metadata.
pub(super) fn write_projection(
    marker: &mut Value,
    project_root: &str,
    continuity_id: &str,
    binding: &ProjectTrajectoryBindingRecord,
    guard: &TrajectoryIntegrityGuardRecord,
) -> Result<(), String> {
    if continuity_id.trim().is_empty() {
        return Err("canonical workstream continuity is required".to_string());
    }
    projection(marker, project_root, Some(continuity_id))?;
    let binding_value = serde_json::to_value(binding).map_err(|error| error.to_string())?;
    let guard_value = serde_json::to_value(guard).map_err(|error| error.to_string())?;
    // A trajectory ID is not a Workstream qualifier. Existing unqualified
    // metadata must remain intact, even when scopes share an ID or a partial slot.
    let update_legacy = marker.get("trajectory_binding").is_none()
        && marker.get("trajectory_integrity_guard").is_none();
    let object = marker
        .as_object_mut()
        .ok_or_else(|| "project marker must be an object".to_string())?;
    {
        let store = object
            .entry(FIELD.to_string())
            .or_insert_with(|| json!({"schema": SCHEMA, "entries": {}}));
        let entries = store
            .get_mut("entries")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| "workstream marker entries must be an object".to_string())?;
        let entry = entries
            .entry(continuity_id.to_string())
            .or_insert_with(|| json!({}));
        let entry = entry
            .as_object_mut()
            .ok_or_else(|| "workstream marker projection must be an object".to_string())?;
        entry.insert("project_root".to_string(), json!(project_root));
        entry.insert("continuity_id".to_string(), json!(continuity_id));
        entry.insert("trajectory_binding".to_string(), binding_value.clone());
        entry.insert(
            "trajectory_integrity_guard".to_string(),
            guard_value.clone(),
        );
    }
    object.insert("schema".to_string(), json!("focusa.project.v2"));
    if update_legacy {
        object.insert("trajectory_binding".to_string(), binding_value);
        object.insert("trajectory_integrity_guard".to_string(), guard_value);
    }
    Ok(())
}
