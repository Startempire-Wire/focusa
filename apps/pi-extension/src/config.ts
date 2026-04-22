// Config loading: .pi/settings.json → env vars → defaults
// Spec: §10.2 (keys), §18 (schema), §19 (env vars), §23 (presets), §25.1 (validation)

import { mkdirSync, readFileSync, writeFileSync } from "fs";
import { dirname, join } from "path";

const RESERVED_PI_KEYS = new Set(["extensions", "skills", "prompts", "themes", "packages"]);

function isPlainObject(value: unknown): value is Record<string, any> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

export interface FocusaConfig {
  enabled: boolean;
  warnPct: number;
  compactPct: number;
  hardPct: number;
  contextStatusMode: "off" | "actionable" | "all";
  cooldownMs: number;
  maxCompactionsPerHour: number;
  minTurnsBetweenCompactions: number;
  compactInstructions: string;
  externalizeThresholdBytes: number;
  externalizeThresholdTokens: number;
  focusaApiBaseUrl: string;
  focusaApiTimeoutMs: number;
  fallbackMode: "passthrough" | "local-compact" | "disabled";
  emitMetrics: boolean;
  autoSuggestForkPct: number;
  autoSuggestHandoffAfterNCompactions: number;
  workLoopPreset: "conservative" | "balanced" | "push" | "audit";
  workLoopMaxTurns: number;
  workLoopMaxWallClockMs: number;
  workLoopMaxRetries: number;
  workLoopCooldownMs: number;
  workLoopAllowDestructiveActions: boolean;
  workLoopRequireOperatorForGovernance: boolean;
  workLoopRequireOperatorForScopeChange: boolean;
  workLoopRequireVerificationBeforePersist: boolean;
  workLoopMaxConsecutiveLowProductivityTurns: number;
  workLoopMaxConsecutiveFailures: number;
  workLoopAutoPauseOnOperatorMessage: boolean;
  workLoopRequireExplainableContinueReason: boolean;
  workLoopMaxSameSubproblemRetries: number;
  workLoopStatusHeartbeatMs: number;
  // Bridge sync/perf mode
  bridgeSyncMode: "event-driven" | "polling";
  bridgePollMs: number;
  // Service URLs (§38.2 multi-machine)
  scoreboardUrl: string;
  scoreboardToken: string;
  contextCoreUrl: string;
  wikiUrl: string;
  focusaToken: string;
  // Micro-compact (§21)
  microCompactEveryNTurns: number;
}

// §23: Tuning presets
const PRESETS: Record<string, Partial<FocusaConfig>> = {
  balanced: { warnPct: 50, compactPct: 70, hardPct: 85, cooldownMs: 180_000, maxCompactionsPerHour: 8 },
  aggressive: { warnPct: 40, compactPct: 60, hardPct: 75, cooldownMs: 120_000, maxCompactionsPerHour: 12 },
  conservative: { warnPct: 60, compactPct: 80, hardPct: 92, cooldownMs: 300_000, maxCompactionsPerHour: 5 },
};

