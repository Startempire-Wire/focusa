#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACT="$ROOT/docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json"
TS_OUT="$ROOT/packages/generated/spec135/typescript/schema.d.ts"
GO_OUT="$ROOT/packages/generated/spec135/go/client.gen.go"

python3 - "$CONTRACT" <<'PY'
import json, sys
contract = json.load(open(sys.argv[1]))
assert contract["openapi"] == "3.0.3", contract.get("openapi")
assert contract["paths"], "OpenAPI paths must not be empty"
assert contract["components"]["schemas"], "OpenAPI schemas must not be empty"
PY

npx --yes openapi-typescript@7.10.1 "$CONTRACT" -o "$TS_OUT"
go run github.com/oapi-codegen/oapi-codegen/v2/cmd/oapi-codegen@v2.7.0 \
  -generate types,client -package spec135client "$CONTRACT" > "$GO_OUT"
gofmt -w "$GO_OUT"

echo "Generated TypeScript and Go clients from $CONTRACT"
