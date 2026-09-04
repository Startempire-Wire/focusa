# Focusa system-daemon installation contract

`focusa install --system-install` is the sole owner of Linux system daemon
promotion, systemd registration, activation, verification, and rollback.
`focusa update apply/rollback` reuse its lock, halt, process-identity, and
system-restart primitives for `/usr/local/bin/focusa-daemon`.
`scripts/install-daemon.sh` remains only as a backward-compatible argument
adapter for existing deployment automation; it delegates to the verified
`scripts/install-focusa.sh` bootstrap and performs no lifecycle mutation itself.

## Canonical surfaces

- Release set: `focusa`, `focusa-daemon`, `focusa-tui`, and
  `focusa-session-runner` from one immutable signed release.
- System binaries: `/usr/local/bin/<name>`.
- System unit: `/etc/systemd/system/focusa-daemon.service`.
- State and data root: `/usr/local/lib/focusa` through both `FOCUSA_HOME` and
  `FOCUSA_DATA_DIR`.
- Health: `FOCUSA_DAEMON_URL`, defaulting to
  `http://127.0.0.1:8787/v1/health`.
- Lock: `/run/lock/focusa-daemon-install.lock`.

SQLite databases, signed leases, node identity, project data, and evidence under
the state root are preservation-only. Installation never replaces the state
root with a source checkout and never deletes it during rollback.

## Transaction

The Rust installer executes this order:

1. Acquire the nonblocking system deployment lock.
2. Fail before dependency, Pi, local-install, or system mutation when
   `RefuseManualStart=yes` is active.
3. Inventory `systemctl`'s `MainPID` and `/proc` processes named
   `focusa-daemon`.
4. Reject an unmanaged process, a duplicate process, a missing active
   `MainPID`, or a `MainPID` whose executable is not
   `/usr/local/bin/focusa-daemon`. It never kills by process name.
5. Validate the exact signed release and all four canonical binaries.
6. Preserve the prior unit in a durable transaction-qualified rollback file,
   atomically render the canonical unit, and reload systemd.
7. Atomically promote the complete release set and verify every `--version`.
8. Enable/start or restart the canonical service.
9. Require exactly one systemd-owned daemon, exact health version, and a
   canonical valid response from `POST /v1/callgraphs/validate`.
10. Remove rollback files only after all gates pass.

Any failure after staging restores the prior binaries and unit in that order,
reloads systemd, and returns the service to its prior active/inactive state. A
rollback failure is reported with the retained rollback path; it is never
hidden as success.

## Compatibility adapter

Existing automation may continue to call:

```bash
bash scripts/install-daemon.sh \
  --binary /tmp/focusa-daemon-v0.9.188-x86_64-unknown-linux-musl \
  --expected-version 0.9.188 \
  --service-name focusa-daemon \
  --health-url http://127.0.0.1:8787/v1/health \
  --require-service
```

The adapter verifies that the local artifact name binds the requested version,
derives the exact immutable tag, and delegates a full signed system install. It
rejects noncanonical install/state roots, binary/service names, tag/version
mismatch, and `--no-verify`. `--no-restart` maps to Rust `--no-service`; it may
not be combined with `--require-service`.

## Operator controls

The installer does not remove, rewrite, or bypass systemd drop-ins. In
particular, an operator `RefuseManualStart=yes` drop-in is an absolute stop. A
human-authorized configuration migration must remove obsolete development
drop-ins separately before activation; artifact installation does not create
that authority.
