import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
const tools = readFileSync(new URL("../src/tools.ts", import.meta.url), "utf8");
const contracts = readFileSync(new URL("../src/tool-contracts.ts", import.meta.url), "utf8");
const start = tools.indexOf('name: "focusa_silent_sessions"');
const end = tools.indexOf("pi.registerTool({", start);
assert(start >= 0 && end > start, "daemon facade tool missing");
const facade = tools.slice(start, end);
for (const marker of [
  "focusaFetchDetailed",
  "/silent-sessions",
  "run_id",
  "generation",
  "approval_id",
  "idempotency_key",
  "direct configs are wrapped automatically",
  '"config" in p.config',
  "{ config: p.config || {} }",
  "body: JSON.stringify(preflightBody)",
  "result?.body ?? result?.data ?? result",
  "payload?.error ?? payload?.message ?? payload?.reason",
  "http_status",
  'authority: "daemon"',
  'parity: "full"',
])
  assert(facade.includes(marker), marker);
for (const forbidden of ["defaultSilentSessionCommand", "silentSessionExec", "execFileSync", "/tmp/"])
  assert(!facade.includes(forbidden), forbidden);
const cstart = contracts.indexOf('name: "focusa_silent_sessions"');
const cend = contracts.indexOf('name: "focusa_tool_doctor"', cstart);
const contract = contracts.slice(cstart, cend);
assert(contract.includes('parity_status: "full"'));
assert(contract.includes('path: "daemon:/v1/silent-sessions"'));
assert(!contract.toLowerCase().includes("tmux"));
console.log("silent session daemon-facade hardening test passed");
