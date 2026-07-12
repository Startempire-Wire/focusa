# 132 E5 native failure repair proof

1. macOS failure repaired: `tests/132-e5-platform-matrix-runtime-test.sh` no longer requires ripgrep; the repository contract assertion uses portable `grep -REiq` and preserves the same Windows/ConPTY/TerminalGuard assertion.
2. Windows stack-overflow defect repaired: Windows preflight no longer uses recursive `which`/PATHEXT probing. `find_command` uses native `where.exe` under `cfg(windows)`, while Unix behavior remains unchanged. This keeps command discovery non-executing and stack-bounded.
3. Windows preflight now avoids all external command probing, Unix path writability checks, and Unix privilege commands under `cfg(windows)`; it reports explicit Windows capability limits instead of entering recursive hosted-agent shims.
4. Linux regression proof passed:

```text
CC=/usr/bin/clang CXX=/usr/bin/clang++ RUSTFLAGS='-C linker=/usr/bin/clang' cargo test -p focusa-cli --bin focusa commands::install::tests::target_auto_resolves_to_platform  PASS
tests/132-e5-platform-matrix-runtime-test.sh                                      PASS
```

The native Windows ConPTY and macOS jobs must rerun on their hosted runners; E5 remains open until both return green. The first rerun used the pre-fix binary and failed with `0xC00000FD`; this correction must be verified by the next native run. Release matrix and protected bootstrapper files were not changed.
