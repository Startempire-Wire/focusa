# 132 E5 ConPTY timeout repair proof

The Windows fixture hang was a harness defect: it waited indefinitely for the child and then awaited output EOF before closing the pseudo-console. The runner now:

1. Waits at most 60 seconds for the owned child.
2. Terminates only that child on timeout and throws a typed timeout.
3. Closes the parent input writer and `ClosePseudoConsole` immediately after normal child exit, before draining output.
4. Uses a bounded post-termination wait and preserves exit/output assertions.
5. Includes a runtime timeout regression using an intentionally long-lived owned `cmd.exe` child with a 1-second test timeout.

The Focusa JSON/ConPTY behavior is not masked; the fixture still requires the real executable, validates JSON schema/read-only state, checks no alternate screen, and requires durable output. A subsequent native run exposed a separate fixture launch defect (`0xC0000142`): the child was launched with a relative executable and inherited working directory. The runner now passes the absolute executable path and its directory explicitly to `CreateProcess`. The next native run isolated an earlier fixture launch defect before Focusa execution: the generic `cmd.exe` probe could not be created. The probe now uses explicit `%WINDIR%\System32\cmd.exe`, and Win32 error codes are surfaced. Focusa has not been reclassified until the probe and Focusa outputs are separately observed. The ConPTY attribute binding was corrected to pass the HPCON value directly as `lpValue` with `IntPtr.Size`, matching the native contract; the process-attribute allocation was removed.
