# Spec 132 — Focusa Installer Animated Terminal Experience

## Binding implementation specification: Hybrid AC — Matrix Core + Glow Base

**Status:** Binding production specification
**Requirement level:** Non-negotiable
**Repository:** `Startempire-Wire/focusa`
**Baseline commit:** `19e7049b7672b66ddbc0036c344c10024c19bfd7`
**Baseline workspace version:** `0.9.91-dev`
**Verified:** 2026-07-11
**Parent specification:** `docs/112-install-binary-architecture-spec.md`
**Required implementation document path:** `docs/132-focusa-installer-animated-terminal-experience-spec.md`
**Approved visual direction:** Hybrid AC — Matrix rain backdrop + assembling continuity core + luminous bottom infrastructure platform, using the approved high-frequency neon palette.

---

## 0. Binding language and precedence

The words **MUST**, **MUST NOT**, **REQUIRED**, **NON-NEGOTIABLE**, **SHALL**, and **SHALL NOT** are binding acceptance requirements.

The implementation is incomplete unless every requirement marked REQUIRED or NON-NEGOTIABLE is implemented and proven. An agent may not reinterpret a requirement as optional, future work, polish, a follow-up, a placeholder, or a “Phase 2” item.

Where sources disagree, apply this precedence:

1. Direct operator decisions embodied in this specification.
2. The stricter security, atomicity, rollback, licensing, scope, and release-gate requirement.
3. Current code and tests at the implementation branch’s actual HEAD.
4. Current documentation.
5. Historical specifications and comments.

This specification is an **additive binding extension of Spec 112**. It does not authorize weakening any existing installer behavior.

### 0.1 Current-head drift gate

Before changing code, the implementing agent MUST:

1. Record:

   * `git rev-parse HEAD`
   * the default remote branch
   * `git status --short`
   * workspace version from root `Cargo.toml`
2. Fetch the remote without deleting or overwriting local work.
3. Compare current HEAD to baseline commit `19e7049b7672b66ddbc0036c344c10024c19bfd7`.
4. Re-read every governing file listed in §2.
5. Create `docs/evidence/INSTALL_ANIMATION_BASELINE_<UTC_TIMESTAMP>.md` containing:

   * starting SHA;
   * remote SHA;
   * dirty-worktree status;
   * files changed since this baseline;
   * installer contract changes discovered;
   * how this specification was reconciled without weakening it.
6. Stop and preserve uncommitted operator work rather than resetting, stashing destructively, or overwriting it.
7. Update the drift evidence if HEAD moves again before merge.

A later HEAD may require integration changes, but it MUST NOT be used to omit the approved animation, fallbacks, tests, or safety behavior.

### 0.2 No unauthorized release or deployment

Implementing this specification does not authorize an agent to:

* create or move a release tag;
* publish a GitHub release;
* deploy a daemon;
* sync the public installer;
* modify the live install host;
* bypass the canonical full release pipeline.

Release or deployment occurs only after explicit operator instruction and through the repository’s canonical release, signing, notarization, parity, and deploy gates.

---

## 1. Required outcome

`focusa install` MUST provide a polished, truthful, terminal-native animated installation presentation when run in a compatible interactive terminal.

The selected production design is:

> **Hybrid AC — Matrix Core + Glow Base, High-Frequency Palette**

The live screen MUST combine:

1. a restrained multicolor Matrix-rain background;
2. an assembling central **Continuity Core** made from terminal block cells;
3. a luminous cyan/blue/magenta infrastructure platform beneath the core;
4. a truthful installation phase rail;
5. real download progress where byte totals exist;
6. clear warning, failure, rollback, and completion states;
7. terminal-safe responsive layouts and nonanimated fallbacks.

The animation MUST be implemented inside the Rust `focusa install` process. It is not a website animation, video, recording, browser experience, separate setup wizard, or invocation of the `focusa-tui` binary.

### 1.1 The central shape is not a replacement logo

The animated ring/hexagonal object shown in the approved visual is named the **Continuity Core**.

It MUST be treated as decorative install art representing infrastructure assembly. It MUST NOT be described, exported, registered, or substituted as a new Focusa logo.

The canonical wordmark remains `FOCUSA`, consistent with the current Mission Deck intro. The live installer header MUST use:

```text
FOCUSA INSTALL
Local-first mission cohesion for AI coding agents.
```

A smaller secondary descriptor MAY read:

```text
Agent continuity infrastructure
```

The concept image controls composition, energy, glow, movement, and palette. It does not authorize replacing canonical brand text or inventing a new trademark.

---

## 2. Governing repository surfaces

The agent MUST inspect and preserve the contracts of at least these current files:

### Installer and CLI

* `crates/focusa-cli/src/commands/install.rs`
* `crates/focusa-cli/src/commands/service.rs`
* `crates/focusa-cli/src/commands/uninstall.rs`
* `crates/focusa-cli/src/commands/upgrade.rs`
* `crates/focusa-cli/src/commands/mod.rs`
* `crates/focusa-cli/src/main.rs`
* `crates/focusa-cli/Cargo.toml`
* root `Cargo.toml`

### TUI and brand references

* `crates/focusa-tui/Cargo.toml`
* `crates/focusa-tui/src/views/intro.rs`
* `crates/focusa-tui/src/theme.rs`
* `docs/27-tui-spec.md`
* `docs/28-ratatui-component-tree.md`
* `docs/MISSION_CONTROL_TUI_DESIGN.md`
* `docs/evidence/SPEC_117_TUI_INTRO_SPLASH_PROOF_2026-07-06.md`

### Public bootstrappers and parity

* `scripts/install-focusa.sh`
* `scripts/install-focusa.ps1`
* `scripts/sync-install-bootstrapper.sh`
* `scripts/sync-install-bootstrapper-windows.sh`
* `scripts/verify-bootstrapper-parity.sh`

### Release and trust

* `.github/workflows/release.yml`
* current release/deploy workflows and watchdogs;
* signing and notarization tests;
* `scripts/create-dev-release-tag.sh`;
* current version-surface verification scripts.

### Specifications and current references

