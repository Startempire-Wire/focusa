//! Mission Deck walkthrough engine (Spec 117 §12).
//!
//! Schema: `focusa.walkthrough.v1` — versioned, declarative walkthrough
//! definitions plus append-only `WalkthroughEvent` records at
//! `~/.focusa/deck/walkthroughs/{project_hash}.jsonl`.
//!
//! This module is read-only with respect to workpoint / evidence /
//! trajectory authority. A walkthrough *proposes* next actions; it never
//! mutates Focusa canonical state directly. Promotion into a canonical
//! Workpoint candidate still requires operator approval through the existing
//! `focusa workpoint checkpoint` path.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const SCHEMA_VERSION: &str = "focusa.walkthrough.v1";
pub const AUDIENCE_BEGINNER: &str = "beginner";
pub const AUDIENCE_OPERATOR: &str = "operator";
pub const AUDIENCE_AGENT: &str = "agent";
pub const AUDIENCE_EVALUATOR: &str = "evaluator";

pub const STEP_KIND_READ: &str = "read";
pub const STEP_KIND_PROPOSE: &str = "propose";
pub const STEP_KIND_WRITE: &str = "write";
pub const STEP_KIND_EXTERNAL: &str = "external";

pub const EVIDENCE_ACTUAL: &str = "actual";
pub const EVIDENCE_PARTIAL: &str = "partial";
pub const EVIDENCE_SURROGATE: &str = "surrogate";
pub const EVIDENCE_BLOCKED: &str = "blocked";
pub const EVIDENCE_MISSING: &str = "missing";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Started,
    Advanced,
    Completed,
    Reset,
    Blocked,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityPosture {
    #[default]
    Ok,
    Advisory,
    Blocked,
    Stale,
    ProofMissing,
    GlobalAdvisory,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Trigger {
    #[serde(default)]
    pub first_run: bool,
    #[serde(default)]
    pub missing_project: bool,
    #[serde(default)]
    pub missing_workpoint: bool,
    #[serde(default)]
    pub missing_evidence: bool,
    #[serde(default)]
    pub scope_mismatch: bool,
    #[serde(default)]
    pub release_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RequiredState {
    #[serde(default)]
    pub daemon: bool,
    #[serde(default)]
    pub project_identity: bool,
    #[serde(default)]
    pub workpoint: bool,
    #[serde(default)]
    pub evidence: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Step {
    pub id: String,
    pub title: String,
    pub explanation: String,
    #[serde(default)]
    pub visual: String,
    #[serde(default = "default_step_kind")]
    pub action_kind: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub api_route: String,
    #[serde(default)]
    pub authority_required: bool,
    #[serde(default)]
    pub success_condition: String,
    #[serde(default)]
    pub recovery_hint: String,
}

fn default_step_kind() -> String {
    STEP_KIND_READ.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Completion {
    #[serde(default)]
    pub success_message: String,
    #[serde(default)]
    pub proof_required: bool,
    #[serde(default = "default_evidence_class")]
    pub evidence_class: String,
    /// Spec 119 §7.8 invariant: proof must precede completion. Defaults to true.
    #[serde(default = "default_proof_precedes_completion")]
    pub proof_precedes_completion: bool,
}

fn default_evidence_class() -> String {
    EVIDENCE_ACTUAL.to_string()
}

fn default_proof_precedes_completion() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Walkthrough {
    pub schema_version: String,
    pub id: String,
    pub title: String,
    pub audience: String,
    pub trigger: Trigger,
    #[serde(default)]
    pub goal: String,
    #[serde(default)]
    pub why_it_matters: String,
    pub required_state: RequiredState,
    pub steps: Vec<Step>,
    pub completion: Completion,
    #[serde(default = "default_resettable")]
    pub resettable: bool,
    #[serde(default)]
    pub side_effects: Vec<String>,
}

fn default_resettable() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkthroughEvent {
    pub walkthrough_id: String,
    pub step_id: String,
    pub project_root: String,
    #[serde(default)]
    pub continuity_id: String,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub authority_posture: AuthorityPosture,
}

pub fn event_log_path(project_root: &Path) -> PathBuf {
    let canonical =
        std::fs::canonicalize(project_root).unwrap_or_else(|_| project_root.to_path_buf());
    let hash = blake_like_hash(&canonical.display().to_string());
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/root"));
    home.join(".focusa")
        .join("deck")
        .join("walkthroughs")
        .join(format!("{hash}.jsonl"))
}

fn blake_like_hash(input: &str) -> String {
    // Cheap, stable, no-dep FNV-1a 64-bit hash. Sufficient as a project-folder
    // identifier for local-only walkthrough event logs.
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

pub fn write_event(event: &WalkthroughEvent) -> Result<()> {
    let path = event_log_path(Path::new(&event.project_root));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("could not create {}", parent.display()))?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("could not open {}", path.display()))?;
    let raw = serde_json::to_string(event)?;
    file.write_all(raw.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

pub fn load_events(project_root: &Path) -> Result<Vec<WalkthroughEvent>> {
    let path = event_log_path(project_root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut events: Vec<WalkthroughEvent> = Vec::new();
    for raw in std::fs::read_to_string(&path)?.lines() {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        let evt: WalkthroughEvent =
            serde_json::from_str(raw).with_context(|| format!("malformed event line: {raw}"))?;
        events.push(evt);
    }
    Ok(events)
}

pub fn progress(project_root: &Path, walkthrough_id: &str) -> Result<BTreeMap<String, EventType>> {
    let mut progress: BTreeMap<String, EventType> = BTreeMap::new();
    for evt in load_events(project_root)? {
        if evt.walkthrough_id != walkthrough_id {
            continue;
        }
        let weight = match evt.event_type {
            EventType::Started => 1,
            EventType::Advanced => 2,
            EventType::Completed => 3,
            EventType::Reset => 0,
            EventType::Blocked => 0,
        };
        let prev = progress.get(&evt.step_id).copied();
        let prev_weight = prev
            .as_ref()
            .map(|kind| match kind {
                EventType::Started => 1,
                EventType::Advanced => 2,
                EventType::Completed => 3,
                EventType::Reset => 0,
                EventType::Blocked => 0,
            })
            .unwrap_or(0);
        if weight >= prev_weight {
            progress.insert(evt.step_id, evt.event_type);
        }
    }
    Ok(progress)
}

pub fn first_mission() -> Walkthrough {
    Walkthrough {
        schema_version: SCHEMA_VERSION.to_string(),
        id: "first-mission".to_string(),
        title: "First Mission".to_string(),
        audience: AUDIENCE_BEGINNER.to_string(),
        trigger: Trigger {
            first_run: true,
            missing_project: true,
            missing_workpoint: true,
            missing_evidence: true,
            ..Trigger::default()
        },
        goal: "Bind project, create Workpoint, attach evidence, resume mission like a new agent would.".to_string(),
        why_it_matters: "An AI agent that loses the mission cannot recover without project, goal, workpoint, proof, and next action. This walkthrough proves Focusa makes that recoverable.".to_string(),
        required_state: RequiredState {
            daemon: true,
            project_identity: true,
            workpoint: true,
            evidence: false,
        },
        steps: vec![
            Step {
                id: "start-daemon".to_string(),
                title: "Start daemon".to_string(),
                explanation: "The Focusa daemon is the local source of truth for project, workpoint, evidence, and recall.".to_string(),
                visual: "[daemon]".to_string(),
                action_kind: STEP_KIND_WRITE.to_string(),
                command: "focusa start".to_string(),
                api_route: "/v1/health".to_string(),
                authority_required: false,
                success_condition: "GET /v1/health returns ok=true".to_string(),
                recovery_hint: "focusa doctor --scope host; bash scripts/install-daemon.sh /usr/local".to_string(),
            },
            Step {
                id: "bind-project".to_string(),
                title: "Bind this project".to_string(),
                explanation: "Bind a folder as the Focusa project so a Workpoint can be scoped to project_root + continuity_id.".to_string(),
                visual: "[project_root]".to_string(),
                action_kind: STEP_KIND_WRITE.to_string(),
                command: "focusa init --quickstart".to_string(),
                api_route: "/v1/project/identity".to_string(),
                authority_required: true,
                success_condition: ".focusa-project.json exists at project root".to_string(),
                recovery_hint: "focusa init --quickstart --project-root <path>".to_string(),
            },
            Step {
                id: "create-workpoint".to_string(),
                title: "Create your first Workpoint".to_string(),
                explanation: "A Workpoint is the canonical save state for in-progress work.".to_string(),
                visual: "[workpoint]".to_string(),
                action_kind: STEP_KIND_WRITE.to_string(),
                command: "focusa workpoint checkpoint --mission \"<what>\" --next-slice \"<why>\"".to_string(),
                api_route: "/v1/workpoint/checkpoint".to_string(),
                authority_required: true,
                success_condition: "Workpoint resume returns a non-empty workpoint_id".to_string(),
                recovery_hint: "focusa workpoint resume --project-root \"$(pwd)\"".to_string(),
            },
            Step {
                id: "attach-evidence".to_string(),
                title: "Attach one proof item".to_string(),
                explanation: "Proof is what makes the workpoint trustworthy across handoff.".to_string(),
                visual: "[evidence]".to_string(),
                action_kind: STEP_KIND_PROPOSE.to_string(),
                command: "focusa workpoint checkpoint --evidence-ref \"tests/...\"".to_string(),
                api_route: "/v1/workpoint/checkpoint".to_string(),
                authority_required: true,
                success_condition: "Workpoint event has at least one evidence_ref".to_string(),
                recovery_hint: "Link a test, file, screenshot, or curl output to make the workpoint prove itself.".to_string(),
            },
            Step {
                id: "resume".to_string(),
                title: "Resume the mission like a new agent would".to_string(),
                explanation: "Resume proves the mission can survive handoff, compaction, and agent restart.".to_string(),
                visual: "[resume]".to_string(),
                action_kind: STEP_KIND_READ.to_string(),
                command: "focusa workpoint resume".to_string(),
                api_route: "/v1/workpoint/resume".to_string(),
                authority_required: false,
                success_condition: "Resume packet shows the same workpoint_id and evidence".to_string(),
                recovery_hint: "If resume is blocked, focusa recover --dry-run".to_string(),
            },
        ],
        completion: Completion {
            success_message: "This mission can now survive handoff, compaction, and agent restart.".to_string(),
            proof_required: true,
            evidence_class: EVIDENCE_ACTUAL.to_string(),
            proof_precedes_completion: true,
        },
        resettable: true,
        side_effects: Vec::new(),
    }
}

pub fn agent_handoff() -> Walkthrough {
    Walkthrough {
        schema_version: SCHEMA_VERSION.to_string(),
        id: "agent-handoff".to_string(),
        title: "Agent Handoff".to_string(),
        audience: AUDIENCE_AGENT.to_string(),
        trigger: Trigger {
            missing_evidence: true,
            ..Trigger::default()
        },
        goal: "Show why Focusa exists: a new agent can recover mission, Workpoint, boundaries, and proof expectations.".to_string(),
        why_it_matters: "Market evaluators need to see the handoff value immediately: compaction or agent restart should not lose what matters.".to_string(),
        required_state: RequiredState {
            daemon: true,
            project_identity: true,
            workpoint: true,
            evidence: false,
        },
        steps: vec![
            Step {
                id: "show-current-mission".to_string(),
                title: "Show current mission".to_string(),
                explanation: "Start with the plain-language objective so the next agent knows what success means.".to_string(),
                visual: "[mission]".to_string(),
                action_kind: STEP_KIND_READ.to_string(),
                command: "focusa trajectory view".to_string(),
                api_route: "/v1/trajectory/view".to_string(),
                authority_required: false,
                success_condition: "Mission summary is visible and scoped to the current project.".to_string(),
                recovery_hint: "If the mission is missing, checkpoint the operator ask before continuing.".to_string(),
            },
            Step {
                id: "show-current-workpoint".to_string(),
                title: "Show current Workpoint".to_string(),
                explanation: "The Workpoint is the canonical mission save: action, evidence, blockers, and next step.".to_string(),
                visual: "[workpoint]".to_string(),
                action_kind: STEP_KIND_READ.to_string(),
                command: "focusa workpoint resume".to_string(),
                api_route: "/v1/workpoint/resume".to_string(),
                authority_required: false,
                success_condition: "Resume packet has a canonical workpoint_id or clearly explains why it is advisory/blocked.".to_string(),
                recovery_hint: "Create a Workpoint checkpoint with mission/current_action/next_action.".to_string(),
            },
            Step {
                id: "render-bootstrap-packet".to_string(),
                title: "Render the handoff packet".to_string(),
                explanation: "A handoff packet gives a new agent enough context without dumping the transcript.".to_string(),
                visual: "[bootstrap]".to_string(),
                action_kind: STEP_KIND_READ.to_string(),
                command: "focusa workpoint resume --mode compact_prompt".to_string(),
                api_route: "/v1/workpoint/resume".to_string(),
                authority_required: false,
                success_condition: "Packet includes mission, current action, proof refs, blockers, next action, and do-not-drift boundaries.".to_string(),
                recovery_hint: "Use focusa workpoint checkpoint before relying on transcript memory.".to_string(),
            },
            Step {
                id: "show-new-agent-receives".to_string(),
                title: "Show what a new agent receives".to_string(),
                explanation: "The next agent should see the same mission and exact next action after reload or compaction.".to_string(),
                visual: "[new-agent]".to_string(),
                action_kind: STEP_KIND_PROPOSE.to_string(),
                command: "focusa context cognition render".to_string(),
                api_route: "/v1/context-cognition/render".to_string(),
                authority_required: false,
                success_condition: "Rendered packet is concise and project-bound.".to_string(),
                recovery_hint: "If scope conflicts, verify project identity before editing files.".to_string(),
            },
            Step {
                id: "show-drift-boundaries".to_string(),
                title: "Show drift boundaries".to_string(),
                explanation: "Do-not-drift boundaries prevent a fast agent from damaging adjacent work.".to_string(),
                visual: "[boundaries]".to_string(),
                action_kind: STEP_KIND_READ.to_string(),
                command: "focusa workpoint resume | grep DO_NOT_DRIFT".to_string(),
                api_route: "/v1/workpoint/resume".to_string(),
                authority_required: false,
                success_condition: "At least one drift boundary is visible when risk exists.".to_string(),
                recovery_hint: "Add do_not_drift lines to the next Workpoint checkpoint.".to_string(),
            },
            Step {
                id: "show-proof-expectations".to_string(),
                title: "Show evidence and proof expectations".to_string(),
                explanation: "Handoff is not done until proof expectations are visible to the next agent.".to_string(),
                visual: "[proof]".to_string(),
                action_kind: STEP_KIND_READ.to_string(),
                command: "focusa workpoint resume | grep -i evidence".to_string(),
                api_route: "/v1/workpoint/resume".to_string(),
                authority_required: false,
                success_condition: "Proof refs or explicit proof gap expectations are visible.".to_string(),
                recovery_hint: "Attach a test, file, screenshot, command output, or an intentional proof-gap note.".to_string(),
            },
        ],
        completion: Completion {
            success_message: "A new agent can now recover mission, next action, boundaries, and proof expectations without transcript memory.".to_string(),
            proof_required: true,
            evidence_class: EVIDENCE_ACTUAL.to_string(),
            proof_precedes_completion: true,
        },
        resettable: true,
        side_effects: Vec::new(),
    }
}

pub fn no_proof_no_done() -> Walkthrough {
    Walkthrough {
        schema_version: SCHEMA_VERSION.to_string(),
        id: "no-proof-no-done".to_string(),
        title: "No Proof, No Done".to_string(),
        audience: AUDIENCE_BEGINNER.to_string(),
        trigger: Trigger {
            missing_evidence: true,
            ..Trigger::default()
        },
        goal: "Teach evidence discipline: an agent completion claim is not done until proof is visible or an explicit proof gap is recorded.".to_string(),
        why_it_matters: "Fast market work only stays safe when every shipped claim has proof a new agent or evaluator can inspect.".to_string(),
        required_state: RequiredState {
            daemon: true,
            project_identity: true,
            workpoint: true,
            evidence: true,
        },
        steps: vec![
            Step {
                id: "display-completion-claim".to_string(),
                title: "Display the agent completion claim".to_string(),
                explanation: "Begin with what the agent says is complete, then require proof before accepting it.".to_string(),
                visual: "[claim]".to_string(),
                action_kind: STEP_KIND_READ.to_string(),
                command: "focusa workpoint resume".to_string(),
                api_route: "/v1/workpoint/resume".to_string(),
                authority_required: false,
                success_condition: "Completion claim or current action is visible.".to_string(),
                recovery_hint: "Checkpoint the claim as current_action before evaluating proof.".to_string(),
            },
            Step {
                id: "check-evidence-refs".to_string(),
                title: "Check evidence refs".to_string(),
                explanation: "Evidence refs are the inspectable handles that make a completion claim trustworthy.".to_string(),
                visual: "[evidence refs]".to_string(),
                action_kind: STEP_KIND_READ.to_string(),
                command: "focusa workpoint resume | grep -i evidence".to_string(),
                api_route: "/v1/workpoint/resume".to_string(),
                authority_required: false,
                success_condition: "At least one proof handle or explicit proof-gap marker is visible.".to_string(),
                recovery_hint: "Use focusa evidence capture or checkpoint with --evidence-ref.".to_string(),
            },
            Step {
                id: "show-proof-gap-if-missing".to_string(),
                title: "Show proof gap if missing".to_string(),
                explanation: "Missing proof is not a failure; hiding it is. Mark the gap before calling work done.".to_string(),
                visual: "[proof gap]".to_string(),
                action_kind: STEP_KIND_PROPOSE.to_string(),
                command: "focusa workpoint checkpoint --blocker \"proof missing\"".to_string(),
                api_route: "/v1/workpoint/checkpoint".to_string(),
                authority_required: true,
                success_condition: "Proof gap is visible as blocked, partial, surrogate, or missing.".to_string(),
                recovery_hint: "Record why actual proof is missing and what would satisfy it.".to_string(),
            },
            Step {
                id: "attach-proof-or-mark-missing".to_string(),
                title: "Attach proof or mark intentionally missing".to_string(),
                explanation: "Attach actual proof when possible; otherwise make the proof gap explicit and bounded.".to_string(),
                visual: "[attach proof]".to_string(),
                action_kind: STEP_KIND_WRITE.to_string(),
                command: "focusa workpoint checkpoint --evidence-ref <proof>".to_string(),
                api_route: "/v1/workpoint/checkpoint".to_string(),
                authority_required: true,
                success_condition: "Workpoint includes evidence_refs or a declared proof-gap blocker.".to_string(),
                recovery_hint: "Accept file path, test id, screenshot, URL, curl output, or explicit proof-gap note.".to_string(),
            },
            Step {
                id: "rerender-proof-meter".to_string(),
                title: "Re-render proof meter".to_string(),
                explanation: "The proof meter should update from none to linked/verified/partial/missing based on the new evidence state.".to_string(),
                visual: "[proof meter]".to_string(),
                action_kind: STEP_KIND_READ.to_string(),
                command: "focusa deck".to_string(),
                api_route: "/v1/deck/proof-meter".to_string(),
                authority_required: false,
                success_condition: "Proof status is visible and no longer silently absent.".to_string(),
                recovery_hint: "Refresh Mission Deck or re-run workpoint resume to inspect evidence state.".to_string(),
            },
        ],
        completion: Completion {
            success_message: "The completion claim now has proof, or the proof gap is explicit and cannot be mistaken for done.".to_string(),
            proof_required: true,
            evidence_class: EVIDENCE_ACTUAL.to_string(),
            proof_precedes_completion: true,
        },
        resettable: true,
        side_effects: Vec::new(),
    }
}

/// JSON envelope for `focusa deck walkthrough list` and similar commands.
pub fn list_catalog() -> Vec<&'static str> {
    vec!["first-mission", "agent-handoff", "no-proof-no-done"]
}

#[derive(Args, Debug)]
pub struct WalkthroughArgs {
    /// Sub-action: list, start, advance, reset, show, completed, or progress.
    /// Defaults to `list` when omitted.
    #[arg(value_name = "ACTION")]
    pub action: Option<String>,

    /// Walkthrough id (e.g. first-mission).
    #[arg(long)]
    pub walkthrough: Option<String>,

    /// Step id used for advance/reset.
    #[arg(long)]
    pub step: Option<String>,

    /// Project root override.
    #[arg(long)]
    pub project_root: Option<String>,
}

pub async fn run(args: WalkthroughArgs, json_mode: bool) -> Result<()> {
    let project_root = match args.project_root.as_deref() {
        Some(value) if !value.trim().is_empty() => PathBuf::from(value),
        _ => std::env::current_dir().context("cwd unavailable")?,
    };

    let action_name = args.action.as_deref().unwrap_or("list");
    match action_name {
        "list" => {
            let catalog = list_catalog();
            let payload = serde_json::json!({
                "schema": SCHEMA_VERSION,
                "catalog": catalog,
            });
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("Focusa Mission Deck walkthroughs ({SCHEMA_VERSION}):");
                for id in catalog {
                    println!("  - {id}");
                }
            }
            Ok(())
        }
        "show" => {
            let id = args.walkthrough.as_deref().unwrap_or("first-mission");
            let payload = render_walkthrough(id)?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!(
                    "{}: {}",
                    payload["id"].as_str().unwrap_or("?"),
                    payload["title"].as_str().unwrap_or("?"),
                );
                let steps = payload["steps"].as_array().cloned().unwrap_or_default();
                for (idx, step) in steps.iter().enumerate() {
                    println!(
                        "  {}. {} — {}",
                        idx + 1,
                        step["title"].as_str().unwrap_or(""),
                        step["explanation"].as_str().unwrap_or(""),
                    );
                }
            }
            Ok(())
        }
        "start" => {
            let id = args.walkthrough.as_deref().unwrap_or("first-mission");
            let first_step = match id {
                "first-mission" => first_mission().steps[0].id.clone(),
                "agent-handoff" => agent_handoff().steps[0].id.clone(),
                "no-proof-no-done" => no_proof_no_done().steps[0].id.clone(),
                _ => "step-1".to_string(),
            };
            let event = WalkthroughEvent {
                walkthrough_id: id.to_string(),
                step_id: first_step,
                project_root: project_root.display().to_string(),
                continuity_id: std::env::var("FOCUSA_CONTINUITY_ID").unwrap_or_default(),
                event_type: EventType::Started,
                timestamp: Utc::now(),
                evidence_refs: vec![],
                authority_posture: AuthorityPosture::Ok,
            };
            write_event(&event)?;
            println!("started walkthrough {id} step={}", event.step_id);
            Ok(())
        }
        "advance" | "reset" | "completed" => {
            let id = args.walkthrough.as_deref().unwrap_or("first-mission");
            let step_id = args.step.clone().unwrap_or_else(|| "step-1".to_string());
            let kind = match action_name {
                "advance" => EventType::Advanced,
                "reset" => EventType::Reset,
                _ => EventType::Completed,
            };
            let event = WalkthroughEvent {
                walkthrough_id: id.to_string(),
                step_id,
                project_root: project_root.display().to_string(),
                continuity_id: std::env::var("FOCUSA_CONTINUITY_ID").unwrap_or_default(),
                event_type: kind,
                timestamp: Utc::now(),
                evidence_refs: vec![],
                authority_posture: AuthorityPosture::Ok,
            };
            write_event(&event)?;
            println!("{} {id} step={}", action_name, event.step_id);
            Ok(())
        }
        "progress" => {
            let id = args.walkthrough.as_deref().unwrap_or("first-mission");
            let prog = progress(&project_root, id)?;
            if json_mode {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "schema": SCHEMA_VERSION,
                        "walkthrough_id": id,
                        "progress": prog,
                    }))?
                );
            } else {
                println!("walkthrough {id} progress:");
                for (step, kind) in prog {
                    println!("  - {step}: {:?}", kind);
                }
            }
            Ok(())
        }
        other => {
            anyhow::bail!(
                "unknown action {other:?}; expected one of list|show|start|advance|reset|completed|progress"
            );
        }
    }
}

