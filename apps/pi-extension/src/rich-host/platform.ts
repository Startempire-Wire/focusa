import { access, chmod, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { constants } from "node:fs";
import { createHash, randomBytes, verify as verifySignature } from "node:crypto";
import { join } from "node:path";
import { spawn } from "node:child_process";
import type {
  HostCapabilityProbe,
  HostRendererResolution,
  InteractionMode,
  RichHostLaunchRequest,
  RichHostPlatform,
  RichHostProcessAdapter,
  RichHostProcessHandle,
} from "./types.js";

export interface RichHostAssetManifest {
  schema: "focusa.rich_host_asset_manifest.v1";
  version: string;
  platform: RichHostPlatform;
  architecture: string;
  entrypoint: string;
  sha256: string;
  signature: string;
  public_key_pem: string;
}

function platformName(platform = process.platform): RichHostPlatform {
  if (platform === "darwin") return "macOS";
  if (platform === "win32") return "Windows";
  return "Linux";
}

async function executable(path: string): Promise<boolean> {
  try {
    await access(path, constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

export async function probeRichHost(packageRoot: string): Promise<HostCapabilityProbe> {
  const platform = platformName();
  const platformDirectory = process.platform === "darwin" ? "darwin" : process.platform;
  const executableName = process.platform === "win32" ? "focusa-rich-host.exe" : "focusa-rich-host";
  const nativeBinaryPath = join(packageRoot, "rich-host", "bin", `${platformDirectory}-${process.arch}`, executableName);
  const nativeBinaryAvailable = await executable(nativeBinaryPath);
  return {
    platform,
    architecture: process.arch,
    native_binary_path: nativeBinaryAvailable ? nativeBinaryPath : undefined,
    native_binary_available: nativeBinaryAvailable,
    system_browser_available: !process.env.FOCUSA_RICH_HOST_DISABLE_BROWSER,
    tui_available: process.stdout.isTTY === true,
    headless: !process.stdout.isTTY && !process.env.DISPLAY && process.platform !== "win32" && process.platform !== "darwin",
    reason: nativeBinaryAvailable ? "signed packaged host binary found" : "packaged host binary unavailable",
  };
}

export function resolveRichHostRenderer(
  interactionMode: InteractionMode,
  probe: HostCapabilityProbe,
  assetVersion: string
): HostRendererResolution {
  if (interactionMode === "headless" || probe.headless) {
    return resolution(interactionMode, probe, "headless_none", "headless", "No graphical display is available", assetVersion);
  }
  if (probe.native_binary_available) {
    return resolution(interactionMode, probe, "focusa_pi_rich_window", "available", probe.reason, assetVersion);
  }
  if (interactionMode === "canvas-guided" && probe.system_browser_available) {
    return resolution(interactionMode, probe, "mission_deck_web", "fallback", "Native host unavailable; using system webview/browser", assetVersion);
  }
  if (probe.tui_available) {
    return resolution(interactionMode, probe, "native_tui", "fallback", "Using stock Pi TUI projection", assetVersion);
  }
  return resolution(interactionMode, probe, "headless_none", "headless", "No compatible renderer is available", assetVersion);
}

function resolution(
  interactionMode: InteractionMode,
  probe: HostCapabilityProbe,
  renderer: HostRendererResolution["selected_renderer"],
  availability: HostRendererResolution["availability"],
  reason: string,
  assetVersion: string
): HostRendererResolution {
  return {
    interaction_mode: interactionMode,
    selected_renderer: renderer,
    platform: probe.platform,
    availability,
    resolution_reason: reason,
    asset_version: renderer === "focusa_pi_rich_window" ? assetVersion : null,
    asset_digest: null,
    resolver_revision: "host-resolver:v1",
    diagnostic_ref: availability === "available" ? null : `diagnostic:rich-host:${renderer}`,
  };
}

export async function verifyRichHostAsset(packageRoot: string, manifest: RichHostAssetManifest): Promise<string> {
  if (manifest.schema !== "focusa.rich_host_asset_manifest.v1") throw new Error("Unsupported rich-host manifest schema");
  const entrypoint = join(packageRoot, "rich-host", manifest.entrypoint);
  const bytes = await readFile(entrypoint);
  const digest = createHash("sha256").update(bytes).digest("hex");
  if (digest !== manifest.sha256) throw new Error("Rich-host asset digest mismatch");
  const signed = Buffer.from(`${manifest.version}\n${manifest.platform}\n${manifest.architecture}\n${manifest.entrypoint}\n${manifest.sha256}`);
  const signature = Buffer.from(manifest.signature, "base64");
  if (!verifySignature(null, signed, manifest.public_key_pem, signature)) throw new Error("Rich-host asset signature invalid");
  return `sha256:${digest}`;
}

export async function writeHandshakeFile(request: RichHostLaunchRequest): Promise<{ path: string; digest: string }> {
  const directory = join(process.env.TMPDIR || process.env.TEMP || "/tmp", "focusa-rich-host");
  await mkdir(directory, { recursive: true, mode: 0o700 });
  const nonce = randomBytes(24).toString("hex");
  const path = join(directory, `${process.pid}-${nonce}.json`);
  const payload = JSON.stringify({
    schema: "focusa.rich_host_handshake.v1",
    protocol_version: "1.0.0",
    daemon_base_url: request.daemon_base_url,
    token: request.token || null,
    scope: request.scope,
    nonce,
    expires_at: new Date(Date.now() + 60_000).toISOString(),
  });
  await writeFile(path, payload, { mode: 0o600, flag: "wx" });
  await chmod(path, 0o600);
  return { path, digest: createHash("sha256").update(payload).digest("hex") };
}

export async function removeHandshakeFile(path: string): Promise<void> {
  await rm(path, { force: true });
}

export class NativeProcessAdapter implements RichHostProcessAdapter {
  async launch(request: RichHostLaunchRequest, resolution: HostRendererResolution, handshakePath: string): Promise<RichHostProcessHandle> {
    if (resolution.selected_renderer === "headless_none" || resolution.selected_renderer === "native_tui") {
      return { window_id: `fallback:${request.scope.attachment_id}`, renderer: resolution.selected_renderer };
    }
    const probe = await probeRichHost(request.package_root);
    let command: string;
    let args: string[];
    if (resolution.selected_renderer === "focusa_pi_rich_window" && probe.native_binary_path) {
      command = probe.native_binary_path;
      args = [];
    } else {
      command = process.execPath;
      args = [join(request.package_root, "rich-host", "host-entrypoint.mjs")];
    }
    const handshakeDigest = createHash("sha256").update(await readFile(handshakePath)).digest("hex");
    const child = spawn(command, args, {
      detached: true,
      stdio: "ignore",
      env: {
        ...process.env,
        FOCUSA_RICH_HOST_HANDSHAKE: handshakePath,
        FOCUSA_RICH_HOST_HANDSHAKE_SHA256: handshakeDigest,
      },
    });
    child.unref();
    return { process_id: child.pid, window_id: `window:${request.scope.attachment_id}`, renderer: resolution.selected_renderer };
  }

  async focus(_handle: RichHostProcessHandle): Promise<void> {}
  async hide(_handle: RichHostProcessHandle): Promise<void> {}
  async close(handle: RichHostProcessHandle): Promise<void> {
    if (handle.process_id) {
      try { process.kill(handle.process_id, "SIGTERM"); } catch { /* already exited */ }
    }
  }
  async isAlive(handle: RichHostProcessHandle): Promise<boolean> {
    if (!handle.process_id) return handle.renderer === "native_tui";
    try { process.kill(handle.process_id, 0); return true; } catch { return false; }
  }
}
