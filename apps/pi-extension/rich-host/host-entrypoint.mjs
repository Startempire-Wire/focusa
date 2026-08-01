#!/usr/bin/env node
import { createServer } from "node:http";
import { readFile, rm, stat } from "node:fs/promises";
import { createHash } from "node:crypto";
import { dirname, extname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawn } from "node:child_process";

const root = dirname(fileURLToPath(import.meta.url));
const handshakePath = process.env.FOCUSA_RICH_HOST_HANDSHAKE;
if (!handshakePath) throw new Error("FOCUSA_RICH_HOST_HANDSHAKE is required");
const metadata = await stat(handshakePath);
if (process.platform !== "win32" && (metadata.mode & 0o077) !== 0) throw new Error("Rich-host handshake permissions are not private");
const serialized = await readFile(handshakePath, "utf8");
const expectedDigest = process.env.FOCUSA_RICH_HOST_HANDSHAKE_SHA256;
if (expectedDigest && createHash("sha256").update(serialized).digest("hex") !== expectedDigest) throw new Error("Rich-host handshake digest mismatch");
const handshake = JSON.parse(serialized);
if (handshake.schema !== "focusa.rich_host_handshake.v1" || handshake.protocol_version !== "1.0.0") throw new Error("Unsupported rich-host handshake");
if (Date.parse(handshake.expires_at) <= Date.now()) throw new Error("Rich-host handshake expired");
for (const key of ["project_root", "continuity_id", "session_id", "attachment_id"]) if (!handshake.scope?.[key]) throw new Error(`Rich-host scope missing ${key}`);
await rm(handshakePath, { force: true });

const server = createServer(async (request, response) => {
  const pathname = new URL(request.url || "/", "http://127.0.0.1").pathname;
  const relative = pathname === "/" ? "index.html" : pathname.replace(/^\//, "");
  if (!/^(index\.html|main\.js|a2ui-runtime\.js|styles\.css)$/.test(relative)) {
    response.writeHead(404).end("Not found");
    return;
  }
  const path = join(root, "assets", relative);
  let body = await readFile(path);
  if (relative === "index.html") {
    const bootstrap = JSON.stringify(handshake).replaceAll("<", "\\u003c");
    body = Buffer.from(String(body).replace("<script type=\"module\"", `<script>globalThis.__FOCUSA_RICH_HOST__=${bootstrap}</script><script type=\"module\"`));
  }
  const contentType = extname(relative) === ".js" ? "text/javascript" : extname(relative) === ".css" ? "text/css" : "text/html";
  response.writeHead(200, {
    "content-type": `${contentType}; charset=utf-8`,
    "cache-control": "no-store",
    "content-security-policy": "default-src 'self'; connect-src http://127.0.0.1:*; script-src 'self' 'unsafe-inline'; style-src 'self'; object-src 'none'; frame-ancestors 'none'",
    "referrer-policy": "no-referrer",
  });
  response.end(body);
});
server.listen(0, "127.0.0.1", () => {
  const address = server.address();
  if (!address || typeof address === "string") throw new Error("Rich-host loopback bind failed");
  const url = `http://127.0.0.1:${address.port}/`;
  process.stdout.write(`${JSON.stringify({ schema: "focusa.rich_host.ready.v1", url, attachment_id: handshake.scope.attachment_id })}\n`);
  if (process.env.FOCUSA_RICH_HOST_NO_OPEN !== "1") {
    const [command, args] = process.platform === "darwin" ? ["open", [url]] : process.platform === "win32" ? ["cmd", ["/c", "start", "", url]] : ["xdg-open", [url]];
    spawn(command, args, { detached: true, stdio: "ignore" }).unref();
  }
});

for (const signal of ["SIGINT", "SIGTERM"]) process.on(signal, () => server.close(() => process.exit(0)));
