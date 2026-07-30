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
    assert len(packet["changes"]) == 5
    for change in packet["changes"]:
        assert change["spec135_impact"] in {"none", "indirect", "direct"}
        assert change["affected_specs"] and change["affected_primitives"]
        assert change["affected_surfaces"] and change["compatibility"]
        assert change["migration"] and change["rollback"]
        assert change["tests"] and change["evidence_refs"]
        for ref in [*change["tests"], *change["evidence_refs"]]:
            assert (ROOT / ref).exists(), (change["change_id"], ref)
    manifest = (ROOT / packet["baseline_manifest_ref"]).read_text()
    assert "135-locked-release-compatibility-delta.v1.yaml" in manifest
    assert "no Spec135L exists" in manifest
    assert packet["agent_handoff"]["release_rule"].startswith("any future unknown")
    print("Spec135 locked-release compatibility/delta gate: PASS")


if __name__ == "__main__":
    main()
