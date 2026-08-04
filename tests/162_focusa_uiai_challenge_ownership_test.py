#!/usr/bin/env python3
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
contract = json.loads((root / "docs/contracts/114-focusa-uiai-challenge-capability.v1.json").read_text())
assert contract["schema"] == "focusa.uiai_challenge_capability_contract.v1"
assert contract["authority_owner"] == "uiai-engine"
assert contract["focusa_role"] == "governance_discovery_evidence_and_recovery_only"
assert contract["capability_status"] == "unsupported"
assert contract["supported_challenge_types"] == []
assert contract["supported_operations"] == []
assert contract["mutation_authority"] is False
assert "invent_solver_capability" in contract["forbidden_in_focusa"]
assert contract["recovery"]["failure_class"] == "uiai_capability_unavailable"
print("GH#114 UIAI ownership: PASS (external owner explicit; no solver capability inferred)")