fn render_walkthrough(id: &str) -> Result<Value> {
    match id {
        "first-mission" => Ok(serde_json::to_value(first_mission())?),
        "agent-handoff" => Ok(serde_json::to_value(agent_handoff())?),
        "no-proof-no-done" => Ok(serde_json::to_value(no_proof_no_done())?),
        _ => anyhow::bail!("unknown walkthrough id: {id}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_constant_matches_spec() {
        assert_eq!(SCHEMA_VERSION, "focusa.walkthrough.v1");
    }

    #[test]
    fn first_mission_round_trips() {
        let wt = first_mission();
        let json = serde_json::to_string(&wt).expect("serialize");
        let back: Walkthrough = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.id, "first-mission");
        assert_eq!(back.steps.len(), 5);
        assert!(back.resettable);
    }

    #[test]
    fn agent_handoff_round_trips() {
        let wt = agent_handoff();
        let json = serde_json::to_string(&wt).expect("serialize");
        let back: Walkthrough = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.id, "agent-handoff");
        assert_eq!(back.steps.len(), 6);
        assert!(back.completion.proof_required);
    }

    #[test]
    fn no_proof_no_done_enforces_proof_precedes_completion() {
        let wt = no_proof_no_done();
        let json = serde_json::to_string(&wt).expect("serialize");
        let back: Walkthrough = serde_json::from_str(&json).expect("deserialize");
        assert!(back.completion.proof_required);
        assert!(back.completion.proof_precedes_completion);
        assert_eq!(back.completion.evidence_class, EVIDENCE_ACTUAL);
    }

    #[test]
    fn no_proof_no_done_round_trips() {
        let wt = no_proof_no_done();
        let json = serde_json::to_string(&wt).expect("serialize");
        let back: Walkthrough = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.id, "no-proof-no-done");
        assert_eq!(back.steps.len(), 5);
        assert!(back.completion.proof_required);
    }

    #[test]
    fn progress_overrides_lower_weight_events() {
        let project_root = std::env::temp_dir().join("focusa-walkthrough-test-progress");
        let _ = std::fs::remove_dir_all(&project_root);
        std::fs::create_dir_all(&project_root).unwrap();
        let wt = first_mission();
        let now = Utc::now();
        let events = vec![
            WalkthroughEvent {
                walkthrough_id: wt.id.clone(),
                step_id: "start-daemon".to_string(),
                project_root: project_root.display().to_string(),
                continuity_id: "test".to_string(),
                event_type: EventType::Started,
                timestamp: now,
                evidence_refs: vec![],
                authority_posture: AuthorityPosture::Ok,
            },
            WalkthroughEvent {
                walkthrough_id: wt.id.clone(),
                step_id: "start-daemon".to_string(),
                project_root: project_root.display().to_string(),
                continuity_id: "test".to_string(),
                event_type: EventType::Completed,
                timestamp: now,
                evidence_refs: vec!["/v1/health".to_string()],
                authority_posture: AuthorityPosture::Ok,
            },
        ];
        for evt in &events {
            write_event(evt).unwrap();
        }
        let prog = progress(&project_root, &wt.id).unwrap();
        assert_eq!(prog.get("start-daemon"), Some(&EventType::Completed));
    }
}