const DEFAULTS: FocusaConfig = {
  enabled: true,
  warnPct: 50,
  compactPct: 70,
  hardPct: 85,
  contextStatusMode: "actionable",
  cooldownMs: 180_000,
  maxCompactionsPerHour: 8,
  minTurnsBetweenCompactions: 3,
  compactInstructions: "Preserve intent, decisions, constraints, next_steps, failures. Prefer handles over blobs.",
  externalizeThresholdBytes: 8192,
  externalizeThresholdTokens: 800,
  focusaApiBaseUrl: "http://127.0.0.1:8787/v1",
  focusaApiTimeoutMs: 5000,
  fallbackMode: "passthrough",
  emitMetrics: true,
  autoSuggestForkPct: 90,
  autoSuggestHandoffAfterNCompactions: 3,
  workLoopPreset: "balanced",
  workLoopMaxTurns: 12,
  workLoopMaxWallClockMs: 1_800_000,
  workLoopMaxRetries: 3,
  workLoopCooldownMs: 1_000,
  workLoopAllowDestructiveActions: false,
  workLoopRequireOperatorForGovernance: true,
  workLoopRequireOperatorForScopeChange: true,
  workLoopRequireVerificationBeforePersist: true,
  workLoopMaxConsecutiveLowProductivityTurns: 3,
  workLoopMaxConsecutiveFailures: 3,
  workLoopAutoPauseOnOperatorMessage: false,
  workLoopRequireExplainableContinueReason: true,
  workLoopMaxSameSubproblemRetries: 2,
  workLoopStatusHeartbeatMs: 5_000,
  bridgeSyncMode: "event-driven",
  bridgePollMs: 15_000,
  scoreboardUrl: "http://127.0.0.1:8100",
  scoreboardToken: "",
  contextCoreUrl: "http://127.0.0.1:7400",
  wikiUrl: "http://127.0.0.1:7325",
  focusaToken: "",
  microCompactEveryNTurns: 5,
};

// §19: Env var mapping (FOCUSA_PI_ prefix)
const ENV_MAP: Record<string, keyof FocusaConfig> = {
  FOCUSA_PI_ENABLED: "enabled",
  FOCUSA_PI_WARN_PCT: "warnPct",
  FOCUSA_PI_COMPACT_PCT: "compactPct",
  FOCUSA_PI_HARD_PCT: "hardPct",
  FOCUSA_PI_CONTEXT_STATUS_MODE: "contextStatusMode",
  FOCUSA_PI_COOLDOWN_MS: "cooldownMs",
  FOCUSA_PI_MAX_COMPACTIONS_PER_HOUR: "maxCompactionsPerHour",
  FOCUSA_PI_MIN_TURNS_BETWEEN_COMPACTIONS: "minTurnsBetweenCompactions",
  FOCUSA_PI_COMPACT_INSTRUCTIONS: "compactInstructions",
  FOCUSA_PI_EXTERNALIZE_BYTES: "externalizeThresholdBytes",
  FOCUSA_PI_EXTERNALIZE_TOKENS: "externalizeThresholdTokens",
  FOCUSA_PI_API_BASE_URL: "focusaApiBaseUrl",
  FOCUSA_PI_API_TIMEOUT_MS: "focusaApiTimeoutMs",
  FOCUSA_PI_FALLBACK_MODE: "fallbackMode",
  FOCUSA_PI_EMIT_METRICS: "emitMetrics",
  FOCUSA_PI_AUTO_SUGGEST_FORK_PCT: "autoSuggestForkPct",
  FOCUSA_PI_AUTO_SUGGEST_HANDOFF_AFTER: "autoSuggestHandoffAfterNCompactions",
  FOCUSA_PI_WORK_LOOP_PRESET: "workLoopPreset",
  FOCUSA_PI_WORK_LOOP_MAX_TURNS: "workLoopMaxTurns",
  FOCUSA_PI_WORK_LOOP_MAX_WALL_CLOCK_MS: "workLoopMaxWallClockMs",
  FOCUSA_PI_WORK_LOOP_MAX_RETRIES: "workLoopMaxRetries",
  FOCUSA_PI_WORK_LOOP_COOLDOWN_MS: "workLoopCooldownMs",
  FOCUSA_PI_WORK_LOOP_ALLOW_DESTRUCTIVE_ACTIONS: "workLoopAllowDestructiveActions",
  FOCUSA_PI_WORK_LOOP_REQUIRE_OPERATOR_FOR_GOVERNANCE: "workLoopRequireOperatorForGovernance",
  FOCUSA_PI_WORK_LOOP_REQUIRE_OPERATOR_FOR_SCOPE_CHANGE: "workLoopRequireOperatorForScopeChange",
  FOCUSA_PI_WORK_LOOP_REQUIRE_VERIFICATION_BEFORE_PERSIST: "workLoopRequireVerificationBeforePersist",
  FOCUSA_PI_WORK_LOOP_MAX_LOW_PRODUCTIVITY_TURNS: "workLoopMaxConsecutiveLowProductivityTurns",
  FOCUSA_PI_WORK_LOOP_MAX_CONSECUTIVE_FAILURES: "workLoopMaxConsecutiveFailures",
  FOCUSA_PI_WORK_LOOP_AUTO_PAUSE_ON_OPERATOR_MESSAGE: "workLoopAutoPauseOnOperatorMessage",
  FOCUSA_PI_WORK_LOOP_REQUIRE_EXPLAINABLE_CONTINUE_REASON: "workLoopRequireExplainableContinueReason",
  FOCUSA_PI_WORK_LOOP_MAX_SAME_SUBPROBLEM_RETRIES: "workLoopMaxSameSubproblemRetries",
  FOCUSA_PI_WORK_LOOP_STATUS_HEARTBEAT_MS: "workLoopStatusHeartbeatMs",
  FOCUSA_PI_BRIDGE_SYNC_MODE: "bridgeSyncMode",
  FOCUSA_PI_BRIDGE_POLL_MS: "bridgePollMs",
  FOCUSA_PI_MICRO_COMPACT_TURNS: "microCompactEveryNTurns",
  SCOREBOARD_URL: "scoreboardUrl",
  SCOREBOARD_TOKEN: "scoreboardToken",
  CONTEXT_CORE_URL: "contextCoreUrl",
  WIKI_URL: "wikiUrl",
  FOCUSA_TOKEN: "focusaToken",
};

