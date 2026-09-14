// Model plan advisory — friendly, one-time guidance when a selected model
// requires a plan upgrade that isn't currently active (e.g. gpt-5.6-sol on
// ChatGPT-backed Codex auth). Instead of raw provider errors being the only
// signal, Focusa explains what happened in plain language and, when possible,
// gracefully falls back to the closest supported model.
//
// Spec alignment: advisory-only UX polish; never changes authority or
// compaction ownership. One notification per session per model (deduped).

const PLAN_GATED_MODELS: Record<string, { label: string; why: string }> = {
  "gpt-5.6-sol": {
    label: "gpt-5.6-sol",
    why: "it needs an active upgrade and isn't available on the current ChatGPT-backed Codex sign-in",
  },
  "gpt-5.3-codex-spark": {
    label: "gpt-5.3-codex-spark",
    why: "it needs an active upgrade and isn't available on the current ChatGPT-backed Codex sign-in",
  },
};

const FALLBACK_PROVIDER = "openai-codex";
// Verified working on the current ChatGPT plan (live probe 2026-08-22).
const FALLBACK_MODEL_ID = "gpt-5.6-luna";

const notifiedModels = new Set<string>();
let switching = false;

function planGate(model: any): { label: string; why: string } | undefined {
  const provider = String(model?.provider ?? "");
  const id = String(model?.id ?? "");
  if (provider !== FALLBACK_PROVIDER) return undefined;
  return PLAN_GATED_MODELS[id];
}

async function ensureNotGated(pi: any, ctx: any, model: any): Promise<void> {
  const gate = planGate(model);
  if (!gate || notifiedModels.has(gate.label)) return;
  notifiedModels.add(gate.label);

  // Gracefully move the session to the closest supported model so no API
  // call ever goes out on the gated model. setModel emits another
  // model_select; the guard below plus the non-gated fallback terminate it.
  let switched = false;
  if (!switching) {
    try {
      const fallback = ctx?.modelRegistry?.find?.(FALLBACK_PROVIDER, FALLBACK_MODEL_ID);
      if (fallback) {
        switching = true;
        switched = await pi.setModel(fallback);
        switching = false;
      }
    } catch (cause) {
      switching = false;
      console.warn(`[focusa] model-plan-advisory: fallback switch failed: ${cause instanceof Error ? cause.message : cause}`);
    }
  }

  try {
    ctx.ui.notify(
      switched
        ? `ℹ️ ${gate.label} ${gate.why}. Switched this session to ${FALLBACK_MODEL_ID} so nothing breaks — flip back anytime with Ctrl+P once your plan renews.`
        : `ℹ️ Heads up: ${gate.label} ${gate.why}. Pick another model with Ctrl+P until your upgrade is active.`,
      "info"
    );
  } catch (cause) {
    console.warn(`[focusa] model-plan-advisory: notify failed: ${cause instanceof Error ? cause.message : cause}`);
  }
}

export function registerModelPlanAdvisory(pi: any): void {
  // Startup / restore: check whatever model is active at session start.
  pi.on("session_start", async (_event: unknown, ctx: any) => {
    try {
      await ensureNotGated(pi, ctx, ctx?.model);
    } catch (cause) {
      console.warn(`[focusa] model-plan-advisory: session_start check failed: ${cause instanceof Error ? cause.message : cause}`);
    }
  });

  // Model changes INCLUDING session restore of the previously selected
  // model — restore lands here after session_start, so this is the hook
  // that must perform the graceful fallback.
  pi.on("model_select", async (event: any, ctx: any) => {
    try {
      await ensureNotGated(pi, ctx, event?.model);
    } catch (cause) {
      console.warn(`[focusa] model-plan-advisory: model_select check failed: ${cause instanceof Error ? cause.message : cause}`);
    }
  });
}