* `docs/112-install-binary-architecture-spec.md`
* `docs/112-install-binary-architecture-audit.md`
* `docs/128-focusa-over-the-air-auto-update-and-dev-mode-license-spec.md`
* `docs/current/INSTALLER_UPDATE_POLICY.md`
* `docs/current/UPGRADE_COMMAND.md`
* `docs/current/CLI_REFERENCE_CURRENT.md`
* `docs/current/TROUBLESHOOTING_CURRENT.md`
* `docs/current/PORTABILITY_AUDIT.md`
* `docs/current/VALIDATION_AND_RELEASE_PROOF.md`

### Existing guards

* `tests/spec_focusa_112_install_cmd_static_test.sh`
* `tests/spec_install_path_walkthrough_static_test.sh`
* current Spec 112 installer, release-matrix, codesign, bootstrapper-parity, uninstall, Pi-extension, OTA, and service-lifecycle tests.

Older documentation that still says the Rust installer is unimplemented MUST be corrected as part of this work. The agent MUST NOT follow a stale status banner over current executable code and tests.

---

## 3. Existing behavior that MUST survive unchanged

The animation is a presentation layer over the installer. It MUST NOT become the source of installation truth.

The following current contracts are protected:

1. The Rust `focusa install` subcommand remains the canonical orchestrator.
2. Bash and PowerShell remain thin security/bootstrap handoff surfaces.
3. Current flags remain accepted:

   * `--target`
   * `--channel`
   * `--dry-run`
   * `--preflight`
   * `--no-animation`
   * `--quiet`
   * `--assume-yes`
   * `--license-key`
   * `--eval`
   * `--accept-license`
   * `--no-service`
   * `--persist-path`
   * `--no-persist-path`
   * `--on-shell`
   * `--json`
   * `--github-repo`
4. `--persist-path` and `--no-persist-path` remain mutually exclusive.
5. The public bootstrapper’s accepted flags and Rust handoff flags remain aligned.
6. Bootstrap and core asset downloads remain staged rather than written directly over live binaries.
7. Existing install atomicity remains:

   * stash existing installation;
   * execute phases;
   * smoke-test the installed CLI;
   * rollback on failure;
   * remove stash only after success.
8. Checksum, signature, codesign, and notarization requirements remain authoritative.
9. macOS codesign verification remains after checksum verification and before user-facing success.
10. PATH persistence remains idempotent and bounded by the existing marker block.
11. Installed TUI/deck discovery continues to resolve the sibling binary under `~/.focusa/bin` without relying solely on PATH.
12. Benign `launchctl unload` replacement states MUST NOT appear as frightening failures.
13. Truthful uninstall reporting and exact retention behavior remain unchanged.
14. Windows x64, Windows ARM64, macOS Intel/Apple Silicon, Linux glibc, and Linux musl release-matrix compilation MUST remain protected.
15. The bundled Pi extension remains supported.
16. JSON contracts retain their existing keys, types, meanings, and single-document output behavior.
17. The required six-step first-install walkthrough remains a durable same-terminal result after the transient animation is removed.
18. Session-transfer, preload, receipt-preview, and explicit receipt-commit behavior added after earlier installer work MUST remain unaffected.
19. No secret, license key, raw authorization header, or unredacted sensitive value may enter the animation state.

---

## 4. Architectural implementation

### 4.1 Required shared library crate

Create:

```text
crates/focusa-terminal-ui/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── capabilities.rs
    ├── terminal_guard.rs
    ├── sanitize.rs
    └── install/
        ├── mod.rs
        ├── event.rs
        ├── state.rs
        ├── presenter.rs
        ├── renderer.rs
        ├── layout.rs
        ├── palette.rs
        ├── canvas.rs
        ├── continuity_core.rs
        ├── matrix_rain.rs
        ├── glow_base.rs
        └── completion.rs
```

This crate is REQUIRED. The implementation MUST NOT place hundreds of rendering lines directly into `install.rs`.

The crate MUST:

* be a library, not another executable;
* contain no HTTP client;
* contain no license validation;
* contain no release selection;
* mutate no installation files;
* start or stop no service;
* own no rollback decision;
* read no secret;
* receive sanitized events and render them;
* compile on every supported release target.

### 4.2 Workspace dependency alignment

Root `Cargo.toml` MUST:

1. add `crates/focusa-terminal-ui` to workspace members;
2. define a workspace path dependency for `focusa-terminal-ui`;
3. centralize the existing compatible Ratatui and Crossterm versions:

   * `ratatui = "0.30"`
   * `crossterm = "0.28"`
4. make both `focusa-tui` and `focusa-terminal-ui` consume the workspace versions;
5. add only dependencies required by the implementation.

`focusa-cli` MUST depend on `focusa-terminal-ui`. It MUST NOT depend on or spawn the `focusa-tui` binary to render installation.

### 4.3 Presenter boundary

The installer MUST emit domain-neutral presentation events through an interface equivalent to:

```rust
pub trait InstallEventSink: Send + Sync {
    fn emit(&self, event: InstallEvent);
}
```

Required presenter implementations:

```text
AnimatedPresenter
MonochromeAnimatedPresenter
ReducedMotionPresenter
PlainPresenter
SilentPresenter
```

Selection MUST occur once near the beginning of `focusa install`.

The install functions MUST NOT branch on colors, frame numbers, Matrix positions, terminal dimensions, or visual state. They emit events only.

The renderer MUST NOT call installer phase functions.

### 4.4 UI failure isolation

A failure to initialize, resize, draw, or update the animated UI MUST:

1. restore terminal state;
2. emit one sanitized warning;
3. switch to `PlainPresenter`;
4. allow the real installer to continue.

A UI failure MUST NOT trigger install rollback unless the real installation phase also failed.

### 4.5 Single final report

Refactor `execute_real_install()` to return a structured internal execution result rather than printing a success walkthrough before the smoke test.

The REQUIRED success order is:

1. real install phases succeed;
2. installed `focusa --version` smoke test succeeds;
3. any existing stash is removed;
4. presenter receives `InstallFinished`;
5. transient terminal UI exits and restores the invoking terminal;
6. human mode prints the permanent summary and existing six-step walkthrough;
7. JSON mode prints exactly one valid JSON document.

