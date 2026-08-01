# Spec 135 Nontechnical Usability Dogfood

Scenario: an operator who does not know Focusa internals opens Mission Canvas, identifies current work, switches the visible activity, reads evidence, opens Controls, and returns to stock Pi.

Results:

1. **Orientation:** application frame says Focusa, exposes Profile and activity navigation, and labels canonical authority in the footer.
2. **Current work:** Active Pi Session and Work Rail use plain labels; internal IDs are secondary diagnostics only.
3. **Empty state:** absent queues, rails, and inspector sections disappear; Add Surface reports when no meaningful surface exists.
4. **Recovery:** bounded notifications state the failed action and retain the previous usable projection.
5. **Controls:** revision, digest, omissions, and operation availability are visible in one diagnostics dialog.
6. **Keyboard:** profile focus, Add Surface, dialog close, pane traversal, prompt send, and direct-manipulation alternatives are keyboard reachable.
7. **Exit:** Canvas OFF restores the stock Pi presentation and preserves drafts/session identity.

Outcome: pass for the scripted tasks. The UIAI screenshot comparison could not run because the engine disallowed loopback URLs; this limitation is recorded in the P10 evaluation evidence and is not represented as a visual pass.

Receipt: `receipt:spec135:p10:nontechnical-dogfood:v1`
