# 132 E5 platform interaction matrix proof

Command:

```text
bash tests/132-e5-platform-matrix-runtime-test.sh
```

Result: PASS for applicable Linux runtime cases:

1. `CI=1 TERM=dumb NO_COLOR=1` JSON preflight.
2. `TERM=xterm NO_COLOR=1 FOCUSA_REDUCE_MOTION=1` JSON preflight.
3. `TERM=xterm FOCUSA_INSTALL_UI=plain` JSON preflight.

Each case used the executable Focusa binary, proved the read-only envelope, and rejected ANSI bytes on stdout. The repository contains the terminal guard and platform capability contract. Windows ConPTY cannot be truthfully executed on this Linux host; that portion remains delegated to the Windows CI terminal host and E5 remains in progress.
