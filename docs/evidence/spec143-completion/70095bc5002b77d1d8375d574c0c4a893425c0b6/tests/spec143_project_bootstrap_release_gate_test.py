#!/usr/bin/env python3
from pathlib import Path
import json

ROOT=Path(__file__).resolve().parents[1]
route=(ROOT/'crates/focusa-api/src/routes/project_bootstrap.rs').read_text()
support=(ROOT/'crates/focusa-api/src/routes/project_bootstrap_support.rs').read_text()
implementation=route+support
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
assert 'Command::new("git")' in implementation and '.args(["remote"])' in implementation
assert '"bd", "br"' in implementation
assert '"init"' in implementation and '"--prefix"' in implementation
assert '"dep"' in implementation and '"add"' in implementation
assert 'project_genesis::start' in implementation and 'project_genesis::commit' in implementation
assert 'implicit_remote_forbidden' in implementation
assert 'programming language' in implementation and 'deployment target' in implementation
assert 'github.com' not in implementation.lower()
for unsafe_root in ('Path::new("/root")','Path::new("/home")','Path::new("/tmp")'):
    assert unsafe_root in support, unsafe_root
assert 'standard_bootstrap_is_previewable_local_only_idempotent_and_rollback_bounded' in e2e
assert 'bootstrap must never create a remote' in e2e
assert 'tasks_after_replay' in e2e
assert 'rolled_back' in e2e
assert len(route.splitlines()) < 500
assert len(support.splitlines()) < 500
print('Spec143 project bootstrap release gate: PASS')
