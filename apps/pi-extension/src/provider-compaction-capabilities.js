function text(value) {
    const candidate = String(value || "").trim();
    return candidate ? candidate.slice(0, 160) : null;
}
function positiveInteger(value) {
    const candidate = Number(value);
    return Number.isFinite(candidate) && candidate > 0 ? Math.floor(candidate) : null;
}
/**
 * Inventory only runtime-observable capability. Unknown provider behavior must
 * stay unknown; provider/model names never imply token, cache, or compaction
 * semantics.
 */
export function providerCompactionCapabilities(context) {
    const ctx = context && typeof context === "object" ? context : {};
    const model = ctx.model && typeof ctx.model === "object" ? ctx.model : {};
    const providerId = text(model.provider || model.providerId || ctx.provider);
    const modelId = text(model.id || model.modelId || model.name);
    const contextWindow = positiveInteger(model.contextWindow);
    const tokenAccounting = typeof ctx.getContextUsage === "function" ? "runtime_observed" : "unknown";
    const nativeCompaction = typeof ctx.compact === "function" ? "supported" : "unknown";
    const cacheBehavior = model.cacheBehavior === "explicit" || model.cacheBehavior === "implicit"
        ? model.cacheBehavior
        : "unknown";
    const evidenceRefs = [
        providerId ? "pi:model.provider" : null,
        modelId ? "pi:model.id" : null,
        contextWindow ? "pi:model.contextWindow" : null,
        tokenAccounting !== "unknown" ? "pi:getContextUsage" : null,
        nativeCompaction === "supported" ? "pi:context.compact" : null,
        cacheBehavior !== "unknown" ? "pi:model.cacheBehavior" : null,
    ].filter((value) => Boolean(value));
    const knownCore = [contextWindow !== null, tokenAccounting !== "unknown", nativeCompaction !== "unknown"];
    const groundingStatus = knownCore.every(Boolean)
        ? "grounded"
        : knownCore.some(Boolean) || Boolean(providerId || modelId)
            ? "partial"
            : "unverified";
    return {
        schema: "focusa.provider_compaction_capabilities.v1",
        providerId,
        modelId,
        contextWindow,
        tokenAccounting,
        nativeCompaction,
        cacheBehavior,
        groundingStatus,
        evidenceRefs,
    };
}
