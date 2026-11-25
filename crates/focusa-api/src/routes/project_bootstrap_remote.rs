//! Remote Bootstrap entry. The generic binding POST is not an attestation source.

use super::{AppState, HeaderMap, Json, ProjectBootstrapRequest, StatusCode, Value, json, reject};
use crate::{
    middleware::principal::{ApiPrincipalSource, request_principal},
    routes::events_sqlite::focusa_db_path,
};
use focusa_core::remote_workspace::{
    RemoteOnboardingScope, RemoteWorkspaceBinding, adopt_remote_bootstrap,
    attest_allowed_workspace, load_controller_attestation, preview_allowed_workspace,
    read_remote_bootstrap,
};

pub(in crate::routes) const MAX_PROBE_AGE: chrono::TimeDelta = chrono::TimeDelta::minutes(5);

pub(in crate::routes) enum Operation {
    Preview,
    Apply,
    Status,
}

pub(in crate::routes) async fn inspect(
    state: &AppState,
    headers: &HeaderMap,
    req: &ProjectBootstrapRequest,
    operation: Operation,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let authority = resolve_allowed(headers, req.remote_binding_id.as_deref()).await?;
    if !matches!(operation, Operation::Status)
        && (req.initialize_git != Some(false)
            || req.initialize_task_provider != Some(false)
            || req
                .task_provider
                .as_deref()
                .is_some_and(|provider| provider != "beads"))
    {
        return Err(reject(
            StatusCode::BAD_REQUEST,
            "remote_bootstrap_adopt_only",
            "explicitly disable Git and task initialization to adopt the existing checkout",
        ));
    }
    if !matches!(operation, Operation::Status)
        && req.project_id != authority.binding.project.project_id
    {
        return Err(authority_required());
    }
    let scope = authority.scope(&req.project_root, &req.continuity_id);
    let path = focusa_db_path(&state.config.data_dir);
    let max_age = MAX_PROBE_AGE;
    let blocked = |_| {
        reject(
            StatusCode::PRECONDITION_FAILED,
            "remote_bootstrap_verification_failed",
            "the attested binding or owner-side workspace could not be verified",
        )
    };
    if matches!(operation, Operation::Status) {
        let ledger = open_ledger(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let receipt = if let Some(attested) = load_controller_attestation(
            &ledger,
            &scope,
            &authority.binding,
            chrono::Utc::now(),
            max_age,
        )
        .map_err(blocked)?
        {
            read_remote_bootstrap(&ledger, &scope, &attested, chrono::Utc::now(), max_age)
                .map_err(blocked)?
        } else {
            None
        };
        return Ok(Json(json!({
            "schema": "focusa.remote_bootstrap_result.v1",
            "status": if receipt.is_some() { "onboarding_required" } else { "not_started" },
            "binding_id": scope.binding_id, "project_root": scope.remote_root, "continuity_id": scope.continuity_id,
            "genesis_ready": false, "receipt": receipt,
        })));
    }
    let apply = matches!(operation, Operation::Apply);
    let receipt = if apply {
        let inspection = attest_allowed_workspace(&path, &scope, &authority.binding)
            .await
            .map_err(blocked)?;
        let mut ledger = open_ledger(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        Some(
            adopt_remote_bootstrap(
                &mut ledger,
                &scope,
                &inspection,
                max_age,
                &req.idempotency_key,
            )
            .map_err(blocked)?,
        )
    } else {
        let inventory = preview_allowed_workspace(&path, &scope, &authority.binding)
            .await
            .map_err(blocked)?;
        if !inventory.beads_present {
            return Err(reject(
                StatusCode::PRECONDITION_FAILED,
                "remote_bootstrap_existing_beads_required",
                "existing Beads export not found",
            ));
        }
        None
    };
    Ok(Json(json!({
        "schema": "focusa.remote_bootstrap_result.v1",
        "status": if apply { "onboarding_required" } else { "preview_ready" },
        "binding_id": scope.binding_id, "project_root": scope.remote_root, "continuity_id": scope.continuity_id,
        "task_provider": "beads", "genesis_ready": false, "receipt": receipt,
    })))
}

pub(in crate::routes) struct RemoteOnboardingAuthority {
    pub binding: RemoteWorkspaceBinding,
    principal_id: String,
}

impl RemoteOnboardingAuthority {
    pub(in crate::routes) fn scope<'a>(
        &'a self,
        root: &'a str,
        continuity: &'a str,
    ) -> RemoteOnboardingScope<'a> {
        let attested = &self.binding;
        RemoteOnboardingScope {
            binding_id: &attested.binding_id,
            controller: &attested.controller,
            project: &attested.project,
            host: &attested.transport.host,
            user: &attested.transport.user,
            port: attested.transport.port,
            remote_root: root,
            working_subpath: attested.roots.working_subpath.as_deref(),
            worktree_identity: attested.roots.worktree_identity.as_deref(),
            continuity_id: continuity,
            principal: &self.principal_id,
        }
    }
}

