# Spec 132 F3 forbidden shortcut audit (partial, no long builds)

Timestamp: 2026-07-13T20:15:00Z
HEAD at audit start: `c3602c979b77d493087d7069e8e0e30bcdd9e17f`
Bead: `focusa-slxpz.6.3`

## Audit command

Focused implementation-surface scan:

```bash
rg -n "TODO|FIXME|unimplemented!\(|todo!\(|future work|Phase 2|placeholder|fake progress|spinner|focusa-tui" \
  crates/focusa-cli/src/commands/install.rs \
  crates/focusa-terminal-ui/src \
  tests/spec_install_animation_static_test.sh \
  tests/spec_install_animation_contract_test.sh \
  tests/spec_install_animation_fallback_static_test.sh \
  tests/spec_install_animation_security_static_test.sh \
  tests/spec_install_pi_integration_rust_static_test.sh
```

## Findings

Actionable stale wording found and fixed:

- `--no-animation` help text no longer says `animation/spinner`; it now says `terminal install animation and plain output`.
- Active installer/docs surfaces no longer contain the stale strings `future work, Phase 2.0` or `sc.exe (Phase 2.0)`.

Remaining matches are not shortcut/defer markers:

- `focusa-tui` appears as an installed binary asset/path, not as a spawned UI renderer.
- `Phase 2` appears as internal numbered install-stage comments or sanitizer pass comments, not as deferred product work.

## Gates rerun after the wording/code update

```text
PASS tests/spec_install_animation_fallback_static_test.sh
PASS tests/spec_focusa_112_install_cmd_static_test.sh
PASS rg spinner over touched installer policy/code surfaces
```

## Open F3 blockers

F3 cannot be closed because the full Definition-of-Done audit depends on:

1. E5 native platform/runtime proof.
2. E7 cargo fmt/clippy/test/release-target gates.
3. Full Phase E gate closure.
4. A final all-surface forbidden-shortcut audit at the ending commit.

## Status

Partial F3 audit complete for current implementation surfaces touched in this session. F3 remains open.
