# First-Run Flow

> **Spec 152 release boundary:** The current menubar component was originally implemented as pairing-first. New evaluator/customer distribution is blocked until first run establishes an authority-issued signed entitlement before pairing and normal product execution. Pairing authenticates a device; it does not create a license.

The Focusa menubar first-run flow is Mac-like, phone-mediated, recovery-oriented, and entitlement-first. It must connect a new Mac without asking the operator to manually type daemon URLs, auth headers, raw license keys, or bearer tokens.

## Required release path

1. Menubar opens `FirstRunWizard.svelte` or its successor when no completed onboarding receipt exists.
2. Menubar verifies the installed app/daemon release identity and obtains recovery-safe daemon health.
3. The canonical entitlement service resolves an existing signed lease or starts a device-code flow.
4. The operator opens the verified authority origin, verifies the account/email, accepts current license/privacy terms, and separately chooses promotional-email consent.
5. The authority issues an Evaluation, paid, or developer lease bound to the selected products and registered node.
6. Menubar/daemon verifies signature, product grant, lease sequence, node binding, time bounds, features, and limits.
7. If the lease grants UIAI Engine, Focusa registers/verifies the UIAI instance and obtains a short-lived scoped child token. UIAI verifies its own product grant independently.
8. Only after entitlement is resolved, Mac generates a local nonce and QR pairing handoff offer.
9. Operator opens Focusa Connect on phone and scans the Mac offer.
10. VPS daemon approves the room and mints a device token that authenticates this paired client but cannot widen entitlement.
11. Mac receives completion via callback/polling and stores daemon/device credentials in the OS-protected store.
12. Main app dispatches the connection/onboarding completion event and starts normal entitled or recovery-only polling.
13. The bounded first-project/first-Workpoint walkthrough begins only when required Focusa grants are active.

## As-built migration note

The current pairing component may still implement the older sequence:

```text
pairing → connection saved → normal polling
```

That sequence is retained as current-code evidence, not approved release behavior. During migration it must terminate in recovery-only mode until a canonical entitlement is verified. A saved pairing token cannot bypass the entitlement step.

## Required UX states

- verify release / starting recovery service
- choose `Evaluate`, `Activate`, or `Manage/Purchase`
- device code and verified authority URL
- waiting for identity/email verification
- terms accepted / promotional consent recorded separately
- lease received / verifying signature and product grants
- evaluation remaining time and bounded usage posture
- paid/developer entitlement summary
- expired, revoked, invalid, offline-grace, and authority-unavailable recovery states
- UIAI included, locked, unavailable, or opted-out state
- clear “Connect to Focusa” pairing title only after entitlement resolution
- visible QR handoff
- phone/Focusa Connect instruction
- five-minute pairing TTL / refresh affordance
- callback polling status
- Mac Completion Payload fallback
- advanced manual settings only behind an explicit toggle
- copy/error fallback for troubleshooting
- no raw key/token/code display except a short user device code explicitly intended for entry
- locked-feature cards with safe manage-license action

## Safety boundaries

- First-run UI is not authority. The license authority issues leases; daemon pairing routes mint/revoke device tokens.
- A device/pairing/local API token authenticates a caller and never creates Evaluation, paid status, product grants, or features.
- The daemon may start without a valid lease only in recovery mode.
- Recovery mode allows health, license start/status/refresh/doctor, safe backup/export, repair, and uninstall; it denies project mutation, Workpoint/Evidence mutation, agent execution, Silent Sessions, protected workers, UIAI execution, and gated update apply.
- Pairing follows `DEVICE_PAIRING_THREAT_MODEL.md`.
- Lease, node-key, and token storage belongs in OS Keychain/Secure Enclave/Tauri storage or the platform security provider, not docs/chat.
- Failed pairing should surface recovery text, not trigger binary install/update assumptions.
- Failed entitlement must not delete or encrypt operator data.
- Test trust roots, fixture statuses, and developer bypasses are forbidden in customer release artifacts.

## Post-entitlement and post-pairing walkthrough

1. Confirm daemon health, displayed version, entitlement state, lease sequence/digest, and product grants without exposing sensitive values.
2. Confirm UIAI health and independent entitlement when selected; absence remains an explicit optional state.
3. Open Mission Canvas and verify the current project, Trajectory gap, Workpoint, Work Rail, and Work Surface scope.
4. Confirm Agent Card discovery reports the complete Pi tool, skill, and runbook inventory without loading every schema; locked tools remain discoverable with `license_feature` metadata.
5. Use `focusa_tool_search`/`focusa_tool_describe` for one bounded entitled action and verify the matching skill/runbook can be opened.
6. Start background work only through daemon-native Silent Sessions; show session/run/generation, limit reservation, and receipt state.
7. Verify updater policy and rollback status; update apply/unattended behavior must match signed features.
8. Verify uninstall guidance states that user data is preserved unless purge is explicit.
9. If context compacts or a worktree is resumed, verify canonical project scope and Workpoint continuation rather than transcript-tail inference.
10. For Evaluation, show remaining time/capacity and demonstrate paid activation without reinstall or loss of state.

## Completion receipt

First-run completion must bind:

- app/daemon/UIAI versions and compatible contract digests;
- entitlement state, lease id, lease sequence, product/feature digests, node id, and signature-verification result;
- device pairing receipt;
- optional UIAI child-token audience/expiry and independent status;
- project identity and first Workpoint when project onboarding is selected;
- data-preservation and recovery posture;
- redacted evidence references.

A connection-saved event alone is not completion.

## Proof

- Component: `apps/menubar/src/lib/components/FirstRunWizard.svelte`
- Existing static guard: `tests/first_run_flow_static_test.sh`
- Required entitlement/documentation gate: `tests/spec152_documentation_consistency_gate.py`
- Menubar check: `cd apps/menubar && bun run check`
- Normative contracts: Spec 152, Spec 150A, Spec 152A
