# Spec 135 generated clients

These TypeScript and Go clients are generated from
`docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json`.
They are transport bindings, not independent DTO authority.

Regenerate from the repository root:

```bash
scripts/generate-spec135-clients.sh
```

Pinned generators:

- `openapi-typescript@7.10.1`
- `oapi-codegen@v2.7.0`

Validate with:

```bash
(cd packages/generated/spec135/typescript && npm ci && npm run check)
(cd packages/generated/spec135/go && go test ./...)
python3 tests/spec135_generated_clients_test.py
```
