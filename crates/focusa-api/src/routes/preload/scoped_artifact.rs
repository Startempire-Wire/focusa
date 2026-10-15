//! Opt-in scoped preload artifact; source validation, not authority activation.
//! Self-contained content avoids a parallel artifact ledger. Canonical sources
//! are recomposed on verification; a digest alone never authorizes client data.
use super::*;
use crate::routes::compaction::{
    BuildCompactionPacketRequest, build_packet as cognition_packet, complete_current_ask,
    semantic_digest,
};
use focusa_core::agent_runtime_constitution::{
    RuntimeConstitutionEvent, RuntimeConstitutionLifecycleState,
};
use focusa_core::agent_runtime_constitution_authority::{
    default_authority_graph, detect_conflicts, discover_project_instructions,
};
use sha2::{Digest, Sha256};

pub const SCHEMA: &str = "focusa.scoped_preload_artifact.v2";
type Error = (StatusCode, Json<Value>);

#[derive(serde::Deserialize, Default)]
pub struct Request {
    #[serde(default)]
    pub artifact_version: Option<u32>,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub include_context_cognition: Option<bool>,
    #[serde(default)]
    pub include_awareness: Option<bool>,
    #[serde(default)]
    pub project_root: Option<String>,
    #[serde(default)]
    pub continuity_id: Option<String>,
    #[serde(default)]
    pub working_subpath_id: Option<String>,
    #[serde(default)]
    pub current_ask: Option<String>,
    #[serde(default)]
    pub constitution_id: Option<String>,
    #[serde(default)]
    pub artifact: Option<Value>,
    #[serde(default)]
    pub artifact_ref: Option<String>,
    #[serde(default)]
    pub expected_digest: Option<String>,
    #[serde(default)]
    pub delivery_target: Option<String>,
}
fn blocked(code: &str) -> Error {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(
            json!({"schema":SCHEMA,"status":"blocked","canonical":false,"advisory":true,"error":{"code":code},"verification_scope":"artifact_and_current_canonical_source","delivery_verified":false,"current_process_refresh_verified":false}),
        ),
    )
}

