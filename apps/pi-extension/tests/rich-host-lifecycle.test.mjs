import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { generateKeyPairSync, createHash, sign } from "node:crypto";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { RichHostLifecycleManager } from "../.tmp-rich-host-test/rich-host/lifecycle.js";
import { MissionCanvasApiClient } from "../.tmp-rich-host-test/rich-host/api-client.js";
import { verifyRichHostAsset } from "../.tmp-rich-host-test/rich-host/platform.js";

const scope = {
  project_root: "/tmp/focusa",
  continuity_id: "mission-canvas",
  session_id: "session:1",
  attachment_id: "attachment:1",
};

class FakeAdapter {
  launches = 0;
  focuses = 0;
  hides = 0;
  closes = 0;
  alive = true;
  async launch(_request, resolution) {
    this.launches += 1;
    return { process_id: 99, window_id: "window:attachment:1", renderer: resolution.selected_renderer };
  }
  async focus() { this.focuses += 1; }
  async hide() { this.hides += 1; }
  async close() { this.closes += 1; this.alive = false; }
  async isAlive() { return this.alive; }
}

class FakeClient {
  writes = [];
  async ensureProjection() { return { projection_revision: 1 }; }
  async updateHostLifecycle(action, state, expected) { this.writes.push({ action, state, expected }); }
  async events() { return []; }
  async getProjection() { return { projection_revision: 1 }; }
  durableEventCursor() { return "event:1"; }
}

const adapter = new FakeAdapter();
const client = new FakeClient();
const manager = new RichHostLifecycleManager(adapter);
const request = {
  scope,
  daemon_base_url: "http://127.0.0.1:8787/v1",
  token: "test-token",
  interaction_mode: "canvas-guided",
  package_root: process.cwd(),
  asset_version: "test",
};
const first = await manager.on(request, client);
const second = await manager.on(request, client);
assert.equal(adapter.launches, 1, "one window per attachment");
assert.equal(adapter.focuses, 1, "second ON focuses existing window");
assert.equal(first.window_id, second.window_id);
await manager.off(scope, false);
assert.equal(adapter.hides, 1);
assert.equal(manager.state(scope).state, "hidden");
await manager.shutdown();
assert.equal(adapter.closes, 1);

const root = await mkdtemp(join(tmpdir(), "focusa-rich-host-test-"));
await mkdir(join(root, "rich-host", "assets"), { recursive: true });
const bytes = Buffer.from("verified rich host asset");
await writeFile(join(root, "rich-host", "assets", "app.js"), bytes);
const sha256 = createHash("sha256").update(bytes).digest("hex");
const { privateKey, publicKey } = generateKeyPairSync("ed25519");
const signed = Buffer.from(`1.0.0\nLinux\nx64\nassets/app.js\n${sha256}`);
const manifest = {
  schema: "focusa.rich_host_asset_manifest.v1",
  version: "1.0.0",
  platform: "Linux",
  architecture: "x64",
  entrypoint: "assets/app.js",
  sha256,
  signature: sign(null, signed, privateKey).toString("base64"),
  public_key_pem: publicKey.export({ type: "spki", format: "pem" }).toString(),
};
assert.equal(await verifyRichHostAsset(root, manifest), `sha256:${sha256}`);
await assert.rejects(() => verifyRichHostAsset(root, { ...manifest, sha256: "0".repeat(64) }), /digest mismatch/);
await rm(root, { recursive: true, force: true });

let revision = 2;
const fetchImpl = async (url) => ({
  ok: true,
  status: 200,
  async json() {
    return {
      schema: "focusa.resolved_workspace_projection.v1",
      scope,
      projection_revision: revision,
      layout_revision: revision,
      durable_event_cursor: `event:${revision}`,
      projection_digest: `sha256:${"1".repeat(64)}`,
    };
  },
});
const api = new MissionCanvasApiClient("http://127.0.0.1:8787/v1", undefined, scope, fetchImpl);
await api.getProjection();
revision = 1;
await assert.rejects(() => api.getProjection(), /revision regressed/);

console.log("Spec 135 rich-host lifecycle, security, and cache discipline: PASS");
