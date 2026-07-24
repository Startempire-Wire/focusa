# Focusa Release Requirement Trace Matrix

| Requirement ID | Implementation | Proof | Result |
|---|---|---|---|
| SPEC112-INSTALL | `scripts/install-focusa.sh`, `scripts/install-focusa.ps1`, installer/TUI crates | strict installer, animation, service, UI, failure/rollback gates | PASS |
| SPEC128-OTA | CLI transactional update/rollback, daemon planning API, release trust metadata | update policy, signature, checksum, rollback, updater static/runtime gates | PASS |
| SPEC130-CONTINUITY | Pi 0.81 compaction coordinator and bounded persistence | Pi lifecycle suite, 1M-event/10K-cycle soak, all rollover crash boundaries | PASS |
| SPEC133-SILENT | daemon control plane, runner, adapters, model safety, supervision, evidence and operator surfaces | Rust workspace, Clippy, Pi runtime, strict Spec suite; `docs/133-silent-sessions-final-release-proof.md` | PASS |
| SPEC135-CANVAS | Mission Canvas critical path, attachment-isolated browser contexts, portable exact resume | PRs #16, #27, #28 and Spec135 strict gates | PASS |
| SPEC141-AGENT | generated capability descriptors and tool documentation | generated-artifact drift checks and strict audit | PASS |
| LIVE-BOOTSTRAP | canonical Bash/PowerShell public bootstrapper | source/live SHA parity and `scripts/verify-bootstrapper-parity.sh` | PASS |
| CROSS-PLATFORM | Linux/macOS/Windows/offline install→OTA→rollback contracts | platform fixtures, dependency fault matrix, integrated customer-lifecycle gate | PASS |
| RELEASE-NOTES | previous-tag delta, merged PR ancestry, issues, contributors, known issues, integrity/rollback | `.github/workflows/release.yml`, `tests/release_tag_template_static_test.py` | PASS |

No requirement is deferred through placeholders, hints-only behavior, inferred authority, ambient model fallback, or an unverified support claim.
