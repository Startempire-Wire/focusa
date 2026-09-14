#!/usr/bin/env node
/**
 * Release-readiness evaluation (#265): aggregate the day's gate evidence
 * into one typed JSON. Runs locally on the anchor server; bounded checks.
 *
 * Usage: node scripts/release-readiness-evaluation.mjs [--json]
 */
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

const ROOT = new URL("..", import.meta.url).pathname;
const jsonMode = process.argv.includes("--json");

function lastLine(log) {
  try {
    const text = readFileSync(log, "utf8").trim();
    return text.split("\n").filter(Boolean).pop() || null;
  } catch {
    return null;
  }
}

const checks = [];
function check(name, ok, detail) {
  checks.push({ name, ok, detail });
}

// 1. Daemon health + version (live).
let daemon = { ok: false };
try {
  const response = await fetch("http://127.0.0.1:8787/v1/health", {
    signal: AbortSignal.timeout(4000),
  });
  if (response.ok) {
    const body = await response.json();
    daemon = { ok: body?.ok === true, version: body?.version ?? null };
  }
} catch {}
check("daemon_health", daemon.ok, JSON.stringify(daemon));

// 2. Distribution parity (drift detection).
let parity = null;
try {
  const output = execFileSync("node", [join(ROOT, "scripts/audit-distribution-parity.mjs"), "--json"], {
    encoding: "utf8",
    timeout: 15000,
  });
  parity = JSON.parse(output);
} catch (error) {
  parity = error?.stdout ? JSON.parse(error.stdout) : null;
}
check("distribution_parity", parity ? parity.parity_ok : false, parity ? `${parity.drift.length} drift rows` : "parity run failed");

// 3. Gate chain markers (background logs on this host).
const gateLogs = [
  "/tmp/focusa-consolidation-gates6.log",
  "/tmp/focusa-consolidation-gates5.log",
  "/tmp/focusa-consolidation-gates4.log",
  "/tmp/focusa-307-243-gates8.log",
  "/tmp/focusa-307-243-gates7.log",
];
const greenMarker = gateLogs.map(lastLine).find((line) => line && line.includes("GATES-GREEN"));
check("gate_chain_green", Boolean(greenMarker), greenMarker || "no green marker yet");

// 4. Extension targeted suites (this host).
const extLogs = ["/tmp/focusa-ext-targeted.log", "/tmp/focusa-ext-mc.log", "/tmp/focusa-ext-full-suite.log"];
const extEvidence = extLogs
  .map((path) => ({ path, tail: lastLine(path) }))
  .filter((entry) => entry.tail);
check("extension_suites_run", extEvidence.length > 0, `${extEvidence.length} suite logs present`);

// 5. Convergence invariants static gate.
let convergence = "not-run";
try {
  execFileSync("bash", [join(ROOT, "tests/convergence_invariants_static_test.sh")], {
    encoding: "utf8",
    timeout: 15000,
  });
  convergence = "pass";
} catch {
  convergence = "fail";
}
check("convergence_invariants", convergence === "pass", convergence);

// 6. Disk headroom (rule enforcement).
const df = execFileSync("df", ["-h", "/home/wirebot"], { encoding: "utf8", timeout: 5000 })
  .split("\n")[1] || "";
const usedPercent = parseInt((df.match(/(\d+)%/)?.[1] || "100"), 10);
check("disk_headroom", usedPercent <= 90, `${usedPercent}% used`);

const manifest = {
  schema: "focusa.release_readiness_evaluation.v1",
  generated_at: new Date().toISOString(),
  checks,
  ready: checks.every((entry) => entry.ok),
};

if (jsonMode) {
  console.log(JSON.stringify(manifest, null, 2));
} else {
  for (const entry of checks) {
    console.log(`${entry.ok ? "PASS" : "FAIL"} ${entry.name}: ${entry.detail}`);
  }
  console.log(`READY: ${manifest.ready}`);
}
process.exit(manifest.ready ? 0 : 1);
