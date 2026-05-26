# Focusa Security Standard Matrix Review — 2026-05-26

Scope: Focusa Rust core/API/CLI, Pi extension, menubar app, local daemon deployment, persistence, and docs. This review maps current posture against five industry-standard security matrices:

1. OWASP Application Security Verification Standard (ASVS-style controls)
2. OWASP API Security Top 10 (2023 categories)
3. MITRE CWE Top 25 weakness classes
4. STRIDE threat model
5. CIS Controls v8 implementation alignment

Evidence baseline:

- Prior review: `docs/current/FOCUSA_SECURITY_REVIEW_2026-05-26.md`.
- Dependency remediation: npm Pi extension 0 vulns; menubar 3 low residual; RustSec vulnerabilities 0.
- Runtime hardening: non-loopback bind requires auth; daemon systemd uses loopback bind, `ProtectSystem=strict`, `NoNewPrivileges=true`, `ReadWritePaths=/home/wirebot/focusa/data`.
- Static gates: `tests/security_cargo_audit_gate.sh`, `tests/security_non_loopback_auth_guard_static_test.sh`, `tests/security_shell_unwrap_static_test.sh`, `tests/security_persisted_state_privacy_static_test.sh`.

## 1. OWASP ASVS-style matrix

| Area | Current status | Evidence | Gap / required next action |
| --- | --- | --- | --- |
| V1 Architecture, design, threat modeling | Partial | Architecture docs, Workpoint/Trajectory authority boundaries, this matrix review | Add explicit threat model diagrams and data-flow trust boundaries for daemon/API/Pi/menubar/sync. |
| V2 Authentication | Partial | `FOCUSA_AUTH_TOKEN` middleware; non-loopback startup guard | Add token storage/rotation docs; config `auth_token` is defined but middleware currently only uses env. |
| V3 Session management | Mostly N/A / partial | Focusa is local daemon; session/continuity IDs exist | Clarify that continuity IDs are not auth/session secrets. Add session fixation/replay notes for sync/remote use. |
| V4 Access control | Partial | Work Loop writer ownership; Workpoint project_root+continuity guard; token routes exist | Add route-level scope/permission matrix for read vs write vs service-control routes. |
| V5 Validation, sanitization, encoding | Partial | TypeBox validation in Pi tools; route validation varies; no runtime unwrap gate | Add Rust route schema/size limits for all mutation endpoints. |
| V6 Stored cryptography/secrets | Partial | Secret scan; P4 secret policy doc | Peer `auth_token` persisted in SQLite needs encryption/redaction/visibility policy. |
| V7 Error handling/logging | Good/partial | Tool result envelopes, failure classes, correlation IDs, error envelope docs | Verify no secret echo in error bodies/logs; add automated redaction regression. |
| V8 Data protection/privacy | Partial → improved | `PERSISTED_STATE_PRIVACY_CLASSES.md` and privacy gate | Add retention/backup/delete policy for SQLite/event stores. |
| V9 Communications | Partial | Loopback default; rustls dependencies patched | Remote/Tailscale deployment must require auth and TLS/reverse-proxy policy. |
| V10 Malicious code/supply chain | Improved | npm/RustSec gates, lockfile updates | Add CI enforcement and dependency review cadence. |
| V11 Business logic | Partial | Workpoint/Trajectory authority boundaries; no-deadend tool envelopes | Add abuse cases for continuous work loop, silent sessions, and prediction/metacog poisoning. |
| V12 Files/resources | Partial | systemd `ReadWritePaths`, cleanup preserve-path guards, command boundary doc | Add static path traversal tests for file/path accepting routes and CLI cleanup. |
| V13 API/Web services | Partial | API routes local-first; auth guard; error envelopes | Add OpenAPI/schema inventory and request-size/rate limits. |
| V14 Configuration | Improved | non-loopback auth guard; daemon hardening docs | Add startup self-check route/report for insecure config and config-file auth support. |

ASVS summary: **medium local-risk, high remote-exposure risk if misconfigured.** Recent fixes reduced accidental remote unauthenticated bind risk and supply-chain risk. Remaining high-value work: route permissions/schemas, persisted-token handling, and data retention.

## 2. OWASP API Security Top 10 (2023) matrix

