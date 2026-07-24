# First-Run Flow

The Focusa menubar first-run flow is Mac-like, phone-mediated, and recovery-oriented. It must connect a new Mac without asking the operator to manually type daemon URLs, auth headers, or tokens.

## Primary path

1. Menubar opens `FirstRunWizard.svelte` when no saved connection exists.
2. Mac generates a local nonce and QR handoff offer.
3. Operator opens Focusa Connect on phone and scans the Mac offer.
4. VPS daemon approves the room and mints a device token.
5. Mac receives completion via callback/polling and stores the daemon URL/token.
6. Main app dispatches `focusa-connection-saved` and starts normal polling.

## Required UX states

- clear “Connect to Focusa” title
- visible QR handoff
- phone/Focusa Connect instruction
- five-minute TTL / refresh affordance
- callback polling status
- Mac Completion Payload fallback
- advanced manual settings only behind an explicit toggle
- copy/error fallback for troubleshooting
- no raw token display unless in completion payload fallback after operator action

## Safety boundaries

- First-run UI is not authority; daemon pairing routes mint/revoke tokens.
- Pairing follows `DEVICE_PAIRING_THREAT_MODEL.md`.
- Token storage belongs in OS/Keychain/Tauri storage via app runtime, not docs/chat.
- Failed pairing should surface recovery text, not trigger binary install/update assumptions.

## Post-pairing walkthrough

1. Confirm daemon health and displayed version.
2. Open Mission Canvas and verify the current project, Trajectory gap, Workpoint, Work Rail, and Work Surface scope.
3. Confirm Agent Card discovery reports the complete Pi tool, skill, and runbook inventory without loading every schema.
4. Use `focusa_tool_search`/`focusa_tool_describe` for one bounded action and verify the matching skill/runbook can be opened.
5. Start background work only through daemon-native Silent Sessions; show session/run/generation and receipt state.
6. Verify updater policy and rollback status; uninstall guidance must state that user data is preserved unless purge is explicit.
7. If context compacts or a worktree is resumed, verify canonical project scope and Workpoint continuation rather than transcript-tail inference.

## Proof

- Component: `apps/menubar/src/lib/components/FirstRunWizard.svelte`
- Static guard: `tests/first_run_flow_static_test.sh`
- Menubar check: `cd apps/menubar && bun run check`
