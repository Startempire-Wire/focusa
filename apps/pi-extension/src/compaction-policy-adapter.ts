import type { ExtensionContext } from "@earendil-works/pi-coding-agent";

import type { FocusaConfig } from "./config.js";
import type { ContextPressureTelemetry } from "./context-pressure-telemetry.js";
import type { ProviderCompactionCapabilities } from "./provider-compaction-capabilities.js";
import { selectCompactionPolicy, type CompactionPolicySelection } from "./compaction-policy-selector.js";
import { focusaFetch } from "./state.js";

interface RustPolicyLease {
  lease_id: string;
  policy_id: string;
  policy_revision: string;
  fallback_policy_id: string;
}

interface RustPolicyBundle {
  policy_id: string;
  revision: string;
  checkpoint_at_tokens: number;
  compact_at_tokens: number | null;
  hard_at_tokens: number;
  actions: string[];
}

interface CachedResolution {
  runtimeKey: string;
  lease: RustPolicyLease;
  policy: RustPolicyBundle;
}

const cache = new Map<string, CachedResolution>();
const MAX_CACHE_ENTRIES = 32;

function runtimeKey(ctx: ExtensionContext): string {
  const model = (ctx as any)?.model ?? {};
  return JSON.stringify([
    ctx.sessionManager.getSessionId(),
    ctx.cwd,
    model.provider ?? model.providerId ?? null,
    model.id ?? model.modelId ?? model.name ?? null,
    model.contextWindow ?? null,
  ]);
}

function bounded(value: unknown, max: number): string | null {
  const text = String(value ?? "").trim();
  return text ? text.slice(0, max) : null;
}

export async function prewarmCompactionPolicy(
  ctx: ExtensionContext,
  config?: Pick<FocusaConfig, "bloatgaurdProfile">
): Promise<void> {
  const key = runtimeKey(ctx);
  const model = (ctx as any)?.model ?? {};
  const response = await focusaFetch("/compaction/policy/resolve", {
    method: "POST",
    body: JSON.stringify({
      schema: "focusa.compaction_policy_resolve_request.v1",
      mode: "shadow",
      objective_profile: config?.bloatgaurdProfile ?? "daily_driver",
      runtime_facts: {
        provider_raw: bounded(model.provider ?? model.providerId, 160),
        api: bounded(model.api, 120),
        model_id_raw: bounded(model.id ?? model.modelId ?? model.name, 256),
        response_model: null,
        endpoint_class: bounded(model.baseUrlClass, 120),
        api_version: bounded(model.apiVersion, 120),
        beta_features: Array.isArray(model.betaFeatures) ? model.betaFeatures.slice(0, 32) : [],
        adapter_revision: "focusa-pi-bridge@0.9.143",
        capability_evidence_revision: "daemon-registry",
        context_window: Number.isFinite(Number(model.contextWindow)) ? Number(model.contextWindow) : null,
        max_output_tokens: Number.isFinite(Number(model.maxOutputTokens))
          ? Number(model.maxOutputTokens)
          : null,
        reasoning_enabled: typeof model.reasoning === "boolean" ? model.reasoning : null,
        transport: bounded(model.transport, 120),
        state_mode: bounded(model.stateMode, 120),
        cache_mode: bounded(model.cacheBehavior, 120),
        harness_mode: "pi",
        objective_profile: config?.bloatgaurdProfile ?? "daily_driver",
        session_id: ctx.sessionManager.getSessionId(),
        attachment_id: ctx.sessionManager.getSessionFile() ?? ctx.sessionManager.getSessionId(),
        project_root: null,
        continuity_id: null,
      },
    }),
  });
  const lease = response?.lease as RustPolicyLease | undefined;
  const policy = response?.resolution?.selected as RustPolicyBundle | undefined;
  if (!lease?.lease_id || !policy?.policy_id) return;
  cache.set(key, { runtimeKey: key, lease, policy });
  while (cache.size > MAX_CACHE_ENTRIES) cache.delete(cache.keys().next().value!);
}

/** Apply one frozen Rust lease. No candidate scoring or capability inference occurs here. */
export function selectFrozenCompactionPolicy(
  ctx: ExtensionContext,
  telemetry: ContextPressureTelemetry,
  capabilities: ProviderCompactionCapabilities
): CompactionPolicySelection {
  const cached = cache.get(runtimeKey(ctx));
  if (!cached) return selectCompactionPolicy(telemetry, capabilities);
  const percent = telemetry.percent;
  const contextWindow = capabilities.contextWindow;
  if (percent === null || contextWindow === null) {
    return selection(cached, "no_op", "none", percent);
  }
  const usedTokens = Math.floor((percent / 100) * contextWindow);
  if (usedTokens < cached.policy.checkpoint_at_tokens) {
    return selection(cached, "no_op", "none", percent);
  }
  if (usedTokens < (cached.policy.compact_at_tokens ?? Number.MAX_SAFE_INTEGER)) {
    return selection(cached, "checkpoint", "focusa", percent);
  }
  if (usedTokens >= cached.policy.hard_at_tokens && capabilities.nativeCompaction !== "supported") {
    return selection(cached, "rollover", "operator", percent);
  }
  return capabilities.nativeCompaction === "supported"
    ? selection(cached, "native_compact", "pi", percent)
    : selection(cached, "rollover", "operator", percent);
}

function selection(
  cached: CachedResolution,
  route: CompactionPolicySelection["route"],
  executionOwner: CompactionPolicySelection["executionOwner"],
  percent: number | null
): CompactionPolicySelection {
  return {
    schema: "focusa.compaction_policy_selection.v1",
    policyVersion: `rust:${cached.policy.revision}`,
    route,
    executionOwner,
    reason: "rust_policy_lease",
    percent,
    deterministicKey: `${cached.lease.lease_id}:${route}`,
  };
}

export function clearCompactionPolicyCache(): void {
  cache.clear();
}
