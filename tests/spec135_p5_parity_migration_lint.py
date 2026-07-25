#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
matrix = json.loads((R / "docs/contracts/spec135-client-parity-matrix.v1.yaml").read_text())
registry = json.loads(
    (R / "docs/contracts/spec135/generated-contract-v1/operation-registry.json").read_text()
)
ts = (R / "packages/generated/spec135/typescript/schema.d.ts").read_text()
clients = {row["client_id"] for row in matrix["clients"]}
assert {"api", "cli", "pi", "typescript", "mission_canvas", "uiai_engine_cockpit"}.issubset(clients)
assert "go" not in clients
assert all(row["contract_source"] == "generated Operation Registry/OpenAPI" for row in matrix["clients"])
operation_ids = {row["operation_id"] for row in registry["operations"]}
for operation_id in operation_ids:
    assert f'operations["{operation_id}"]' in ts, operation_id
assert matrix["canonical_contracts"]
assert len(matrix["requirements"]) >= 70
print("Spec 135 P5 provider/connector/client/stream parity migration lint: PASS")
