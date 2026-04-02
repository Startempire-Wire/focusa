// focusa-pi-bridge — Focusa cognitive integration for Pi
// See: /home/wirebot/focusa/docs/44-pi-focusa-integration-spec.md (§28-§38)
// See: /home/wirebot/focusa/docs/UNIFIED_ORGANISM_SPEC.md (§9.9)

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";

export default function (pi: ExtensionAPI) {
  pi.on("session_start", async (_event, ctx) => {
    ctx.ui.setStatus("focusa", "🧠 Focusa");
  });
}