| API risk | Status | Focusa-specific assessment | Next action |
| --- | --- | --- | --- |
| API1 Broken Object Property Level Authorization | Watch | Many routes expose rich state; auth is all-or-none when token enabled. | Add read/write/sensitive field permission matrix and response redaction tests. |
| API2 Broken Authentication | Improved | Non-loopback now requires auth; loopback unauth is intentional local-first. | Implement config-file `auth_token` middleware path or remove unsupported config claim. |
| API3 Broken Object Property Level Authorization | Watch | Workpoint/project scopes exist; generic event/telemetry routes may expose broad payloads. | Add field-level privacy class filters for P3/P4 fields. |
| API4 Unrestricted Resource Consumption | Watch | Hot-path budget work exists; daemon memory previously high; no universal request body limits. | Add request-size limits, per-route timeout/rate caps, and stress gates for mutation routes. |
| API5 Broken Function Level Authorization | Watch | Token auth is not scoped by route/function; service-control actions rely on operator/tool policy. | Add route family scopes: read, write, admin/service-control. |
| API6 Unrestricted Access to Sensitive Business Flows | Watch | Work-loop/silent sessions/prediction capture can influence agent behavior. | Add abuse-case tests and approval gates for background execution controls. |
| API7 Server Side Request Forgery | Low/watch | API mostly local; proxy route and external adapters call providers/tools. | Review proxy/upstream URL construction and deny arbitrary internal fetches. |
| API8 Security Misconfiguration | Improved | Non-loopback guard, systemd hardening, docs. | Add `focusa doctor security` config posture report. |
| API9 Improper Inventory Management | Partial | API reference generated, tool contracts exist. | Generate machine-readable OpenAPI-ish route inventory with auth/scope/privacy class. |
| API10 Unsafe Consumption of APIs | Watch | LLM/provider/proxy/wb integrations consume external/local service responses. | Add provider response size/timeouts/redaction and SSRF-style URL tests where applicable. |

API Top 10 summary: **primary residual API risk is coarse authorization and resource bounds**, not known public exposure.

## 3. MITRE CWE Top 25 matrix

| CWE class | Applicability | Current mitigations | Gaps |
| --- | --- | --- | --- |
| CWE-787 Out-of-bounds Write | Low | Rust memory safety; RustSec 0 vulns | TUI `lru` unsound warning remains via dependency. |
| CWE-79 XSS | Medium for menubar/docs UI | Tauri/local app; npm high/moderate fixed | Menubar still has low SvelteKit/cookie advisories; UI escaping tests should be added. |
| CWE-89 SQL Injection | Low/medium | `rusqlite` parameterized queries used in many paths | Add static query construction review for all SQL strings. |
| CWE-416 Use After Free | Low | Rust safe code baseline | Dependency unsound warnings need triage. |
| CWE-78 OS Command Injection | Medium/high | Shell hotspots documented/static allowlist | Convert `bash -c`/`bash -lc` hotspots to argv or curated command enums. |
| CWE-20 Improper Input Validation | Medium | Some schema validation; non-loopback guard | Add universal request body limits and route schema tests. |
| CWE-22 Path Traversal | Medium | Project scope guards; systemd write path limits | Add path canonicalization tests for cleanup, ECS, project refs, attachments. |
| CWE-352 CSRF | Low currently | Local API, no browser auth cookies | If menubar/browser writes are added, require CSRF posture. |
| CWE-434 Unrestricted File Upload | Low/watch | Attachment routes exist | Review attachment attach/detach size/type/path constraints. |
| CWE-862 Missing Authorization | Medium/high if exposed | Auth middleware + bind guard | Need route scopes and admin/read/write separation. |
| CWE-798 Hard-coded Credentials | Low | Secret scan no confirmed raw secrets | Add committed secret scanning gate with allowlist. |
| CWE-400 Uncontrolled Resource Consumption | Medium | LowMem/resource modes, hot-path warnings | Add route-level body/time/rate caps. |
| CWE-918 SSRF | Watch | Proxy/provider surfaces exist | Review proxy/upstream construction and block arbitrary internal URLs. |
| CWE-502 Deserialization of Untrusted Data | Watch | JSON parsing; SvelteKit devalue findings reduced | Add JSON depth/size limits for public-ish endpoints. |
| CWE-94 Code Injection | Improved | protobufjs critical fixed in npm lockfile | Keep npm audit gate in CI. |

CWE summary: highest relevant classes are **OS command injection, missing authorization, path traversal, input validation, and resource consumption**.

## 4. STRIDE threat model matrix