No “installed,” “complete,” “operational,” success card, success JSON, or six-step walkthrough may be emitted before the smoke-test gate passes.

---

## 5. Required install event contract

### 5.1 Stable phase identifiers

Implement these stable phase identifiers:

```rust
pub enum InstallPhase {
    InitializeEnvironment,
    DetectSystem,
    ValidateLicense,
    ResolveRelease,
    DownloadAssets,
    VerifyIntegrity,
    InstallBinaries,
    IntegratePi,
    RegisterService,
    PersistPath,
    RunHealthChecks,
    Finalize,
    Complete,
    Rollback,
}
```

Human labels are fixed:

| ID                      | Required label             |
| ----------------------- | -------------------------- |
| `InitializeEnvironment` | Initialize environment     |
| `DetectSystem`          | Detect system              |
| `ValidateLicense`       | Validate license           |
| `ResolveRelease`        | Resolve release            |
| `DownloadAssets`        | Download assets            |
| `VerifyIntegrity`       | Verify checksums and trust |
| `InstallBinaries`       | Install binaries           |
| `IntegratePi`           | Integrate Pi               |
| `RegisterService`       | Register service           |
| `PersistPath`           | Persist PATH               |
| `RunHealthChecks`       | Run health checks          |
| `Finalize`              | Finalize                   |
| `Complete`              | Complete                   |
| `Rollback`              | Roll back safely           |

These identifiers MUST NOT be repurposed for unrelated operations.

### 5.2 Events

Implement an event model with at least:

```rust
pub enum InstallEvent {
    PhaseStarted {
        phase: InstallPhase,
        message: String,
    },
    PhaseMessage {
        phase: InstallPhase,
        message: String,
    },
    AssetStarted {
        asset: String,
        total_bytes: Option<u64>,
    },
    AssetProgress {
        asset: String,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
    },
    AssetFinished {
        asset: String,
        downloaded_bytes: u64,
    },
    PhaseSucceeded {
        phase: InstallPhase,
        detail: Option<String>,
    },
    PhaseSkipped {
        phase: InstallPhase,
        reason: String,
    },
    PhaseWarning {
        phase: InstallPhase,
        message: String,
        recovery_hint: Option<String>,
    },
    PhaseFailed {
        phase: InstallPhase,
        message: String,
        recovery_hint: Option<String>,
    },
    RollbackStarted {
        reason: String,
    },
    RollbackSucceeded,
    RollbackFailed {
        message: String,
        recovery_hint: String,
    },
    InstallFinished {
        summary: InstallCompletionSummary,
    },
}
```

All strings entering this model MUST pass the sanitizer in §11.

### 5.3 Truthfulness rules

NON-NEGOTIABLE:

* A phase is never marked successful before its function returns success.
* A skipped phase uses `Skipped`, not `Succeeded`.
* A warning remains visible and is included in the completion summary.
* Missing verification metadata MUST NOT display as “verified.”
* A byte percentage is displayed only when a trustworthy total byte count exists.
* Unknown-length activity uses an indeterminate motion and no percentage.
* The decorative core assembly percentage is never printed as install completion percentage.
* No timer, sleep, random duration, or expected phase duration may fabricate progress.
* The final progress indicator reaches 100% only after smoke test and cleanup succeed.

### 5.4 Phase-to-current-code mapping

The implementation MUST preserve and expose these current operations:

| Required phase         | Current responsibility                                                      |
| ---------------------- | --------------------------------------------------------------------------- |
| Initialize environment | presenter/capability setup and existing atomic stash preparation            |
| Detect system          | target, OS, arch, shell, terminal, package manager, service manager         |
| Validate license       | `phase_license()` and existing registry authority                           |
| Resolve release        | separate release manifest/tag/target resolution from asset transfer         |
| Download assets        | streamed staged downloads for CLI/daemon/TUI                                |
| Verify integrity       | SHA256, cosign/release trust, and macOS codesign/notarization state         |
| Install binaries       | atomic promotion, permissions, and symlink placement                        |
| Integrate Pi           | current bundled Pi-extension behavior, migrated as specified below          |
| Register service       | current systemd/launchd/Windows service path, respecting `--no-service`     |
| Persist PATH           | existing marker-bounded idempotent shell changes                            |
| Run health checks      | installed binary version checks plus actual available service/health checks |
| Finalize               | report construction and stash cleanup                                       |
| Complete               | presentation-only stable final state after all gates                        |
| Rollback               | current rollback/cleanup result, not a decorative simulation                |

---

## 6. Mandatory migration of Pi integration into the Rust orchestrator

At the baseline, the Bash bootstrapper installs the bundled Focusa Pi extension before handing off to Rust. This conflicts with the canonical thin-bootstrapper architecture and prevents the Rust presentation from reporting that integration truthfully.

This specification therefore REQUIRES:

1. Keep release packaging of `focusa-pi-extension-<tag>.tar.gz`.
2. Move installation, extraction, replacement, dependency setup, and result reporting into Rust-owned install logic.
3. Remove runtime Pi-extension mutation logic from Bash after parity tests are updated.
4. Provide equivalent Windows behavior through the same Rust implementation.
5. Detect Pi without failing non-Pi installations.
6. Preserve the current best-effort product rule:

   * Pi absent → `Skipped`;
   * Pi present and integration succeeds → `Succeeded`;
   * Pi present but archive/dependency setup fails → `Warning` with exact recovery;
   * the core Focusa install MUST NOT falsely fail solely because optional Pi integration failed.
7. Never mark Pi as integrated merely because `pi` exists.
8. Verify the final extension directory and required entry files before reporting success.
9. Record the actual integration path in the final summary.
10. Never expose npm output directly inside the TUI without sanitization and line bounds.

This migration is part of this specification and MUST NOT be deferred.

---

## 7. Download progress and staging

To support truthful download progress:

1. Separate release resolution from asset transfer.
2. Stream response chunks to the existing staged `.download` path.
3. Emit `AssetProgress` from bytes actually written.
4. Preserve atomic promotion semantics.
5. Preserve executable permissions.
6. Preserve anti-rollback behavior.
7. Delete abandoned staged files on cancellation or failure.
8. Never render the asset URL if it contains credentials or sensitive query parameters.
9. The progress bar MUST use actual bytes:

   * `downloaded / total` when `Content-Length` is trusted;
   * byte count with indeterminate bar when total is absent.
