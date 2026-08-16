import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

const ACTIVATE_COMMAND = "focusa-activate-updated-extension";
const RESTART_MARKER = "pi-extension-restart-required.json";
const ACTIVATING_MARKER = "pi-extension-activating.json";
const ACTIVATION_RECEIPT = "pi-extension-activation-receipt.json";
const POLL_MS = 2_500;
const TIMER_KEY = Symbol.for("focusa.pi_extension.ota_activation_timer.v1");

type OtaMarker = {
  schema?: string;
  version?: string;
  installed_at?: string;
  action?: string;
};

type OtaTimerGlobal = typeof globalThis & {
  [TIMER_KEY]?: ReturnType<typeof setInterval>;
};

export function otaActivationStateRoot(env: NodeJS.ProcessEnv = process.env): string {
  const stateHome =
    String(env.XDG_STATE_HOME || "").trim() ||
    join(String(env.HOME || "").trim() || homedir(), ".local", "state");
  return join(stateHome, "focusa", "update");
}

export function otaActivationPaths(root = otaActivationStateRoot()): {
  restart: string;
  activating: string;
  receipt: string;
} {
  return {
    restart: join(root, RESTART_MARKER),
    activating: join(root, ACTIVATING_MARKER),
    receipt: join(root, ACTIVATION_RECEIPT),
  };
}

function readMarker(path: string): OtaMarker {
  try {
    return JSON.parse(readFileSync(path, "utf8")) as OtaMarker;
  } catch {
    return {};
  }
}

function writeReceipt(activating: string, receipt: string): void {
  if (!existsSync(activating)) return;
  const marker = readMarker(activating);
  const payload = {
    schema: "focusa.pi_extension_activation_receipt.v1",
    status: "activated",
    version: marker.version || null,
    installed_at: marker.installed_at || null,
    activated_at: new Date().toISOString(),
    activation: "pi_runtime_reload",
  };
  mkdirSync(join(receipt, ".."), { recursive: true, mode: 0o700 });
  const temporary = `${receipt}.tmp-${process.pid}-${Date.now()}`;
  writeFileSync(temporary, `${JSON.stringify(payload, null, 2)}\n`, { mode: 0o600 });
  renameSync(temporary, receipt);
  rmSync(activating, { force: true });
}

/**
 * Complete Pi-extension OTA activation without operator intervention.
 *
 * The CLI atomically replaces the package and writes RESTART_MARKER. A running
 * Pi process notices it and queues a private slash command. Slash commands have
 * ExtensionCommandContext authority, so the handler can wait for idle and call
 * ctx.reload(). The marker is retained/restored on failure and becomes a durable
 * activation receipt only after reload succeeds.
 */
export function registerAutomaticOtaActivation(pi: ExtensionAPI): () => void {
  const paths = otaActivationPaths();
  const timerGlobal = globalThis as OtaTimerGlobal;
  let queued = false;

  // A process starting after package promotion already loaded the new files. An
  // activating marker proves a prior runtime crossed the reload boundary.
  try {
    writeReceipt(paths.activating, paths.receipt);
  } catch {
    // Keep the marker for the command path; startup must never crash Pi.
  }

  const queueActivation = (): void => {
    if (queued || !existsSync(paths.restart)) return;
    queued = true;
    try {
      pi.sendUserMessage(`/${ACTIVATE_COMMAND}`, { deliverAs: "followUp" });
    } catch {
      queued = false;
    }
  };

  pi.registerCommand(ACTIVATE_COMMAND, {
    description: "Activate an atomically installed Focusa Pi extension update",
    handler: async (_args, ctx) => {
      if (!existsSync(paths.restart)) {
        queued = false;
        return;
      }
      await ctx.waitForIdle();
      mkdirSync(otaActivationStateRoot(), { recursive: true, mode: 0o700 });
      rmSync(paths.activating, { force: true });
      renameSync(paths.restart, paths.activating);
      try {
        await ctx.reload();
        writeReceipt(paths.activating, paths.receipt);
        queued = false;
        ctx.ui.notify("Focusa Pi extension OTA activated automatically.", "info");
      } catch (error) {
        if (existsSync(paths.activating)) renameSync(paths.activating, paths.restart);
        queued = false;
        ctx.ui.notify(
          `Focusa Pi extension OTA activation deferred safely: ${error instanceof Error ? error.message : String(error)}`,
          "warning"
        );
      }
    },
  });

  pi.on("session_start", async () => queueActivation());
  pi.on("agent_end", async () => queueActivation());
  pi.on("session_shutdown", async () => {
    const timer = timerGlobal[TIMER_KEY];
    if (timer) clearInterval(timer);
    delete timerGlobal[TIMER_KEY];
  });

  const previous = timerGlobal[TIMER_KEY];
  if (previous) clearInterval(previous);
  const timer = setInterval(queueActivation, POLL_MS);
  timer.unref?.();
  timerGlobal[TIMER_KEY] = timer;
  queueActivation();

  return () => {
    clearInterval(timer);
    if (timerGlobal[TIMER_KEY] === timer) delete timerGlobal[TIMER_KEY];
  };
}
