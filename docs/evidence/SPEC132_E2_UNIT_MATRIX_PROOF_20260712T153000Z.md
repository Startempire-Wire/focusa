# Spec 132 E2 Rust unit/state matrix proof

Command:

```text
CC=/usr/bin/clang CXX=/usr/bin/clang++ RUSTFLAGS='-C linker=/usr/bin/clang' cargo test -p focusa-terminal-ui
```

Result: **46 passed, 0 failed**; doc-tests passed.

Coverage includes half-block mapping, deterministic continuity/matrix frames, occupancy bounds, exact palettes and ANSI mapping, sanitizer ANSI/credential/email/license redaction, legal/illegal phase transitions, monotonic progress, warning/recovery retention, verification truth, breakpoint selection, terminal guard idempotence/cancellation, deterministic fixed-seed initialization, and the <=700ms completion hold bound.

A real behavior defect was fixed: credentialed URLs were correctly userinfo-redacted, then incorrectly classified as emails because the redaction marker contains `@`. Email redaction now preserves `[REDACTED_CREDENTIALS]@` URL output while query-secret redaction remains active.
