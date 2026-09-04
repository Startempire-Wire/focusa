#!/usr/bin/env node
/**
 * Distribution parity audit (#260): one generated manifest describing
 * source, installed, and live surfaces, plus a drift report.
 *
 * Usage: node scripts/audit-distribution-parity.mjs [--json]
 *
 * Emits focusa.distribution_parity.v1 with per-surface version/digest
 * facts and a typed drift list. Exit code 1 when drift is detected.
 */
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { existsSync, lstatSync, readFileSync, readdirSync } from "node:fs";
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
    return `sha256:${createHash("sha256").update(readFileSync(path)).digest("hex")}`;
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

const treeExcludedDirs = new Set([
  ".git",
  ".beads",
  "node_modules",
  "target",
  "dist",
  ".svelte-kit",
  "__pycache__",
]);
const manifestRelative =
  "docs/contracts/spec141/generated-capability-v2/distribution-manifest.json";

function treeDigestMappings(root, mappings) {
  try {
    const files = [];
    function walk(directory, emittedPrefix) {
      for (const entry of readdirSync(directory, { withFileTypes: true })) {
        if (treeExcludedDirs.has(entry.name)) continue;
        const absolute = join(directory, entry.name);
        const emitted = emittedPrefix ? `${emittedPrefix}/${entry.name}` : entry.name;
        if (entry.isSymbolicLink()) throw new Error(`symlink:${emitted}`);
        if (entry.isDirectory()) walk(absolute, emitted);
        else if (entry.isFile() && emitted !== manifestRelative) files.push([emitted, absolute]);
        else if (!entry.isFile()) throw new Error(`special:${emitted}`);
      }
    }
    for (const [source, emitted] of mappings) {
      const absolute = join(root, source);
      if (!existsSync(absolute)) throw new Error(`missing:${source}`);
      const metadata = lstatSync(absolute);
      if (metadata.isSymbolicLink()) throw new Error(`symlink:${source}`);
      if (metadata.isFile()) files.push([emitted, absolute]);
      else if (metadata.isDirectory()) walk(absolute, emitted);
      else throw new Error(`special:${source}`);
    }
    files.sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0));
    const digest = createHash("sha256");
    for (const [relative, absolute] of files) {
      digest.update(relative, "utf8");
      digest.update(Buffer.from([0]));
      digest.update(createHash("sha256").update(readFileSync(absolute)).digest());
      digest.update(Buffer.from([0]));
    }
    return { algorithm: "sha256-tree-v1", sha256: `sha256:${digest.digest("hex")}`, file_count: files.length };
  } catch (error) {
    return { algorithm: "sha256-tree-v1", sha256: null, file_count: null, error: String(error) };
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

const sourceManifestPath = join(
  ROOT,
  "docs/contracts/spec141/generated-capability-v2/distribution-manifest.json",
);
const sourceManifest = readJson(sourceManifestPath);
const installedManifestPath =
  process.env.FOCUSA_DISTRIBUTION_MANIFEST ||
  [
    "/usr/local/lib/focusa/distribution-manifest.json",
    join(process.env.HOME || "", ".focusa/distribution-manifest.json"),
  ].find((candidate) => existsSync(candidate)) ||
  null;
const installedManifest = installedManifestPath ? readJson(installedManifestPath) : null;
const releaseManifestPath = process.env.FOCUSA_RELEASE_MANIFEST || null;
const releaseManifest = releaseManifestPath ? readJson(releaseManifestPath) : null;
const releaseAssetSuffix = process.env.FOCUSA_RELEASE_ASSET_SUFFIX || null;

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

function binaryVersion(path, fallback = null) {
  const raw = run(path, ["--version"]) ?? (fallback ? run(fallback, ["--version"]) : null);
  if (!raw) return null;
  const match = raw.match(/\b\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?\b/);
  return match?.[0] ?? null;
}

const runtimeContract = sourceManifest?.components?.runtime_contract ?? {};
const binaryPaths = runtimeContract.binary_paths ?? {
  cli: "/usr/local/bin/focusa",
  daemon: "/usr/local/bin/focusa-daemon",
  tui: "/usr/local/bin/focusa-tui",
  session_runner: "/usr/local/bin/focusa-session-runner",
};
const binaryVersions = {};
const binaryDigests = {};
for (const [surface, path] of Object.entries(binaryPaths)) {
  binaryVersions[surface] = binaryVersion(path, surface === "cli" ? "focusa" : null);
  binaryDigests[surface] = sha256(path);
}
const agentContextDir =
  process.env.FOCUSA_AGENT_CONTEXT_DIR || join(process.env.HOME || "", ".focusa/agent-context");

const installedComponents = {
  pi_extension: piExtDir
    ? treeDigestMappings(piExtDir, [[".", "apps/pi-extension"]])
    : null,
  agent_skills: existsSync(join(agentContextDir, "skills"))
    ? treeDigestMappings(agentContextDir, [
        ["skills", ".pi/skills"],
        ["bin/focusa-skill-doctor", "scripts/focusa-skill-doctor"],
      ])
    : null,
  documentation: existsSync(join(agentContextDir, "docs"))
    ? treeDigestMappings(agentContextDir, [
        ["AGENTS.md", "AGENTS.md"],
        ["README.md", "README.md"],
        ["docs/current", "docs/current"],
        ["docs/contracts/spec141/generated-capability-v2", "docs/contracts/spec141/generated-capability-v2"],
        ["docs/07-reference-store.md", "docs/07-reference-store.md"],
        ["docs/82-focusa-memory-optimization-spec.md", "docs/82-focusa-memory-optimization-spec.md"],
        ["docs/94-focusa-intent-preserving-memory-rpc-optimization-sow.md", "docs/94-focusa-intent-preserving-memory-rpc-optimization-sow.md"],
        ["docs/canonical-live-release-pipeline.md", "docs/canonical-live-release-pipeline.md"],
      ])
    : null,
  generated_clients: existsSync(join(agentContextDir, "packages/generated/spec135"))
    ? treeDigestMappings(agentContextDir, [
        ["packages/generated/spec135", "packages/generated/spec135"],
        ["docs/contracts/spec135/generated-contract-v1", "docs/contracts/spec135/generated-contract-v1"],
      ])
    : null,
};

const installed = {
  cli_version: binaryVersions.cli,
  binary_versions: binaryVersions,
  binary_digests: binaryDigests,
  extension_version: piExtDir ? (readJson(join(piExtDir, "package.json"))?.version ?? null) : null,
  installed_skills_count: piExtDir ? dirCount(join(piExtDir, "skills")) : null,
  manifest_path: installedManifestPath,
  manifest_sha256: installedManifestPath ? sha256(installedManifestPath) : null,
  manifest_version: installedManifest?.release_version ?? null,
  agent_context_path: existsSync(agentContextDir) ? agentContextDir : null,
  agent_context_manifest_sha256: sha256(join(agentContextDir, "distribution-manifest.json")),
  release_manifest_path: releaseManifestPath,
  release_manifest_sha256: releaseManifestPath ? sha256(releaseManifestPath) : null,
  agent_context_docs_present: existsSync(join(agentContextDir, "docs/current")),
  component_digests: installedComponents,
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
const sourceManifestCheckRaw = run("python3", [
  join(ROOT, "scripts/distribution_manifest.py"),
  "--root",
  ROOT,
  "--manifest",
  sourceManifestPath,
  "--check",
  "--json",
]);
const sourceManifestCheck = sourceManifestCheckRaw ? JSON.parse(sourceManifestCheckRaw) : null;
if (!sourceManifestCheck?.ok) {
  drift.push({
    surface: "source_distribution_manifest",
    expected: "current full SHA-256 component contract",
    source_value: sourceManifest?.source_commit ?? null,
    observed_value: sourceManifestCheck?.failures?.join("; ") ?? "validation unavailable",
  });
}
if (!installedManifestPath || !installedManifest) {
  drift.push({
    surface: "installed_distribution_manifest",
    expected: runtimeContract.installed_manifest_path ?? "/usr/local/lib/focusa/distribution-manifest.json",
    source_value: sourceManifest?.release_version ?? null,
    observed_value: "missing_or_invalid",
  });
} else {
  const sourceManifestDigest = sha256(sourceManifestPath);
  if (sourceManifestDigest !== installed.manifest_sha256) {
    drift.push({
      surface: "installed_distribution_manifest",
      expected: "byte-identical signed source manifest",
      source_value: sourceManifestDigest,
      observed_value: installed.manifest_sha256,
    });
  }
  if (!installed.agent_context_manifest_sha256) {
    drift.push({
      surface: "agent_context_distribution_manifest",
      expected: "installed agent context manifest",
      source_value: installed.manifest_sha256,
      observed_value: "missing",
    });
  } else if (installed.agent_context_manifest_sha256 !== installed.manifest_sha256) {
    drift.push({
      surface: "agent_context_distribution_manifest",
      expected: "same manifest as installed runtime",
      source_value: installed.manifest_sha256,
      observed_value: installed.agent_context_manifest_sha256,
    });
  }
}
if (
  releaseManifest &&
  (releaseManifest.schema !== "focusa.release_manifest.v1" ||
    releaseManifest.tag !== `v${sourceManifest?.release_version}`)
) {
  drift.push({
    surface: "signed_release_identity",
    expected: `v${sourceManifest?.release_version}`,
    source_value: releaseManifest.tag ?? null,
    observed_value: releaseManifest.commit ?? null,
  });
}
if (releaseManifest) {
  const releasedManifestDigest = releaseManifest.assets?.["distribution-manifest.json"]?.sha256;
  const sourceManifestDigest = sha256(sourceManifestPath);
  if (!releasedManifestDigest || sourceManifestDigest !== `sha256:${releasedManifestDigest}`) {
    drift.push({
      surface: "released_distribution_manifest_digest",
      expected: "signed release asset digest",
      source_value: releasedManifestDigest ? `sha256:${releasedManifestDigest}` : "release_asset_missing",
      observed_value: sourceManifestDigest,
    });
  }
}
const releaseBinaryNames = {
  cli: "focusa",
  daemon: "focusa-daemon",
  tui: "focusa-tui",
  session_runner: "focusa-session-runner",
};
for (const [surface, path] of Object.entries(binaryPaths)) {
  if (!installed.binary_digests[surface] || !installed.binary_versions[surface]) {
    drift.push({
      surface: `installed_binary:${surface}`,
      expected: path,
      source_value: sourceManifest?.release_version ?? null,
      observed_value: "missing_or_unexecutable",
    });
  } else if (
    installed.manifest_version &&
    installed.binary_versions[surface] !== installed.manifest_version
  ) {
    drift.push({
      surface: `installed_binary:${surface}`,
      expected: "installed distribution manifest version",
      source_value: installed.manifest_version,
      observed_value: installed.binary_versions[surface],
    });
  }
  if (releaseManifest && releaseAssetSuffix) {
    const assetName = `${releaseBinaryNames[surface]}-${releaseManifest.tag}-${releaseAssetSuffix}`;
    const releasedDigest = releaseManifest.assets?.[assetName]?.sha256 ?? null;
    if (!releasedDigest || installed.binary_digests[surface] !== `sha256:${releasedDigest}`) {
      drift.push({
        surface: `installed_binary_digest:${surface}`,
        expected: assetName,
        source_value: releasedDigest ? `sha256:${releasedDigest}` : "release_asset_missing",
        observed_value: installed.binary_digests[surface],
      });
    }
  }
}
if (!releaseManifest || !releaseAssetSuffix) {
  drift.push({
    surface: "signed_release_binary_contract",
    expected: "FOCUSA_RELEASE_MANIFEST and FOCUSA_RELEASE_ASSET_SUFFIX",
    source_value: sourceManifest?.release_version ?? null,
    observed_value: "release_contract_missing",
  });
}
if (!installed.agent_context_path || !installed.agent_context_docs_present) {
  drift.push({
    surface: "installed_agent_documentation",
    expected: "agent-context/docs/current",
    source_value: sourceManifest?.components?.documentation?.sha256 ?? null,
    observed_value: installed.agent_context_path ? "docs_missing" : "agent_context_missing",
  });
}
for (const component of ["pi_extension", "agent_skills", "documentation", "generated_clients"]) {
  const expected = sourceManifest?.components?.[component];
  const observed = installedComponents[component];
  if (!expected || !observed || observed.sha256 !== expected.sha256 || observed.file_count !== expected.file_count) {
    drift.push({
      surface: `installed_component:${component}`,
      expected: "full SHA-256 tree and file-count parity",
      source_value: expected ? `${expected.sha256}:${expected.file_count}` : "source_contract_missing",
      observed_value: observed ? `${observed.sha256}:${observed.file_count}` : "installed_component_missing",
      detail: observed?.error ?? null,
    });
  }
}
if (!piExtDir) {
  drift.push({
    surface: "installed_pi_extension",
    expected: "one active focusa-pi-bridge package",
    source_value: source.extension_version,
    observed_value: "missing",
  });
}

// The complete Pi-extension and documentation tree contracts above include
// runtime tool sources and the inert capability registry. Do not reimplement
// those digest checks per file or execute installed TypeScript during audit.
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

if (live.daemon_ok !== true || !live.daemon_version) {
  drift.push({
    surface: "live_daemon_health",
    expected: "healthy daemon with exact version",
    source_value: installed.manifest_version,
    observed_value: live.daemon_version ?? "unreachable_or_unhealthy",
  });
} else if (installed.manifest_version && live.daemon_version !== installed.manifest_version) {
  drift.push({
    surface: "live_daemon_version",
    expected: "installed distribution manifest version",
    source_value: installed.manifest_version,
    observed_value: live.daemon_version,
  });
}

const manifest = {
  schema: "focusa.distribution_parity.v1",
  generated_at: new Date().toISOString(),
  source,
  source_manifest: {
    path: sourceManifestPath,
    sha256: sha256(sourceManifestPath),
    release_version: sourceManifest?.release_version ?? null,
    source_commit: sourceManifest?.source_commit ?? null,
    check: sourceManifestCheck,
  },
  installed,
  live,
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
