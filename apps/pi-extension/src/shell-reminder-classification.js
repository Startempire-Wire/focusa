const SHELL_TOOLS = new Set(["bash", "sh", "fish", "zsh", "csh", "dash"]);
function commandText(args) {
    if (typeof args === "string")
        return args;
    if (!args || typeof args !== "object")
        return "";
    const record = args;
    for (const key of ["command", "cmd", "script"]) {
        if (typeof record[key] === "string")
            return String(record[key]);
    }
    return "";
}
function equivalentForCommand(command) {
    const routeMappings = [
        [/\/v1\/project\/(?:identity|verify)\b/i, "focusa_project_identity"],
        [/\/v1\/trajectory\/(?:view|assess|define-goal|propose-workpoint)\b/i, "focusa_trajectory_view"],
        [/\/v1\/workpoint\/(?:resume|checkpoint|current|evidence)\b/i, "focusa_workpoint_resume"],
        [/\/v1\/(?:lineage|focus\/snapshots)\b/i, "focusa_tree_head"],
        [/\/v1\/focus\/(?:stack|update)\b/i, "focusa_context_cognition"],
        [/\/v1\/prediction(?:s|_authority)?\b/i, "focusa_predict_recent"],
        [/\/v1\/temporal\b/i, "focusa_temporal_authority"],
        [/\/v1\/work-loop\b/i, "focusa_work_loop_status"],
        [/\/v1\/(?:agent\/runtime|instructions)\b/i, "focusa_agent_runtime_effective"],
    ];
    return routeMappings.find(([pattern]) => pattern.test(command))?.[1] || null;
}
export function classifyShellReminderInteraction(toolName, args) {
    const tool = String(toolName || "").trim().toLowerCase();
    if (!SHELL_TOOLS.has(tool)) {
        return {
            classification: "shell_used",
            confidence: "high",
            equivalent_tool: null,
            reason: "not_a_shell_tool",
        };
    }
    const command = commandText(args).trim();
    if (!command) {
        return {
            classification: "shell_used",
            confidence: "uncertain",
            equivalent_tool: null,
            reason: "missing_command_text",
        };
    }
    const targetsFocusaApi = /(?:127\.0\.0\.1|localhost)(?::8787)?\/v1\//i.test(command);
    const rawHttpClient = /(?:^|[;&|\s])(curl|wget|http|xh)(?:\s|$)/i.test(command);
    const equivalentTool = equivalentForCommand(command);
    if (targetsFocusaApi && rawHttpClient && equivalentTool) {
        return {
            classification: "actual_focusa_bypass",
            confidence: "high",
            equivalent_tool: equivalentTool,
            reason: "raw_focusa_api_call_with_governed_equivalent",
        };
    }
    return {
        classification: "shell_used",
        confidence: targetsFocusaApi ? "uncertain" : "high",
        equivalent_tool: null,
        reason: targetsFocusaApi
            ? "focusa_like_command_without_reliable_equivalent"
            : "unrelated_shell_command",
    };
}
