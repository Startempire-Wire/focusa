import { selectCompactionPolicy } from "./compaction-policy-selector.js";
import { focusaFetch, focusaPost } from "./state.js";
const cache = new Map();
const MAX_CACHE_ENTRIES = 32;
function runtimeKey(ctx) {
    const model = ctx?.model ?? {};
    return JSON.stringify([
        ctx.sessionManager.getSessionId(),
        ctx.cwd,
        model.provider ?? model.providerId ?? null,
        model.id ?? model.modelId ?? model.name ?? null,
        model.contextWindow ?? null,
    ]);
}
function bounded(value, max) {
    const text = String(value ?? "").trim();
    return text ? text.slice(0, max) : null;
}
export async function prewarmCompactionPolicy(ctx, config) {
    const key = runtimeKey(ctx);
    const model = ctx?.model ?? {};
    const response = await focusaFetch("/compaction/policy/resolve", {
        method: "POST",
        body: JSON.stringify({
            schema: "focusa.compaction_policy_resolve_request.v1",
            mode: config?.compactionPolicyMode ?? "shadow",
            objective_profile: config?.bloatgaurdProfile ?? "daily_driver",
            sample_size: 0,
            confidence: config?.compactionAdaptiveConfidence ?? 0.95,
            minimum_samples: config?.compactionAdaptiveMinSamples ?? 20,
            required_confidence: config?.compactionAdaptiveConfidence ?? 0.95,
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
    const lease = response?.lease;
    const policy = response?.resolution?.selected;
    const runtimeSegment = String(response?.runtime_fingerprint?.segment_key ?? "");
    const workstreamHash = String(response?.workstream_hash ?? "");
    if (!lease?.lease_id || !policy?.policy_id || !runtimeSegment || !workstreamHash)
        return;
    cache.set(key, { runtimeKey: key, runtimeSegment, workstreamHash, lease, policy });
    while (cache.size > MAX_CACHE_ENTRIES)
        cache.delete(cache.keys().next().value);
}
/** Apply one frozen Rust lease. No candidate scoring or capability inference occurs here. */
export function selectFrozenCompactionPolicy(ctx, telemetry, capabilities) {
    const cached = cache.get(runtimeKey(ctx));
    if (!cached)
        return selectCompactionPolicy(telemetry, capabilities);
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
function selection(cached, route, executionOwner, percent) {
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
export function observeFrozenCompactionOutcome(ctx, input) {
    const cached = cache.get(runtimeKey(ctx));
    if (!cached)
        return;
    focusaPost("/compaction/policy/observe", {
        schema: "focusa.compaction_policy_observe_request.v1",
        observation: {
            schema: "focusa.compaction_policy_observation.v1",
            runtime_segment: cached.runtimeSegment,
            workstream_hash: cached.workstreamHash,
            epoch_id: input.epochId,
            policy_id: cached.policy.policy_id,
            trigger_class: input.triggerClass,
            tokens_before: Math.max(0, input.tokensBefore ?? 0),
            tokens_after: input.tokensAfter === null ? null : Math.max(0, input.tokensAfter),
            context_release_ratio: input.tokensBefore && input.tokensAfter !== null
                ? Math.max(0, Math.min(1, (input.tokensBefore - input.tokensAfter) / input.tokensBefore))
                : null,
            projection_tokens: Math.max(0, input.projectionTokens),
            prepare_latency_ms: null,
            compaction_latency_ms: null,
            verify_latency_ms: null,
            first_productive_action_ms: null,
            workpoint_revision_delta: 0,
            repeat_error_delta: 0,
            rehydrate_calls: 0,
            rehydrated_bytes: 0,
            hard_findings: input.hardFindings.slice(0, 32),
            rollback_triggered: input.rollbackTriggered,
        },
        promotion: null,
        drift: null,
    });
}
export function clearCompactionPolicyCache() {
    cache.clear();
}