// Stable material contains the exact source fields; generated packet ids and
// wall-clock presentation are not source identities.
async fn material(state: &Arc<AppState>, req: &Request) -> Result<Value, Error> {
    let _fence = state.write_serial_lock.lock().await;
    material_under_fence(state, req).await
}
async fn material_under_fence(state: &Arc<AppState>, req: &Request) -> Result<Value, Error> {
    if req.artifact_version != Some(2) {
        return Err(blocked("unsupported_artifact_version"));
    }
    let root = req
        .project_root
        .as_deref()
        .ok_or_else(|| blocked("project_root_required"))?;
    let continuity = req
        .continuity_id
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| blocked("continuity_id_required"))?;
    let root_path = std::path::Path::new(root);
    if !root_path.is_absolute() || root_path.canonicalize().ok().as_deref() != Some(root_path) {
        return Err(blocked("exact_canonical_project_root_required"));
    }
    let identity = project_identity_payload_for_scope(Some(root), Some(root), None);
    if identity
        .pointer("/project_identity/status")
        .and_then(Value::as_str)
        != Some("verified")
    {
        return Err(blocked("project_identity_unverified"));
    }
    let ask = complete_current_ask(req.current_ask.as_deref())
        .ok_or_else(|| blocked("complete_current_request_required"))?;
    let profile = req.profile.as_deref().unwrap_or(PROFILE_RULES_AND_CONTEXT);
    if profile_by_id(profile).is_none() {
        return Err(blocked("profile_unknown"));
    }
    let id = req
        .constitution_id
        .as_deref()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| blocked("active_constitution_reference_required"))?;
    let constitution = state
        .persistence
        .load_runtime_constitution(id)
        .map_err(|_| blocked("constitution_source_unreadable"))?
        .ok_or_else(|| blocked("constitution_not_found"))?;
    constitution
        .validate()
        .map_err(|_| blocked("constitution_invalid"))?;
    let canonical_scope_id = identity
        .pointer("/project_identity/scope_ref/scope_id")
        .and_then(Value::as_str);
    if constitution.project_ref != root
        && Some(constitution.project_ref.as_str()) != canonical_scope_id
    {
        return Err(blocked("foreign_constitution_scope"));
    }
    let events = state
        .persistence
        .runtime_constitution_events(id)
        .map_err(|_| blocked("constitution_history_unreadable"))?;
    let source_identity = constitution
        .instruction_sources
        .iter()
        .map(|s| s.content_sha256.as_str())
        .collect::<Vec<_>>()
        .join(":");
    let mut approved = false;
    let mut active = false;
    for stored in &events {
        let event: RuntimeConstitutionEvent = serde_json::from_str(&stored.payload_json)
            .map_err(|_| blocked("constitution_history_invalid"))?;
        match event {
            RuntimeConstitutionEvent::RuntimeConstitutionApproved(v) => {
                approved = v.version == constitution.revision.to_string()
                    && v.content_sha256 == source_identity
                    && v.lifecycle == RuntimeConstitutionLifecycleState::Approved;
                active = false;
            }
            RuntimeConstitutionEvent::RuntimeConstitutionActivated(v)
            | RuntimeConstitutionEvent::ContractRollbackActivated(v) => {
                active = approved
                    && v.version == constitution.revision.to_string()
                    && v.content_sha256 == source_identity
                    && v.lifecycle == RuntimeConstitutionLifecycleState::Active;
            }
            RuntimeConstitutionEvent::RuntimeConstitutionRevoked(_) => {
                approved = false;
                active = false;
            }
            RuntimeConstitutionEvent::RuntimeConstitutionDrafted(_) => {
                active = false;
            }
            _ => {}
        }
    }
    if !active {
        return Err(blocked("approved_active_constitution_required"));
    }
    let discovered = discover_project_instructions(root_path, 256 * 1024)
        .map_err(|_| blocked("instruction_sources_unreadable"))?;
    if !discovered.findings.is_empty()
        || !detect_conflicts(&discovered.claims, &default_authority_graph()).is_empty()
    {
        return Err(blocked("instruction_sources_degraded_or_conflicted"));
    }
    if constitution.instruction_sources.len() != discovered.sources.len()
        || constitution.instruction_sources.is_empty()
    {
        return Err(blocked("constitution_instruction_inventory_drift"));
    }
    for accepted in &constitution.instruction_sources {
        if !discovered.sources.iter().any(|current| {
            current.source_id == accepted.source_id
                && current.source_ref == accepted.source_ref
                && current.content_sha256 == accepted.content_sha256
                && current.scope_ref == accepted.scope_ref
                && current.trust == accepted.trust
                && current.authority == accepted.authority
        }) {
            return Err(blocked("constitution_instruction_source_drift"));
        }
    }
    if constitution.claim_refs.is_empty() {
        return Err(blocked("approved_instruction_claims_required"));
    }
    let mut rules = Vec::new();
    for claim_id in &constitution.claim_refs {
        let claim = discovered
            .claims
            .iter()
            .find(|c| &c.claim_id == claim_id)
            .ok_or_else(|| blocked("constitution_claim_drift"))?;
        rules.push(claim.normalized_text.clone());
    }
    let focus = state.focusa.read().await;
    let source = cognition_packet(
        &focus,
        &BuildCompactionPacketRequest {
            resume_source: Some("manual".into()),
            project_root: Some(root.into()),
            continuity_id: Some(continuity.into()),
            current_ask: Some(ask),
            ..Default::default()
        },
    );
    if source["scope"]["scope_status"] != "verified"
        || source["workpoint"]["canonical"] != true
        || source["workpoint"]["status"] != "ready"
        || source["trajectory"]["generic_bootstrap"] == true
    {
        return Err(blocked("canonical_context_not_ready"));
    }
    let wp = active_workpoint_for_context(
        &focus,
        Some(root),
        Some(continuity),
        req.working_subpath_id.as_deref(),
    );
    if wp.map(|v| v.workpoint_id.to_string()).as_deref()
        != source["workpoint"]["workpoint_id"].as_str()
    {
        return Err(blocked("working_subpath_mismatch"));
    }
    let mut temporal = source["temporal"].clone();
    if let Some(fields) = temporal.as_object_mut() {
        fields.remove("as_of");
    }
    Ok(
        json!({"profile_id":profile,"temporal":temporal,"scope":source["scope"],"working_subpath_id":req.working_subpath_id.as_deref().unwrap_or("primary"),"source_revision":source["source_revision"],"current_ask":source["current_ask"],"trajectory":source["trajectory"],"workpoint":source["workpoint"],"focus_state":source["focus_state"],"evidence":source["evidence"],"rehydrate_refs":source["rehydrate_refs"],"constitution":constitution,"approved_rule_lines":rules}),
    )
}
fn compose(material: Value) -> Result<Value, Error> {
    let mut packet = build_packet(
        material["profile_id"]
            .as_str()
            .ok_or_else(|| blocked("profile_missing"))?,
    )
    .map_err(|_| blocked("profile_unknown"))?;
    packet.static_rule_lines = vec![
        serde_json::to_string(&material["constitution"])
            .map_err(|_| blocked("constitution_encoding_failed"))?,
    ];
    packet.static_rule_lines.extend(
        material["approved_rule_lines"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned),
    );
    // Required scoped meaning is composed first; never squeeze a late restriction
    // into a lossy profile selection or leave old template placeholders rendered.
    let mut context = material.clone();
    context
        .as_object_mut()
        .expect("owned material object")
        .remove("constitution");
    packet.dynamic_context_lines =
        vec![serde_json::to_string(&context).map_err(|_| blocked("context_encoding_failed"))?];
    packet.bounded_dynamic_items = packet.dynamic_context_lines.len();
    let rendered = render_packet(&packet);
    if rendered.len() > 512 * 1024 {
        return Err(blocked("required_preload_meaning_over_budget"));
    }
    let body = json!({"schema":SCHEMA,"material":material,"rendered":rendered,"advisory":true});
    let digest = semantic_digest(&body)
        .trim_start_matches("sha256:")
        .to_string();
    Ok(
        json!({"schema":SCHEMA,"artifact_ref":format!("preload:sha256:{digest}"),"content_sha256":digest,"packet":body}),
    )
}
pub async fn build(state: &Arc<AppState>, req: &Request) -> Result<Json<Value>, Error> {
    let artifact = compose(material(state, req).await?)?;
    Ok(Json(
        json!({"schema":SCHEMA,"status":"completed","canonical":false,"advisory":true,"context_kind":"scoped_artifact","artifact":artifact,"delivery_verified":false,"current_process_refresh_verified":false}),
    ))
}
pub async fn validate(state: &Arc<AppState>, req: &Request) -> Result<Value, Error> {
    let _fence = state.write_serial_lock.lock().await;
    validate_under_fence(state, req).await
}
async fn validate_under_fence(state: &Arc<AppState>, req: &Request) -> Result<Value, Error> {
    let supplied = req
        .artifact
        .as_ref()
        .ok_or_else(|| blocked("exact_artifact_required"))?;
    let expected = compose(material_under_fence(state, req).await?)?;
    let digest = expected["content_sha256"]
        .as_str()
        .ok_or_else(|| blocked("digest_missing"))?;
    if req.expected_digest.as_deref() != Some(digest)
        || req.artifact_ref.as_deref() != expected["artifact_ref"].as_str()
        || supplied != &expected
    {
        return Err(blocked("artifact_identity_or_canonical_source_mismatch"));
    }
    Ok(expected)
}
pub async fn inspect(
    state: &Arc<AppState>,
    req: &Request,
    step: &str,
) -> Result<Json<Value>, Error> {
    let artifact = validate(state, req).await?;
    Ok(Json(
        json!({"schema":SCHEMA,"step":step,"status":"verified","canonical":false,"advisory":true,"context_kind":"scoped_artifact","artifact":artifact,"verification_scope":"artifact_and_current_canonical_source","delivery_verified":false,"current_process_refresh_verified":false}),
    ))
}

