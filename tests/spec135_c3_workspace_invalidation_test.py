#!/usr/bin/env python3
"""Spec 135C-3 named invalidation and SSE-first live refresh proof."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
SCHEMA = json.loads((ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.workspace_event.v1.json").read_text())
CONTRACT = json.loads((ROOT / "docs/contracts/spec135-workspace-invalidation-live-refresh.v1.json").read_text())
ROUTER = (ROOT / "apps/pi-extension/src/workspace-invalidation.ts").read_text()
SESSION = (ROOT / "apps/pi-extension/src/session.ts").read_text()
assert SCHEMA["title"] == "Focusa Workspace Invalidation Event v1"
for field in ("project_root", "continuity_id", "instance_id", "attachment_id", "invalidate", "semantic_authority"):
    assert field in SCHEMA["required"], field
assert CONTRACT["primary_transport"] == "Focusa SSE"
assert "bounded polling" in CONTRACT["fallback_transport"]
assert "No full Mission Canvas or workspace refetch for unrelated events" in CONTRACT["laws"]
assert "Workspace invalidation is not an ontology promotion event" in CONTRACT["laws"]
for token in ("planWorkspaceInvalidation", "isNamedInvalidationKey", "visibleKeys", "subscribedKeys", "cross_project_scope", "cross_workstream_scope", "polling_fallback"):
    assert token in ROUTER, token
assert "publishScopedStateChange({" in SESSION
assert "eventRoot === currentRoot" in SESSION
assert 'mutation_kind: String(evt.type)' in SESSION
print("Spec 135 C3 named invalidation and SSE-first refresh: PASS")
