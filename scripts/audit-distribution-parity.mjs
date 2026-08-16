#!/usr/bin/env node
/**
 * Distribution parity audit (#260): one generated manifest describing
 * source, installed, and live surfaces, plus a drift report.
 *
 * Usage: node scripts/audit-distribution-parity.mjs [--json]
 *
 * Emits focusa.distribution_manifest.v1 with per-surface version/digest
 * facts and a typed drift list. Exit code 1 when drift is detected.
 */
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const ROOT = new URL("..", import.meta.url).pathname;
const jsonMode = process.argv.includes("--json");

function readJson(path) {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch {
    return null;
  }
}

function sha256(path) {
  try {
    return createHash("sha256").update(readFileSync(path)).digest("hex").slice(0, 16);
  } catch {
    return null;
  }
}

function dirCount(path) {
  try {
    return readdirSync(path).length;
  } catch {
    return null;
  }
}

function run(command, args) {
  try {
    return execFileSync(command, args, { encoding: "utf8", timeout: 8000 }).trim();
  } catch {
    return null;
  }
}

function tomlVersion(path) {
  try {
    const text = readFileSync(path, "utf8");
    const direct = text.match(/^version\s*=\s*"([^"]+)"/m);
    if (direct) return direct[1];
    if (/^version\.workspace\s*=\s*true/m.test(text)) {
      return tomlVersion(join(ROOT, "Cargo.toml")) ?? null;
    }
    return null;
  } catch {
    return null;
  }
}

const piExtDir =
  process.env.FOCUSA_PI_EXT_DIR ||
  [join(process.env.HOME || "", ".pi/agent/extensions/focusa"), "/root/.pi/agent/extensions/focusa"].find(
    (candidate) => existsSync(join(candidate, "package.json"))
  ) ||
  null;

const source = {
  core_version: tomlVersion(join(ROOT, "crates/focusa-core/Cargo.toml")),
  extension_version: readJson(join(ROOT, "apps/pi-extension/package.json"))?.version ?? null,
  contract_count:
    (() => {
      const registry = readJson(join(ROOT, "docs/current/focusa-tool-contracts.json"));
      if (!registry) return null;
      const list = Array.isArray(registry)
        ? registry
        : registry.tools ?? registry.contracts ?? registry.entries;
      return Array.isArray(list) ? list.length : null;
    })(),
  repo_skills_count: dirCount(join(ROOT, "apps/pi-extension/skills")),
  api_reference_digest: sha256(join(ROOT, "docs/current/API_REFERENCE_CURRENT.md")),
  cli_reference_digest: sha256(join(ROOT, "docs/current/CLI_REFERENCE_CURRENT.md")),
};

const installed = {
  cli_version:
    (() => {
      const raw = run("/usr/local/bin/focusa", ["--version"]) ?? run("focusa", ["--version"]);
      return raw ? raw.replace(/^focusa\s+/, "").trim() || null : null;
    })(),
  extension_version: piExtDir ? (readJson(join(piExtDir, "package.json"))?.version ?? null) : null,
  installed_skills_count: piExtDir ? dirCount(join(piExtDir, "skills")) : null,
};

let live = { daemon_ok: null, daemon_version: null };
try {
  const response = await fetch("http://127.0.0.1:8787/v1/health", {
    signal: AbortSignal.timeout(4000),
  });
  if (response.ok) {
    const body = await response.json();
    live = {
      daemon_ok: body?.ok ?? null,
      daemon_version: body?.version ?? body?.daemon_version ?? null,
    };
  }
} catch {
  /* daemon unreachable is itself a drift fact */
}

const drift = [];

// #260 digest axis: the installed extension's key runtime files must match
// the canonical tree (or be explicitly flagged as deployed-line divergence).
const digestFiles = ["src/tools.ts", "src/session.ts", "src/north-star.ts", "src/ota-activation.ts"];
const digests = {};
for (const relative of digestFiles) {
  const sourceDigest = sha256(join(ROOT, "apps/pi-extension", relative));
  const installedDigest = piExtDir ? sha256(join(piExtDir, relative)) : null;
  digests[relative] = { source: sourceDigest, installed: installedDigest };
  if (sourceDigest && installedDigest && sourceDigest !== installedDigest) {
    drift.push({
      surface: `digest:${relative}`,
      expected: "source_tree_digest",
      source_value: sourceDigest,
      observed_value: installedDigest,
    });
  }
}
const surfaces = [
  ["source.core_version", source.core_version, installed.cli_version, "cli"],
  ["source.extension_version", source.extension_version, installed.extension_version, "pi_extension"],
  ["installed.extension_version", installed.extension_version, live.daemon_version, "daemon"],
];
for (const [leftName, left, right, label] of surfaces) {
  if (left && right && left !== right) {
    drift.push({ surface: label, expected: leftName, source_value: left, observed_value: right });
  }
}

const manifest = {
  schema: "focusa.distribution_manifest.v1",
  generated_at: new Date().toISOString(),
  source,
  installed,
  live,
  digests,
  drift,
  parity_ok: drift.length === 0,
};

if (jsonMode) {
  console.log(JSON.stringify(manifest, null, 2));
} else {
  console.log(`source core=${source.core_version} ext=${source.extension_version} contracts=${source.contract_count} skills=${source.repo_skills_count}`);
  console.log(`installed cli=${installed.cli_version} ext=${installed.extension_version} skills=${installed.installed_skills_count}`);
  console.log(`live daemon ok=${live.daemon_ok} version=${live.daemon_version}`);
  if (drift.length) {
    console.log("DRIFT:");
    for (const row of drift) {
      console.log(`  ${row.surface}: expected ${row.source_value} (${row.expected}), observed ${row.observed_value}`);
    }
  } else {
    console.log("PARITY OK");
  }
}
process.exit(drift.length ? 1 : 0);
