# Spec 132 C7 Render Loop Proof — 2026-07-12

## Scope

`focusa-slxpz.3.7` — render-loop scheduling, accessibility behavior, and bounded renderer state.

## Implementation proof

- `crates/focusa-terminal-ui/src/install/renderer.rs` uses a 33 ms frame deadline, drops cosmetic late frames instead of queueing them, drains installer events in order, and holds the truthful completed frame for at most 700 ms.
- Completion and failure pause Matrix rain; reduced-motion mode does not tick animation state; plain fallback returns before alternate-screen art rendering.
- `crates/focusa-terminal-ui/src/install/canvas.rs` reuses the renderer-owned logical pixel buffer across frames and resize clears it deterministically.
- Ratatui `Terminal::draw` supplies buffered diff output; no raw animation escape sequences, blink, sound, or sleep-based progress fabrication are used.

## Executed evidence

```text
rustfmt --check crates/focusa-terminal-ui/src/install/canvas.rs crates/focusa-terminal-ui/src/install/renderer.rs
RUSTFLAGS='-C linker=clang' cargo check -p focusa-terminal-ui
RUSTFLAGS='-C linker=clang' cargo test -p focusa-terminal-ui
RUSTFLAGS='-C linker=clang' cargo clippy -p focusa-terminal-ui --all-targets --all-features -- -D warnings
```

Results: touched-file rustfmt check passed; focused check passed; 42 unit tests and 0 doctests passed; clippy passed with `-D warnings`. A full `cargo fmt --all -- --check` also ran and reported pre-existing formatting drift outside this task (including API/core/TUI files), so it was not used to rewrite unrelated work. The default linker was unavailable in this host (`cc: Permission denied`), so the equivalent installed `clang` linker was selected explicitly; no source or release artifacts were changed by that workaround.

## Release boundary

No build, release, deploy, or installer execution was performed. Phase A baseline evidence and the unrelated release workflow changes remain preserved.