10. Core installer behavior MUST remain correct when the presenter is silent.

The implementation MUST NOT regress to downloading the entire response into memory merely to animate it.

---

## 8. Visual system: Hybrid AC

### 8.1 Screen composition

The real installation screen contains one cohesive presentation, not the multi-panel concept poster.

Required vertical structure:

```text
┌──────────────────────────────────────────────────────────────┐
│ FOCUSA INSTALL                                               │
│ Local-first mission cohesion for AI coding agents.           │
│                                                              │
│        Matrix rain + assembling Continuity Core              │
│        luminous infrastructure platform beneath              │
│                                                              │
│ Current phase / real asset progress                          │
│ Phase rail or compact phase status                           │
│ Warning/recovery area when needed                            │
└──────────────────────────────────────────────────────────────┘
```

The current phase and failure information MUST remain readable over all animation.

### 8.2 Canonical high-frequency palette

Truecolor mode MUST use these semantic anchors:

| Token           |             RGB |       Hex | Purpose                        |
| --------------- | --------------: | --------: | ------------------------------ |
| `background`    |      `2, 3, 10` | `#02030A` | base screen                    |
| `text`          | `234, 247, 255` | `#EAF7FF` | primary readable text          |
| `muted`         | `120, 144, 168` | `#7890A8` | secondary text                 |
| `cyan`          |   `0, 229, 255` | `#00E5FF` | core energy and glow           |
| `electric_blue` |   `0, 136, 255` | `#0088FF` | structure and platform         |
| `violet`        |  `138, 92, 255` | `#8A5CFF` | core outer structure           |
| `magenta`       |  `255, 43, 214` | `#FF2BD6` | high-frequency accent          |
| `lime`          |   `57, 255, 20` | `#39FF14` | Matrix rain and success energy |
| `yellow`        |   `255, 230, 0` | `#FFE600` | sparse signal accent           |
| `orange`        |   `255, 138, 0` | `#FF8A00` | sparse assembly sparks         |
| `success`       |  `53, 255, 120` | `#35FF78` | completed phase                |
| `warning`       |  `255, 216, 74` | `#FFD84A` | warning state                  |
| `error`         |   `255, 51, 79` | `#FF334F` | failure and rollback alert     |
| `border`        |    `39, 58, 78` | `#273A4E` | subdued panel boundaries       |

Rules:

* Black remains dominant.
* Cyan, blue, violet, and magenta form the main core.
* Lime dominates Matrix rain but MUST NOT dominate foreground text.
* Yellow and orange occupy no more than 8% of lit art cells in steady state.
* Red is reserved for actual error state.
* High-frequency color does not authorize visual noise, illegible text, flashing, or rainbow cycling.

### 8.3 Terminal pixel technique

The block canvas MUST support truecolor half-block rendering:

* one terminal cell represents two vertical logical pixels;
* top pixel uses background color;
* bottom pixel uses foreground color;
* glyph is `▄`;
* solid single-pixel alternatives may use `▀`, `█`, `▓`, `▒`, or `░`;
* every line ends with a style reset through Ratatui/Crossterm state, not raw uncontrolled escape leakage.

No image protocol, sixel, Kitty graphics, iTerm image escape, browser canvas, or embedded raster is permitted.

### 8.4 Continuity Core

The Continuity Core MUST:

1. use a fixed deterministic logical mask;
2. fit inside 32 logical columns by 32 logical rows in standard mode;
3. begin as dispersed colored fragments;
4. assemble progressively as real phases complete;
5. stabilize into the approved luminous ring/core structure;
6. retain an open dark center;
7. include cyan/blue lower energy, violet/magenta outer structure, and sparse orange/yellow sparks;
8. never jitter after cells lock into place;
9. never obscure the phase rail;
10. stop active movement in the final stable frame.

The mask MUST live as data or a focused generator in `continuity_core.rs`, not as an unreadable monolithic ANSI string.

### 8.5 Matrix rain

Matrix rain is a background system, not the main content.

Required behavior:

* deterministic pseudorandom seed per run;
* injectable fixed seed for tests;
* columns spaced at least one terminal cell apart;
* tail length from 4–12 cells;
* speeds from 8–18 logical cells per second;
* colors drawn from dim lime, cyan, violet, magenta, and rare yellow;
* brightness attenuated behind text and phase rail;
* maximum active rain occupancy of 18% of available background cells;
* no full-screen white flashes;
* no ANSI blink modifier;
* no more than three high-contrast luminance transitions per second in a fixed region;
* pause or heavily slow during failure presentation;
* freeze into a stable low-energy field on completion.

Use width-stable glyphs only. Approved rain glyph set:

```text
0 1 2 3 4 5 6 7 8 9 A B C D E F : + * · │
```

Do not use combining characters, emoji, ambiguous-width glyphs, or random Unicode.

### 8.6 Glow base

The lower infrastructure platform MUST:

1. occupy approximately the bottom 20–28% of the art region in full mode;
2. use cyan/electric-blue horizontal layers;
3. include restrained magenta/violet reflections;
4. brighten from the center outward as phases succeed;
5. emit a slow pulse between 0.55–0.9 Hz while active;
6. avoid strobing;
7. shift to success-green accents only after final success;
8. shift to red with dim amber remnants on failure;
9. remain readable in 256-color and monochrome modes.

The platform is abstract infrastructure, not a literal 3D floor that requires perspective-perfect graphics.

### 8.7 Verification scan

During `VerifyIntegrity`, a bright cyan-to-magenta scan line MUST cross the core once per asset verification cycle.

It MUST:

* correspond to a real asset being checked;
* end green only on actual verification success;
* turn warning amber when verification metadata is unavailable;
* turn red and stop on verification failure.

### 8.8 Completion animation

After every real success gate passes:

1. the core locks into its stable mask;
2. the glow base emits one outward energy wave;
3. completed phase markers become green;
4. Matrix rain settles;
5. `FOCUSA INSTALL COMPLETE` appears;
6. the stable frame remains visible for **700 ms maximum**;
7. the alternate screen is restored;
8. the durable permanent summary and walkthrough print normally.