fn authority_required() -> (StatusCode, Json<Value>) {
    reject(
        StatusCode::FORBIDDEN,
        "remote_onboarding_authority_required",
        "remote onboarding requires controller-attested scope and administrator authority",
    )
}

pub(in crate::routes) async fn resolve_authority(
    state: &AppState,
    headers: &HeaderMap,
    binding_id: Option<&str>,
    root: &str,
    continuity: &str,
) -> Result<RemoteOnboardingAuthority, (StatusCode, Json<Value>)> {
    let mut authority = resolve_allowed(headers, binding_id).await?;
    let path = focusa_db_path(&state.config.data_dir);
    let ledger = open_ledger(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let attested = load_controller_attestation(
        &ledger, &authority.scope(root, continuity), &authority.binding, chrono::Utc::now(), MAX_PROBE_AGE,
    ).map_err(|error| {
        tracing::warn!(failure = %error, "remote onboarding verification not current");
        reject(StatusCode::PRECONDITION_FAILED, "remote_onboarding_verification_failed", "verify the exact registered binding and refresh its owner-side probe if expired")
    })?.ok_or_else(|| reject(StatusCode::PRECONDITION_FAILED, "remote_onboarding_current_probe_required", "apply remote Bootstrap with the original replay key to verify this allowlisted checkout"))?;
    authority.binding = attested;
    Ok(authority)
}

async fn resolve_allowed(
    headers: &HeaderMap,
    binding_id: Option<&str>,
) -> Result<RemoteOnboardingAuthority, (StatusCode, Json<Value>)> {
    let principal = request_principal(headers)
        .await
        .filter(|p| p.source == ApiPrincipalSource::AdminToken)
        .ok_or_else(authority_required)?;
    // Controller configuration, never a client binding registration.
    let id = binding_id.ok_or_else(authority_required)?;
    let binding = std::env::var("FOCUSA_REMOTE_ONBOARDING_TARGETS")
        .ok()
        .and_then(|value| select_target(&value, id))
        .ok_or_else(authority_required)?;
    Ok(RemoteOnboardingAuthority {
        binding,
        principal_id: principal.principal.principal_id,
    })
}

fn open_ledger(
    path: &std::path::Path,
    flags: rusqlite::OpenFlags,
) -> Result<rusqlite::Connection, (StatusCode, Json<Value>)> {
    rusqlite::Connection::open_with_flags(path, flags).map_err(|_| {
        reject(
            StatusCode::SERVICE_UNAVAILABLE,
            "remote_bootstrap_ledger_unavailable",
            "controller ledger is unavailable",
        )
    })
}

/// Explicit selection only. Neither an absent ID nor duplicate IDs can
/// fall through to another customer's attestation.
fn select_target(input: &str, binding_id: &str) -> Option<RemoteWorkspaceBinding> {
    if input.len() > 64 * 1024 || binding_id.trim().is_empty() {
        return None;
    }
    let mut candidates = serde_json::from_str::<Vec<RemoteWorkspaceBinding>>(input)
        .ok()?
        .into_iter()
        .filter(|binding| binding.binding_id == binding_id);
    let selected = candidates.next()?;
    candidates.next().is_none().then_some(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_selection_requires_one_exact_id() {
        let sample = |id| {
            json!({
                "schema": "focusa.remote_workspace_binding.v1", "binding_id": id,
                "controller": {"daemon_identity":"controller", "controller_origin":"fixture"},
                "project": {"project_id":"demo", "repo_remote":"https://example.test/demo.git"},
                "transport": {"kind":"ssh", "host":"example.test", "user":"owner", "port":22, "verification_evidence":[]},
                "roots": {"canonical_remote_root":"/work/demo"},
                "session": {"continuity_id":"demo-work", "principal":"principal:admin-token"},
                "state": {"status":"pending"}
            })
        };
        let input = json!([sample("left"), sample("right")]).to_string();
        assert_eq!(select_target(&input, "right").unwrap().binding_id, "right");
        assert!(select_target(&input, "missing").is_none());
        let duplicate = json!([sample("left"), sample("left")]).to_string();
        assert!(select_target(&duplicate, "left").is_none());
    }
}