// §25.1: Validation
function validate(cfg: FocusaConfig): string[] {
  const errs: string[] = [];
  if (!(0 < cfg.warnPct && cfg.warnPct < cfg.compactPct && cfg.compactPct < cfg.hardPct && cfg.hardPct < 100))
    errs.push(`Invalid tier ordering: 0 < warnPct(${cfg.warnPct}) < compactPct(${cfg.compactPct}) < hardPct(${cfg.hardPct}) < 100`);
  if (!["off", "actionable", "all"].includes(cfg.contextStatusMode))
    errs.push(`contextStatusMode(${cfg.contextStatusMode}) must be one of: off, actionable, all`);
  if (cfg.cooldownMs < 30_000) errs.push(`cooldownMs(${cfg.cooldownMs}) must be >= 30000`);
  if (cfg.maxCompactionsPerHour < 1) errs.push(`maxCompactionsPerHour must be >= 1`);
  if (cfg.externalizeThresholdBytes < 2048) errs.push(`externalizeThresholdBytes must be >= 2048`);
  if (cfg.externalizeThresholdTokens < 200) errs.push(`externalizeThresholdTokens must be >= 200`);
  if (!["conservative", "balanced", "push", "audit"].includes(cfg.workLoopPreset))
    errs.push(`workLoopPreset(${cfg.workLoopPreset}) must be one of: conservative, balanced, push, audit`);
  if (cfg.workLoopMaxTurns < 1) errs.push(`workLoopMaxTurns must be >= 1`);
  if (cfg.workLoopMaxWallClockMs < 60_000) errs.push(`workLoopMaxWallClockMs must be >= 60000`);
  if (cfg.workLoopMaxRetries < 0) errs.push(`workLoopMaxRetries must be >= 0`);
  if (cfg.workLoopCooldownMs < 0) errs.push(`workLoopCooldownMs must be >= 0`);
  if (cfg.workLoopMaxConsecutiveLowProductivityTurns < 1) errs.push(`workLoopMaxConsecutiveLowProductivityTurns must be >= 1`);
  if (cfg.workLoopMaxConsecutiveFailures < 1) errs.push(`workLoopMaxConsecutiveFailures must be >= 1`);
  if (cfg.workLoopMaxSameSubproblemRetries < 0) errs.push(`workLoopMaxSameSubproblemRetries must be >= 0`);
  if (cfg.workLoopStatusHeartbeatMs < 1_000) errs.push(`workLoopStatusHeartbeatMs must be >= 1000`);
  if (!["event-driven", "polling"].includes(cfg.bridgeSyncMode))
    errs.push(`bridgeSyncMode(${cfg.bridgeSyncMode}) must be one of: event-driven, polling`);
  if (cfg.bridgePollMs < 5_000) errs.push(`bridgePollMs(${cfg.bridgePollMs}) must be >= 5000`);
  return errs;
}

