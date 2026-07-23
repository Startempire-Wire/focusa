#!/usr/bin/env python3
"""Spec98/99 Phase B: ProjectRootKey + WorkstreamKey partition contract validation."""

from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/worksheets/focusa-877z.20-project-workstream-keys.yaml"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def require(container, key, label):
    if key not in container or container[key] in (None, "", []):
        fail(f"{label} missing {key}")
    return container[key]


def main() -> None:
    if not CONTRACT.exists():
        fail(f"contract missing: {CONTRACT}")
    data = yaml.safe_load(CONTRACT.read_text())
    if data.get("schema_version") != "focusa.project_workstream_partition_contract.v1":
        fail("unexpected schema_version")
    if data.get("work_item_id") != "focusa-877z.20":
        fail("wrong work_item_id")
    if data.get("status") != "partition_contract_defined":
        fail("contract status is not partition_contract_defined")
    keys = require(data, "keys", "root")
    for key in ["ProjectRootKey", "WorkstreamKey", "SessionKey"]:
        require(keys, key, "keys")
    project = keys["ProjectRootKey"]
    workstream = keys["WorkstreamKey"]
    session = keys["SessionKey"]
    if "reject_broad_roots" not in project.get("normalization", []):
        fail("ProjectRootKey must reject broad roots")
    if "never_replace_with_session_id" not in workstream.get("normalization", []):
        fail("WorkstreamKey must never be replaced with session_id")
    if session.get("authority_role") != "correlation_only":
        fail("SessionKey must be correlation_only")
    partitions = require(data, "state_partitions", "root")
    for name in [
        "project_registry",
        "workstream_state",
        "project_timeline",
        "runtime_session_cache",
    ]:
        require(partitions, name, "state_partitions")
    if partitions["workstream_state"].get("key") != "ProjectRootKey + WorkstreamKey":
        fail("workstream_state must be keyed by ProjectRootKey + WorkstreamKey")
    route = require(data, "route_requirements", "root")
    canonical = require(route, "canonical_write", "route_requirements")
    fields = set(canonical.get("required_fields") or [])
    for field in ["project_root", "continuity_id", "mutation_class"]:
        if field not in fields:
            fail(f"canonical_write missing required field {field}")
    surfaces = require(data, "surface_requirements", "root")
    for surface in ["daemon", "api", "cli", "pi_extension", "menubar", "uiai"]:
        require(surfaces, surface, "surface_requirements")
    proofs = set(require(data, "proof_requirements", "root"))
    for proof in [
        "two-project bleed test",
        "same-root multi-session timeline test",
        "ambiguous cwd degradation test",
        "session_id_not_authority test",
    ]:
        if proof not in proofs:
            fail(f"proof_requirements missing {proof}")
    print("✓ PASS: ProjectRootKey/WorkstreamKey partition contract is valid")


if __name__ == "__main__":
    main()
