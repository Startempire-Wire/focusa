# Spec 135 Mission Canvas Rich Host Threat Model

## Assets and trust zones

1. **Focusa Core authority** — canonical projection, events, permissions, revisions, Evidence, Receipts.
2. **Pi extension** — exact attachment binding and host lifecycle owner; trusted local process, not canonical workspace storage.
3. **One-time handshake file** — secret-bearing transition boundary; mode `0600`, 60-second expiry, nonce, digest, atomic deletion.
4. **Rich host loopback webview** — presentation process; receives a scoped short-lived bootstrap and holds credentials in memory only.
5. **Generated A2UI/Lit runtime** — untrusted message input constrained to a trusted component catalog and allowlisted Operation Registry actions.
6. **UIAI Work Surface** — bounded evidence projection; UIAI Engine Cockpit retains browser execution ownership.

## Threats and mitigations

| Threat | Mitigation | Proof |
|---|---|---|
| Token leakage through argv, URL, logs, projection, or local storage | Token exists only in private handshake file and host memory; tests reject argv/localStorage usage | `rich-host-entrypoint.integration.mjs` |
| Handshake theft or replay | `0600` mode, random nonce, one-minute expiry, SHA-256 environment binding, atomic consume/delete | `host-entrypoint.mjs` |
| Cross-project or cross-attachment mutation | Exact project/continuity/session/attachment scope on every API call; response scope checked; mismatch fails closed | API and client tests |
| Stale overwrite | Projection/layout expected revisions and idempotency keys required; SQLite transaction rejects stale writers | Core concurrency tests |
| Arbitrary generated action | A2UI action names intersect projected Operation Registry allowlist | A2UI renderer tests |
| Arbitrary component/script injection | Permanent Focusa Custom Elements catalog; no `innerHTML`, `eval`, `document.write`, or arbitrary iframe | generated-surface gate |
| Malicious screenshot/artifact URL | Image origins restricted to blob, image data, or loopback; textual evidence uses `textContent` | frontend gate |
| Webview navigation/exfiltration | Loopback bind, `default-src 'self'`, loopback-only `connect-src`, no referrer, no objects/frames | entrypoint CSP |
| UIAI authority confusion | Surface labels Cockpit ownership and never executes page tools directly | UIAI isolation gate |
| Native binary substitution | Versioned SHA-256 plus Ed25519 manifest verification before native promotion | lifecycle security test |
| Orphan process/resource leak | One host per attachment, heartbeat, session-shutdown close, timer unref, handshake cleanup | lifecycle/stress tests |

## Origin policy

Production content is served only from an ephemeral `127.0.0.1` port owned by the host process. The server allowlists four immutable assets and sends `no-store`, CSP, and `frame-ancestors 'none'`. External navigation is not rendered inside the host. Daemon access is limited to loopback by CSP.

## Credential projection rule

`ResolvedWorkspaceProjection`, events, diagnostics, Evidence, Receipts, screenshots, and generated messages cannot contain bearer tokens, handshake nonces, private keys, OAuth refresh tokens, cookies, or raw environment values. The rich host never persists bootstrap data to local/session storage or IndexedDB.
