# Spec 135 Pi-native interaction proof

The interaction-mode contract is implemented by `MissionCanvasShell` and `MissionCanvasView` inside the current Pi terminal.

- Canvas mode mounts one Pi custom component in the current session.
- Terminal mode leaves stock Pi visible while canonical Focusa state remains active.
- Headless mode performs no UI mount.
- `/mission-canvas off` disposes the component and restores stock Pi without spawning a browser, webview, sidecar, or second host.
- Profile, activity, Work Surface, prompt-editor, queue, and sparse-state transitions are covered by the Mission Canvas Pi tests.
