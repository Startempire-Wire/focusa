import assert from "node:assert/strict";
import { createHash, randomBytes } from "node:crypto";
import { chmod, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { spawn } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const directory = await mkdtemp(join(tmpdir(), "focusa-rich-host-integration-"));
const handshakePath = join(directory, "handshake.json");
const token = `token-${randomBytes(8).toString("hex")}`;
const handshake = JSON.stringify({
  schema: "focusa.rich_host_handshake.v1",
  protocol_version: "1.0.0",
  daemon_base_url: "http://127.0.0.1:8787/v1",
  token,
  scope: {
    project_root: "/tmp/focusa",
    continuity_id: "mission-canvas",
    session_id: "session:integration",
    attachment_id: "attachment:integration",
  },
  nonce: randomBytes(16).toString("hex"),
  expires_at: new Date(Date.now() + 60_000).toISOString(),
});
await writeFile(handshakePath, handshake, { mode: 0o600 });
await chmod(handshakePath, 0o600);
const digest = createHash("sha256").update(handshake).digest("hex");
const child = spawn(process.execPath, ["rich-host/host-entrypoint.mjs"], {
  cwd: join(dirname(fileURLToPath(import.meta.url)), ".."),
  env: {
    ...process.env,
    FOCUSA_RICH_HOST_HANDSHAKE: handshakePath,
    FOCUSA_RICH_HOST_HANDSHAKE_SHA256: digest,
    FOCUSA_RICH_HOST_NO_OPEN: "1",
  },
  stdio: ["ignore", "pipe", "pipe"],
});
const ready = await new Promise((resolve, reject) => {
  const timeout = setTimeout(() => reject(new Error("rich host ready timeout")), 10_000);
  child.once("error", reject);
  child.stderr.on("data", (chunk) => reject(new Error(String(chunk))));
  child.stdout.once("data", (chunk) => {
    clearTimeout(timeout);
    resolve(JSON.parse(String(chunk).trim()));
  });
});
assert.equal(ready.schema, "focusa.rich_host.ready.v1");
assert.equal(ready.attachment_id, "attachment:integration");
const response = await fetch(ready.url);
assert.equal(response.status, 200);
const html = await response.text();
assert.match(html, /focusa\.rich_host_handshake\.v1/);
assert.match(html, /attachment:integration/);
assert.match(html, new RegExp(token));
assert.ok(!child.spawnargs.join(" ").includes(token), "token must not appear in process arguments");
await assert.rejects(() => readFile(handshakePath), /ENOENT/);
child.kill("SIGTERM");
await new Promise((resolve) => child.once("exit", resolve));
await rm(directory, { recursive: true, force: true });
console.log("Spec 135 Pi-to-rich-host integration: PASS");
