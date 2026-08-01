import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";
import { RichHostLifecycleManager } from "../.tmp-rich-host-test/rich-host/lifecycle.js";

const scope = { project_root: "/tmp/focusa", continuity_id: "stress", session_id: "session:stress", attachment_id: "attachment:stress" };
class Adapter {
  launches = 0; focuses = 0; hides = 0; closes = 0; alive = true;
  async launch(_request, resolution) { this.launches++; this.alive = true; return { process_id: 42, window_id: "window:stress", renderer: resolution.selected_renderer }; }
  async focus() { this.focuses++; }
  async hide() { this.hides++; }
  async close() { this.closes++; this.alive = false; }
  async isAlive() { return this.alive; }
}
class Client {
  writes = 0;
  async ensureProjection() { return { projection_revision: 1 }; }
  async updateHostLifecycle() { this.writes++; }
  async events() { return []; }
  async getProjection() { return { projection_revision: 1 }; }
  durableEventCursor() { return "event:stress"; }
}
const adapter = new Adapter();
const client = new Client();
const manager = new RichHostLifecycleManager(adapter);
const request = { scope, daemon_base_url: "http://127.0.0.1:8787/v1", interaction_mode: "canvas-guided", package_root: process.cwd(), asset_version: "stress" };
const memoryBefore = process.memoryUsage().heapUsed;
const started = performance.now();
await manager.on(request, client);
for (let index = 0; index < 1_000; index++) {
  await manager.off(scope, false);
  await manager.on(request, client);
}
await manager.shutdown();
const elapsed = performance.now() - started;
const memoryGrowth = process.memoryUsage().heapUsed - memoryBefore;
assert.equal(adapter.launches, 1);
assert.equal(adapter.closes, 1);
assert.equal(manager.state(scope), undefined);
assert.ok(elapsed < 10_000, `stress elapsed ${elapsed}ms`);
assert.ok(memoryGrowth < 64 * 1024 * 1024, `heap growth ${memoryGrowth}`);
console.log(`Spec 135 rich-host long-session stress: PASS (${elapsed.toFixed(1)}ms, heap ${memoryGrowth})`);
