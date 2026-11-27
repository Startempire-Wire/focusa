//! Remote Bootstrap entry. The generic binding POST is not an attestation source.

use super::{AppState, HeaderMap, Json, ProjectBootstrapRequest, StatusCode, Value, json, reject};
use crate::{
    middleware::principal::{ApiPrincipalSource, request_principal},
    routes::events_sqlite::focusa_db_path,
};
use focusa_core::remote_workspace::{
    RemoteGenesisIntent, RemoteGenesisPublication, RemoteOnboardingScope, RemoteWorkspaceBinding,
    adopt_remote_bootstrap, attest_allowed_workspace, load_controller_attestation,
    preview_allowed_workspace, read_remote_bootstrap, read_remote_genesis_publication,
};
use focusa_core::types::{
    FocusaState, TrajectoryProjectionRecord, WorkpointRecord, WorkpointStatus,
};
use std::path::Path;

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
    let root = root.to_owned();
    let continuity = continuity.to_owned();
    tokio::task::spawn_blocking(move || {
        let ledger = open_ledger(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let attested = load_controller_attestation(
            &ledger, &authority.scope(&root, &continuity), &authority.binding, chrono::Utc::now(), MAX_PROBE_AGE,
        ).map_err(|error| {
            tracing::warn!(failure = %error, "remote onboarding verification not current");
            reject(StatusCode::PRECONDITION_FAILED, "remote_onboarding_verification_failed", "verify the exact registered binding and refresh its owner-side probe if expired")
        })?.ok_or_else(|| reject(StatusCode::PRECONDITION_FAILED, "remote_onboarding_current_probe_required", "apply remote Bootstrap with the original replay key to verify this allowlisted checkout"))?;
        authority.binding = attested;
        Ok(authority)
    })
    .await
    .map_err(|_| authority_required())?
}

/// Read-only resolution of an enrolled remote scope. A configured remote root
/// never falls back to a controller checkout when its exact binding cannot be
/// verified. The selector is not a grant and never trusts the general registry.
pub(in crate::routes) async fn resolve_read_scope(
    state: &AppState,
    headers: &HeaderMap,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    working_subpath: Option<&str>,
) -> Result<Option<RemoteOnboardingAuthority>, (StatusCode, Json<Value>)> {
    let binding_header = "x-scope-remote-binding-id";
    let binding_id = headers
        .get(binding_header)
        .map(|value| value.to_str().unwrap_or(""));
    if headers.get_all(binding_header).iter().count() > 1 || binding_id == Some("") {
        return Err(read_scope_mismatch());
    }
    let targets = match std::env::var("FOCUSA_REMOTE_ONBOARDING_TARGETS") {
        Ok(input) => configured_targets(&input).ok_or_else(read_scope_mismatch)?,
        Err(std::env::VarError::NotPresent) if binding_id.is_none() => return Ok(None),
        Err(_) => return Err(read_scope_mismatch()),
    };
    let requested_root = project_root.unwrap_or("").trim();
    if binding_id.is_none()
        && !targets.iter().any(|target| {
            target.roots.canonical_remote_root == requested_root.trim_end_matches('/')
        })
    {
        return Ok(None);
    }
    let scoped_value = |key, requested: Option<&str>| -> Result<Option<String>, _> {
        let supplied = headers.get(key).map(|value| value.to_str().unwrap_or(""));
        if headers.get_all(key).iter().count() > 1
            || supplied.is_some_and(|value| value.trim().is_empty())
            || matches!((requested, supplied), (Some(left), Some(right)) if left.trim() != right.trim())
        {
            return Err(read_scope_mismatch());
        }
        Ok(requested.or(supplied).map(str::trim).map(ToOwned::to_owned))
    };
    let root = scoped_value("x-scope-project-root", project_root)?;
    let continuity = scoped_value("x-scope-continuity-id", continuity_id)?;
    let subpath = scoped_value("x-scope-working-subpath-id", working_subpath)?;
    let (Some(root), Some(continuity)) = (root.as_deref(), continuity.as_deref()) else {
        return Err(read_scope_mismatch());
    };
    let id = select_read_target(&targets, root, continuity, binding_id, subpath.as_deref())?;
    resolve_authority(state, headers, Some(id), root, continuity)
        .await
        .map(Some)
}

