//! Stored-graph preflight consumed by the HTTP route and its registry snapshot.
use focusa_core::callgraph::{
    AdapterCapability, Disposition, FocusaCallGraphDefinition, eligibility_for_frame_with_adapters,
    validate_graph,
};
use rusqlite::Connection;
use serde_json::{Value, json};

pub(super) fn registered_adapters(conn: &Connection) -> anyhow::Result<Vec<AdapterCapability>> {
    focusa_core::adapter_registry::ensure_schema(conn)?;
    Ok(focusa_core::adapter_registry::list_adapters(conn)?
        .into_iter()
        .map(|record| AdapterCapability {
            adapter_id: record.adapter_id,
            model: record.model,
            capabilities: record.capabilities,
            healthy: record.healthy,
        })
        .collect())
}

pub(super) fn preflight_stored_graph(
    conn: &Connection,
    graph_id: &str,
    revision: u64,
) -> anyhow::Result<Value> {
    focusa_core::callgraph_store::ensure_schema(conn)?;
    let Some(stored) = focusa_core::callgraph_store::load_definition(conn, graph_id, revision)?
    else {
        return Ok(
            json!({"status":"rejected_missing_definition","graph_id":graph_id,"revision":revision}),
        );
    };
    let graph: FocusaCallGraphDefinition = serde_json::from_str(&stored.definition_json)?;
    let report = validate_graph(&graph);
    if !report.valid {
        return Ok(json!({"status":"rejected_invalid","issues":report.issues}));
    }
    let adapters = registered_adapters(conn)?;
    let mut blockers = Vec::new();
    for entry in &graph.entry_frame_ids {
        let disposition = eligibility_for_frame_with_adapters(
            &graph,
            entry,
            None,
            &std::collections::HashSet::new(),
            &adapters,
        );
        if disposition != Disposition::Eligible {
            blockers.push(json!({"frame_id":entry,"disposition":disposition}));
        }
    }
    Ok(
        json!({"status":if blockers.is_empty(){"preflighted"}else{"blocked"},"graph_id":graph_id,"revision":revision,"blockers":blockers}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stored_preflight_consumes_real_registry_health_and_capabilities() {
        let conn = Connection::open_in_memory().unwrap();
        focusa_core::callgraph_store::ensure_schema(&conn).unwrap();
        let mut graph: FocusaCallGraphDefinition = serde_json::from_str(include_str!(
            "../../../focusa-core/tests/fixtures/callgraph-golden.v1.json"
        ))
        .unwrap();
        graph.scope.project_root = std::env::current_dir().unwrap().display().to_string();
        let entry = graph.entry_frame_ids[0].clone();
        graph.frames.retain(|frame| frame.frame_id == entry);
        graph.edges.clear();
        graph.entry_frame_ids = vec![entry];
        graph.workpoint_refs.clear();
        graph.frames[0].preconditions.clear();
        graph.frames[0].authority_requirement = None;
        graph.frames[0].capability_refs = vec!["cap.verify".into()];
        focusa_core::callgraph_store::upsert_definition(&conn, &graph).unwrap();
        let before = preflight_stored_graph(&conn, &graph.graph_id, graph.revision).unwrap();
        assert_eq!(before["status"], "blocked");
        assert_eq!(before["blockers"][0]["disposition"], "waiting_capability");
        let adapter = focusa_core::adapter_registry::AdapterRecord {
            adapter_id: "test-adapter".into(),
            model: "test-model".into(),
            harness: "test".into(),
            capabilities: vec!["cap.verify".into()],
            healthy: true,
            last_seen: "2026-01-01T00:00:00Z".into(),
        };
        focusa_core::adapter_registry::upsert_adapter(&conn, &adapter).unwrap();
        let after = preflight_stored_graph(&conn, &graph.graph_id, graph.revision).unwrap();
        assert_eq!(after["status"], "preflighted");
        assert_eq!(after["blockers"], json!([]));
        focusa_core::adapter_registry::set_healthy(
            &conn,
            &adapter.adapter_id,
            &adapter.model,
            false,
        )
        .unwrap();
        assert_eq!(
            preflight_stored_graph(&conn, &graph.graph_id, graph.revision).unwrap()["status"],
            "blocked"
        );
    }
}
