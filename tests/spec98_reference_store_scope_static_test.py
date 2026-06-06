#!/usr/bin/env python3
"""Spec98 focusa-877z.6: Reference Store scoped exact-handle guard."""
from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/worksheets/focusa-877z.6-reference-store-scope-contract.yaml"
TYPES = ROOT / "crates/focusa-core/src/types.rs"
STORE = ROOT / "crates/focusa-core/src/reference/store.rs"
DAEMON = ROOT / "crates/focusa-core/src/runtime/daemon.rs"
ECS = ROOT / "crates/focusa-api/src/routes/ecs.rs"
VISUAL = ROOT / "crates/focusa-api/src/routes/visual_workflow.rs"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    if data.get("schema_version") != "focusa.reference_store_scope_contract.v1":
        fail("unexpected .6 contract schema")
    if data.get("status") != "reference_handles_scope_bound_and_exact_handle_returned":
        fail("unexpected .6 contract status")

    types = TYPES.read_text()
    store = STORE.read_text()
    daemon = DAEMON.read_text()
    ecs = ECS.read_text()
    visual = VISUAL.read_text()

    handle_struct = types[types.find("pub struct HandleRef"):types.find("pub enum HandleKind")]
    for field in ["pub session_id: Option<SessionId>", "pub project_root: Option<String>", "pub continuity_id: Option<String>", "pub trajectory: Option<TrajectoryLadderContext>"]:
        if field not in handle_struct:
            fail(f"HandleRef missing scope field {field}")

    action = types[types.find("StoreArtifact {"):types.find("ResolveHandle", types.find("StoreArtifact {"))]
    for field in ["handle_id: Option<HandleId>", "project_root: Option<String>", "continuity_id: Option<String>"]:
        if field not in action:
            fail(f"Action::StoreArtifact missing {field}")

    signature = "handle_id: Option<HandleId>,\n        project_root: Option<String>,\n        continuity_id: Option<String>"
    if signature not in store:
        fail("ReferenceStore::store must accept handle_id/project_root/continuity_id")
    if "let id = handle_id.unwrap_or_else(Uuid::now_v7);" not in store:
        fail("ReferenceStore::store must use caller-supplied handle_id when provided")
    for field in ["project_root,", "continuity_id,"]:
        if field not in store:
            fail(f"ReferenceStore::store must persist {field.strip(',')}")

    if "project_root.or_else(|| session.and_then(|s| s.project_root.clone()))" not in daemon:
        fail("daemon must fill handle project_root from action or active session")
    if "continuity_id.or_else(|| session.and_then(|s| s.continuity_id.clone()))" not in daemon:
        fail("daemon must fill handle continuity_id from action or active session")

    for name, text in [("ecs.rs", ecs), ("visual_workflow.rs", visual)]:
        if "let handle_id = uuid::Uuid::now_v7();" not in text:
            fail(f"{name} must pre-generate exact handle_id")
        if "handle_id: Some(handle_id)" not in text:
            fail(f"{name} must dispatch exact handle_id")
        if ".find(|h| h.id == handle_id)" not in text:
            fail(f"{name} must poll by exact handle id")
        if ".find(|h| h.label ==" in text:
            fail(f"{name} must not poll newly-created handles by label")
    if '"handle": handle' not in ecs:
        fail("ECS store response must return exact handle object")

    print("✓ PASS: Reference Store writes are scope-bound and store routes return exact created handles")


if __name__ == "__main__":
    main()
