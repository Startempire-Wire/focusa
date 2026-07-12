# Spec 132 E3 deterministic golden frame proof

Command:

```text
CC=/usr/bin/clang CXX=/usr/bin/clang++ RUSTFLAGS='-C linker=/usr/bin/clang' cargo test -p focusa-terminal-ui deterministic_golden_frames_cover_required_sizes_and_modes
```

Result: PASS. Ratatui `TestBackend` renders fixed-seed frames and hashes every cell symbol/foreground/background. Reviewed fingerprints:

| Frame | Fingerprint |
|---|---:|
| 120x40 truecolor | 15614631799357408319 |
| 100x30 ANSI-256 | 17647664474310074821 |
| 80x24 truecolor compact | 9784246533569277816 |
| 80x24 monochrome | 167468927385124613 |
| 80x24 reduced motion | 167468927385124613 |
| 69x30 plain fallback | 15850258645686606196 |

The test uses no sleeps and fails on fingerprint drift; snapshots are not blindly regenerated. The plain fallback is rendered as an intentionally blank presentation frame while durable output remains outside the renderer.