| STRIDE category | Focusa threat | Existing mitigation | Residual work |
| --- | --- | --- | --- |
| Spoofing | Unauthorized local/remote client impersonates Focusa tool/agent | Bearer auth support; non-loopback requires auth | Route-scoped tokens and audit identity per writer/tool. |
| Tampering | Malicious client mutates Focus State, Workpoints, predictions, metacog | Workpoint project_root+continuity guards; writer ownership | Route scopes, event integrity checks, append-only audit verification. |
| Repudiation | Agent/operator cannot prove who changed state | Event logs, correlation IDs, Workpoint/evidence refs | Sign/hash event chains or add tamper-evident audit log. |
| Information disclosure | API leaks project state, secrets, evidence payloads, peer tokens | Local bind, auth guard, privacy classes, no raw secret findings | Field-level privacy redaction and P4 persistence prevention tests. |
| Denial of service | Large requests, hot paths, daemon memory/resource exhaustion | LowMem/resource telemetry, systemd memory caps, request timeouts in tools | Request-size/rate limits and more stress gates. |
| Elevation of privilege | Shell/service restart/cleanup commands exceed intended authority | Command boundary doc, static shell allowlist, operator approval policy | Replace shell hotspots with argv/approved command enums; add admin scope. |

STRIDE summary: strongest mitigations are locality and state-continuity gates; weakest areas are route-scoped auth, command boundary hardening, and tamper-evident audit.

## 5. CIS Controls v8 alignment matrix

| CIS Control | Status | Focusa/local VPS relevance | Next action |
| --- | --- | --- | --- |
| 1 Inventory and Control of Enterprise Assets | Partial | Service unit, daemon paths, docs inventory exist | Add Focusa deployment asset manifest. |
| 2 Inventory and Control of Software Assets | Improved | npm/Rust lockfiles and audit gates | Add CI dependency inventory artifact. |
| 3 Data Protection | Partial | Privacy classes added | Add retention, backup, deletion, encryption-at-rest decision. |
| 4 Secure Configuration of Assets and Software | Improved | systemd hardening; non-loopback auth guard | Add `focusa doctor security` config check. |
| 5 Account Management | N/A/partial | Local daemon; OS users matter | Document supported local user model and service account assumptions. |
| 6 Access Control Management | Partial | Bearer token all-or-nothing | Add scoped tokens/roles. |
| 7 Continuous Vulnerability Management | Improved | npm/RustSec gates | Schedule recurring audit gate. |
| 8 Audit Log Management | Partial | Event logs/correlation IDs | Add log retention and tamper-evidence policy. |
| 9 Email/Web Browser Protections | Mostly N/A | Menubar/browser-like UI not public web | Ensure no dev server exposure in production. |
| 10 Malware Defenses | External | Host controls/Guardian/Imunify outside Focusa | Document reliance on host controls. |
| 11 Data Recovery | Partial | SQLite/data dir exists; no explicit backup policy | Add backup/restore runbook for Focusa data. |
| 12 Network Infrastructure Management | Partial | Loopback bind; Tailscale/remote possible | Document allowed network exposure patterns. |
| 13 Network Monitoring and Defense | External/partial | Local route telemetry exists | Integrate daemon health/security events into monitoring. |
| 14 Security Awareness and Skills Training | Partial | AGENTS/safety docs | Add agent-facing security playbook. |
| 15 Service Provider Management | Partial | LLM/provider SDKs and `wb` integrations | Document provider trust boundaries and data sent externally. |
| 16 Application Software Security | Improved | static gates, dependency gates, auth guard | Add PR/CI security gate suite. |
| 17 Incident Response Management | Partial | troubleshooting docs; evidence refs | Add Focusa security incident runbook. |
| 18 Penetration Testing | Not done | No dynamic fuzz/pentest proof yet | Add local API fuzz/smoke and threat scenario tests. |

CIS summary: good progress on controls 2, 4, 7, 16; next highest value controls are 3, 6, 8, 11, 18.

## Prioritized remediation from matrix review

1. **Route scopes and API permission matrix** — distinguish read/write/admin/service-control routes and enforce token scopes.
2. **Request size/rate/depth limits** — prevent API4/CWE-400 DoS on mutation and JSON-heavy routes.
3. **Command boundary reduction** — replace shell hotspots with argv or command enums.
4. **Path traversal/static tests** — cover cleanup, attachments, ECS/reference, project identity, and export paths.
5. **Tamper-evident event log** — hash chain or signed checkpoint for audit repudiation protection.
6. **Persistence retention/backup policy** — classify stores, define deletion/backup/restore behavior.
7. **Security CI suite** — run npm audit, cargo audit, static shell/unwrap, auth guard, privacy tests, link checks.
8. **Dynamic local API fuzz/smoke** — bounded fuzz for route schemas, oversized payloads, malformed JSON, auth failures.

## Overall posture

Focusa is now significantly safer for local-first use: supply-chain vulnerabilities are remediated to zero high/critical known findings, non-loopback unauthenticated startup is blocked, command/panic/privacy static gates exist, and systemd hardening constrains write paths.

Focusa is **not yet ready for broad network exposure** without additional scoped authorization, request limits, route privacy classes, and dynamic API abuse testing.