function readSettingsFile(path: string): any {
  try {
    const raw = JSON.parse(readFileSync(path, "utf-8"));
    return isPlainObject(raw) ? raw : {};
  } catch {
    return {};
  }
}

function extractFocusaConfig(raw: any): Partial<FocusaConfig> {
  const rootConfig = isPlainObject(raw?.focusaPiBridge) ? raw.focusaPiBridge : null;
  const legacyConfig = isPlainObject(raw?.extensions) && isPlainObject(raw.extensions?.focusaPiBridge)
    ? raw.extensions.focusaPiBridge
    : null;
  const ext = rootConfig ?? legacyConfig;
  if (!ext) return {};
  let fileConfig: Partial<FocusaConfig> = {};
  if (ext.preset && PRESETS[ext.preset]) fileConfig = { ...PRESETS[ext.preset] };
  return { ...fileConfig, ...ext };
}

function resolveSettingsPaths(cwd?: string): string[] {
  return [cwd ? join(cwd, ".pi/settings.json") : "", join(process.env.HOME || "", ".pi/agent/settings.json")].filter(Boolean);
}

// Load config: §18 precedence: env vars > settings.json > defaults
export function loadConfig(cwd?: string): { config: FocusaConfig; errors: string[] } {
  let fileConfig: Partial<FocusaConfig> = {};

  for (const p of resolveSettingsPaths(cwd)) {
    const raw = readSettingsFile(p);
    const ext = extractFocusaConfig(raw);
    if (Object.keys(ext).length > 0) {
      fileConfig = ext;
      break;
    }
  }

  // Merge: defaults → file → env
  const cfg: FocusaConfig = { ...DEFAULTS, ...fileConfig };

  // §19: Env var overrides (highest precedence)
  for (const [envKey, cfgKey] of Object.entries(ENV_MAP)) {
    const val = process.env[envKey];
    if (val === undefined) continue;
    const d = DEFAULTS[cfgKey];
    if (typeof d === "boolean") (cfg as any)[cfgKey] = val === "true";
    else if (typeof d === "number") (cfg as any)[cfgKey] = Number(val);
    else (cfg as any)[cfgKey] = val;
  }

  const errors = validate(cfg);
  return { config: cfg, errors };
}

export function saveConfigOverrides(cwd: string | undefined, overrides: Partial<FocusaConfig>, scope: "project" | "user" = "project"):
  { config: FocusaConfig; errors: string[]; path: string } {
  const path = scope === "project" && cwd
    ? join(cwd, ".pi/settings.json")
    : join(process.env.HOME || "", ".pi/agent/settings.json");

  const raw = readSettingsFile(path);
  if (!isPlainObject(raw)) throw new Error(`Refusing to write Focusa config into non-object settings file: ${path}`);
  if (RESERVED_PI_KEYS.has("extensions") && isPlainObject(raw.extensions) && "focusaPiBridge" in raw.extensions) {
    delete raw.extensions.focusaPiBridge;
    if (Object.keys(raw.extensions).length === 0) delete raw.extensions;
  }
  raw.focusaPiBridge = {
    ...(isPlainObject(raw.focusaPiBridge) ? raw.focusaPiBridge : {}),
    ...overrides,
  };

  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, `${JSON.stringify(raw, null, 2)}\n`, "utf-8");

  const { config, errors } = loadConfig(cwd);
  return { config, errors, path };
}

// §23: Get preset names
export function getPresetNames(): string[] { return Object.keys(PRESETS); }
export function getPreset(name: string): Partial<FocusaConfig> | undefined { return PRESETS[name]; }