There is no artificial install delay beyond the maximum 700 ms final hold.

---

## 9. Phase rail and progress presentation

### 9.1 Status symbols

Status MUST never rely on color alone.

Required symbols:

```text
○ pending
◆ active
✓ succeeded
– skipped
! warning
✗ failed
↶ rollback active
```

### 9.2 Full phase rail

Full layouts MUST show all relevant phases. Conditional phases remain visible and become skipped when not applicable.

Example:

```text
✓ Detect system
✓ Validate license
✓ Resolve release
◆ Download assets
○ Verify checksums and trust
○ Install binaries
○ Integrate Pi
○ Register service
○ Persist PATH
○ Run health checks
```

### 9.3 Progress bar

The live progress bar MUST:

* show asset name;
* show bytes transferred;
* show percent only when total is known;
* update no more frequently than the render frame rate;
* preserve monotonic byte progress;
* never regress visually;
* never display 100% until the asset write completes;
* use terminal width safely.

No overall numeric percent is required. If an overall phase bar is included, it MUST be labeled **phase completion**, not download or time remaining.

### 9.4 Messages

Current-phase messages:

* maximum visible length: terminal width minus layout chrome;
* maximum retained visible history: 3 lines;
* ellipsized safely;
* stripped of control characters;
* never display stack traces in the animated screen.

Full error details remain available in durable plain output/logs after terminal restoration.

---

## 10. Responsive layout contract

The renderer MUST re-evaluate layout on every terminal-size change without losing phase state.

### 10.1 Full layout

Minimum: `120 × 36`

Required:

* centered header;
* 32×32 logical Continuity Core;
* full Matrix field;
* full glow base;
* phase rail at right;
* status/progress at bottom;
* warnings below progress.

### 10.2 Standard layout

Range: `90–119 × 28–35`

Required:

* smaller core;
* reduced Matrix density;
* compact right-side phase rail;
* one-line current message;
* full truthful progress.

### 10.3 Compact animated layout

Range: `70–89 × 22–27`

Required:

* core centered above progress;
* no separate right rail;
* phases collapsed to completed count plus current and next phase;
* reduced Matrix field;
* glow base retained in at least two rows.

### 10.4 Plain fallback

Any terminal below `70 × 22` MUST use `PlainPresenter`.

It MUST NOT attempt to clip the core, hide recovery text, or overflow the terminal.

### 10.5 Resize behavior

On resize:

* clear only through Ratatui’s render diff/alternate-screen redraw;
* preserve phase statuses and real download counters;
* recompute rain columns deterministically;
* avoid duplicate completion or phase events;
* degrade to plain if size falls below minimum;
* do not automatically re-enter alternate screen after degrading during the same run.

---

## 11. Sanitization and secret safety

Create one mandatory sanitization boundary in `sanitize.rs`.

Every dynamic string MUST:

1. remove ANSI CSI, OSC, DCS, APC, PM, and C1 control sequences;
2. remove control characters except permitted whitespace;
3. replace tabs/newlines according to the target field;
4. bound length;
5. avoid terminal escape injection;
6. redact detected license keys;
7. redact authorization headers;
8. redact sensitive query parameters;
9. never include raw customer email in the animated screen;
10. never include a full license-response payload.

Required tests include malicious strings such as:

```text
"\x1b[2Jforged success"
"\x1b]8;;https://evil.invalid\x07click\x1b]8;;\x07"
"focusa_live_super_secret"
"Authorization: Bearer secret"
"normal\n✗ fake failure"
```

The renderer must show harmless sanitized text only.

---

## 12. Terminal capability and fallback matrix

### 12.1 Output streams

* Animated UI renders to `stderr`.
* JSON remains on `stdout`.
* The renderer MUST NOT corrupt stdout.
* Durable human summary may use normal stdout after the transient UI exits.
* Logs and real errors remain durable.

### 12.2 Capability detection

Use `std::io::IsTerminal` on `stderr`.

Do not require stdin to be a TTY because the public `curl | bash` flow may leave stdin piped or closed while stderr remains interactive.

Full animation requires:

* `stderr` is a terminal;
* terminal size is at least `70 × 22`;
* `TERM` is not empty or `dumb`;
* not CI;
* not `--json`;
* not `--quiet`;
* not `--no-animation`;
* `FOCUSA_INSTALL_UI` is not `plain`.

### 12.3 Renderer modes

Implement:

```rust
pub enum InstallRendererMode {
    TrueColorAnimated,
    Ansi256Animated,
    MonochromeAnimated,
    ReducedMotion,
    Plain,
    Silent,
}
```

Selection rules:

| Condition                                  | Required mode                  |
| ------------------------------------------ | ------------------------------ |
| `--json`                                   | `Silent`                       |
| `--quiet`                                  | `Silent` except durable errors |
| `--no-animation`                           | `Plain`                        |
| CI                                         | `Plain`                        |
| non-TTY stderr                             | `Plain`                        |
| `TERM=dumb`                                | `Plain`                        |
| too-small terminal                         | `Plain`                        |
| `NO_COLOR` or `CLICOLOR=0` on suitable TTY | `MonochromeAnimated`           |
| reduced-motion env on suitable TTY         | `ReducedMotion`                |
| truecolor-capable TTY                      | `TrueColorAnimated`            |
| 256-color TTY                              | `Ansi256Animated`              |
| otherwise suitable TTY                     | `MonochromeAnimated`           |

`NO_COLOR` disables color, not motion. The current preflight behavior that disables all animation solely because `NO_COLOR` is present MUST be updated.

### 12.4 Supported environment controls

Implement and document:

```text
FOCUSA_INSTALL_UI=auto|full|mono|reduced|plain
FOCUSA_INSTALL_SEED=<u64>
FOCUSA_REDUCE_MOTION=0|1
```

Rules:

* default is `auto`;
* `full` may request color but cannot override `--json`, `--quiet`, non-TTY, CI, `TERM=dumb`, or minimum-size safety;
* invalid values fail early in preflight with an actionable error and no mutation;
* seed is for deterministic diagnostics/tests, not security;
* environment values must not appear in machine JSON unless an additive schema revision is explicitly approved.