fn read_scope_mismatch() -> (StatusCode, Json<Value>) {
    reject(
        StatusCode::PRECONDITION_FAILED,
        "remote_read_scope_unverified",
        "verified remote reads require one exact registered root, continuity and worktree",
    )
}

fn select_read_target<'a>(
    targets: &'a [RemoteWorkspaceBinding],
    root: &str,
    continuity: &str,
    binding_id: Option<&str>,
    working_subpath: Option<&str>,
) -> Result<&'a str, (StatusCode, Json<Value>)> {
    let mut exact = targets.iter().filter(|target| {
        target.roots.canonical_remote_root == root
            && !continuity.is_empty()
            && target.session.continuity_id == continuity
            && binding_id.is_none_or(|id| id == target.binding_id)
            && match working_subpath {
                Some(subpath) => {
                    target.roots.working_subpath.as_deref().unwrap_or("primary") == subpath
                }
                None => {
                    binding_id.is_some()
                        || (target.roots.working_subpath.is_none()
                            && target.roots.worktree_identity.is_none())
                }
            }
    });
    let selected = exact.next().ok_or_else(read_scope_mismatch)?;
    if exact.next().is_some() {
        return Err(read_scope_mismatch());
    }
    Ok(selected.binding_id.as_str())
}

pub(in crate::routes) type RemotePublication = (RemoteGenesisIntent, RemoteGenesisPublication);

impl RemoteOnboardingAuthority {
    /// Read the existing binding-qualified Genesis journal, never a same-path
    /// local record. Presence of an intent alone does not prove publication.
    pub(in crate::routes) async fn read_publication(
        &self,
        state: &AppState,
    ) -> Result<Option<RemotePublication>, (StatusCode, Json<Value>)> {
        let path = focusa_db_path(&state.config.data_dir);
        let binding = self.binding.clone();
        let principal_id = self.principal_id.clone();
        tokio::task::spawn_blocking(move || {
            let authority = RemoteOnboardingAuthority {
                binding,
                principal_id,
            };
            let binding = &authority.binding;
            let scope = authority.scope(
                &binding.roots.canonical_remote_root,
                &binding.session.continuity_id,
            );
            let ledger = open_ledger(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            read_remote_genesis_publication(
                &ledger,
                &scope,
                binding,
                chrono::Utc::now(),
                MAX_PROBE_AGE,
                None,
            )
            .map(|observed| {
                observed.and_then(|(intent, publication)| {
                    publication.map(|publication| (intent, publication))
                })
            })
            .map_err(|_| read_scope_mismatch())
        })
        .await
        .map_err(|_| read_scope_mismatch())?
    }

    pub(in crate::routes) fn bound_trajectory<'a>(
        &self,
        state: &'a FocusaState,
        intent: &RemoteGenesisIntent,
        publication: &RemoteGenesisPublication,
    ) -> Option<&'a TrajectoryProjectionRecord> {
        let binding = &self.binding;
        let trajectory =
            publication.verified_trajectory(state, intent, &binding.session.continuity_id)?;
        let scope_ref = trajectory.scope_ref.as_ref()?;
        (scope_ref.scope_id == format!("remote-workspace:{}", binding.binding_id)
            && scope_ref.root_path == Path::new(&binding.roots.canonical_remote_root)
            && scope_ref.fingerprint == intent.projection_key())
        .then_some(trajectory)
    }

    pub(in crate::routes) fn initial_records<'a>(
        &self,
        state: &'a FocusaState,
        intent: &RemoteGenesisIntent,
        publication: &RemoteGenesisPublication,
    ) -> Option<(&'a TrajectoryProjectionRecord, &'a WorkpointRecord)> {
        let trajectory = self.bound_trajectory(state, intent, publication)?;
        let first = intent.scoped_workpoint(
            state,
            &self.binding.session.continuity_id,
            publication.workpoint_id,
        )?;
        matches!(
            first.status,
            WorkpointStatus::Active | WorkpointStatus::Superseded
        )
        .then_some((trajectory, first))
    }

    /// Live reads follow the authoritative pointer, never the first published
    /// task or the daemon-wide active task. The caller still proves attestation.
    pub(in crate::routes) fn active_records<'a>(
        &self,
        state: &'a FocusaState,
        intent: &RemoteGenesisIntent,
        publication: &RemoteGenesisPublication,
    ) -> Option<(&'a TrajectoryProjectionRecord, &'a WorkpointRecord)> {
        let trajectory = self.bound_trajectory(state, intent, publication)?;
        let active = intent.scoped_workpoint(
            state,
            &self.binding.session.continuity_id,
            trajectory.active_workpoint_id?,
        )?;
        let subpath = active
            .session_identity
            .as_ref()
            .and_then(|identity| identity.working_subpath_id.as_deref());
        (active.status == WorkpointStatus::Active
            && subpath.is_none_or(|id| {
                id == self
                    .binding
                    .roots
                    .working_subpath
                    .as_deref()
                    .unwrap_or("primary")
            }))
        .then_some((trajectory, active))
    }
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
fn configured_targets(input: &str) -> Option<Vec<RemoteWorkspaceBinding>> {
    if input.len() > 64 * 1024 {
        return None;
    }
    serde_json::from_str(input).ok()
}