fn target_path(target: &str) -> Result<std::path::PathBuf, Error> {
    let path = std::path::Path::new(target);
    if !is_safe_target(target) {
        return Err(blocked("unsafe_target_path"));
    }
    Ok(path.to_path_buf())
}
fn delivered_file(artifact: &Value, target: &str) -> Result<Value, Error> {
    let path = target_path(target)?;
    let expected = artifact["packet"]["rendered"]
        .as_str()
        .ok_or_else(|| blocked("rendered_missing"))?;
    let metadata = std::fs::metadata(&path).map_err(|_| blocked("delivery_target_missing"))?;
    if !metadata.is_file() || metadata.len() > 512 * 1024 {
        return Err(blocked("delivery_target_invalid"));
    }
    let bytes = std::fs::read(&path).map_err(|_| blocked("delivery_target_unreadable"))?;
    if bytes != expected.as_bytes() {
        return Err(blocked("delivered_content_mismatch"));
    }
    delivery_descriptor(artifact, target)
}
fn delivery_descriptor(artifact: &Value, target: &str) -> Result<Value, Error> {
    let rendered = artifact["packet"]["rendered"]
        .as_str()
        .ok_or_else(|| blocked("rendered_missing"))?;
    Ok(
        json!({"target":target,"artifact_ref":artifact["artifact_ref"],"content_sha256":hex::encode(Sha256::digest(rendered.as_bytes())),"verification_scope":"exact_file_bytes","current_process_refresh_verified":false}),
    )
}
fn delivery_receipt(artifact: &Value, delivery: Value, key: &str) -> Value {
    json!({"schema":SCHEMA,"scope":artifact["packet"]["material"]["scope"],"working_subpath_id":artifact["packet"]["material"]["working_subpath_id"],"artifact_ref":artifact["artifact_ref"],"content_sha256":artifact["content_sha256"],"delivery":delivery,"idempotency_key":key})
}
fn scoped_receipt_path(receipt: &Value, key: &str) -> std::path::PathBuf {
    receipt_ledger_path(&format!(
        "v2-{}",
        semantic_digest(&json!([
            receipt["scope"],
            receipt["working_subpath_id"],
            key
        ]))
    ))
}
fn receipt_matches(receipt: &Value, key: &str) -> Result<bool, Error> {
    let path = scoped_receipt_path(receipt, key);
    if !path.exists() {
        return Ok(false);
    }
    let saved: Value =
        serde_json::from_slice(&std::fs::read(&path).map_err(|_| blocked("receipt_unreadable"))?)
            .map_err(|_| blocked("receipt_invalid"))?;
    if &saved != receipt {
        return Err(blocked("receipt_identity_conflict"));
    }
    Ok(true)
}
fn persist_delivery_receipt(receipt: &Value, key: &str) -> Result<bool, Error> {
    if receipt_matches(receipt, key)? {
        return Ok(true);
    }
    let path = scoped_receipt_path(receipt, key);
    std::fs::create_dir_all(path.parent().expect("owned receipt parent"))
        .map_err(|_| blocked("receipt_directory_unwritable"))?;
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|_| blocked("receipt_creation_conflict"))?;
    file.write_all(&serde_json::to_vec(receipt).map_err(|_| blocked("receipt_encoding_failed"))?)
        .and_then(|_| file.sync_all())
        .map_err(|_| blocked("receipt_commit_incomplete"))?;
    Ok(false)
}
pub async fn write(
    state: &Arc<AppState>,
    req: &Request,
    target: &str,
    key: &str,
) -> (StatusCode, Json<Value>) {
    let result: Result<Value, Error> = async {
        if key.trim().is_empty() { return Err(blocked("idempotency_key_required")); }
        let _fence = state.write_serial_lock.lock().await;
        let artifact = validate_under_fence(state, req).await?;
        let path = target_path(target)?;
        let receipt = delivery_receipt(&artifact, delivery_descriptor(&artifact, target)?, key);
        receipt_matches(&receipt, key)?;
        let body = artifact["packet"]["rendered"].as_str().ok_or_else(|| blocked("rendered_missing"))?;
        let replay = path.exists();
        if !replay {
            let parent = path.parent().ok_or_else(|| blocked("target_parent_missing"))?;
            std::fs::create_dir_all(parent).map_err(|_| blocked("target_parent_unwritable"))?;
            target_path(target)?;
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&path).map_err(|_| blocked("target_creation_conflict"))?;
            file.write_all(body.as_bytes()).and_then(|_| file.sync_all()).map_err(|_| blocked("artifact_write_incomplete"))?;
        }
        let delivery = delivered_file(&artifact, target)?;
        persist_delivery_receipt(&receipt, key)?;
        Ok(json!({"schema":SCHEMA,"status":"completed","artifact":artifact,"delivery":delivery,"idempotent_replay":replay,"delivery_verified":true,"current_process_refresh_verified":false}))
    }.await;
    match result {
        Ok(body) => (StatusCode::OK, Json(body)),
        Err(error) => error,
    }
}
pub async fn commit_receipt(
    state: &Arc<AppState>,
    req: &Request,
    key: &str,
) -> (StatusCode, Json<Value>) {
    let result: Result<Value, Error> = async {
        if key.trim().is_empty() { return Err(blocked("idempotency_key_required")); }
        let _fence = state.write_serial_lock.lock().await;
        let artifact = validate_under_fence(state, req).await?;
        let target = req.delivery_target.as_deref().ok_or_else(|| blocked("exact_delivery_target_required"))?;
        let delivery = delivered_file(&artifact, target)?;
        let receipt = delivery_receipt(&artifact, delivery, key);
        let replay = persist_delivery_receipt(&receipt, key)?;
        Ok(json!({"schema":SCHEMA,"status":"completed","artifact":artifact,"receipt":receipt,"idempotent_replay":replay,"delivery_verified":true,"current_process_refresh_verified":false}))
    }.await;
    match result {
        Ok(body) => (StatusCode::OK, Json(body)),
        Err(error) => error,
    }
}