### 12.5 Backward-compatible preflight fields

Keep existing `TerminalUxPreflight` fields and add, without renaming/removing current keys:

```text
renderer_mode
color_depth
minimum_size_met
reduced_motion
stderr_is_terminal
```

`intro_animation_enabled` continues to mean animated rendering is enabled. Under `NO_COLOR`, it may be true with `renderer_mode=monochrome`.

---

## 13. Terminal lifecycle and cancellation

### 13.1 Alternate screen

Interactive animated modes MUST:

1. use a `CrosstermBackend<Stderr>`;
2. enter the alternate screen;
3. hide the cursor;
4. avoid raw mode unless a proven requirement exists;
5. render through Ratatui;
6. restore cursor and leave alternate screen on every handled exit.

The animated screen is transient. It is not the persistent six-step walkthrough prohibited by current Spec 112.

### 13.2 RAII guard

`TerminalGuard` MUST restore terminal state from `Drop`.

It MUST be safe when:

* initialization partially succeeds;
* drawing fails;
* a phase returns error;
* a panic occurs;
* Ctrl+C occurs;
* SIGTERM occurs on Unix;
* presenter is replaced by plain fallback.

Restoration MUST be idempotent.

### 13.3 Panic and signal behavior

Install UI setup MUST install a scoped panic hook that restores terminal state before delegating to the prior hook.

Handle:

* Ctrl+C on all supported systems;
* SIGTERM on Unix;
* terminal resize;
* renderer channel closure.

SIGKILL cannot be handled and is excluded.

### 13.4 Cancellation semantics

Use a cancellation primitive shared by the orchestrator and presentation task.

On cancellation:

1. stop accepting new mutating phases;
2. close the animated renderer;
3. restore the terminal immediately;
4. delete incomplete staged downloads;
5. invoke the existing appropriate rollback/clean-state path;
6. print a durable cancellation result;
7. exit nonzero;
8. never print completion.

The animation MUST NOT swallow Ctrl+C.

---

## 14. Error and rollback presentation

When a real phase fails:

1. set that phase to `✗`;
2. stop or heavily slow Matrix rain;
3. recolor the core and platform to red with limited amber;
4. display the sanitized failure reason;
5. show `↶ Rolling back safely` while rollback is active;
6. distinguish:

   * rollback succeeded;
   * rollback failed;
   * no prior install existed;
7. display one actionable recovery hint;
8. restore the terminal;
9. print the full durable error, recovery hint, and log/evidence path;
10. preserve the original nonzero exit class.

Never show “rolled back safely” unless rollback or clean-state cleanup actually succeeded.

Benign launchd replacement states are not errors and must not trigger the red error state.

---

## 15. Completion summary

After success, print a durable summary containing real values:

```text
FOCUSA INSTALL COMPLETE

Version:          <actual installed version>
Target:           <actual target triple>
Channel:          <actual channel>
Install root:     <actual path>
CLI:              <actual path>
Daemon:           <actual path and health/status>
TUI:              <actual path>
Service:          <registered | skipped | warning>
PATH:             <persisted | already present | skipped | warning>
Pi integration:   <integrated path | not detected | warning>
Integrity:        <verified status and warnings>
Atomicity:        <fresh install | prior install replaced and stash cleared>
```

Then print the existing structured six-step first-install walkthrough.

Requirements:

* no hard-coded mock version;
* no hard-coded `/usr/local/bin` if actual path differs;
* no claim that daemon is operational unless checked;
* warnings remain visible;
* JSON mode emits one document and no decorative text.

---

## 16. Frame rate, rendering, and performance

### 16.1 Timing

* target: 30 FPS;
* frame interval: approximately 33 ms;
* drop late frames rather than queueing them;
* no 60 FPS requirement;
* animation state is time-based, not dependent on exact frame count;
* phase events are never dropped;
* cosmetic frame ticks may be dropped.

### 16.2 Allocation and diffing

The renderer MUST:

* use Ratatui’s buffered diff rendering;
* preallocate core and rain buffers;
* avoid allocating one `String` per cell per frame;
* reuse layouts when size is unchanged;
* cap rain columns based on width;
* stop ticking after final frame or plain fallback;
* avoid blocking install network/filesystem work on a slow terminal.

### 16.3 Performance budgets

At `120 × 40` on a normal modern terminal:

* renderer state target: under 8 MiB;
* sustained renderer CPU target: under 5% of one modern CPU core;
* no unbounded memory growth;
* no terminal output flood after diffing;
* no observable slowdown to download or verification work.

A headless render benchmark and evidence must be added.

---

## 17. Accessibility and operator safety

NON-NEGOTIABLE:

* status is conveyed with symbols and text, not color alone;
* no ANSI blink;
* no rapid full-screen flashing;
* no sound;
* no seizure-risk strobing;
* reduced-motion mode freezes rain and uses phase-triggered discrete updates;
* monochrome mode retains the core, base, phase rail, and progress hierarchy;
* plain mode remains complete and actionable;
* error text is copyable after alternate-screen restoration;
* the user never has to interact with the animation to finish installation.

---

## 18. Exact implementation sequence

The implementing agent MUST execute in this order:

### Phase A — Baseline and contract proof

1. Complete current-head drift gate.
2. Run existing installer/static tests before changes.
3. Save baseline outputs.
4. Confirm public bootstrapper parity state.
5. Confirm release freeze/authorization state.
6. Document all pre-existing failures; do not claim them as caused by this work.

### Phase B — Shared terminal library

1. Add workspace crate and aligned dependencies.
2. Implement sanitizer.
3. Implement capability detection.
4. Implement terminal guard.
5. Implement events/state machine.
6. Implement plain and silent presenters first.
7. Add unit tests before animated integration.

### Phase C — Visual engine

1. Implement block canvas.
2. Implement fixed Continuity Core mask.
3. Implement high-frequency palette.
4. Implement Matrix rain with deterministic seed.
5. Implement glow base.
6. Implement phase rail/progress.
7. Implement responsive layouts.
8. Implement truecolor, 256-color, monochrome, and reduced-motion renderers.
9. Add snapshots/golden frames.