fn select_target(input: &str, binding_id: &str) -> Option<RemoteWorkspaceBinding> {
    if binding_id.trim().is_empty() {
        return None;
    }
    let mut candidates = configured_targets(input)?
        .into_iter()
        .filter(|binding| binding.binding_id == binding_id);
    let selected = candidates.next()?;
    candidates.next().is_none().then_some(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(id: &str) -> Value {
        json!({
                "schema": "focusa.remote_workspace_binding.v1", "binding_id": id,
                "controller": {"daemon_identity":"controller", "controller_origin":"fixture"},
                "project": {"project_id":"demo", "repo_remote":"https://example.test/demo.git"},
                "transport": {"kind":"ssh", "host":"example.test", "user":"owner", "port":22, "verification_evidence":[]},
                "roots": {"canonical_remote_root":"/work/demo"},
                "session": {"continuity_id":"demo-work", "principal":"principal:admin-token"},
                "state": {"status":"pending"}
        })
    }

    #[test]
    fn target_selection_requires_one_exact_id() {
        let input = json!([target("left"), target("right")]).to_string();
        assert_eq!(select_target(&input, "right").unwrap().binding_id, "right");
        assert!(select_target(&input, "missing").is_none());
        let duplicate = json!([target("left"), target("left")]).to_string();
        assert!(select_target(&duplicate, "left").is_none());
    }

    #[test]
    fn controller_attested_identity_does_not_depend_on_a_local_checkout() {
        let root = tempfile::tempdir().unwrap();
        let absent = root.path().join("remote-only");
        let mut binding: RemoteWorkspaceBinding =
            serde_json::from_value(target("enrolled")).unwrap();
        binding.roots.canonical_remote_root = absent.to_str().unwrap().into();
        binding.state.status = focusa_core::remote_workspace::BindingStatus::Verified;
        binding.transport.verified_at = Some(chrono::Utc::now().to_rfc3339());
        let authority = RemoteOnboardingAuthority {
            binding,
            principal_id: "principal:admin-token".into(),
        };
        assert!(!absent.exists());
        let packet = crate::routes::project::attested_identity_payload(&authority, None);
        assert_eq!(packet["canonical"], true);
        assert_eq!(
            packet["project_identity"]["scope_ref"]["scope_id"],
            "remote-workspace:enrolled"
        );
        assert_eq!(
            packet["project_identity"]["project_root"],
            absent.to_str().unwrap()
        );
        assert_eq!(packet["controller_binding"]["continuity_id"], "demo-work");
        assert_eq!(packet["verification"]["verified"], true);
    }

    #[test]
    fn remote_successor_is_selected_without_changing_initial_readiness_identity() {
        use focusa_core::scoped_state::ScopeRef;
        use focusa_core::types::{FocusaSessionIdentity, HltLedgerEntry, TrajectoryLadderEvent};
        use uuid::Uuid;

        let binding: RemoteWorkspaceBinding = serde_json::from_value(target("exact")).unwrap();
        let authority = RemoteOnboardingAuthority {
            binding,
            principal_id: "fixture".into(),
        };
        let continuity = authority.binding.session.continuity_id.as_str();
        let intent = RemoteGenesisIntent {
            intent_id: Uuid::now_v7(),
            bootstrap_receipt_id: Uuid::now_v7(),
            plan_digest: "fixture".into(),
        };
        let key = intent.projection_key();
        let initial = Uuid::now_v7();
        let current = Uuid::now_v7();
        let goal = "Verify remote task continuity";
        let trajectory = TrajectoryProjectionRecord {
            trajectory_id: "fixture-remote-trajectory".into(),
            canonical: true,
            project_root: Some(key.clone()),
            continuity_id: Some(continuity.into()),
            scope_ref: Some(
                ScopeRef::project(
                    "remote-workspace:exact",
                    &authority.binding.roots.canonical_remote_root,
                    &authority.binding.project.project_id,
                    &key,
                )
                .unwrap(),
            ),
            long_term_goal: goal.into(),
            active_workpoint_id: Some(initial),
            ..Default::default()
        };
        let hlt = HltLedgerEntry::new(key.clone(), goal.into(), "fixture", 1)
            .with_scope(Some(continuity.into()), None);
        let publication = RemoteGenesisPublication {
            trajectory_id: trajectory.trajectory_id.clone(),
            workpoint_id: initial,
            ladder: crate::routes::trajectory::trajectory_commit_events(
                &trajectory,
                None,
                &hlt,
                &[],
                Some("fixture"),
            ),
            hlt,
        };
        let record = |id, status| WorkpointRecord {
            workpoint_id: id,
            status,
            canonical: true,
            project_root: Some(key.clone()),
            continuity_id: Some(continuity.into()),
            work_item_id: Some("fixture-task".into()),
            next_slice: Some("verify the selected active task".into()),
            ..Default::default()
        };
        let mut state = FocusaState::default();
        state.trajectory.records.push(trajectory);
        state
            .workpoint
            .records
            .push(record(initial, WorkpointStatus::Active));
        assert!(publication.verifies(&state, &intent, continuity));
        state.trajectory.records[0].active_workpoint_id = Some(current);
        state.workpoint.records[0].status = WorkpointStatus::Superseded;
        state
            .workpoint
            .records
            .push(record(current, WorkpointStatus::Active));
        // Genesis publication still proves the original transaction; only the
        // active view follows the successor, regardless of global activity.
        state.workpoint.active_workpoint_id = Some(initial);
        assert!(!publication.verifies(&state, &intent, continuity));
        assert_eq!(
            authority
                .initial_records(&state, &intent, &publication)
                .unwrap()
                .1
                .workpoint_id,
            initial
        );
        assert_eq!(
            authority
                .active_records(&state, &intent, &publication)
                .unwrap()
                .1
                .workpoint_id,
            current
        );
        let duplicate = state.workpoint.records[1].clone();
        state.workpoint.records.push(duplicate);
        assert!(
            authority
                .active_records(&state, &intent, &publication)
                .is_none()
        );
        state.workpoint.records.pop();
        state.workpoint.records[1].session_identity = Some(FocusaSessionIdentity {
            working_subpath_id: Some("sibling".into()),
            ..Default::default()
        });
        assert!(
            authority
                .active_records(&state, &intent, &publication)
                .is_none()
        );
        state.workpoint.records[1].session_identity = None;
        state.workpoint.records[1].project_root =
            Some(authority.binding.roots.canonical_remote_root.clone());
        assert!(
            authority
                .active_records(&state, &intent, &publication)
                .is_none()
        );
    }

    #[test]
    fn remote_read_targets_do_not_fall_back_to_sibling_or_ambiguous_bindings() {
        let main: RemoteWorkspaceBinding = serde_json::from_value(target("left")).unwrap();
        let mut sibling: RemoteWorkspaceBinding = serde_json::from_value(target("right")).unwrap();
        sibling.roots.working_subpath = Some("linked".into());
        sibling.roots.worktree_identity = Some("linked-checkout".into());
        let root = main.roots.canonical_remote_root.clone();
        let continuity = main.session.continuity_id.clone();
        let targets = [main.clone(), sibling];
        let select = |id, subpath| select_read_target(&targets, &root, &continuity, id, subpath);
        assert_eq!(select(None, None).unwrap(), "left");
        assert_eq!(select(None, Some("linked")).unwrap(), "right");
        assert_eq!(select(Some("right"), None).unwrap(), "right");
        assert!(select(Some("left"), Some("linked")).is_err());
        assert!(select(None, Some("unknown-subpath")).is_err());
        assert!(select_read_target(&targets, &root, "other-continuity", None, None).is_err());
        let duplicate = [main.clone(), main];
        assert!(select_read_target(&duplicate, &root, &continuity, None, None).is_err());
    }
}
