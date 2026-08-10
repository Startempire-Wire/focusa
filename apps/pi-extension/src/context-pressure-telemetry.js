function finiteNonNegative(value) {
    const candidate = Number(value);
    return Number.isFinite(candidate) && candidate >= 0 ? candidate : null;
}
/** Capture counts and provider-reported usage only; never copy prompt/tool text. */
export function contextPressureTelemetry(context, capabilities) {
    const ctx = context && typeof context === "object" ? context : {};
    const usage = typeof ctx.getContextUsage === "function" ? ctx.getContextUsage() : undefined;
    const branch = typeof ctx.sessionManager?.getBranch === "function" ? ctx.sessionManager.getBranch() : [];
    const entries = Array.isArray(branch) ? branch : [];
    const tokens = finiteNonNegative(usage?.tokens);
    const contextWindow = finiteNonNegative(usage?.contextWindow) || capabilities.contextWindow;
    const runtimePercent = finiteNonNegative(usage?.percent);
    const percent = runtimePercent !== null
        ? Math.min(100, runtimePercent)
        : tokens !== null && contextWindow
            ? Math.min(100, (tokens / contextWindow) * 100)
            : null;
    const messageEntryCount = entries.filter((entry) => entry?.type === "message").length;
    const toolResultCount = entries.filter((entry) => entry?.type === "message" && entry?.message?.role === "toolResult").length;
    const priorCompactionCount = entries.filter((entry) => entry?.type === "compaction").length;
    const knownSignals = [tokens !== null, contextWindow !== null, entries.length > 0];
    return {
        schema: "focusa.context_pressure_telemetry.v1",
        tokens,
        contextWindow,
        percent,
        branchEntryCount: entries.length,
        messageEntryCount,
        toolResultCount,
        priorCompactionCount,
        tokenSource: usage ? "pi_runtime" : "unknown",
        cacheBehavior: capabilities.cacheBehavior,
        contentIncluded: false,
        groundingStatus: knownSignals.every(Boolean)
            ? "grounded"
            : knownSignals.some(Boolean)
                ? "partial"
                : "unverified",
    };
}
