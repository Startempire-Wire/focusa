//! Read-only qualification of the existing Genesis journal and marker.
//! Project history is not another Workstream's readiness or session admission.
use super::*;

const STATUS_SCHEMA: &str = "focusa.project_genesis_status.v1";

pub(super) async fn read(
    scope: ScopeContext,
    query: ProjectGenesisStatusQuery,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match crate::routes::project::project_inspection_task(move || inspect(scope, query)).await {
        Ok(result) => result,
        Err(failure) => Err((StatusCode::SERVICE_UNAVAILABLE, failure)),
    }
}

fn inspect(
    scope: ScopeContext,
    query: ProjectGenesisStatusQuery,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let query_root = canonical_root(&query.project_root)?;
    let root = match scope.project_root.as_deref() {
        Some(value) => canonical_root(value)?,
        None => query_root.clone(),
    };
    let active_root = scope
        .active_worktree_root
        .as_deref()
        .and_then(|value| fs::canonicalize(value).ok());
    let query_continuity = clean(query.continuity_id.as_deref());
    let context_continuity = clean(scope.continuity_id.as_deref());
    let requested_continuity = query_continuity.clone().or(context_continuity.clone());
    let requested = json!({
        "project_root":query_root,
        "continuity_id":requested_continuity,
        "working_subpath_id":scope.working_subpath_id,
        "active_worktree_root":active_root,
        "session_id":scope.session_id
    });
    if (query_root != root && active_root.as_ref() != Some(&query_root))
        || matches!((&query_continuity, &context_continuity), (Some(a), Some(b)) if a != b)
    {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({
                "schema":STATUS_SCHEMA, "status":"scope_conflict", "canonical":false,
                "code":"GENESIS_STATUS_SCOPE_CONFLICT", "requested_scope":requested,
                "resolved_scope":Value::Null,
                "reason":"explicit Genesis query and request scope disagree"
            })),
        ));
    }
    let packet = read_json(&packet_path(&root)).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "schema":STATUS_SCHEMA, "status":"not_started", "canonical":false,
                "code":"genesis_not_started", "requested_scope":requested,
                "resolved_scope":Value::Null,
                "next_action":"inspect existing scoped saved work before considering initialization"
            })),
        )
    })?;
    let record_root = clean(packet["authority"]["scope"].as_str());
    let identity_root = clean(packet["project_identity"]["project_root"].as_str());
    let actual_continuity = clean(packet["authority"]["continuity_id"].as_str());
    let resolved = json!({
        "project_root":record_root.as_ref().or(identity_root.as_ref()),
        "continuity_id":actual_continuity,
        "working_subpath_id":packet["authority"]["working_subpath_id"],
        "session_id":packet["authority"]["session_id"]
    });
    let canonical_root_string = root.to_string_lossy();
    let identity_matches = record_root.as_deref() == Some(canonical_root_string.as_ref())
        && identity_root.as_deref() == Some(canonical_root_string.as_ref())
        && actual_continuity.is_some()
        && actual_continuity == requested_continuity;
    let marker = read_json(&root.join(".focusa-project.json")).unwrap_or(Value::Null);
    let binding = &marker["genesis_binding"];
    let references_match = [
        (&binding["workpoint_id"], &packet["first_workpoint"]["workpoint_id"]),
        (&binding["receipt_id"], &packet["readiness_receipt"]["receipt_id"]),
        (&binding["trajectory_id"], &packet["readiness_receipt"]["trajectory_id"]),
    ]
    .iter()
    .all(|(left, right)| {
        matches!((left.as_str(), right.as_str()), (Some(a), Some(b)) if !a.trim().is_empty() && a == b)
    });
    let committed = binding["status"] == "ready"
        && binding["continuity_id"].as_str() == actual_continuity.as_deref()
        && references_match
        && packet["first_workpoint"]["canonical"] == true;
    let supported =
        packet["schema"] == GENESIS_SCHEMA && packet["project_identity"]["verified"] == true;
    let ready = packet["status"] == "ready";
    let reason = if requested_continuity.is_none() {
        "legacy_unqualified"
    } else if !supported || record_root.is_none() || actual_continuity.is_none() {
        "identity_missing"
    } else if record_root != identity_root {
        "ambiguous_identity"
    } else if !identity_matches {
        "scope_mismatch"
    } else if ready && !committed {
        "stale_readiness"
    } else if !ready {
        "not_ready"
    } else {
        "verified"
    };
    let canonical = reason == "verified";
    let validation = json!({
        "status":reason, "scope_kind":"project_workstream_record",
        "canonical_project_root":root,
        "request_working_subpath_id":scope.working_subpath_id,
        "session_binding":"not_asserted", "execution_admission":false
    });
    let mut result = if canonical || (identity_matches && supported && !ready) {
        packet.clone()
    } else {
        json!({
            "schema":STATUS_SCHEMA,
            "status":if requested_continuity.is_none() { "historical" } else { reason },
            "next_action":"resume existing saved work in the requested scope; historical Genesis does not authorize initialization or takeover",
            "historical_project_receipt":{
                "continuity_id":actual_continuity,
                "receipt_id":packet["readiness_receipt"]["receipt_id"],
                "workpoint_id":packet["first_workpoint"]["workpoint_id"]
            }
        })
    };
    if requested_continuity.is_none() {
        // Supported legacy project-level inspection is explicit history, never
        // current-workstream readiness; the saved journal itself is untouched.
        result["project_history"] = packet;
    }
    result["status_projection_schema"] = json!(STATUS_SCHEMA);
    result["canonical"] = json!(canonical);
    result["requested_scope"] = requested;
    result["resolved_scope"] = resolved;
    result["scope_validation"] = validation;
    Ok(Json(result))
}
