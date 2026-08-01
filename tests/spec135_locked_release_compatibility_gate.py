#!/usr/bin/env python3
from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[1]
PACKET = ROOT / "docs/contracts/135-locked-release-compatibility-delta.v1.yaml"


def main() -> None:
    packet = yaml.safe_load(PACKET.read_text())
    assert packet["schema"] == "focusa.spec135_locked_release_compatibility_delta.v1"
    assert packet["status"] == "verified_complete"
    assert packet["release_admitted"] is True
    assert packet["frozen_series_terminal"] == "135K"
    assert packet["spec135l_created"] is False
    assert packet["unknown_impact_count"] == 0
    expected_changes = {
        "temporal-authority-137-137a",
        "epistemic-authority-138-138a",
        "instruction-integrity-140-140a",
        "working-subpath-workloop-recovery",
        "agent-capability-and-doc-parity",
        "startup-project-binding-v1",
        "cross-project-context-isolation",
        "scoped-mission-canvas-refresh",
        "advisory-scope-poisoning-guard",
    }
    assert {change["change_id"] for change in packet["changes"]} == expected_changes
    parity_surfaces = {
        surface for change in packet["changes"] for surface in change["affected_surfaces"]
    }
    assert {"API", "Pi", "Mission Canvas"} <= parity_surfaces
    assert not list((ROOT / "docs").glob("135l-*"))
    required_fields = set(packet["agent_handoff"]["required_change_fields"])
    for change in packet["changes"]:
        assert required_fields <= set(change)

        assert change["spec135_impact"] in {"none", "indirect", "direct"}
        assert change["affected_specs"] and change["affected_primitives"]
        assert change["affected_docs"] and change["affected_contracts"]
        assert change["affected_surfaces"] and change["compatibility"]
        assert change["migration"] and change["rollback"] and change["agent_handoff"]
        assert change["tests"] and change["evidence_refs"]
        for ref in [
            *change["affected_docs"],
            *change["affected_contracts"],
            *change["tests"],
            *change["evidence_refs"],
        ]:
            assert (ROOT / ref).exists(), (change["change_id"], ref)
    manifest = (ROOT / packet["baseline_manifest_ref"]).read_text()
    assert "135-locked-release-compatibility-delta.v1.yaml" in manifest
    assert "no Spec135L exists" in manifest
    assert packet["agent_handoff"]["release_rule"].startswith("any future unknown")
    print("Spec135 locked-release compatibility/delta gate: PASS")


if __name__ == "__main__":
    main()
