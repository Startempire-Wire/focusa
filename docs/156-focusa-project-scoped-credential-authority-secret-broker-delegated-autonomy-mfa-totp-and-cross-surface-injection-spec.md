# Spec 156 — Project-Scoped Credential Authority, Secret Broker, Delegated Autonomy, MFA/TOTP, and Cross-Surface Injection

**Status:** Normative draft; primitive-owning; implementation not implied  
**Owner:** Focusa core and security runtime  
**Created:** 2026-08-14  
**Canonical label:** **Spec 156 — Project-Scoped Credential Authority, Secret Broker, Delegated Autonomy, MFA/TOTP, and Cross-Surface Injection**  
**Source issue:** [Startempire-Wire/focusa#299](https://github.com/Startempire-Wire/focusa/issues/299)  
**Primary implementation surfaces:** Focusa core, reducer, daemon, SQLite metadata/event persistence, API, Operation Registry, generated contracts, CLI, Pi extension, Agent Card, Skills, CallGraph, Workpoints, governed procedures, UIAI Engine, Cockpit, Mission Canvas, Menu Bar, future Focusa Desktop, provider/custodian adapters, consumer/injection adapters, Evidence, Receipts, Capability Truth, tests, conformance, and incident recovery  
**Depends on:** Specs 16, 34, 43, 49, 53, 72, 88, 96, 97, 100, 103, 104, 111, 119, 120, 124, 125, 130, 133, 135, 136, 137, 138, 139, 140, 141, 149, and 155; issues #252, #254, #256–#260, #277–#281, #283–#284, #289–#296, and #298; WPUIAI/uiai-engine#65  
**Supersedes:** none  
**Research basis:** rbw; Bitwarden Secrets Manager machine accounts; OpenBao; Infisical; 1Password Secure AI Access/Credential Broker patterns; Browser Use domain-scoped sensitive data; RFC 6238 TOTP; RFC 8628 OAuth Device Authorization Grant; WebAuthn/passkey semantics; current Focusa privacy/authority/runtime contracts; current UIAI TOTP, Aegis, rbw, browser-auth, diagnostics, and Cockpit credential designs

---

## 0. Executive requirement

Focusa MUST make credentials and authentication capabilities explicit, typed, preflighted execution dependencies. It MUST allow the operator to delegate anything from one manual credential use to broad standing project or provider autonomy while preserving exact scope, attribution, expiry, revocation, consequence boundaries, and secret-safe proof.

Focusa MUST separate three authorities:

1. **Credential authority** — Focusa decides who may use, reveal, manage, delegate, export, or revoke which credential role, for which project/workstream/frame/attempt/target/consumer, under which limits and evidence policy.
2. **Secret custody** — an approved provider adapter stores, retrieves, generates, rotates, or revokes raw secret material without making Focusa a password vault.
3. **Secret consumption** — an approved browser, API, process, SSH, Desktop, device, or operator adapter injects or uses secret material inside a controlled boundary without gaining authority to authorize itself.

The default autonomous path MUST give agents the capability to complete work without receiving plaintext credentials. The operator MAY separately grant plaintext reveal, management, delegation, deletion, or export rights when desired. Broad autonomy MUST remain possible and non-obstructive inside an accepted policy horizon; stronger convenience MUST NOT erase target binding, effect truth, audit, revocation, or incident recovery.

The central rule is:

> **FOCUSA AUTHORIZES. PROVIDER ADAPTERS KEEP CUSTODY. EXECUTION ADAPTERS INJECT OR USE. AGENTS RECEIVE ONLY THE CAPABILITY AND RESULT THEIR CURRENT GRANT ALLOWS.**

This specification is normative design authority. It does not by itself activate a provider, unlock a vault, grant credential access, expose a secret, approve an authentication challenge, install software, or admit implementation. Source mutation requires approved typed design, dependency-ordered IR5 work, and current execution authority.

## 0.1 Normative language

`MUST`, `MUST NOT`, `SHALL`, `SHALL NOT`, `REQUIRED`, `SHOULD`, `SHOULD NOT`, `MAY`, and `OPTIONAL` are normative. Example YAML and operation names define required semantics; exact generated language bindings may differ only when their schema mapping is deterministic and lossless.

## 0.2 Traceability

Parent program: #252
CallGraph/autonomous execution: #254/#294/#295
Runtime Constitution/context: #256
Remote/environment placement: #257 and Spec 139
Tool/Skill/docs distribution: #258/#259/#260/#296
Completion/proof/Receipts: #277/#278 and Specs 119/136
Browser missions: #281
Cockpit/operator authority: #283/#284/#290/#291
Governed procedures: #298
UIAI integration: WPUIAI/uiai-engine#65
UIAI credential design basis: `UIAI_COCKPIT_008_FOCUSA_MISSION_CANVAS_INTERLOCK_HANDOFF_CREDENTIALS_SUPERVISION_2026-08-01_v1.0.md` §8 and `AGENT_2FA_INTEGRATION.md`
Research case study: https://github.com/Dicklesworthstone/asimposium.org/blob/main/INSTRUCTIONS_FOR_COMPUTER_USE.md

## Implementation readiness

`IR0 — P0 security architecture/acceptance epic; not directly executable.` #294 must produce exact current Focusa/UIAI/Cockpit/menubar/Desktop/provider seams, approved Call Stack Designs, threat model, stable contracts, operations/events/errors/state machines, golden secret-safe fixtures, and dependency-ordered IR5 packets before any source or credential mutation.

This specification authorizes no vault access, credential reveal, provider enrollment, secret creation/rotation/deletion, 2FA action, session import, package installation, or infrastructure mutation.

## Classification

This is the missing **provider-neutral, project-scoped Credential Authority and autonomous authentication runtime** for Focusa.

Credentials are formal execution dependencies like tools, environments, tasks, and verifiers. Agents cannot perform reliable continuous work if password/API/SSH/OAuth/2FA/session requirements appear only after execution starts or are handled through ad hoc chat, clipboard, environment variables, or undocumented unlocked vault state.

The required architecture enables broad, low-friction autonomous access when the operator wants it while preserving exact operator-selected boundaries and immediate revocation.

## Verified current baseline — read-only

### Focusa

Focusa currently has valuable fragments but no coalesced runtime:

- `docs/current/PERSISTED_STATE_PRIVACY_CLASSES.md` classifies passwords, API keys, bearer tokens, private keys, and TOTP seeds as P4 and forbids raw persistence in Focusa stores;
- Spec 136 `AuthorityDecision` already allows opaque `credential_handle_refs`, actor/resource/scope/time-specific authorization, max uses, expiry, and revocation;
- #281/#298 propose secret slots, origin-bound BlindFill, auth handoffs, spend/destructive policy, and multi-surface routing;
- device pairing has token issuance/revocation and menubar Keychain storage;
- the generated `focusa-security-auth-licensing` Skill says it covers secrets but its required workflow currently routes only through device-pairing operations;
- no canonical credential provider registry, credential requirement preflight, project policy, credential grant/lease/use/reveal distinction, TOTP/auth challenge state machine, secret-use operation family, or cross-surface Credential Center was found;
- no installed Focusa tool family lets an agent safely discover credential readiness and use a credential without receiving its value.

### UIAI Engine

UIAI has more implementation/design than Focusa currently incorporates:

- authenticated `/api/2fa/code` exists;
- native RFC 6238 TOTP profiles and optional Aegis CLI profiles exist;
- `${rbw:item[:field]}` config expansion exists;
- current 2FA response returns the short-lived OTP code plus expiry metadata to the caller;
- UIAI browser sessions support cookies/auth save/load, but current identity-gap documentation marks opaque encrypted auth-state lifecycle and durable persona handling partial;
- diagnostics and research packets have strong redaction rules;
- UIAI Cockpit 008 already proposes the key split:

```text
Focusa CredentialUseGrant = authority without raw secret
UIAI SecretBinding       = custody/injection without self-authorization
```

That browser-specific split should become one provider-neutral Focusa contract. UIAI remains one secret consumer/custodian adapter, not the global credential authority.

### Current environment lesson

The approved credential CLI inventory and actual installed binary state can drift. Therefore declared provider choice is not readiness: Focusa must verify current binary/socket/service/profile/unlock/capability truth before admitting credential-dependent work. Exact local vault details remain private evidence.

## Research synthesis

### `rbw`

[`doy/rbw`](https://github.com/doy/rbw) uses a background agent to retain Bitwarden decryption state, similar to `ssh-agent`/`gpg-agent`; supports multiple profiles, password/custom-field retrieval, an SSH-agent socket, TOTP code generation, and Email/Authenticator/Yubico OTP account 2FA. Its docs state WebAuthn/passkey and Duo are unsupported.

Useful pattern: **stateful local custodian process** with short model-free calls. Limitation: human password vault semantics, static secrets, and host-local unlock/readiness are not a complete distributed machine-identity/lease system.

### Bitwarden Secrets Manager

[Machine accounts](https://bitwarden.com/help/machine-accounts/) represent non-human users, scope project secret access, issue access tokens, support read/read-write permissions, and retain event logs.

Useful pattern: separate human password vault from project-scoped machine automation. Limitation: provider-specific access tokens and product entitlement; not a universal credential contract.

### OpenBao

[OpenBao](https://openbao.org/docs/what-is-openbao/) provides identity authentication, path policies, audit, encrypted storage, dynamic credentials, leases/renewal, and tree revocation.

Useful pattern: short-lived dynamic credentials and machine identities should replace reusable static secrets when providers support them.

### Infisical

Official documentation provides machine identities, multiple machine-auth methods, temporary access, approval workflows, dynamic secrets, rotation, and audit streams.

Useful pattern: pluggable workload identity and dynamic-secret providers. Adoption must remain optional and capability/edition verified.

### 1Password secure AI patterns

[1Password secure AI access](https://www.1password.dev/get-started/secure-ai-access) documents runtime process injection, MCP configuration without plaintext secrets, an MCP server that does not return secrets to the agent, and agentic browser autofill over an encrypted channel after operator authorization.

Useful pattern: **agents can act on secrets without seeing them**. This should be Focusa's default even when the operator grants broad autonomous use.

### Browser Use sensitive-data pattern

Browser Use supports placeholder-based sensitive data, domain-specific mappings, allowed domains, and vision disabling to reduce screenshot exposure.

Useful pattern: binding secret role to target domain. Limitation: in-process framework configuration is not Focusa authority, custody, or full leak prevention.

### Standards

- [RFC 6238](https://datatracker.ietf.org/doc/html/rfc6238): TOTP, shared secret plus time; time sync and seed protection are mandatory.
- [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628): OAuth Device Authorization Grant; operator can authenticate/consent in a separate user agent without giving long-term credentials to the client.
- WebAuthn/passkeys/security keys are phishing-resistant possession/user-verification factors and require a verified device/authenticator path; they cannot be treated as ordinary retrievable secret strings.

### Operational case study

The linked ASImposium procedure correctly separates terminal-automatable work from browser-only remainder, forbids fresh authentication/password/2FA by default, verifies origin before input, sends one-time secrets only into approved secret stores, bounds spending/destructive actions, captures checkpoints, and hands unexpected authentication to the operator.

Focusa must turn those prose safeguards into typed project policy, grants, leases, bindings, uses, challenges, Receipts, and operator controls—not copy the checklist into prompts.

## Architectural decision

```text
Focusa Credential Authority
  owns requirement, project/workflow/frame scope, actor, purpose, target,
  operation/effect, autonomy policy, approval, grant, expiry, use limits,
  revocation, event/Receipt, completion dependency

Secret Custodian Adapter
  rbw | Aegis | Bitwarden SM | OpenBao | Infisical | 1Password |
  OS Keychain/Secret Service/Credential Manager | cloud provider | operator
  owns encrypted/raw secret material, provider session, retrieval/generation,
  optional rotation/revocation, and provider audit

Secret Consumer Adapter
  UIAI | API proxy | CLI/shell runner | SSH agent | Desktop autofill |
  device connector | provider SDK
  receives a one-use binding or injects/uses the value in a controlled boundary

Agent/model
  receives requirement, role, availability, policy, use result, errors,
  and evidence refs; receives raw value only when an explicit exposure grant allows it
```

Focusa never becomes a password vault. Provider adapters remain replaceable. The same Credential Authority governs every shell and execution surface.

## Operator discretion is multidimensional

A single “autonomy” slider is insufficient. The operator controls independent dimensions:

```yaml
CredentialDelegationDimensions:
  scope_kind: exact_secret|credential_role|frame|workflow_run|workstream|project|provider_account|vault
  actor_scope: exact_actor|role|team|any_project_agent
  operation_scope: discover_metadata|use|reveal|create|update|rotate|revoke|delete|export|delegate
  exposure_mode: none|metadata_only|blind_use|consumer_injection|agent_plaintext|export
  target_scope: exact_resource|origin|host|process|API_route|account|tenant|environment|declared_set
  duration: one_use|frame|attempt|run|workflow|until_timestamp|renewable|standing
  approval_mode: every_use|every_session|every_run|workflow_preapproved|project_preapproved|standing_no_prompt
  use_limit: integer|unlimited_within_grant
  side_effect_ceiling: typed_ref
  budget_ceiling: optional
  network_egress_policy_ref: typed_ref
  evidence_and_notification_policy_ref: typed_ref
```

Operator presets may simplify these dimensions:

1. **Manual handoff** — no autonomous use; operator types/approves each challenge.
2. **Approve each use** — agent requests; operator sees consequence preview.
3. **Blind delegated** — preapproved exact role/targets; agent can use but cannot see.
4. **Workflow autonomous** — one accepted workflow may use required roles for its duration.
5. **Project autonomous** — standing project-scoped use/refresh within declared targets/effects.
6. **Operator-trusted broad** — broad use and optionally management/reveal rights across an explicit provider/vault until revoked.

The operator may choose broad/full access. Focusa still preserves attribution, target checks, secret-safe logs, revocation, and effect truth. `can_use`, `can_reveal`, `can_manage`, `can_delegate`, and `can_export` remain separate rights.

## Credential and authentication classes

### Secret material classes

```text
password
API token/key
OAuth client secret/refresh token/access token
database credential
SSH private key/certificate/passphrase
TLS/private signing key
TOTP seed
HOTP seed/recovery code
session cookie/browser storage/auth-state capsule
provider service-account credential
dynamic leased credential
license/pairing/bearer token
```

### Authentication challenge classes

```text
password
TOTP
email OTP
SMS OTP
push approval
Duo/vendor challenge
WebAuthn/passkey
hardware security key
biometric/native OS approval
OAuth user consent
OAuth device authorization
CAPTCHA/anti-bot challenge
recovery code
account recovery
legal/terms consent
```

Each has a different automation posture. “2FA available” is not sufficient.

## Canonical contracts

### CredentialProviderDescriptor

```yaml
schema: focusa.credential_provider_descriptor.v1
provider_id: stable_id
provider_kind: rbw|bitwarden_password_manager|bitwarden_secrets_manager|aegis|openbao|vault|infisical|onepassword|os_keychain|secret_service|windows_credential_manager|cloud_secret_manager|environment|operator|custom
adapter_version: string
custody_location_ref: private_typed_ref
supported_secret_classes: []
supported_operations: []
supported_auth_methods: []
supports_machine_identity: boolean
supports_dynamic_secrets: boolean
supports_leases: boolean
supports_rotation: boolean
supports_revocation: boolean
supports_audit: boolean
supports_blind_use: boolean
supports_process_injection: boolean
supports_browser_injection: boolean
availability: available|locked|degraded|unavailable|unknown|version_incompatible
freshness: current|stale|unknown
health_evidence_refs: []
private_configuration_ref: optional
content_digest: sha256
```

Descriptors contain no account identifier or secret value in model/public projections.

### CredentialRole

```yaml
schema: focusa.credential_role.v1
credential_role_id: stable_id
human_label: bounded_nonsecret
project_scope_ref: typed_ref
purpose: bounded
secret_class: enum
provider_binding_ref: opaque
allowed_target_refs: []
allowed_origin_refs: []
allowed_host_process_route_refs: []
default_exposure_mode: enum
rotation_policy_ref: optional
owner_ref: typed_ref
status: configured|ready|locked|missing|expired|revoked|rotation_due|incident
metadata_digest: sha256
```

Agents request roles such as `production_dns_admin` or `github_release_writer`, never vault item names or raw values unless the operator deliberately permits discovery.

### CredentialRequirement

```yaml
schema: focusa.credential_requirement.v1
requirement_id: stable_id
project_scope_ref: typed_ref
workstream_ref: typed_ref
callgraph_frame_ref: typed_ref
attempt_generation: integer
credential_role_ref: typed_ref
required_operation: use|reveal|manage|rotate|revoke
required_exposure_mode: enum
exact_target_refs: []
exact_consumer_ref: typed_ref
required_auth_challenge_support: []
precondition_refs: []
validity_minimum: duration
use_count_required: integer
evidence_requirement_refs: []
```

Credential requirements are compiled before IR5 admission. Unknown credential readiness blocks before mutation rather than surprising the worker mid-frame.

### CredentialAutonomyPolicy

```yaml
schema: focusa.credential_autonomy_policy.v1
policy_id: UUIDv7
project_scope_ref: typed_ref
applies_to_actor_refs: []
applies_to_role_refs: []
applies_to_workflow_refs: []
credential_role_refs: []
operation_scope: []
exposure_modes: []
target_scope_refs: []
duration_policy_ref: typed_ref
approval_mode: enum
max_uses: optional
side_effect_ceiling_ref: typed_ref
budget_ceiling_ref: optional
notification_policy_ref: typed_ref
challenge_policy_ref: typed_ref
freshness_policy_ref: typed_ref
issued_by_operator_ref: typed_ref
issued_at: timestamp
expires_at: optional
revocation_ref: optional
policy_revision: integer
content_digest: sha256
```

### CredentialUseGrant

```yaml
schema: focusa.credential_use_grant.v1
grant_id: UUIDv7
policy_ref: typed_versioned_ref
project_scope_ref: typed_ref
workstream_ref: typed_ref
frame_attempt_ref: typed_ref
requesting_actor_ref: typed_ref
credential_role_ref: typed_ref
operation: use|reveal|manage|rotate|revoke|export
exposure_mode: enum
exact_target_refs: []
exact_consumer_ref: typed_ref
allowed_operation_or_field_refs: []
side_effect_ceiling_ref: typed_ref
approval_refs: []
valid_from: timestamp
expires_at: timestamp
max_uses: integer
uses_consumed: integer
generation: integer
revocation_ref: optional
receipt_policy_ref: typed_ref
status: proposed|approval_required|active|consumed|expired|revoked|blocked|incident
content_digest: sha256
```

Grant contains no secret/provider item identifier that untrusted consumers can dereference independently.

### SecretBinding

```yaml
schema: focusa.secret_binding.v1
binding_id: UUIDv7
grant_ref: typed_ref
custodian_provider_ref: private_typed_ref
credential_role_ref: typed_ref
consumer_ref: typed_ref
target_refs: []
injection_mechanism: browser_blind_fill|api_proxy|process_fd|process_stdin|ephemeral_env|ssh_agent|keychain_autofill|device_approval|operator_input|agent_plaintext
secret_material_ref: provider_private_opaque_ref
lease_ref: typed_ref
use_counter_ref: typed_ref
redaction_policy_ref: typed_ref
status: bound|ready|used|expired|revoked|failed|incident
```

The model-visible binding excludes `secret_material_ref`.

### CredentialLease

```yaml
schema: focusa.credential_lease.v1
lease_id: UUIDv7
grant_ref: typed_ref
binding_ref: typed_ref
consumer_instance_ref: typed_ref
session_process_or_connection_ref: typed_ref
issued_at: timestamp
expires_at: timestamp
renewable: boolean
max_uses: integer
uses_consumed: integer
fencing_generation: integer
revocation_epoch: integer
cleanup_policy_ref: typed_ref
status: active|renewing|consumed|expired|revoked|orphaned|cleanup_pending|incident
```

### AuthenticationChallenge

```yaml
schema: focusa.authentication_challenge.v1
challenge_id: UUIDv7
project_scope_ref: typed_ref
frame_attempt_ref: typed_ref
consumer_session_ref: typed_ref
exact_target_ref: typed_ref
origin_account_assertion_ref: typed_ref
challenge_kind: enum
observed_challenge_ref: redacted_evidence_ref
eligible_resolver_refs: []
selected_resolution: blind_secret|totp|device_authorization|operator_takeover|push_wait|security_key|recovery_code|unsupported
required_grant_ref: optional
expires_at: optional
attempts_remaining: optional
lockout_risk: low|medium|high|unknown
status: observed|validating|approval_required|ready|resolving|resolved|failed|expired|blocked|handoff_required
```

### SecretUseIntent

```yaml
schema: focusa.secret_use_intent.v1
use_intent_id: UUIDv7
grant_ref: typed_ref
binding_ref: typed_ref
lease_ref: typed_ref
frame_attempt_ref: typed_ref
exact_target_ref: typed_ref
operation_or_field_ref: typed_ref
consumer_ref: typed_ref
idempotency_key: string
pre_use_observation_ref: typed_ref
issued_at: timestamp
expires_at: timestamp
content_digest: sha256
```

### SecretUseReceipt

```yaml
schema: focusa.secret_use_receipt.v1
receipt_id: UUIDv7
use_intent_ref: typed_ref
grant_binding_lease_refs: []
project_frame_attempt_refs: []
provider_ref: nonsecret_descriptor_ref
consumer_ref: typed_ref
target_origin_host_process_refs: []
operation_class: string
exposure_mode: enum
used_at: timestamp
result: used|rejected|expired|origin_mismatch|target_mismatch|consumer_mismatch|provider_locked|injection_failed|possible_exposure|revoked
side_effect_observation_ref: typed_ref
redaction_verification_ref: typed_ref
provider_audit_ref: optional
uses_remaining: integer
lease_status: enum
secret_material_in_receipt: false
```

### CredentialIncident

Covers suspected transcript/log/screenshot/diagnostics/clipboard/env/process-list/file/export exposure, wrong target/account/origin, reused OTP/recovery code, stale lease, unexpected provider access, brute-force/lockout risk, leaked session state, provider compromise, and failed cleanup. It triggers freeze, scope impact analysis, revocation, rotation/re-authentication, evidence quarantine, and recovery proof.

## Credential readiness and autonomous flow

```text
accepted CallGraph frame
→ compile CredentialRequirements
→ resolve project AutonomyPolicy
→ verify provider/adapter/profile/unlock/freshness
→ verify target/account/origin/host/process/consumer
→ check secret/challenge capability and validity horizon
→ preview required grants/handoffs
→ operator approval only when policy requires
→ issue fenced CredentialUseGrant
→ custodian creates SecretBinding
→ consumer obtains temporary CredentialLease
→ just-in-time use/injection
→ immediate pre/post observation
→ secret-safe use Receipt + provider audit ref
→ consume/renew/revoke/cleanup lease
→ independent step verification
```

For workflow/project-autonomous policies, this path is deterministic and noninteractive while conditions remain valid. Expiry, revocation, target drift, changed operation/effect, challenge-class change, lockout risk, or possible exposure pauses before further credential use.

## 2FA and authentication policy

### TOTP

- TOTP seed remains provider-only P4 material;
- generate just in time after exact challenge/origin/account validation;
- verify trusted clock/time sync and algorithm/digits/period metadata;
- do not use a code with insufficient remaining lifetime;
- bind generated code to one challenge/use and redact it from ordinary tool output;
- UIAI blind-fills the code directly by default;
- raw code return to an agent requires explicit `agent_plaintext` grant;
- retries count against lockout policy and require fresh code/window when needed;
- provider options include rbw `code`, UIAI native RFC 6238, and Aegis adapter.

Current `/api/2fa/code` is a useful primitive but returning the OTP to the caller is not the final default Focusa contract. Add an opaque `generate_and_inject`/binding path while preserving an explicitly authorized reveal/debug path.

### Email/SMS OTP

Use a separately scoped mailbox/SMS connector only if the operator delegated read/use for the exact target and message class. Otherwise hand off. Do not grant ambient mailbox access merely to satisfy one login.

### Push/Duo

Create a mobile/operator approval handoff with target/account/action details and expiration. Agent waits on canonical challenge state; it cannot claim approval from elapsed time.

### WebAuthn/passkey/hardware key/biometric

Use a verified local device/Desktop/UIAI operator-persona path. User-presence/user-verification requirements remain real; remote software must not fabricate them. A standing operator policy may authorize supported platform automation where authenticator semantics permit it, but Focusa records the exact device and verification posture.

### OAuth authorization/device flow

Prefer delegated scopes, machine identity, service account, OAuth device flow, or existing authorized refresh token over collecting human passwords. Consent scopes and account identity are displayed and receipted. Refresh/access tokens remain leased secrets.

### Recovery codes/account recovery

Single-use, high-consequence credential consumption. Require dedicated grant, use count, immediate provider reconciliation, remaining-code state, and rotation/replenishment guidance. Account recovery cannot be inferred as routine login.

### Authenticated browser/session state

Cookies, local/session storage, refresh tokens, and browser profile auth state are credentials:

- encrypt at rest through UIAI/Desktop custodian;
- expose only opaque session/persona refs;
- bind project/workstream/account/origin/profile;
- expiration/revocation/logout/deletion semantics;
- no cross-project/session reuse by default;
- broad reuse only under explicit operator policy;
- reobserve account/origin before material action;
- never export to Focusa state/evidence/transcript.

## Consumer-specific injection

### UIAI Engine

- receives active grant + opaque provider binding;
- validates project/workstream/frame/session/origin/account/field/action;
- retrieves/generates only at injection time;
- fills password/TOTP without model/snapshot/diagnostics/recording exposure;
- suppresses or redacts captures during secret input;
- returns secret-safe observation and Receipt candidate;
- authenticated profile/session lease remains separately revocable;
- UIAI cannot issue its own grant or settle completion.

UIAI Cockpit 008 §8 is adopted as the browser-specific contract and generalized through provider adapters.

### API/provider adapters

Prefer proxy/signing semantics: adapter injects header/body/signature internally and returns redacted result/effect metadata. Never put bearer/API keys in tool inputs or URLs.

### CLI/shell/process

Preferred mechanisms:

1. agent/socket/keyring-backed native authentication;
2. inherited file descriptor or protected stdin;
3. temporary protected file with immediate cleanup only where required;
4. ephemeral environment variable for the child process only;
5. command argument only when unavoidable and explicitly risk-accepted.

No shell interpolation, command echo, process-list exposure, trace mode, core dump, or child inheritance beyond policy. Stdout/stderr undergo leak scanning/redaction before model/events.

### SSH

Prefer ssh-agent/rbw-agent, short-lived OpenSSH certificates, hardware-backed key use, or brokered agent forwarding with exact host/user/command policy. Raw private key material never enters model context.

### Future Focusa Desktop

Desktop can host the native broker/client and OS Keychain/Secret Service/Credential Manager adapters, passkey/biometric/security-key handoffs, UIAI browser extension/autofill bridge, and secure operator input. It remains a consumer/custodian shell; Focusa daemon grants remain canonical.

## Cross-surface product contract

### Focusa daemon/headless

- provider-neutral registry;
- project/workstream policy;
- requirement preflight;
- grants/leases/revocation/fencing;
- challenge/handoff state;
- secret-safe events/Receipts;
- adapter health and Capability Truth;
- no P4 storage.

### UIAI Engine

- challenge detection;
- exact-origin/account/session validation;
- BlindFill/TOTP generation-and-injection;
- encrypted auth profiles;
- secure takeover/reobservation;
- redacted diagnostics/evidence;
- no self-authorization.

### Cockpit

Full **Credential Center**:

- project credential readiness by role/class—not values;
- provider/adapter health, lock, expiry, rotation, audit status;
- grant builder across scope/actor/operation/exposure/target/duration/approval/use/effect/budget dimensions;
- autonomy presets plus advanced controls;
- pending challenges and operator handoffs;
- active/recent grants, bindings, leases, use counters, expiry, consumer, target;
- reveal/manage/rotate/export/delete separated and consequence-previewed;
- browser/CLI/remote/Desktop session posture;
- provider audit and Focusa Receipt correlation;
- leak/lockout/origin/session incidents;
- emergency project freeze, revoke-all, provider lock, and rotation workflow;
- comments/suggestions/history under #290/#291.

Cockpit never displays secret values by default. Explicit raw reveal uses a secure, non-recorded, time-bounded view and remains a separate auditable operator action.

### Menu Bar

Compact, fast, non-secret controls:

- current project credential readiness;
- provider locked/degraded/rotation-due status;
- active lease count and nearest expiry;
- pending credential/2FA/operator approval notification;
- approve once / approve workflow / open Credential Center;
- lock provider;
- revoke project leases / emergency stop;
- deep link to exact Cockpit/Desktop object.

Menu Bar never lists vault contents, passwords, OTP values, cookies, or raw tokens. Keychain stores its own pairing token; credential grants/leases remain daemon authority.

### Future Focusa Desktop

- all Cockpit Credential Center capabilities;
- secure native input and OS keychain/provider setup;
- passkey/biometric/security key presence;
- browser extension/autofill integration;
- multi-node/provider profile administration;
- private local audit/redaction review;
- offline/status behavior that cannot fabricate grants.

### Pi/agents

Agents receive bounded tools for provider readiness, credential role availability, requirement preflight, grant/request status, challenge resolution status, use dispatch, and recovery. Secret values are not ordinary tool outputs.

## Provider adapter policy

Focusa ships contracts and adapters, not one mandatory vault.

### Initial compatibility priorities

1. **rbw adapter** — password/custom fields/TOTP/SSH agent for existing operator vaults;
2. **Aegis adapter** — TOTP-only existing VPS/mobile-export profile;
3. **OS keychain adapters** — macOS Keychain, Linux Secret Service, Windows Credential Manager;
4. **Bitwarden Secrets Manager adapter** — project-scoped machine accounts;
5. **OpenBao adapter** — dynamic secrets, leases, PKI/database/cloud credentials;
6. **operator handoff adapter** — typed manual input/consent;
7. **environment adapter** — compatibility only, strict process scope and provenance.

Later optional adapters: Infisical, HashiCorp Vault, 1Password, cloud-native secret managers, SPIFFE/SPIRE workload identity, Teleport/SSH certificate authorities, custom enterprise brokers.

A provider adapter must implement only supported capabilities. No fabricated rotation/revocation/audit/dynamic-secret claims.

## Operation/tool family

Exact names require #258 conflict review and approved Call Stack Design. Required intents:

```text
focusa_credentials_status
focusa_credential_providers_list
focusa_credential_provider_show
focusa_credential_provider_verify
focusa_credential_roles_list
focusa_credential_role_show
focusa_credential_role_bind_preview
focusa_credential_role_bind_commit
focusa_credential_requirements_preflight
focusa_credential_policy_show
focusa_credential_policy_preview
focusa_credential_policy_commit
focusa_credential_grants_list
focusa_credential_grant_preview
focusa_credential_grant_issue
focusa_credential_grant_revoke
focusa_credential_leases_list
focusa_credential_lease_revoke
focusa_auth_challenge_status
focusa_auth_challenge_resolve_preview
focusa_auth_challenge_resolve
focusa_credential_use_preview
focusa_credential_use_dispatch
focusa_credential_rotation_preview
focusa_credential_rotation_commit
focusa_credential_incident_report
focusa_credential_emergency_freeze
focusa_credentials_doctor
```

No general `get_secret` tool is enabled by default. Explicit reveal uses a separate high-consequence operation unavailable to ordinary workers unless the operator grants `agent_plaintext`.

Required Skills:

```text
focusa-credential-authority
focusa-authentication-challenges
focusa-credential-provider-administration
```

Runbooks cover project setup, autonomous delegation, browser BlindFill, CLI/API/SSH use, TOTP/MFA/device flows, rotation/revocation, incidents, and provider-specific recovery without embedding vault data.

## State machines

### Grant/lease

```text
requirement_detected
→ provider_resolved
→ policy_evaluated
→ previewed
→ approval_required | granted
→ bound
→ leased
→ used
→ consumed | renewable
→ expired | revoked | incident
→ cleanup_verified
```

### Authentication challenge

```text
observed
→ origin_account_validating
→ classified
→ policy_evaluated
→ autonomous_resolver_ready | approval_required | handoff_required | unsupported
→ resolving
→ reobserving
→ resolved | failed | expired | lockout_risk | incident
```

### Incident

```text
suspected_exposure_or_misuse
→ freeze affected grants/consumers
→ preserve redacted evidence
→ determine secret/target/project/provider blast radius
→ revoke
→ rotate/re-authenticate/recover
→ verify cleanup and downstream health
→ operator settlement
```

## Errors

Required typed classes:

```text
credential_provider_unconfigured
credential_provider_unavailable
credential_provider_locked
credential_provider_version_incompatible
credential_provider_capability_unsupported
credential_role_missing
credential_role_ambiguous
credential_requirement_unsatisfied
credential_policy_missing
credential_policy_expired
credential_grant_required
credential_grant_scope_mismatch
credential_grant_exposure_not_allowed
credential_grant_use_limit_exhausted
credential_lease_expired
credential_lease_revoked
credential_binding_consumer_mismatch
credential_target_mismatch
credential_origin_or_account_mismatch
secret_injection_failed
secret_possible_exposure
secret_redaction_failed
totp_clock_untrusted
totp_window_too_short
totp_profile_missing
auth_challenge_unsupported
authentication_handoff_required
passkey_or_user_presence_required
account_lockout_risk
recovery_code_consumption_requires_approval
auth_session_stale_or_revoked
credential_rotation_required
credential_cleanup_unverified
```

Every error exposes non-secret expected/observed metadata, effect/exposure uncertainty, retry safety, exact recovery operations, evidence refs, and one next action. No raw provider stderr containing secrets, blank output, generic failure, or circular approval/handoff.

## Security laws

1. Focusa stores refs/policies/grants/leases/Receipts, never P4 values.
2. Default agent access is capability use without plaintext knowledge.
3. Operator may explicitly grant plaintext/reveal/export; it remains separate, scoped, expiring, revocable, and auditable.
4. Every use is project/workstream/frame/attempt/actor/target/consumer/time/generation bound unless the operator deliberately chooses broader scope.
5. Prompt/page/tool/document/MCP content cannot request or expand credential authority.
6. Vault/provider metadata is minimized; unauthorized agents cannot enumerate names/accounts.
7. Secret values never appear in prompts, transcripts, Focus State, Workpoints, CallGraph, events, logs, diagnostics, screenshots, recordings, URLs, command lines, process lists, crash dumps, evidence, Receipts, or exports under blind/injection modes.
8. Redaction is defense-in-depth, not authorization; raw secret should not enter observable surface first.
9. Credential use checks revocation/expiry/target/origin/account/consumer/generation immediately before dispatch.
10. Possible use/exposure/effect is reconciled before retry.
11. Provider unlock/session state is a credential dependency with expiry and liveness.
12. Secret memory is minimized and zeroized best-effort; no caching beyond lease/provider policy.
13. Adapters run least-privileged and use authenticated local IPC/peer identity.
14. Public/cross-project telemetry receives only redacted non-secret projections.
15. Emergency revoke/freeze remains available even when model/UI/provider execution is degraded.
16. Backup/export/restore of provider data requires separate operator policy; Focusa backup does not absorb secret stores.

## Capability Truth

Report separately:

```text
credential_policy_authority
provider_registry
provider_installed_health
credential_requirement_preflight
grant_and_lease_runtime
blind_consumer_injection
agent_plaintext_reveal
TOTP_generation
TOTP_blind_injection
non-TOTP_challenge_handling
encrypted_browser_auth_state
CLI_API_SSH_injection
Cockpit_Credential_Center
Menu_Bar_controls
Desktop_native_broker
rotation_and_revocation
incident_response
installed_end_to_end_autonomous_auth
```

A schema, Skill, configured env var, unlocked provider, generated OTP, UI card, or successful login cannot claim the full profile.

## Normative requirements

- **FCA-CRED-REQ-001:** Credentials SHALL be explicit typed dependencies in CallGraph/readiness before credential-dependent frames become IR5.
- **FCA-CRED-REQ-002:** Focusa SHALL own authorization/grants/leases/Receipts while approved provider adapters retain secret custody.
- **FCA-CRED-REQ-003:** Operator SHALL independently control scope, actor, operation, exposure, target, duration, approval frequency, uses, effects, budget, notification, and revocation.
- **FCA-CRED-REQ-004:** Broad project/standing autonomy SHALL be supported without forcing per-use prompts when the operator explicitly chooses it.
- **FCA-CRED-REQ-005:** `use`, `reveal`, `manage`, `delegate`, `delete`, and `export` SHALL be independent authorities.
- **FCA-CRED-REQ-006:** Default autonomous use SHALL not expose plaintext secret material to the model.
- **FCA-CRED-REQ-007:** Every use SHALL bind exact current project/workstream/frame/attempt/actor/consumer/target/time/generation or an explicit broader operator grant.
- **FCA-CRED-REQ-008:** Provider declaration SHALL not imply availability; runtime capability/unlock/profile/freshness verification is mandatory.
- **FCA-CRED-REQ-009:** TOTP SHALL be generated just-in-time, clock/window checked, one-challenge bound, and blind-injected by default.
- **FCA-CRED-REQ-010:** Authentication challenge classes SHALL have explicit resolver/handoff/unsupported policies; no generic “2FA handled.”
- **FCA-CRED-REQ-011:** WebAuthn/passkey/security-key/biometric/user-presence semantics SHALL not be fabricated or reduced to retrievable strings.
- **FCA-CRED-REQ-012:** Browser/session auth state SHALL be encrypted, opaque, origin/account/profile scoped, expiring, revocable, and non-exported to Focusa.
- **FCA-CRED-REQ-013:** UIAI SHALL inject browser credentials only with matching active Focusa grant and SecretBinding; UIAI cannot authorize itself.
- **FCA-CRED-REQ-014:** Shell/API/SSH/Desktop consumers SHALL use the least observable supported injection mechanism and prove cleanup/redaction.
- **FCA-CRED-REQ-015:** Provider/audit and Focusa use Receipts SHALL correlate without secret material.
- **FCA-CRED-REQ-016:** Possible exposure/use/effect SHALL freeze affected retries until reconciliation/revocation/rotation policy completes.
- **FCA-CRED-REQ-017:** Operator steering/revocation/emergency freeze SHALL fence every consumer and active lease across hosts/surfaces.
- **FCA-CRED-REQ-018:** Cockpit SHALL provide full non-secret Credential Center administration; Menu Bar SHALL provide compact readiness/approval/revoke/deep-link controls.
- **FCA-CRED-REQ-019:** Headless/API/CLI/Pi/MCP/SDK/Cockpit/Menu Bar/Desktop SHALL share canonical policy/grant/lease/challenge state and operation semantics.
- **FCA-CRED-REQ-020:** Provider adapters SHALL remain optional and capability-truthful; no one vault is mandatory.
- **FCA-CRED-REQ-021:** Workflow packs/#298 SHALL reference CredentialRoles/Requirements, never embed values or ambient provider access.
- **FCA-CRED-REQ-022:** Completion SHALL require secret-safe evidence that required authentication/use succeeded, not the secret or OTP itself.
- **FCA-CRED-REQ-023:** Current UIAI OTP-return path SHALL remain explicit reveal posture or migrate to opaque generation-and-injection for default autonomous browser use.
- **FCA-CRED-REQ-024:** Capability Truth SHALL distinguish every credential layer and prevent static/configured/demo overclaim.

## Required tests

- **FCA-CRED-TEST-001:** Project frame with missing role blocks at readiness before mutation.
- **FCA-CRED-TEST-002:** Declared but absent/locked/stale provider reports exact recovery; no fallback switch without policy.
- **FCA-CRED-TEST-003:** Manual, per-use, workflow, project-standing, and operator-broad policies produce distinct valid behavior.
- **FCA-CRED-TEST-004:** Broad blind-use grant completes an autonomous workflow without exposing plaintext or interrupting operator.
- **FCA-CRED-TEST-005:** Use grant cannot reveal/manage/export; separately granted reveal can, with secure/audited projection.
- **FCA-CRED-TEST-006:** Wrong project/workstream/frame/attempt/actor/consumer/generation is fenced.
- **FCA-CRED-TEST-007:** Wrong origin/account/host/process/API target blocks injection.
- **FCA-CRED-TEST-008:** Malicious page/runbook/tool/MCP prompt cannot request or expand secret access.
- **FCA-CRED-TEST-009:** rbw adapter retrieves/injects a fixture password/TOTP role without outputting it.
- **FCA-CRED-TEST-010:** Aegis fixture generates one current TOTP and blind-fills exact challenge.
- **FCA-CRED-TEST-011:** RFC 6238 vectors, clock skew, near-expiry window, algorithm/digits/period, reuse, and retry cases pass.
- **FCA-CRED-TEST-012:** Raw `/api/2fa/code` is unavailable to ordinary blind-use worker; authorized reveal remains explicit.
- **FCA-CRED-TEST-013:** Email/SMS connector absent causes handoff, not ambient mailbox permission.
- **FCA-CRED-TEST-014:** Push/Duo waits for observed approval and does not infer success.
- **FCA-CRED-TEST-015:** Passkey/security-key/biometric requires verified supported device/user-presence path.
- **FCA-CRED-TEST-016:** OAuth device flow grants only reviewed scopes/account and stores resulting token as leased secret.
- **FCA-CRED-TEST-017:** Recovery code use is single-use, receipted, reconciled, and updates remaining posture.
- **FCA-CRED-TEST-018:** UIAI screenshot/snapshot/diagnostics/FPV/recording/network packet contains no password/TOTP/cookie/token.
- **FCA-CRED-TEST-019:** API/CLI/SSH consumers use controlled injection and outputs/process list/files/core dumps pass secret scan.
- **FCA-CRED-TEST-020:** Browser auth capsule restart/reopen preserves correct account/origin and remains opaque/revocable.
- **FCA-CRED-TEST-021:** Cross-project/session/profile credential reuse is rejected unless an explicit broader policy allows it.
- **FCA-CRED-TEST-022:** Grant expiry/revocation/emergency freeze propagates to local/remote/UIAI/Desktop consumers.
- **FCA-CRED-TEST-023:** Crash before/after retrieval/injection/use/Receipt/cleanup produces no blind retry or orphan credential.
- **FCA-CRED-TEST-024:** Suspected leak freezes, revokes, rotates/re-authenticates, quarantines evidence, and verifies cleanup.
- **FCA-CRED-TEST-025:** Provider audit and Focusa Receipt correlate by opaque refs without secret values.
- **FCA-CRED-TEST-026:** Cockpit manages policy/grants/leases/challenges/incidents; Menu Bar status/approve/revoke deep-links exact object.
- **FCA-CRED-TEST-027:** Pi/MCP/API/CLI/Cockpit/Desktop return semantically identical non-secret grant/lease state.
- **FCA-CRED-TEST-028:** Weak model completes password+TOTP workflow with opaque role/binding and no architectural inference.
- **FCA-CRED-TEST-029:** 1k-role/lease projection remains bounded and cannot enumerate unauthorized metadata.
- **FCA-CRED-TEST-030:** Static schema/config/Skill/UI/OTP demo/login cannot satisfy installed autonomous-credential capability profile.

## Reference implementation phases

### Phase 0 — authority/threat model/reconciliation

Reconcile Focusa P4 privacy, Spec 136, #254/#281/#291/#294/#295/#298, UIAI Cockpit 008 §8, UIAI Agent 2FA/RBW integration, menubar Keychain/pairing, and Desktop keychain plans. Approve exact owner matrix and threat model.

### Phase 1 — provider-neutral metadata/preflight

Provider descriptors, role metadata, requirements, project policies, health/unlock/freshness, no secret use.

### Phase 2 — grants/leases + rbw/Aegis fixtures

Canonical policy/grant/lease/revoke/Receipt runtime; opaque fixture adapters; then approved rbw/Aegis adapters.

### Phase 3 — UIAI BlindFill/auth challenges

Replace ordinary OTP/password return with grant-bound generation/retrieval-and-injection; encrypted auth-state/persona lifecycle; reobservation and proof.

### Phase 4 — API/CLI/SSH consumers + machine/dynamic providers

Controlled process/proxy/agent injection; Bitwarden Secrets Manager/OpenBao optional adapters; rotation/reconciliation.

### Phase 5 — Cockpit/Menu Bar/Pi/headless parity

Credential Center, autonomy policies, challenges, leases, emergency controls, Skills, operation registry, cross-shell conformance.

### Phase 6 — Desktop/native authentication

OS keychain, secure input, passkey/biometric/security-key support, browser autofill, multi-node administration after separate native adapter approval.

### Phase 7 — autonomous/security certification

Repeated project-scoped password/TOTP/API/SSH/OAuth/session workflows; malicious prompts/pages/tools; crash/revoke/rotate/leak/lockout; weak models; full installed proof.

## Atomic decomposition — blocked below IR5

- **FCA-CRED-TASK-001:** exact Focusa/UIAI/menubar/Desktop/provider/auth-state/2FA current inventory and ownership matrix.
- **FCA-CRED-TASK-002:** approve Credential Authority/broker/provider/consumer/challenge/Cockpit Call Stack Designs and threat model.
- **FCA-CRED-TASK-003:** finalize contracts/states/events/operations/errors and secret-safe cross-language fixtures.
- **FCA-CRED-TASK-004:** implement provider registry/role metadata/requirement preflight.
- **FCA-CRED-TASK-005:** implement project autonomy policy/grants/leases/revoke/fencing/Receipts.
- **FCA-CRED-TASK-006:** implement fixture then rbw/Aegis adapters without secret output.
- **FCA-CRED-TASK-007:** implement UIAI grant-bound BlindFill/TOTP/auth-state lifecycle.
- **FCA-CRED-TASK-008:** implement API/CLI/process/SSH consumer injection and cleanup proof.
- **FCA-CRED-TASK-009:** implement optional machine/dynamic provider adapters.
- **FCA-CRED-TASK-010:** implement auth challenge/handoff/recovery/incident state machines.
- **FCA-CRED-TASK-011:** implement Cockpit/Menu Bar/Pi/headless tools/Skills/projections using current components.
- **FCA-CRED-TASK-012:** implement Desktop native adapters after separate approval.
- **FCA-CRED-TASK-013:** execute security/crash/scale/weak-model/cross-surface/autonomous dogfood.

Every task needs a separate #294 IR5 packet with exact files/symbols/commands/mutations/proof.

## Acceptance

- [ ] FCA-CRED-REQ-001–024 map to exact contracts/owners/operations/events/errors/tests/evidence.
- [ ] Credentials are first-class project/workflow/frame requirements and block before mutation when unavailable.
- [ ] Operator can select manual through project-standing/broad autonomy across every delegation dimension.
- [ ] Agents complete autonomous password/TOTP/API/SSH/OAuth/session workflows without routine prompts under approved policies.
- [ ] Default use does not expose plaintext; operator can explicitly grant separate reveal/manage/export rights.
- [ ] rbw/Aegis and at least one machine/dynamic provider pass provider-neutral conformance.
- [ ] UIAI BlindFill/auth state, Cockpit Credential Center, Menu Bar controls, Pi/headless tools, and future Desktop seams share canonical state.
- [ ] Revocation/fencing/rotation/leak/lockout/crash/side-effect recovery passes across local/remote/browser/Desktop consumers.
- [ ] Secret-safe Receipts prove use/outcome while no P4 material enters Focusa or public/model-visible artifacts.
- [ ] Capability Truth and installed end-to-end multi-workflow dogfood pass.

## Non-duplication

- Provider systems own encryption/storage/provider sessions and their own audit.
- Spec 136 owns generic authority/risk/settlement.
- #254/#295 own graph/runtime/type/effect execution.
- #281/UIAI Cockpit 008 own browser-specific binding/injection/takeover.
- #291 owns operator authority UI.
- #298 owns procedure routing and references credential requirements.
- Device pairing owns Focusa-client bearer pairing, not workload credentials.
- This specification owns provider-neutral credential requirements, operator autonomy policy, grants/leases, provider/consumer adapters, authentication challenges, and cross-surface credential control.

## Closure prohibition

Do not close for a password manager integration, `rbw get`, environment expansion, TOTP endpoint, browser autofill demo, Keychain storage, a secrets UI, service account, dynamic-secret provider, or one successful autonomous login. Closure requires project-scoped operator-configurable delegation, provider-neutral custody, fenced grants/leases, exact-target multi-surface injection, comprehensive challenge handling, no-default-plaintext exposure, broad autonomous mode, Cockpit/Menu Bar/Pi/Desktop/headless parity, revocation/rotation/incident/crash proof, secret-safe settlement, and repeated installed multi-provider/multi-workflow success.
