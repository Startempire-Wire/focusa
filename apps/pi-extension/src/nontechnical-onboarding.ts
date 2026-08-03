import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { getActiveWorkpointPacket, getSessionCwd } from "./state.js";

export function onboardingSummary(packet: any): string {
  const ready = Boolean(packet?.workpoint_id || packet?.workpoint?.workpoint_id);
  return [
    "# Welcome to Focusa",
    ready ? "Your project is ready to continue." : "Let’s get your project ready.",
    "",
    ready ? "Primary action: Continue your project" : "Primary action: Start guided setup",
    "Focusa saves reviewed answers, so you will not be asked for them again.",
    "You can pause safely and resume here later.",
    "Advanced details stay hidden unless you choose to inspect them.",
    "If setup cannot continue, Focusa explains why and offers a safe recovery action.",
  ].join("\n");
}

export function registerNontechnicalOnboarding(pi: ExtensionAPI): void {
  pi.registerCommand("focusa-start", {
    description: "Start or continue Focusa with plain-language guided onboarding",
    handler: async (_args, ctx) => {
      if (!ctx.hasUI) return;
      const packet = getActiveWorkpointPacket();
      const choice = await ctx.ui.select("What would you like to do?", [
        packet ? "Continue my project" : "Start guided setup",
        "Add project documents",
        "Review saved answers",
        "Recover a paused setup",
      ]);
      if (!choice) return;
      pi.sendMessage({
        customType: "focusa-onboarding",
        content: `${onboardingSummary(packet)}\n\nSelected: ${choice}\nProject folder: ${getSessionCwd()}`,
        display: true,
      });
    },
  });
}
