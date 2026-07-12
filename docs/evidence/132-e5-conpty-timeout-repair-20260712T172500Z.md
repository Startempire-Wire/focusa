# 132 E5 ConPTY timeout repair proof

The Windows fixture hang was a harness defect: it waited indefinitely for the child and then awaited output EOF before closing the pseudo-console. The runner now:

1. Waits at most 60 seconds for the owned child.
2. Terminates only that child on timeout and throws a typed timeout.
3. Closes the parent input writer and `ClosePseudoConsole` immediately after normal child exit, before draining output.
4. Uses a bounded post-termination wait and preserves exit/output assertions.
5. Includes a runtime timeout regression using an intentionally long-lived owned `cmd.exe` child with a 1-second test timeout.

The Focusa JSON/ConPTY behavior is not masked; the fixture still requires the real executable, validates JSON schema/read-only state, checks no alternate screen, and requires durable output. The native matrix must be rerun after this fix.
