# Presentation Command Manifest

Status: implemented and browser-accepted

`src/lib/shell/command-manifest.ts` is the versioned semantic source for Focusa Desktop's **Find or do** palette. Every command is explicitly marked `presentation-only`.

Supported actions:

- navigate to a workspace shell;
- replace the complete inner interface with Mission Canvas or Agent TUI;
- expand or compact the local sidebar;
- select system, full, or reduced motion.

The manifest cannot select a project, create a Workstream, attach a runtime, steer Pi, approve contention, mutate canonical state, or call a domain write route. Navigation to a workspace changes presentation only.

Interaction contract:

- open through the sidebar or `⌘K` / `Ctrl+K`;
- filter by label, hint, and keywords;
- navigate with Up/Down, Home, and End;
- execute with Enter;
- dismiss with Escape or backdrop selection;
- keep the Desktop shell stable while the palette appears;
- use opacity/transform transitions and reduced-motion fallback.

UIAI Engine evidence:

- screenshot: `docs/contracts/evidence/spec158-desktop-command-palette.png`;
- session: `2lJjZJ9S`;
- keyboard execution selected `Show Agent TUI` and rendered `.agent-surface`;
- diagnostics: `uiai-diagnostics:session=2lJjZJ9S:seq=0`, with zero console errors, warnings, exceptions, and failed requests.
