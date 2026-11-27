//! Project identity from a current controller attestation, not controller Git lineage.

use super::{
    IdentityCandidate, ProjectSignal, ProjectVerifyRequest, RemoteProjectHint, ScopeRef, Value,
    candidate_payload, identity_name_matches, json, stable_fingerprint,
};
use crate::routes::project_bootstrap::remote::RemoteOnboardingAuthority;

pub(super) fn payload(
    authority: &RemoteOnboardingAuthority,
    hint: &RemoteProjectHint,
    expected: Option<&ProjectVerifyRequest>,
    pi_session_id: Option<&str>,
    projected_scope: Option<&ScopeRef>,
) -> Value {
    let binding = &authority.binding;
    let root = &binding.roots.canonical_remote_root;
    let project = &binding.project;
    let transport = &binding.transport;
    let scope_id = format!("remote-workspace:{}", binding.binding_id);
    let fingerprint = projected_scope
        .map(|scope| scope.fingerprint.clone())
        .unwrap_or_else(|| {
            stable_fingerprint(&[
                scope_id.clone(),
                binding.controller.daemon_identity.clone(),
                project.project_id.clone(),
                project.repo_remote.clone(),
                root.clone(),
                transport.host.clone(),
                transport.user.clone(),
                transport.port.to_string(),
                binding.session.continuity_id.clone(),
                binding.roots.working_subpath.clone().unwrap_or_default(),
                binding.roots.worktree_identity.clone().unwrap_or_default(),
            ])
        });
    let workspace_kind = "remote_ssh";
    let mut mismatches = Vec::new();
    let port = transport.port.to_string();
    let hint_port = hint.remote_port.map(|value| value.to_string());
    for (field, supplied, attested) in [
        (
            "remote_host",
            hint.remote_host.as_deref(),
            Some(transport.host.as_str()),
        ),
        (
            "remote_user",
            hint.remote_user.as_deref(),
            Some(transport.user.as_str()),
        ),
        ("remote_port", hint_port.as_deref(), Some(port.as_str())),
        (
            "remote_repo_remote",
            hint.remote_repo_remote.as_deref(),
            Some(project.repo_remote.as_str()),
        ),
        (
            "remote_deploy_root",
            hint.remote_deploy_root.as_deref(),
            binding.roots.deploy_root.as_deref(),
        ),
        (
            "persisted_project_root",
            hint.persisted_project_root.as_deref(),
            Some(root.as_str()),
        ),
        (
            "persisted_project_id",
            hint.persisted_project_id.as_deref(),
            Some(project.project_id.as_str()),
        ),
    ] {
        if supplied.is_some() && supplied != attested {
            mismatches.push(json!({"source":"controller_attestation","field":field,"severity":"high","expected":supplied,"actual":attested}));
        }
    }
    if let Some(name) = &hint.persisted_canonical_name
        && !identity_name_matches(
            name,
            &project.project_id,
            &project.project_id,
            &[],
            Some(root),
            Some(root),
        )
    {
        mismatches.push(json!({"source":"persisted_session_identity_canonical_name","severity":"high","expected":name,"actual":project.project_id}));
    }
    if let Some(previous) = &hint.persisted_project_fingerprint
        && *previous != fingerprint
    {
        // Previous locator fingerprints were advisory, not a binding grant.
        mismatches.push(json!({"source":"persisted_session_identity_fingerprint","severity":"warning","expected":fingerprint,"actual":previous}));
    }
    let remote_context = json!({
        "schema": "focusa.remote_project_locator.v1",
        "source": "controller_attestation",
        "verification_status": "verified",
        "binding_id": binding.binding_id,
        "continuity_id": binding.session.continuity_id,
        "remote_project_root": root,
        "remote_repo_remote": project.repo_remote,
        "remote_host": transport.host,
        "remote_user": transport.user,
        "remote_port": transport.port,
        "remote_deploy_root": binding.roots.deploy_root,
        "working_subpath": binding.roots.working_subpath,
        "worktree_identity": binding.roots.worktree_identity,
        "locator_fingerprint": fingerprint,
        "verified_at": transport.verified_at,
    });
    let scope = projected_scope
        .cloned()
        .or_else(|| ScopeRef::project(scope_id, root, &project.project_id, &fingerprint).ok());
    let candidate = IdentityCandidate {
        project_id: project.project_id.clone(),
        canonical_name: project.project_id.clone(),
        project_root: root.clone(),
        repo_remote: Some(project.repo_remote.clone()),
        beads_prefix: None,
        workspace_kind: Some(workspace_kind.into()),
        aliases: vec![],
        project_urls: json!({}),
        deployment: json!({"deploy_location":binding.roots.deploy_root,"environment":"remote"}),
        remote_context,
        // Neither Git ancestry nor a checkout on the controller is evidence for this locator.
        working_context: Value::Null,
        fingerprint,
        confidence: if scope.is_some() { "high" } else { "low" },
        status: if scope.is_some() {
            "verified"
        } else {
            "degraded"
        },
        scope_ref: scope,
        signals: vec![ProjectSignal {
            source: "controller_attestation",
            root: Some(root.clone()),
            confidence: "high",
            independent: true,
            details: json!({"binding_id":binding.binding_id,"verified_at":transport.verified_at}),
        }],
        mismatches,
        verified_at: transport.verified_at.clone().unwrap_or_default(),
    };
    let mut payload = candidate_payload(candidate, expected);
    if let Some(session) = pi_session_id {
        payload["pi_session_id"] = json!(session);
    }
    payload["controller_binding"] = json!({
        "binding_id": binding.binding_id,
        "status": binding.state.status,
        "continuity_id": binding.session.continuity_id,
        "project_id": project.project_id,
        "repo_remote": project.repo_remote,
        "canonical_remote_root": root,
        "working_subpath": binding.roots.working_subpath,
        "worktree_identity": binding.roots.worktree_identity,
    });
    payload
}
