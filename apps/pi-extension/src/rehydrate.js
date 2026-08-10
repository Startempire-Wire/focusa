const HANDLE_ID = /^[A-Za-z0-9._:-]{1,160}$/;
const DEFAULT_MAX_TOKENS = 300;
const MAX_TOKENS = 4000;
export function parseRehydrateArgs(args) {
    const [handleId = "", maxTokensRaw] = String(args || "").trim().split(/\s+/, 3);
    if (!HANDLE_ID.test(handleId)) {
        throw new Error("Usage: /focusa-rehydrate <handle_id> [max_tokens]");
    }
    const parsed = maxTokensRaw === undefined ? DEFAULT_MAX_TOKENS : Number(maxTokensRaw);
    if (!Number.isInteger(parsed) || parsed < 1) {
        throw new Error("max_tokens must be a positive integer");
    }
    return { handleId, maxTokens: Math.min(parsed, MAX_TOKENS) };
}
function clipLocal(content, maxTokens) {
    const maxChars = maxTokens * 4;
    if (content.length <= maxChars)
        return { content, truncated: false };
    return { content: `${content.slice(0, maxChars)}…`, truncated: true };
}
export async function rehydrateHandle(args, dependencies) {
    const { handleId, maxTokens } = parseRehydrateArgs(args);
    for (const kind of ["text", "report-summary"]) {
        const local = dependencies.getLocal(kind, handleId);
        if (local !== null) {
            const clipped = clipLocal(local, maxTokens);
            return { handleId, source: "local", ...clipped, originalSize: local.length };
        }
    }
    const response = (await dependencies.fetchRemote(`/ecs/rehydrate/${encodeURIComponent(handleId)}?max_tokens=${maxTokens}`, { method: "POST" }));
    if (!response || typeof response.content !== "string") {
        throw new Error(`Handle ${handleId} is unavailable locally and ECS did not return content`);
    }
    return {
        handleId,
        content: response.content,
        source: "ecs",
        truncated: response.truncated === true,
        originalSize: typeof response.original_size === "number" ? response.original_size : undefined,
    };
}
