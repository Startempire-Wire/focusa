//! Spec108 Awareness Substrate - `focusa.utility_card.v2` / `AwarenessPacket`
//!
//! "One substrate, many outputs." Takes a rich `AwarenessInput` bundle, generates
//! scored candidate lines, selects mode/surface/renderers, and produces typed
//! `AwarenessPacket` output for every surface that needs to tell the agent/operator
//! something.
//!
//! Surfaces: reload | post_compaction | warning | tool_guidance | uiai_bridge
//! Modes:    minimal    | standard    | rich   | onboarding

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Top-level types
// ---------------------------------------------------------------------------

pub type AwarenessLayer = String;
pub type Surface = String;
pub type Mode = String;

pub const SURFACE_RELOAD: &str = "reload";
pub const SURFACE_POST_COMPACTION: &str = "post_compaction";
pub const SURFACE_WARNING: &str = "warning";
pub const SURFACE_TOOL_GUIDANCE: &str = "tool_guidance";
pub const SURFACE_UIAI_BRIDGE: &str = "uiai_bridge";

pub const MODE_MINIMAL: &str = "minimal";
pub const MODE_STANDARD: &str = "standard";
pub const MODE_RICH: &str = "rich";
pub const MODE_ONBOARDING: &str = "onboarding";

/// AwarenessPacket - primary output of the awareness substrate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AwarenessPacket {
    pub schema: &'static str,
    pub generated_at: u64,
    pub mode: Mode,
    pub surface: Surface,
    pub status: &'static str,

    /// Lines surfaced to operators / visible cards.
    pub visible_lines: Vec<AwarenessCandidateLine>,
    /// Lines surfaced to system-awareness kernel (internal only).
    pub system_lines: Vec<AwarenessCandidateLine>,

    /// Top tool guidance recommendations.
    pub next_tools: Vec<ToolGuidance>,
    /// Tool guidance for recovery scenarios.
    pub recovery_tools: Vec<ToolGuidance>,

    /// Candidates that were scored but excluded, with reasons.
    pub suppressed_lines: Vec<SuppressedLine>,

    /// Packet metadata.
    pub metadata: PacketMetadata,

    /// Rehydrate ID for continuation across sessions.
    pub rehydrate_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PacketMetadata {
    pub dvs_cutoff: f64,
    pub total_candidates: usize,
    pub visible_count: usize,
    pub suppressed_count: usize,
    pub freshness_score: u8,
    pub authority_score: u8,
    pub confidence: &'static str,
    pub mode_reason: String,
    pub surface_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuppressedLine {
    pub line: AwarenessCandidateLine,
    pub suppress_reason: String,
    pub dvs: f64,
}

// ---------------------------------------------------------------------------
// AwarenessInput - gathered from daemon state helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AwarenessInput {
    // Authority layer
    pub project_identity: ProjectIdentityInput,
    pub project_root_safety: ProjectRootSafetyInput,

    // Execution layer
    pub workpoint_resume: Option<WorkpointResumeInput>,
    pub trajectory_view: Option<TrajectoryViewInput>,
    pub context_cognition: Option<ContextCognitionInput>,

    // Session layer
    pub session_transfer: SessionTransferInput,
    pub dxux_digest: Option<DxuxDigestInput>,

    // Risk / pressure layer
    pub context_pressure: ContextPressureInput,
    pub uiai_state: UiaiStateInput,

    // Operator steering layer
    pub operator_steering: OperatorSteeringInput,

    // Evidence / learning layer
    pub evidence: EvidenceInput,
    pub prediction: PredictionInput,
    pub metacog: MetacogInput,

    // Tool ecosystem layer
    pub tool_graph: ToolGraphInput,

    // Cadence / state layer
    pub cadence_state: Option<ContextPressureState>,

    // Render controls
    pub mode: String,
    pub surface: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIdentityInput {
    pub project_root: String,
    pub canonical_name: String,
    pub continuity_id: String,
    pub session_id: String,
    pub confidence: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRootSafetyInput {
    pub safe: bool,
    pub path: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkpointResumeInput {
    pub workpoint_id: String,
    pub canonical: bool,
    pub degraded: bool,
    pub mission: String,
    pub next_action: String,
    pub target_objects: Vec<String>,
    pub verified_evidence: Vec<String>,
    pub blockers: Vec<String>,
    pub do_not_drift: Vec<String>,
    pub action_authority: bool,
    pub continuity_id: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrajectoryViewInput {
    pub trajectory_id: String,
    pub canonical: bool,
    pub degraded: bool,
    pub hlt: Option<String>,
    pub mlg: Option<String>,
    pub stg: Option<String>,
    pub desired_end_state: Option<String>,
    pub active_gap: Option<String>,
    pub waypoints: Vec<String>,
    pub clarity_gate: String,
    pub next_tools: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextCognitionInput {
    pub rehydrate_id: String,
    pub workpoint_id: String,
    pub action_authority: String,
    pub scope_status: String,
    pub score: f64,
    pub evidence_refs: Vec<String>,
    pub advisory: bool,
    pub canonical: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTransferInput {
    pub action: String,
    pub saved: bool,
    pub resume_found: bool,
    pub continuity_id: String,
    pub mission: Option<String>,
    pub next_action: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DxuxDigestInput {
    pub status: String,
    pub authority: String,
    pub why: String,
    pub exact_next_action: String,
    pub evidence_refs: Vec<String>,
    pub rehydrate_refs: Vec<String>,
    pub canonical: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPressureInput {
    pub percentage: u8,
    pub tier: String,
    pub compaction_pending: bool,
    pub compaction_count: u32,
    pub last_compaction_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiaiStateInput {
    pub pressure: u8,
    pub session_count: u32,
    pub saturated: bool,
    pub browser_failures: u32,
    pub private_url_blocks: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorSteeringInput {
    pub current_ask: String,
    pub explicit_steer: Option<String>,
    pub scope_kind: Option<String>,
    pub carryover_policy: Option<String>,
    pub excluded_context_labels: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceInput {
    pub recent_refs: Vec<String>,
    pub proof_gaps: Vec<String>,
    pub active_object_refs: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionInput {
    pub recent_predictions: Vec<PredictionEntry>,
    pub stats: PredictionStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionEntry {
    pub id: String,
    pub predicted_outcome: String,
    pub confidence: f64,
    pub evaluated: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionStats {
    pub total: u32,
    pub evaluated: u32,
    pub accuracy: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetacogInput {
    pub recent_lessons: Vec<MetacogLesson>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetacogLesson {
    pub kind: String,
    pub content: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolGraphInput {
    pub total_tools: u32,
    pub families: HashMap<String, u32>,
    pub top_next_tools: Vec<String>,
    pub top_recovery_tools: Vec<String>,
    pub next_tools_by_family: HashMap<String, Vec<String>>,
    pub side_effect_profiles: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// AwarenessCandidateLine - scored line output
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AwarenessCandidateLine {
    pub id: String,
    pub layer: AwarenessLayer,
    pub category: String,
    pub text: String,
    pub authority_value: f64,
    pub actionability: f64,
    pub risk_reduction: f64,
    pub novelty: f64,
    pub proof_value: f64,
    pub redundancy_penalty: f64,
    pub staleness_penalty: f64,
    pub dvs: f64,
    pub mode_allowed: Vec<String>,
    pub surface_allowed: Vec<String>,
    pub suppress_reason: Option<String>,
    pub source_ref: Option<String>,
    pub evidence_ref: Option<String>,
}

// ---------------------------------------------------------------------------
// ToolGuidance - per-tool guidance output
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolGuidance {
    pub tool_name: String,
    pub family: String,
    pub why_included: String,
    pub authority_value: f64,
    pub actionability: f64,
    pub side_effect_risk: String,
    pub next_tools: Vec<String>,
}

// ---------------------------------------------------------------------------
// ContextPressureState - cadence/dedupe engine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPressureState {
    pub last_shown_at_ms: Option<u64>,
    pub last_pct: u8,
    pub last_tier: String,
    pub last_anchor_state: String,
    pub compaction_count_at_last_shown: u32,
    pub transition_count: u32,
    pub suppression_count: u32,
}

// ---------------------------------------------------------------------------
// Mode selection
// ---------------------------------------------------------------------------

/// Select mode from input per Spec108 §8 priority rules.
pub fn select_mode(input: &AwarenessInput) -> String {
    // Priority 1: safety and authority signals
    if !input.project_root_safety.safe {
        return MODE_STANDARD.to_string();
    }
    if let Some(ref wp) = input.workpoint_resume
        && !wp.action_authority
    {
        return MODE_STANDARD.to_string();
    }

    // Priority 2: post-compaction always gets standard
    if input.surface == SURFACE_POST_COMPACTION {
        return MODE_STANDARD.to_string();
    }

    // Priority 3: warnings get standard
    if input.surface == SURFACE_WARNING {
        return MODE_STANDARD.to_string();
    }

    // Priority 4: operator asks for architecture/design
    if let Some(ref steer) = input.operator_steering.explicit_steer
        && (steer.contains("architecture") || steer.contains("design") || steer.contains("explain"))
    {
        return MODE_RICH.to_string();
    }

    // Priority 5: first-ever project onboarding
    if input.session_transfer.action == "continue" && !input.session_transfer.resume_found {
        return MODE_ONBOARDING.to_string();
    }

    // Priority 6: high context pressure
    if input.context_pressure.tier == "critical" || input.context_pressure.tier == "high" {
        return MODE_MINIMAL.to_string();
    }

    // Priority 7: canonical Workpoint present
    if input
        .workpoint_resume
        .as_ref()
        .is_some_and(|wp| wp.canonical)
    {
        return MODE_MINIMAL.to_string();
    }

    // Priority 8: UIAI bridge
    if input.surface == SURFACE_UIAI_BRIDGE {
        return MODE_STANDARD.to_string();
    }

    MODE_STANDARD.to_string()
}

// ---------------------------------------------------------------------------
// DVS thresholds
// ---------------------------------------------------------------------------

fn dvs_threshold(mode: &str) -> f64 {
    match mode {
        MODE_MINIMAL => 7.0,
        MODE_STANDARD => 4.0,
        MODE_RICH => 1.5,
        MODE_ONBOARDING => 0.5,
        _ => 4.0,
    }
}

fn authority_exception_threshold(mode: &str) -> f64 {
    match mode {
        MODE_MINIMAL => 8.0,
        _ => f64::MAX,
    }
}

// ---------------------------------------------------------------------------
// Candidate line generation
// ---------------------------------------------------------------------------

static ALL_SURFACES: [&str; 5] = [
    SURFACE_RELOAD,
    SURFACE_POST_COMPACTION,
    SURFACE_WARNING,
    SURFACE_TOOL_GUIDANCE,
    SURFACE_UIAI_BRIDGE,
];
static ALL_MODES: [&str; 4] = [MODE_MINIMAL, MODE_STANDARD, MODE_RICH, MODE_ONBOARDING];

fn to_string_vec(slice: &[&str]) -> Vec<String> {
    slice.iter().map(|s| (*s).to_string()).collect()
}

/// Generate all candidate lines from the input bundle per Spec108 §7 source map.
pub fn generate_candidates(input: &AwarenessInput) -> Vec<AwarenessCandidateLine> {
    let mut lines = Vec::new();

    macro_rules! push {
        ($layer:expr, $cat:expr, $text:expr, $av:expr, $ac:expr, $rr:expr, $nv:expr, $pv:expr, $rp:expr, $sp:expr, $src:expr) => {{
            let dvs = compute_dvs($av, $ac, $rr, $nv, $pv, $rp, $sp);
            lines.push(AwarenessCandidateLine {
                id: format!("{:03}", lines.len()),
                layer: $layer.to_string(),
                category: $cat.to_string(),
                text: $text.to_string(),
                authority_value: $av,
                actionability: $ac,
                risk_reduction: $rr,
                novelty: $nv,
                proof_value: $pv,
                redundancy_penalty: $rp,
                staleness_penalty: $sp,
                dvs,
                mode_allowed: to_string_vec(&ALL_MODES),
                surface_allowed: to_string_vec(&ALL_SURFACES),
                suppress_reason: None,
                source_ref: Some($src.to_string()),
                evidence_ref: None,
            });
        }};
    }

    // --- Identity layer ---
    push!(
        "identity",
        "project_root",
        format!("project: {}", input.project_identity.project_root),
        10.0,
        8.0,
        0.0,
        5.0,
        0.0,
        0.0,
        0.0,
        "project_identity.project_root"
    );

    if !input.project_root_safety.safe {
        push!(
            "identity",
            "safety",
            format!(
                "⚠ Unsafe project root: {} ({})",
                input.project_root_safety.path,
                input
                    .project_root_safety
                    .reason
                    .as_deref()
                    .unwrap_or("unknown")
            ),
            10.0,
            9.0,
            8.0,
            10.0,
            0.0,
            0.0,
            0.0,
            "project_root_safety"
        );
    }

    push!(
        "identity",
        "continuity",
        format!(
            "session: {} | continuity: {}",
            input.project_identity.session_id, input.project_identity.continuity_id
        ),
        8.0,
        3.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        "project_identity"
    );

    // --- Authority layer ---
    if let Some(ref wp) = input.workpoint_resume {
        let auth = if wp.canonical { 10.0 } else { 6.0 };
        push!(
            "authority",
            "workpoint",
            format!(
                "Workpoint {} [{}]",
                wp.workpoint_id,
                if wp.canonical {
                    "canonical"
                } else {
                    "advisory"
                }
            ),
            auth,
            9.0,
            5.0,
            3.0,
            0.0,
            0.0,
            0.0,
            "workpoint_resume"
        );

        if !wp.mission.is_empty() {
            push!(
                "authority",
                "mission",
                format!("mission: {}", wp.mission),
                auth,
                9.0,
                3.0,
                3.0,
                0.0,
                0.0,
                0.0,
                "workpoint_resume.mission"
            );
        }

        if !wp.next_action.is_empty() {
            push!(
                "authority",
                "next_action",
                format!("next: {}", wp.next_action),
                auth,
                10.0,
                3.0,
                3.0,
                0.0,
                0.0,
                0.0,
                "workpoint_resume.next_action"
            );
        }

        if !wp.target_objects.is_empty() {
            push!(
                "authority",
                "targets",
                format!("targets: {}", wp.target_objects.join(" · ")),
                auth - 1.0,
                7.0,
                2.0,
                2.0,
                0.0,
                0.0,
                0.0,
                "workpoint_resume.target_objects"
            );
        }

        for blocker in &wp.blockers {
            push!(
                "risk",
                "blocker",
                format!("⚠ blocker: {}", blocker),
                auth,
                8.0,
                9.0,
                5.0,
                0.0,
                0.0,
                0.0,
                "workpoint_resume.blockers"
            );
        }

        for dn in &wp.do_not_drift {
            push!(
                "authority",
                "do_not_drift",
                format!("⛔ do not drift: {}", dn),
                auth,
                7.0,
                6.0,
                2.0,
                0.0,
                0.0,
                0.0,
                "workpoint_resume.do_not_drift"
            );
        }

        if wp.degraded {
            push!(
                "risk",
                "degraded",
                format!("⚠ Workpoint degraded: authority may be unreliable"),
                10.0,
                9.0,
                8.0,
                4.0,
                0.0,
                0.0,
                0.0,
                "workpoint_resume.degraded"
            );
        }
    }

    // --- Goal layer ---
    if let Some(ref tv) = input.trajectory_view {
        if let Some(ref hlt) = tv.hlt {
            push!(
                "goal",
                "hlt",
                format!("HLT: {}", hlt),
                5.0,
                4.0,
                2.0,
                2.0,
                0.0,
                1.0,
                1.0,
                "trajectory_view.hlt"
            );
        }
        if let Some(ref mlg) = tv.mlg {
            push!(
                "goal",
                "mlg",
                format!("MLG: {}", mlg),
                5.0,
                5.0,
                2.0,
                2.0,
                0.0,
                1.0,
                1.0,
                "trajectory_view.mlg"
            );
        }
        if let Some(ref stg) = tv.stg {
            push!(
                "goal",
                "stg",
                format!("STG: {}", stg),
                5.0,
                6.0,
                3.0,
                3.0,
                0.0,
                1.0,
                1.0,
                "trajectory_view.stg"
            );
        }
        if let Some(ref gap) = tv.active_gap {
            push!(
                "goal",
                "gap",
                format!("gap: {}", gap),
                5.0,
                7.0,
                5.0,
                3.0,
                0.0,
                1.0,
                1.0,
                "trajectory_view.active_gap"
            );
        }
        for wp in &tv.waypoints {
            push!(
                "goal",
                "waypoint",
                format!("→ {}", wp),
                4.0,
                5.0,
                2.0,
                1.0,
                0.0,
                1.0,
                1.0,
                "trajectory_view.waypoints"
            );
        }
        if tv.degraded {
            push!(
                "risk",
                "trajectory_degraded",
                "⚠ Trajectory degraded - advisory only",
                7.0,
                8.0,
                6.0,
                3.0,
                0.0,
                0.0,
                0.0,
                "trajectory_view.degraded"
            );
        }
    }

    // --- Risk layer ---
    let cp = &input.context_pressure;
    if cp.tier == "critical" || cp.tier == "high" {
        push!(
            "risk",
            "pressure",
            format!(
                "⛔ context pressure: {}% [{}] - compaction imminent",
                cp.percentage, cp.tier
            ),
            10.0,
            10.0,
            10.0,
            5.0,
            0.0,
            0.0,
            0.0,
            "context_pressure"
        );
    } else if cp.tier == "medium" {
        push!(
            "risk",
            "pressure",
            format!("⚠ context pressure: {}% [{}]", cp.percentage, cp.tier),
            6.0,
            6.0,
            5.0,
            3.0,
            0.0,
            0.0,
            0.0,
            "context_pressure"
        );
    }

    let uiai = &input.uiai_state;
    if uiai.saturated {
        push!(
            "risk",
            "uiai_saturated",
            format!(
                "⚠ UIAI pressure: {}% [saturated, {} sessions]",
                uiai.pressure, uiai.session_count
            ),
            7.0,
            7.0,
            6.0,
            3.0,
            0.0,
            0.0,
            0.0,
            "uiai_state"
        );
    }
    if uiai.browser_failures > 0 {
        push!(
            "risk",
            "browser_failures",
            format!("⚠ {} browser failure(s) detected", uiai.browser_failures),
            6.0,
            6.0,
            5.0,
            3.0,
            0.0,
            0.0,
            0.0,
            "uiai_state"
        );
    }

    // --- Proof layer ---
    if !input.evidence.recent_refs.is_empty() {
        push!(
            "proof",
            "recent_evidence",
            format!(
                "recent evidence: {} handle(s)",
                input.evidence.recent_refs.len()
            ),
            5.0,
            5.0,
            3.0,
            2.0,
            8.0,
            0.0,
            0.0,
            "evidence.recent_refs"
        );
    }
    for gap in &input.evidence.proof_gaps {
        push!(
            "proof",
            "proof_gap",
            format!("proof gap: {}", gap),
            5.0,
            7.0,
            5.0,
            3.0,
            0.0,
            0.0,
            0.0,
            "evidence.proof_gaps"
        );
    }

    // --- Recovery layer ---
    if !input.tool_graph.top_next_tools.is_empty() {
        push!(
            "recovery",
            "next_tools",
            format!(
                "top next tools: {}",
                input
                    .tool_graph
                    .top_next_tools
                    .iter()
                    .take(3)
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            5.0,
            8.0,
            4.0,
            3.0,
            0.0,
            0.0,
            0.0,
            "tool_graph.top_next_tools"
        );
    }

    // --- Learning layer ---
    if input.prediction.stats.total > 0 {
        let acc_pct = (input.prediction.stats.accuracy * 100.0).round() as u8;
        push!(
            "learning",
            "prediction_stats",
            format!(
                "prediction accuracy: {}% ({}/{} evaluated)",
                acc_pct, input.prediction.stats.evaluated, input.prediction.stats.total
            ),
            3.0,
            2.0,
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            "prediction.stats"
        );
    }

    for lesson in &input.metacog.recent_lessons {
        push!(
            "learning",
            "lesson",
            format!("[{}] {}", lesson.kind, lesson.content),
            3.0,
            4.0,
            2.0,
            2.0,
            0.0,
            0.0,
            1.0,
            "metacog.recent_lessons"
        );
    }

    // --- DXUX digest ---
    if let Some(ref dx) = input.dxux_digest
        && (dx.status != "ok" || dx.canonical)
    {
        push!(
            "recovery",
            "dxux_digest",
            format!("DXUX: {} - {}", dx.status, dx.exact_next_action),
            6.0,
            8.0,
            5.0,
            3.0,
            0.0,
            0.0,
            0.0,
            "dxux_digest"
        );
    }

    lines
}

/// Compute DVS per Spec108 §6 formula.
fn compute_dvs(av: f64, ac: f64, rr: f64, nv: f64, pv: f64, rp: f64, sp: f64) -> f64 {
    let score = (av * 3.0) + (ac * 2.5) + (rr * 2.0) + (nv * 1.5) + (pv * 1.5);
    let penalty = (rp * 2.0) + (sp * 1.5);
    (score - penalty).max(0.0)
}

// ---------------------------------------------------------------------------
// ContextPressureState dedupe engine
// ---------------------------------------------------------------------------

/// Per Spec108 §9 - determine whether to show a pressure warning.
pub fn should_show_pressure_warning(
    state: &ContextPressureState,
    input: &AwarenessInput,
) -> PressureWarning {
    use std::time::UNIX_EPOCH;
    let now_ms = UNIX_EPOCH
        .elapsed()
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let pct = input.context_pressure.percentage;
    let tier = &input.context_pressure.tier;
    let anchor = input
        .workpoint_resume
        .as_ref()
        .map(|w| w.workpoint_id.clone())
        .unwrap_or_else(|| "none".to_string());
    let comp_count = input.context_pressure.compaction_count;

    // Never show within 30 seconds
    if let Some(last) = state.last_shown_at_ms
        && now_ms.saturating_sub(last) < 30_000
    {
        return PressureWarning {
            show: false,
            reason: "within_30s_dedupe".to_string(),
            escalation: "none".to_string(),
        };
    }

    let tier_order = |t: &str| match t {
        "low" => 0u8,
        "medium" => 1,
        "high" => 2,
        "critical" => 3,
        _ => 0,
    };

    // Tier escalated
    if tier_order(tier) > tier_order(&state.last_tier) {
        return PressureWarning {
            show: true,
            reason: "tier_escalation".to_string(),
            escalation: "hard".to_string(),
        };
    }

    // Workpoint anchor changed
    if anchor != state.last_anchor_state {
        return PressureWarning {
            show: true,
            reason: "anchor_changed".to_string(),
            escalation: "soft".to_string(),
        };
    }

    // Percentage jumped >20 and tier is high/critical
    if pct.saturating_sub(state.last_pct) > 20 && (tier == "high" || tier == "critical") {
        return PressureWarning {
            show: true,
            reason: "pct_jump".to_string(),
            escalation: "soft".to_string(),
        };
    }

    // Compaction count escalated
    if comp_count >= state.compaction_count_at_last_shown + 3 {
        return PressureWarning {
            show: true,
            reason: "compaction_count_escalation".to_string(),
            escalation: "hard".to_string(),
        };
    }

    // After 5 minutes, re-show if still pressure
    if let Some(last) = state.last_shown_at_ms
        && now_ms.saturating_sub(last) > 300_000
        && pct > 50
    {
        return PressureWarning {
            show: true,
            reason: "stale_reminder".to_string(),
            escalation: "soft".to_string(),
        };
    }

    PressureWarning {
        show: false,
        reason: "no_state_change".to_string(),
        escalation: "none".to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct PressureWarning {
    pub show: bool,
    pub reason: String,
    pub escalation: String,
}

// ---------------------------------------------------------------------------
// Tool-family selector
// ---------------------------------------------------------------------------

/// Select top N tool guidance recommendations per Spec108 §10.
pub fn select_top_tools(input: &AwarenessInput, count: usize) -> Vec<ToolGuidance> {
    let blockers: Vec<String> = input
        .workpoint_resume
        .as_ref()
        .map(|w| w.blockers.clone())
        .unwrap_or_default();

    let mut candidates: Vec<ToolGuidance> = input
        .tool_graph
        .top_next_tools
        .iter()
        .take(count * 2)
        .map(|tool| {
            // families maps tool → count; use tool itself as family identifier
            let family = tool.clone();
            let side_effect = input
                .tool_graph
                .side_effect_profiles
                .get(tool)
                .cloned()
                .unwrap_or_default();
            let blocker_relevant = blockers
                .iter()
                .any(|b| b.to_lowercase().contains(&tool.to_lowercase()));

            let (av, ac) = if blocker_relevant {
                (8.0, 9.0)
            } else {
                (5.0, 6.0)
            };
            let risk =
                if side_effect.contains("write_state") || side_effect.contains("control_state") {
                    "risky"
                } else if side_effect.contains("write_") {
                    "moderate"
                } else {
                    "safe"
                };

            ToolGuidance {
                tool_name: tool.clone(),
                family: family.clone(),
                why_included: if blocker_relevant {
                    "directly relevant to current blocker".to_string()
                } else {
                    "top next tool from choreography graph".to_string()
                },
                authority_value: av,
                actionability: ac,
                side_effect_risk: risk.to_string(),
                next_tools: input
                    .tool_graph
                    .next_tools_by_family
                    .get(tool)
                    .cloned()
                    .unwrap_or_default(),
            }
        })
        .collect();

    // Sort by authority + actionability descending
    candidates.sort_by(|a, b| {
        let sa = a.authority_value + a.actionability;
        let sb = b.authority_value + b.actionability;
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Filter risky in minimal mode
    if input.mode == MODE_MINIMAL {
        candidates.retain(|t| t.side_effect_risk != "risky");
    }

    candidates.truncate(count);
    candidates
}

// ---------------------------------------------------------------------------
// Per-surface renderer
// ---------------------------------------------------------------------------

/// Render the final `AwarenessPacket` for the given surface + mode.
pub fn render_packet(input: &AwarenessInput) -> AwarenessPacket {
    let mode = if input.mode.is_empty() {
        select_mode(input)
    } else {
        input.mode.clone()
    };

    let surface = input.surface.clone();
    let threshold = dvs_threshold(&mode);
    let auth_exception = authority_exception_threshold(&mode);
    let now_ms = std::time::UNIX_EPOCH
        .elapsed()
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let candidates = generate_candidates(input);
    let total_candidates = candidates.len();
    let candidates_clone = candidates.clone();

    let mut visible = Vec::new();
    let mut suppressed = Vec::new();

    for mut line in candidates {
        // Filter by surface
        if !line.surface_allowed.iter().any(|s| s == &surface) {
            line.suppress_reason = Some("surface_not_allowed".to_string());
            suppressed.push(SuppressedLine {
                line: line.clone(),
                suppress_reason: line.suppress_reason.clone().unwrap(),
                dvs: line.dvs,
            });
            continue;
        }

        // Filter by mode
        if !line.mode_allowed.iter().any(|m| m == &mode) {
            line.suppress_reason = Some("mode_not_allowed".to_string());
            suppressed.push(SuppressedLine {
                line: line.clone(),
                suppress_reason: line.suppress_reason.clone().unwrap(),
                dvs: line.dvs,
            });
            continue;
        }

        // DVS threshold OR authority exception
        let passes = line.dvs >= threshold || line.authority_value >= auth_exception;
        if !passes {
            line.suppress_reason = Some(format!(
                "dvs_below_threshold:{:.2}<{:.2}",
                line.dvs, threshold
            ));
            suppressed.push(SuppressedLine {
                line: line.clone(),
                suppress_reason: line.suppress_reason.clone().unwrap(),
                dvs: line.dvs,
            });
            continue;
        }

        visible.push(line);
    }

    // Sort visible by DVS descending
    visible.sort_by(|a, b| {
        b.dvs
            .partial_cmp(&a.dvs)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Tool guidance
    let next_tools = select_top_tools(input, 3);
    let recovery_tools = input
        .tool_graph
        .top_recovery_tools
        .iter()
        .take(3)
        .map(|tool| {
            let family = tool.clone();
            ToolGuidance {
                tool_name: tool.clone(),
                family,
                why_included: "recovery tool from choreography graph".to_string(),
                authority_value: 6.0,
                actionability: 7.0,
                side_effect_risk: "moderate".to_string(),
                next_tools: vec![],
            }
        })
        .collect();

    // Compute metadata scores
    let freshness_score = compute_freshness_score(input);
    let authority_score = compute_authority_score(input);
    let visible_count = visible.len();
    let suppressed_count = suppressed.len();

    let rehydrate_id = format!(
        "awareness:{}{}{}{}",
        now_ms, input.project_identity.continuity_id, mode, surface
    );

    let confidence = if authority_score >= 80 {
        "high"
    } else if authority_score >= 50 {
        "medium"
    } else {
        "low"
    };
    let mode_reason = mode_selected_reason(input, &mode);
    let surface_reason = surface_selected_reason(&surface);

    AwarenessPacket {
        schema: "focusa.awareness_packet.v1",
        generated_at: now_ms,
        mode,
        surface,
        status: if input.workpoint_resume.as_ref().is_some_and(|w| !w.degraded)
            && input
                .trajectory_view
                .as_ref()
                .map(|t| !t.degraded)
                .unwrap_or(true)
        {
            "fresh"
        } else {
            "degraded"
        },
        visible_lines: visible,
        system_lines: candidates_clone
            .into_iter()
            .filter(|l| l.suppress_reason.is_none())
            .collect(),
        next_tools,
        recovery_tools,
        suppressed_lines: suppressed,
        metadata: PacketMetadata {
            dvs_cutoff: threshold,
            total_candidates,
            visible_count,
            suppressed_count,
            freshness_score,
            authority_score,
            confidence,
            mode_reason,
            surface_reason,
        },
        rehydrate_id,
    }
}

fn compute_freshness_score(input: &AwarenessInput) -> u8 {
    let mut score = 100u8;

    // Penalize stale workpoint
    if let Some(ref cp) = input.context_pressure.last_compaction_at_ms {
        let age_ms = std::time::UNIX_EPOCH
            .elapsed()
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
            .saturating_sub(*cp);
        if age_ms > 300_000 {
            score = score.saturating_sub(30);
        } else if age_ms > 60_000 {
            score = score.saturating_sub(10);
        }
    }

    // Penalize degraded state
    if input.workpoint_resume.as_ref().is_some_and(|w| w.degraded) {
        score = score.saturating_sub(20);
    }
    if input.trajectory_view.as_ref().is_some_and(|t| t.degraded) {
        score = score.saturating_sub(10);
    }

    score
}

fn compute_authority_score(input: &AwarenessInput) -> u8 {
    let mut score = 0u8;

    if input.project_root_safety.safe {
        score += 20;
    }
    if input.project_identity.verified {
        score += 20;
    }
    if input.workpoint_resume.as_ref().is_some_and(|w| w.canonical) {
        score += 30;
    } else if input.workpoint_resume.is_some() {
        score += 10;
    }
    if input.trajectory_view.is_some() {
        score += 15;
    }
    if !input.tool_graph.top_next_tools.is_empty() {
        score += 15;
    }

    score.min(100)
}

fn mode_selected_reason(input: &AwarenessInput, mode: &str) -> String {
    if !input.project_root_safety.safe {
        return "unsafe_project_root → standard".to_string();
    }
    if let Some(ref wp) = input.workpoint_resume {
        if !wp.action_authority {
            return "workpoint.action_authority=false → standard".to_string();
        }
        if wp.canonical {
            return "canonical_workpoint → minimal".to_string();
        }
    }
    if input.context_pressure.tier == "critical" || input.context_pressure.tier == "high" {
        return "high_critical_pressure → minimal".to_string();
    }
    if let Some(ref steer) = input.operator_steering.explicit_steer
        && (steer.contains("architecture") || steer.contains("design"))
    {
        return "explicit_steer=architecture/design → rich".to_string();
    }
    format!("surface={} → {}", input.surface, mode)
}

fn surface_selected_reason(surface: &str) -> String {
    match surface {
        SURFACE_RELOAD => "Pi/agent reload - operator-visible bootstrap".to_string(),
        SURFACE_POST_COMPACTION => "post-compaction handoff - standard mode enforced".to_string(),
        SURFACE_WARNING => "pressure/risk warning surface - standard mode enforced".to_string(),
        SURFACE_TOOL_GUIDANCE => "tool guidance surface - next-tools + recovery".to_string(),
        SURFACE_UIAI_BRIDGE => "UIAI proof/risk bridge - standard mode enforced".to_string(),
        _ => "unknown surface".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Backward-compat shim for existing utility_card() callers
// ---------------------------------------------------------------------------

use super::utility_card::{UtilityCard, utility_card as legacy_utility_card};

/// Thin wrapper: renders the awareness packet as a legacy UtilityCard shape.
/// Used by routes that haven't migrated to the new surface-aware API yet.
pub fn awareness_as_utility_card(input: &AwarenessInput) -> UtilityCard {
    let packet = render_packet(input);
    let visible: Vec<String> = packet
        .visible_lines
        .iter()
        .map(|l| format!("[{}:{}] {}", l.layer, l.category, l.text))
        .collect();

    let next_tools: Vec<String> = packet
        .next_tools
        .iter()
        .map(|t| t.tool_name.clone())
        .collect();

    let recovery: Vec<String> = packet
        .recovery_tools
        .iter()
        .map(|t| t.tool_name.clone())
        .collect();

    UtilityCard {
        schema: "focusa.utility_card.v2_awareness".to_string(),
        status: packet.status.to_string(),
        purpose: format!(
            "awareness_packet surface={} mode={}",
            packet.surface, packet.mode
        ),
        preferred_layer: packet.mode.to_string(),
        authority_boundary: "project_root + continuity_id".to_string(),
        usefulness_bar: vec![format!(
            "authority_score:{} freshness:{} visible:{}",
            packet.metadata.authority_score,
            packet.metadata.freshness_score,
            packet.metadata.visible_count
        )],
        scope_gate: vec![],
        bootstrap_card: vec![],
        post_compaction_card: visible.clone(),
        exact_next_actions: visible
            .iter()
            .filter(|l| l.contains("next:") || l.contains("next_action"))
            .take(3)
            .cloned()
            .collect(),
        do_not_drift: packet
            .visible_lines
            .iter()
            .filter(|l| l.layer == "authority" && l.category == "do_not_drift")
            .map(|l| l.text.clone())
            .collect(),
        evidence_policy: vec![],
        brevity_rules: vec![],
        recovery_order: recovery,
        proof_commands: vec![],
        next_tools,
    }
}

/// Fallback: when no input is available, return the legacy static card.
pub fn fallback_card() -> UtilityCard {
    legacy_utility_card()
}
