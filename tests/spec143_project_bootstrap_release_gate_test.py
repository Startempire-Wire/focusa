#!/usr/bin/env python3
from pathlib import Path
import json

ROOT=Path(__file__).resolve().parents[1]
route=(ROOT/'crates/focusa-api/src/routes/project_bootstrap.rs').read_text()
support=(ROOT/'crates/focusa-api/src/routes/project_bootstrap_support.rs').read_text()
safety=(ROOT/'crates/focusa-api/src/routes/project_bootstrap_safety.rs').read_text()
provider=(ROOT/'crates/focusa-api/src/routes/project_bootstrap_provider.rs').read_text()
filesystem=(ROOT/'crates/focusa-api/src/routes/project_bootstrap_fs.rs').read_text()
journal=(ROOT/'crates/focusa-api/src/routes/project_bootstrap_journal.rs').read_text()
pre_root=(ROOT/'crates/focusa-api/src/routes/project_bootstrap_pre_root.rs').read_text()
status_route=(ROOT/'crates/focusa-api/src/routes/project_bootstrap_status.rs').read_text()
implementation=route+support+safety+provider+filesystem+journal+status_route
cli=(ROOT/'crates/focusa-cli/src/commands/project.rs').read_text()
e2e=(ROOT/'crates/focusa-cli/tests/project_genesis_e2e.rs').read_text()
tools=(ROOT/'apps/pi-extension/src/tools.ts').read_text()
contracts=(ROOT/'apps/pi-extension/src/tool-contracts.ts').read_text()
api=(ROOT/'docs/current/API_REFERENCE_CURRENT.md').read_text()
cli_docs=(ROOT/'docs/current/CLI_REFERENCE_CURRENT.md').read_text()
registry=json.loads((ROOT/'docs/contracts/spec135/generated-contract-v1/operation-registry.json').read_text())

for action in ('preview','apply','status','repair'):
    endpoint=f'/v1/project/bootstrap/{action}'
    assert endpoint in route and endpoint in api, action
    assert any(op['operation_id']==f'focusa.project.bootstrap.{action}' for op in registry['operations'])
assert 'name: "focusa_project_bootstrap"' in tools
assert 'name: "focusa_project_bootstrap"' in contracts
for command in ('bootstrap preview','bootstrap apply','bootstrap status','bootstrap repair'):
    assert command in cli_docs, command
for required in ('planned_changes','preserved_choices','rollback','verification','created_by_this_transaction','idempotency_key','marker_ref','identity_confidence','cross_project_marker_conflict','malformed_project_marker'):
    assert required in implementation, required
assert '"git", &["init"]' in implementation
assert 'run(&root, "git", &["remote"])' in route
assert 'provider_output_limit_exceeded' in provider and 'provider_timeout' in provider
assert 'require_owner_context(&root)' in route and 'owner_runner_required' in filesystem
assert 'artifact_write_rejection("marker_create", error)' in route
assert 'check_write_access(&root)' in route and 'fn check_write_access(' in filesystem
assert '"applying"' in safety and 'receipt["status"] == "applying"' in safety
assert route.index('pre_root::reserve(&root, &req.idempotency_key, &request_digest)?') < route.index('fs::create_dir_all(&root)')
assert 'create_json_atomic(&path(root)?, &record)' in pre_root and 'bootstrap_pre_root_interrupted' in pre_root
assert 'rollback_without_root' in route and 'root_creation_uncertain' in status_route
assert 'options.mode(0o600)' in filesystem
for stage in ('marker_create','settings_create','docs_create','git_init','task_provider','genesis'):
    assert f'&created, "{stage}"' in route
assert 'write_json_atomic(&receipt_path(root), &progress)' in journal
assert '"unsupported_repair_action"' in route and 'Some("retry_apply")' in route
assert '"unverified_existing_artifacts"' in status_route and '"bootstrap_status_artifact_unreadable"' in status_route
assert '"unverified_existing_artifacts"' in safety
for code in ('bootstrap_artifact_already_exists','bootstrap_permission_denied','bootstrap_quota_exceeded','bootstrap_read_only_filesystem'):
    assert code in filesystem
assert '"bd", "br"' in implementation
assert '"init"' in implementation and '"--prefix"' in implementation
assert '"dep"' in implementation and '"add"' in implementation
assert 'project_genesis::start' in implementation and 'project_genesis::commit' in implementation
assert 'implicit_remote_forbidden' in implementation
assert route.count('validate_marker(&root, &req.project_id, &req.canonical_name)?;') == 3
assert 'safety::validate_project_marker(root, project_id, canonical_name)' in route
assert 'focusa_core::project_marker::ProjectMarker' in route
assert 'focusa_core::project_marker::read_marker(root)' in safety
assert 'programming language' in implementation and 'deployment target' in implementation
assert 'github.com' not in implementation.lower()
# Bootstrap delegates both supplied and canonicalized paths to the shared
# classifier; duplicating its path list here would reward divergent policy.
classifier=(ROOT/'crates/focusa-core/src/scope_safety.rs').read_text()
assert 'focusa_core::scope_safety::classify_project_root(raw).is_safe()' in support
assert 'focusa_core::scope_safety::classify_project_root(&rendered)' in support
assert 'unsafe_project_root' in support
for unsafe_root in ('"/root"','"/home"','"/tmp"'):
    assert unsafe_root in classifier, unsafe_root
assert 'fn rejects_broad_roots()' in classifier
assert 'fn rejects_user_homes()' in classifier
assert 'standard_bootstrap_is_previewable_local_only_idempotent_and_rollback_bounded' in e2e
assert 'bootstrap must never create a remote' in e2e
assert 'tasks_after_replay' in e2e
assert 'rolled_back' in e2e
assert len(route.splitlines()) < 500
assert len(support.splitlines()) < 500
assert len(filesystem.splitlines()) < 240
print('Spec143 project bootstrap release gate: PASS')