### Phase D — Installer integration

1. Refactor install phase output into events.
2. Separate release resolution and streamed download.
3. Preserve staged promotion and verification order.
4. Migrate Pi-extension mutation to Rust.
5. Move final report/walkthrough after smoke test and stash cleanup.
6. Wire cancellation and rollback events.
7. Preserve JSON output.
8. Remove superseded direct `println!/eprintln!` phase noise or route it through the presenter, except durable logs.

### Phase E — Cross-platform and failure proof

1. macOS interactive install.
2. Linux interactive install.
3. Windows ConPTY install.
4. non-TTY/plain path.
5. JSON path.
6. CI path.
7. `TERM=dumb`.
8. `NO_COLOR`.
9. reduced motion.
10. resize.
11. cancellation.
12. checksum failure.
13. codesign/notarization failure.
14. service warning.
15. Pi present/absent/failure.
16. upgrade with stash.
17. clean install failure cleanup.

### Phase F — Docs and final gates

1. Update all docs in §22.
2. Run all tests in §21.
3. Capture evidence.
4. Ensure no TODO/placeholder/deferred requirement remains.
5. Stop before release/deploy unless explicitly authorized.

Skipping or reordering a phase requires a written reason in evidence and may not weaken acceptance.

---

## 19. Mandatory tests

### 19.1 New test files

Create at minimum:

```text
tests/spec_install_animation_static_test.sh
tests/spec_install_animation_contract_test.sh
tests/spec_install_animation_fallback_static_test.sh
tests/spec_install_animation_security_static_test.sh
tests/spec_install_pi_integration_rust_static_test.sh
```

Add Rust unit/integration tests under the new crate and CLI.

### 19.2 Unit tests

Required:

* half-block top/background and bottom/foreground mapping;
* full-block edge cases;
* deterministic core mask;
* deterministic Matrix frame from fixed seed;
* rain occupancy cap;
* color palette values;
* ANSI-256 mapping;
* monochrome mapping;
* sanitizer escape removal;
* secret redaction;
* legal phase transition table;
* illegal phase regression rejection;
* progress monotonicity;
* unknown-length download behavior;
* warning retention;
* layout selection at every breakpoint;
* terminal guard idempotence;
* completion hold not above 700 ms;
* no success before smoke-test event;
* Pi integration skipped/warning/success truthfulness.

### 19.3 Snapshot/golden tests

Snapshots MUST cover at least:

```text
120x40 truecolor initializing
120x40 truecolor downloading at known progress
120x40 truecolor verifying
120x40 truecolor warning
120x40 truecolor failure and rollback
120x40 truecolor complete
100x30 ANSI-256 active
80x24 compact active
80x24 monochrome
80x24 reduced motion
plain fallback
```

Snapshots must be deterministic and reviewed. Updating all snapshots blindly is forbidden.

### 19.4 PTY/ConPTY integration tests

Required:

* alternate screen entered and exited;
* cursor restored;
* no ANSI on stdout JSON;
* Ctrl+C restores terminal;
* resize preserves state;
* renderer failure falls back to plain;
* non-TTY never enters alternate screen;
* `NO_COLOR` contains no truecolor/256-color sequences;
* plain output contains every failed phase and recovery hint;
* completion summary appears only after smoke-test proof.

Use a portable PTY approach. Windows CI MUST include ConPTY coverage or an equivalent automated terminal-host proof; Windows cannot be deferred to compile-only acceptance.

### 19.5 Existing guards to extend

Extend without weakening:

* `tests/spec_focusa_112_install_cmd_static_test.sh`
* `tests/spec_install_path_walkthrough_static_test.sh`
* release matrix guards;
* bootstrapper parity guards;
* codesign/notarization guards;
* Pi-extension packaging/install guards;
* uninstall and service lifecycle guards;
* OTA/version-surface guards.

### 19.6 No sleep-based flaky tests

Tests MUST use injected clocks/manual ticks where possible.

A test MUST NOT rely on “sleep two seconds and hope the animation reached frame N.”

---

## 20. Required acceptance transcripts

### 20.1 Interactive truecolor success

Must prove:

* animation enters;
* Matrix/core/base visible;
* phases advance truthfully;
* real asset progress appears;
* verification scan corresponds to actual verification;
* final success occurs only after smoke test;
* terminal restores;
* permanent summary and six-step walkthrough remain.

### 20.2 JSON

Command equivalent:

```text
focusa install --json ...
```

Must prove:

* no alternate screen;
* no ANSI;
* no spinner;
* one valid JSON document;
* existing keys/types preserved;
* exit code truthful.

### 20.3 NO_COLOR

Must prove:

* animated monochrome mode on suitable TTY;
* no color escape sequences;
* same phase information;
* same error/recovery semantics.

### 20.4 Pi present

Must prove:

* Pi detected;
* matching extension archive resolved;
* extension installed by Rust;
* dependencies handled truthfully;
* final integration path verified;
* core install survives optional integration warning.

### 20.5 Integrity failure

Must prove:

* failed asset marked red;
* completion never shown;
* rollback state shown truthfully;
* alternate screen restored;
* durable mismatch details and recovery printed;
* exit code remains failure.

### 20.6 Cancellation

Must prove:

* Ctrl+C does not hang;
* staged download removed;
* previous install restored where applicable;
* cursor/screen restored;
* no completion output.

---

## 21. Required validation commands and gates

