import { Type } from "@sinclair/typebox";
import { focusaFetch } from "./state.js";
function toolResult(payload) {
    const body = payload ?? {
        ok: false,
        status: "unavailable",
        error: "agent_runtime_route_unavailable",
    };
    return {
        content: [{ type: "text", text: JSON.stringify(body, null, 2) }],
        details: body,
    };
}
async function get(path) {
    return toolResult(await focusaFetch(path));
}
async function post(path, body) {
    return toolResult(await focusaFetch(path, {
        method: "POST",
        body: JSON.stringify(body),
    }));
}
function projectQuery(projectRoot, maxSourceBytes) {
    const root = encodeURIComponent(projectRoot);
    const limit = Math.max(1, Math.floor(maxSourceBytes || 262_144));
    return `project_root=${root}&max_source_bytes=${limit}`;
}
export function registerAgentRuntimeTools(pi) {
    pi.registerTool({
        name: "focusa_agent_runtime_effective",
        label: "Focusa Agent Runtime Effective",
        description: "Read effective project instruction claims and unresolved conflicts under Spec 140.",
        parameters: Type.Object({
            project_root: Type.String({ description: "Verified absolute project root." }),
            max_source_bytes: Type.Optional(Type.Number({ minimum: 1 })),
        }),
        async execute(_id, params) {
            const p = params;
            return get(`/agent-runtime/instructions/effective?${projectQuery(p.project_root, p.max_source_bytes)}`);
        },
    });
    pi.registerTool({
        name: "focusa_instruction_sources",
        label: "Focusa Instruction Sources",
        description: "Discover bounded, registered project instruction sources with trust and authority metadata.",
        parameters: Type.Object({
            project_root: Type.String(),
            max_source_bytes: Type.Optional(Type.Number({ minimum: 1 })),
        }),
        async execute(_id, params) {
            const p = params;
            return get(`/agent-runtime/instructions/sources?${projectQuery(p.project_root, p.max_source_bytes)}`);
        },
    });
    pi.registerTool({
        name: "focusa_instruction_conflicts",
        label: "Focusa Instruction Conflicts",
        description: "Read deterministic instruction conflicts; unresolved equal-authority claims remain blocked.",
        parameters: Type.Object({
            project_root: Type.String(),
            max_source_bytes: Type.Optional(Type.Number({ minimum: 1 })),
        }),
        async execute(_id, params) {
            const p = params;
            return get(`/agent-runtime/instructions/conflicts?${projectQuery(p.project_root, p.max_source_bytes)}`);
        },
    });
    pi.registerTool({
        name: "focusa_instruction_explain",
        label: "Focusa Instruction Explain",
        description: "Explain one instruction claim from the current bounded source inventory.",
        parameters: Type.Object({
            project_root: Type.String(),
            claim_id: Type.String(),
            max_source_bytes: Type.Optional(Type.Number({ minimum: 1 })),
        }),
        async execute(_id, params) {
            const p = params;
            const response = await focusaFetch(`/agent-runtime/instructions/claims?${projectQuery(p.project_root, p.max_source_bytes)}`);
            const claim = response?.claims?.find((candidate) => candidate?.claim_id === p.claim_id);
            return toolResult(claim
                ? { schema: "focusa.instruction_explanation.v1", claim, source_linked: true }
                : { ok: false, status: "not_found", error: "instruction_claim_not_found" });
        },
    });
    pi.registerTool({
        name: "focusa_instruction_simulate",
        label: "Focusa Instruction Simulate",
        description: "Preview path/profile/target-specific instruction behavior without committing changes.",
        parameters: Type.Object({
            project_root: Type.String(),
            path: Type.Optional(Type.String()),
            profile: Type.Optional(Type.String()),
            target: Type.Optional(Type.String()),
            max_source_bytes: Type.Optional(Type.Number({ minimum: 1 })),
        }),
        async execute(_id, params) {
            return post("/agent-runtime/instructions/simulate", params);
        },
    });
    pi.registerTool({
        name: "focusa_runtime_constitution_preview",
        label: "Focusa Runtime Constitution Preview",
        description: "Preview a compiled Runtime Constitution without activation or artifact delivery.",
        parameters: Type.Object({
            constitution_id: Type.String(),
            request: Type.Any({ description: "Typed PromptCompileInput request." }),
        }),
        async execute(_id, params) {
            const p = params;
            return post(`/agent-runtime/constitutions/${encodeURIComponent(p.constitution_id)}/preview`, p.request);
        },
    });
    pi.registerTool({
        name: "focusa_prompt_variant_preview",
        label: "Focusa Prompt Variant Preview",
        description: "Compile and preview a target prompt variant without activation.",
        parameters: Type.Object({ request: Type.Any({ description: "Typed PromptCompileInput request." }) }),
        async execute(_id, params) {
            return post("/agent-runtime/compile/system-prompt", params.request);
        },
    });
    pi.registerTool({
        name: "focusa_prompt_variant_diff",
        label: "Focusa Prompt Variant Diff",
        description: "Compare two caller-supplied prompt variant projections without mutating Focusa state.",
        parameters: Type.Object({ baseline: Type.Any(), candidate: Type.Any() }),
        async execute(_id, params) {
            const p = params;
            return toolResult({
                schema: "focusa.prompt_variant_diff.v1",
                baseline_sha256: p.baseline?.variant?.prompt_sha256 || p.baseline?.prompt_sha256 || null,
                candidate_sha256: p.candidate?.variant?.prompt_sha256 || p.candidate?.prompt_sha256 || null,
                changed: JSON.stringify(p.baseline) !== JSON.stringify(p.candidate),
                advisory: true,
            });
        },
    });
    pi.registerTool({
        name: "focusa_agent_artifact_preview",
        label: "Focusa Agent Artifact Preview",
        description: "Preview a Spec 140 artifact delivery manifest; never writes files.",
        parameters: Type.Object({ request: Type.Any({ description: "Typed delivery preview request." }) }),
        async execute(_id, params) {
            return post("/agent-runtime/delivery/preview", params.request);
        },
    });
    pi.registerTool({
        name: "focusa_agent_artifact_delivery",
        label: "Focusa Agent Artifact Delivery",
        description: "Commit verified agent artifacts with explicit operator confirmation and a durable Receipt reference.",
        parameters: Type.Object({
            request: Type.Any({ description: "Typed delivery request." }),
            confirmed: Type.Boolean({ description: "Explicit operator confirmation." }),
        }),
        async execute(_id, params) {
            const p = params;
            return post("/agent-runtime/delivery/commit", {
                ...p.request,
                operator_confirmed: p.confirmed,
            });
        },
    });
    pi.registerTool({
        name: "focusa_agent_artifact_verify",
        label: "Focusa Agent Artifact Verify",
        description: "Verify content hashes and evidence for a Runtime Constitution delivery manifest.",
        parameters: Type.Object({ request: Type.Any({ description: "Typed delivery verification request." }) }),
        async execute(_id, params) {
            return post("/agent-runtime/delivery/verify", params.request);
        },
    });
    pi.registerTool({
        name: "focusa_instruction_integrity_evaluate",
        label: "Focusa Instruction Integrity Evaluate",
        description: "Evaluate the foundational headless InstructionIntegrityGuard and durably record its fail-closed decision.",
        parameters: Type.Object({ request: Type.Any({ description: "Typed integrity event envelope." }) }),
        async execute(_id, params) {
            return post("/agent-runtime/instruction-integrity/evaluate", params.request);
        },
    });
    pi.registerTool({
        name: "focusa_canonical_instruction_amendment_propose",
        label: "Focusa Canonical Instruction Amendment Propose",
        description: "Record an operator-originated canonical instruction amendment proposal without activating it.",
        parameters: Type.Object({ request: Type.Any({ description: "Typed amendment proposal envelope." }) }),
        async execute(_id, params) {
            return post("/agent-runtime/amendments/propose", params.request);
        },
    });
    pi.registerTool({
        name: "focusa_canonical_instruction_amendment_activate",
        label: "Focusa Canonical Instruction Amendment Activate",
        description: "Activate a separately operator-approved amendment only after its official documentation sweep is complete.",
        parameters: Type.Object({
            request: Type.Any({ description: "Typed approved amendment activation envelope." }),
            confirmed: Type.Boolean({ description: "Explicit operator confirmation for activation." }),
        }),
        async execute(_id, params) {
            const input = params;
            if (!input.confirmed) {
                const details = {
                    status: "blocked",
                    failure_class: "operator_confirmation_required",
                    recovery: ["obtain explicit operator confirmation and preserve the documentation sweep receipt"],
                };
                return {
                    content: [{ type: "text", text: JSON.stringify(details) }],
                    details,
                };
            }
            return post("/agent-runtime/amendments/activate", input.request);
        },
    });
    pi.registerTool({
        name: "focusa_agent_runtime_headless_verify",
        label: "Focusa Agent Runtime Headless Verify",
        description: "Verify foundational runtime capability parity without Mission Canvas or generated UI availability.",
        parameters: Type.Object({ request: Type.Any({ description: "Typed headless parity envelope." }) }),
        async execute(_id, params) {
            return post("/agent-runtime/headless/verify", params.request);
        },
    });
    pi.registerTool({
        name: "focusa_instruction_integrity_status",
        label: "Focusa Instruction Integrity Status",
        description: "Read foundational guard availability, amendment authority, and outage posture.",
        parameters: Type.Object({}),
        async execute() {
            return get("/agent-runtime/instruction-integrity/status");
        },
    });
    pi.registerTool({
        name: "focusa_agent_runtime_doctor",
        label: "Focusa Agent Runtime Doctor",
        description: "Diagnose Runtime Constitution compiler defaults, replacement gates, and delivery readiness.",
        parameters: Type.Object({}),
        async execute() {
            return get("/agent-runtime/doctor");
        },
    });
}
