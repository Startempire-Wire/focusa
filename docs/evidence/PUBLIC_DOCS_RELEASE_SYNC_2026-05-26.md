# Public Docs Release Sync — 2026-05-26

## Scope

Public docs were checked against the current Focusa runtime snapshot after the UIAI browser diagnostics/evidence integration and project-card/session-transfer changes.

## Current repo heads

- Focusa: `40b362f feat: update project card and session transfer`
- Public docs/UIAI integration guidance: `15d5ded docs: update uiai focusa integration guidance`
- UIAI Engine companion docs: `63f06fb docs: document focusa browser evidence contract`

## Docs verified current

- `docs/README.md`
- `docs/current/CURRENT_RUNTIME_STATUS.md`
- `docs/current/VALIDATION_AND_RELEASE_PROOF.md`
- `docs/current/UIAI_BROWSER_DIAGNOSTICS_FOCUSA_INTEGRATION_SPEC.md`
- `docs/current/TAURI_MENUBAR_UP_TO_SPEED_SPEC.md`
- `docs/focusa-tools/README.md`
- `docs/focusa-tools/tools/focusa_browser_diagnostics_intake.md`
- `docs/focusa-tools/tools/focusa_evidence_capture.md`
- Focusa Pi skill docs under `.pi/skills/` and `apps/pi-extension/skills/`

## Secret/publication audit

Guardian was run before publication checks. The Guardian daemon was offline, but `guardian scan /home/wirebot/focusa` completed and reported mostly dependency/build/token-named false positives plus ignored local data artifacts. Tracked-public secret regex scan found no live credentials.

Tracked-public scan command:

```bash
git grep -n -I -E '(sk-or-v1-[a-f0-9]{64}|ghp_[A-Za-z0-9_]{36,}|github_pat_[A-Za-z0-9_]{82,}|AKIA[0-9A-Z]{16}|xox[baprs]-[A-Za-z0-9-]+|-----BEGIN (RSA |OPENSSH |EC |DSA )?PRIVATE KEY-----)' -- . ':!docs/evidence/DOCS_SECRET_AUDIT_2026-04-28.md'
```

Result: no matches.

## Runtime and app proof

- `cargo test --workspace --all-targets` passed.
- `cargo build --release --bins` passed.
- Installed `/usr/local/bin/focusa`, `/usr/local/bin/focusa-daemon`, and `/usr/local/bin/focusa-tui`.
- `focusa-daemon` restarted and `/v1/health` returned `ok=true`.
- Menubar web app proof passed: `npm run check` and `npm run build` under `apps/menubar`.
- Native Tauri bundle proof is host-bound: AlmaLinux exposes `glib-2.0` 2.56.4, while `glib-sys` requires `glib-2.0 >= 2.70`; build should run on Mac or a newer GTK/GLib Linux builder.
- Scoped UIAI browser stress produced `/tmp/uiai-focusa-scope-verify.json` with `focusa_evidence` including `workpoint_id`, `continuity_id`, `project_root`, and `evidence_ref`.

## Publication status

Docs use current snapshot language, not final-product claims. Older design docs remain present as historical/design-direction docs and the current index points operators to `docs/current/*` for runtime behavior.