The final implementation evidence MUST include successful results for all applicable commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
bash tests/spec_focusa_112_install_cmd_static_test.sh
bash tests/spec_install_path_walkthrough_static_test.sh
bash tests/spec_install_animation_static_test.sh
bash tests/spec_install_animation_contract_test.sh
bash tests/spec_install_animation_fallback_static_test.sh
bash tests/spec_install_animation_security_static_test.sh
bash tests/spec_install_pi_integration_rust_static_test.sh
bash scripts/verify-bootstrapper-parity.sh
```

Also run:

* current release-matrix static test;
* current macOS codesign/notarization static test;
* current Pi-extension package test;
* current uninstall tests;
* current service lifecycle tests;
* current OTA/update tests;
* current version-surface verifier;
* Pi-extension typecheck/lint/tests after integration migration;
* target builds for every release target in current `.github/workflows/release.yml`.

No failing gate may be hidden behind `|| true`, ignored, marked flaky without proof, or removed to obtain green CI.

---

## 22. Documentation updates

Implementation is incomplete until these are updated:

1. Add this specification at:

   * `docs/132-focusa-installer-animated-terminal-experience-spec.md`
2. Add a binding addendum link from:

   * `docs/112-install-binary-architecture-spec.md`
3. Correct stale implementation status in Spec 112 without rewriting historical intent.
4. Update:

   * `docs/current/CLI_REFERENCE_CURRENT.md`
   * `docs/current/INSTALLER_UPDATE_POLICY.md`
   * `docs/current/TROUBLESHOOTING_CURRENT.md`
   * `docs/current/PORTABILITY_AUDIT.md`
   * `docs/current/VALIDATION_AND_RELEASE_PROOF.md`
   * `docs/128-focusa-over-the-air-auto-update-and-dev-mode-license-spec.md`
5. Document:

   * renderer selection;
   * environment controls;
   * `NO_COLOR`;
   * reduced motion;
   * non-TTY behavior;
   * JSON behavior;
   * minimum terminal size;
   * recovery from a damaged terminal;
   * Pi integration migration;
   * terminal safety and secret redaction.
6. Add evidence:

   * baseline/drift report;
   * screenshot or text-frame captures for all required visual states;
   * PTY/ConPTY proof;
   * CPU/memory benchmark;
   * macOS/Linux/Windows proof;
   * final gate transcript.

Docs MUST describe actual implemented behavior. Concept mock values and placeholder versions are forbidden.

---

## 23. Forbidden shortcuts

An implementation fails this specification if it does any of the following:

* substitutes a spinner or ordinary progress bar for the approved block animation;
* renders only a static ASCII logo;
* implements the effect on the website rather than in core software;
* starts `focusa-tui` as the installer UI;
* moves installer business logic into the renderer;
* puts the full renderer in Bash or PowerShell;
* uses fake progress;
* uses fixed demo timing instead of real events;
* claims verification when verification was skipped or warned;
* prints success before the smoke test;
* emits multiple JSON documents;
* writes ANSI to JSON stdout;
* removes the existing walkthrough;
* replaces the canonical `FOCUSA` wordmark;
* calls the Continuity Core a new logo;
* omits monochrome, reduced-motion, plain, CI, or non-TTY behavior;
* ignores terminal resizing;
* leaves the cursor hidden;
* leaves the alternate screen active after failure;
* uses uncontrolled raw ANSI strings from external errors;
* displays license keys, customer email, or authorization values;
* directly overwrites live binaries while downloading;
* regresses staged asset handling;
* regresses PATH idempotency;
* regresses sibling TUI discovery;
* moves Pi integration back to a manual post-install step;
* treats optional Pi integration warning as core install success without warning;
* treats benign launchd replacement as failure;
* bypasses signing/notarization;
* drops Windows ARM64 or Linux musl compilation;
* adds `TODO`, `FIXME`, `unimplemented!()`, `todo!()`, “future work,” “Phase 2,” or placeholder text for a required item;
* deletes or weakens a test to make implementation pass;
* publishes or deploys without explicit operator authorization.

---

## 24. Definition of done

This specification is complete only when all statements below are true:

* [ ] Current-head drift evidence exists.
* [ ] New shared terminal UI crate exists and is in the workspace.
* [ ] CLI and TUI consume aligned Ratatui/Crossterm versions.
* [ ] Installer emits typed events independent of rendering.
* [ ] Full truecolor Hybrid AC animation exists.
* [ ] Matrix rain meets density, glyph, speed, and safety requirements.
* [ ] Continuity Core uses deterministic block-cell assembly.
* [ ] Glow base meets approved high-frequency design.
* [ ] Canonical `FOCUSA` branding is preserved.
* [ ] Truecolor, ANSI-256, monochrome, reduced-motion, plain, and silent modes exist.
* [ ] `NO_COLOR` produces monochrome animation rather than disabling all motion.
* [ ] All responsive breakpoints work.
* [ ] Resize is safe.
* [ ] Alternate screen and cursor always restore.
* [ ] Ctrl+C/SIGTERM cancellation is safe.
* [ ] Renderer failure falls back without failing installation.
* [ ] Download progress is byte-truthful and streamed.
* [ ] Staged download and atomic promotion remain.
* [ ] Integrity/codesign/notarization states are truthful.
* [ ] No success prints before smoke test and stash cleanup.
* [ ] Permanent summary contains real values.
* [ ] Existing six-step walkthrough remains.
* [ ] JSON emits one clean document with compatible contract.
* [ ] Pi integration is Rust-owned and cross-platform.
* [ ] Pi absent/success/warning states are truthful.
* [ ] Secret sanitization tests pass.
* [ ] Snapshot tests pass.
* [ ] PTY and ConPTY tests pass.
* [ ] Windows ARM64 and Linux musl gates pass.
* [ ] Existing installer, uninstall, service, OTA, and release gates pass.
* [ ] Performance evidence meets budget.
* [ ] Required docs and evidence are current.
* [ ] No required work is deferred.
* [ ] No release/deploy occurred without authorization.

An agent may not close the implementation bead, issue, PR, or task until every box is proven or the operator explicitly changes this specification.

---

## 25. Required agent completion report

The implementing agent’s final report MUST contain:

1. starting and ending commit SHAs;
2. exact files created and modified;
3. architecture summary;
4. visual behavior summary;
5. compatibility/fallback matrix;
6. security and sanitization proof;
7. install phase-to-event mapping;
8. Pi integration migration proof;
9. terminal restoration and cancellation proof;
10. tests run with pass/fail counts;
11. target builds completed;
12. performance measurements;
13. documentation/evidence paths;
14. known limitations.

“Implemented,” “looks good,” “tests pass,” or a screenshot alone is not an acceptable completion report.

Known limitations may describe external constraints only. They may not be used to excuse a missing non-negotiable requirement.
