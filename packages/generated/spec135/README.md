# Spec 135 generated clients

The TypeScript client is generated from
`docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json`.
It is a transport binding, not independent DTO authority. External language adapters consume the same portable OpenAPI and JSON Schema contracts outside Focusa core.

Regenerate from the repository root:

```bash
scripts/generate-spec135-clients.sh
```

Pinned generators:

- `openapi-typescript@7.10.1`

Validate with:

```bash
(cd packages/generated/spec135/typescript && npm ci && npm run check)
python3 tests/spec135_generated_clients_test.py
```
