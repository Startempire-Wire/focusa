# Spec128 Pre-license macOS Beta Bootstrap Proof — 2026-07-23

## Durable business boundary

Paid Apple Developer membership is not required for demos, pilots, first customers, or trusted beta OTA before Focusa has revenue. `beta_ad_hoc` is explicit and unnotarized; `production_notarized` remains fail-closed on complete Apple credentials.

## Mac bootstrap E2E

Isolated test on the operator Mac used:

- a synthetic `Focusa.app` with bundle id `com.focusa.menubar`;
- real macOS ad-hoc `codesign` signing and strict verification;
- a synthetic Tauri `latest.json` platform entry and `.app.tar.gz` release archive;
- the real `scripts/install-focusa-menubar-beta.sh`;
- explicit `FOCUSA_BETA_ACCEPT=1` test consent;
- a temporary destination outside the installed Focusa app.

Verified flow:

1. HTTPS release URL allowlist parsing;
2. archive extraction;
3. strict bundle signature and identifier verification;
4. post-consent quarantine removal;
5. no-sudo user installation;
6. application launch/process proof;
7. final installed bundle verification.

Result: `MAC_BETA_BOOTSTRAP_E2E=PASS`.

## Automatic update trust

- Tauri updater private/public key match proof remains mandatory.
- Tagged beta release creates signed updater artifacts and `latest.json`.
- Both built and archived beta apps must report `Signature=adhoc` and pass strict verification.
- Menubar Settings explicitly states that beta is Tauri-signed but not Apple-notarized.
- Production mode requires Developer ID signing plus app and DMG notarization/stapling.
