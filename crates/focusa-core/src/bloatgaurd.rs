//! Spec101 Focusa Bloatgaurd — read-only budget domain model.
//!
//! The core crate owns the deterministic budget taxonomy. API/CLI/Pi/menubar
//! surfaces are thin adapters over this compact report.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BloatgaurdBudget {
    pub max_hot_response_bytes: u64,
    pub max_hot_response_items: u64,
    pub full_payload_requires_opt_in: bool,
    pub deletion_requires_human_review: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BloatgaurdDomainState {
    pub name: String,
    pub title: String,
    pub section: String,
    pub status: String,
    pub budget: BloatgaurdBudget,
    pub checks: Vec<String>,
    pub potential_findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BloatgaurdReport {
    pub schema: String,
    pub status: String,
    pub spec_ref: String,
    pub mode: String,
    pub summary: String,
    pub domains: Vec<BloatgaurdDomainState>,
}

pub fn bloatgaurd_report() -> BloatgaurdReport {
    let domains = bloatgaurd_domains();
    BloatgaurdReport {
        schema: "focusa.bloatgaurd.report.v1".to_string(),
        status: "completed".to_string(),
        spec_ref: "docs/101-focusa-bloatgaurd-spec.md#5-budget-domains".to_string(),
        mode: "read_only_budget_report".to_string(),
        summary: format!(
            "Spec101 Bloatgaurd read-only budget report: {} domains, compact hot-path envelopes, explicit full-payload gates.",
            domains.len()
        ),
        domains,
    }
}

pub fn bloatgaurd_domain(name: &str) -> Option<BloatgaurdDomainState> {
    let key = normalize_domain_name(name);
    bloatgaurd_domains().into_iter().find(|domain| {
        normalize_domain_name(&domain.name) == key || normalize_domain_name(&domain.title) == key
    })
}

pub fn bloatgaurd_domains() -> Vec<BloatgaurdDomainState> {
    vec![
        domain(
            "output-firewall",
            "5.1 Output firewall",
            "5.1",
            vec![
                "compact summary present",
                "evidence/ref handles present when larger data exists",
                "raw/full payload path gated by explicit opt-in",
                "line/byte/item caps documented for hot routes",
            ],
            vec![
                "route_without_compact_envelope",
                "ungated_full_payload",
                "hot_path_unbounded_output",
            ],
        ),
        domain(
            "tool-call-compression",
            "5.2 Tool-call compression",
            "5.2",
            vec![
                "repeated read/rg workflows have aggregate helper candidates",
                "proof commands return summaries and handles",
                "inspection helpers expose json or bounded text mode",
            ],
            vec![
                "repeated_manual_probe_pattern",
                "proof_output_too_verbose",
                "missing_summary_mode",
            ],
        ),
        domain(
            "docs-diet",
            "5.3 Docs diet",
            "5.3",
            vec![
                "doc size budget by directory/class",
                "repeated command blocks detected",
                "current docs link to generated proof bundle maps",
                "historical material marked archive/spec/worksheet",
            ],
            vec![
                "oversized_current_doc",
                "duplicated_proof_commands",
                "stale_history_in_current_doc",
            ],
        ),
        domain(
            "test-diet",
            "5.4 Test diet",
            "5.4",
            vec![
                "shell suite length budget",
                "new shell scripts require comment/head justification",
                "static checks prefer Rust/core logic where stable",
            ],
            vec![
                "oversized_shell_suite",
                "script_duplicates_core_logic",
                "missing_rust_migration_note",
            ],
        ),
        domain(
            "prompt-context-diet",
            "5.5 Prompt/context diet",
            "5.5",
            vec![
                "Focus Slice section caps",
                "evidence refs instead of raw blobs",
                "full lineage/ontology/telemetry gated behind traversal/cold opt-in",
            ],
            vec![
                "raw_context_dump",
                "transcript_tail_authority",
                "uncapped_focus_slice_section",
            ],
        ),
        domain(
            "rust-first-core",
            "5.6 Rust-first core",
            "5.6",
            vec![
                "packet building/filtering/scoring/proof mapping lives in Rust or has migration note",
                "JS/Pi/CLI surfaces remain adapters where practical",
                "new scripts do not duplicate core logic",
            ],
            vec![
                "adapter_contains_core_logic",
                "script_duplicates_core_logic",
                "rust_migration_candidate",
            ],
        ),
        domain(
            "dead-code-safety",
            "5.7 Dead code and brownfield cleanup safety",
            "5.7",
            vec![
                "dead-code grade A-D assigned before removal",
                "dynamic/public/future-facing surfaces protected",
                "deletion never automatic",
            ],
            vec![
                "unsafe_delete_candidate",
                "missing_protection_reason",
                "dead_code_grade_missing",
            ],
        ),
        domain(
            "adaptive-router",
            "5.8 Adaptive Bloatgaurd router/classifier",
            "5.8",
            vec![
                "classifier is advisory for cleanup/context routing",
                "enforcement stays deterministic",
                "router cannot authorize deletion or full-payload exposure",
            ],
            vec![
                "classifier_over_authorized",
                "gate_mode_missing",
                "retrieval_budget_missing",
            ],
        ),
    ]
}

fn domain(
    name: &str,
    title: &str,
    section: &str,
    checks: Vec<&str>,
    potential_findings: Vec<&str>,
) -> BloatgaurdDomainState {
    BloatgaurdDomainState {
        name: name.to_string(),
        title: title.to_string(),
        section: section.to_string(),
        status: "ready".to_string(),
        budget: BloatgaurdBudget {
            max_hot_response_bytes: 8_000,
            max_hot_response_items: 50,
            full_payload_requires_opt_in: true,
            deletion_requires_human_review: true,
        },
        checks: checks.into_iter().map(str::to_string).collect(),
        potential_findings: potential_findings.into_iter().map(str::to_string).collect(),
    }
}

fn normalize_domain_name(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace(['_', ' '], "-")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenbloatControl {
    pub name: String,
    pub title: String,
    pub section: String,
    pub status: String,
    pub prompt_visible_fields: Vec<String>,
    pub required_boundaries: Vec<String>,
    pub potential_findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenbloatReport {
    pub schema: String,
    pub status: String,
    pub spec_ref: String,
    pub summary: String,
    pub controls: Vec<TokenbloatControl>,
}

pub fn tokenbloat_report() -> TokenbloatReport {
    let controls = tokenbloat_controls();
    TokenbloatReport {
        schema: "focusa.bloatgaurd.tokenbloat_report.v1".to_string(),
        status: "completed".to_string(),
        spec_ref: "docs/101-focusa-bloatgaurd-spec.md#59-tokenbloat-control-domain".to_string(),
        summary: format!(
            "Spec101 Tokenbloat Control report: {} controls covering stable-prefix compression and tool-call history elision.",
            controls.len()
        ),
        controls,
    }
}

pub fn tokenbloat_control(name: &str) -> Option<TokenbloatControl> {
    let key = normalize_domain_name(name);
    tokenbloat_controls().into_iter().find(|control| {
        normalize_domain_name(&control.name) == key || normalize_domain_name(&control.title) == key
    })
}

pub fn tokenbloat_controls() -> Vec<TokenbloatControl> {
    vec![
        token_control(
            "tokenbloat-control",
            "5.9 Tokenbloat Control Domain",
            "5.9",
            vec![
                "tool or route name",
                "target object/path/endpoint",
                "compact result and evidence handle",
                "rehydrate route for exact raw output",
            ],
            vec![
                "safety and authority boundaries are not compressed without allowlist",
                "full payload exposure remains cold opt-in",
                "classifier output is advisory; deterministic checks enforce",
            ],
            vec![
                "stable_prefix_churn",
                "raw_transcript_leakage",
                "duplicate_block_repeated",
                "full_payload_recommended_without_cold_opt_in",
            ],
        ),
        token_control(
            "tool-call-history-elision",
            "5.10 Tool-call history elision and structured rehydration",
            "5.10",
            vec![
                "tool or route name",
                "action type",
                "omitted byte/line count",
                "linked decision/constraint/failure/workpoint",
            ],
            vec![
                "lossless raw output lives outside the hot prompt",
                "lossy prompt summary is compact and rehydratable",
                "evidence handles replace transcript blobs",
            ],
            vec![
                "tool_output_flood",
                "missing_rehydrate_ref",
                "raw_tool_history_in_focus_slice",
                "omitted_count_missing",
            ],
        ),
    ]
}

fn token_control(
    name: &str,
    title: &str,
    section: &str,
    prompt_visible_fields: Vec<&str>,
    required_boundaries: Vec<&str>,
    potential_findings: Vec<&str>,
) -> TokenbloatControl {
    TokenbloatControl {
        name: name.to_string(),
        title: title.to_string(),
        section: section.to_string(),
        status: "ready".to_string(),
        prompt_visible_fields: prompt_visible_fields
            .into_iter()
            .map(str::to_string)
            .collect(),
        required_boundaries: required_boundaries
            .into_iter()
            .map(str::to_string)
            .collect(),
        potential_findings: potential_findings.into_iter().map(str::to_string).collect(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BloatgaurdGateThresholds {
    pub max_hot_response_bytes: u64,
    pub max_hot_response_items: u64,
    pub max_findings_before_warning: u64,
    pub max_findings_before_fail_candidate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BloatgaurdGateMode {
    pub code: String,
    pub name: String,
    pub title: String,
    pub status: String,
    pub enforcement: String,
    pub thresholds: BloatgaurdGateThresholds,
    pub allowlist: Vec<String>,
    pub report_schema_fields: Vec<String>,
    pub allowed_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BloatgaurdGateModesReport {
    pub schema: String,
    pub status: String,
    pub spec_ref: String,
    pub summary: String,
    pub modes: Vec<BloatgaurdGateMode>,
}

pub fn bloatgaurd_gate_modes_report() -> BloatgaurdGateModesReport {
    let modes = bloatgaurd_gate_modes();
    BloatgaurdGateModesReport {
        schema: "focusa.bloatgaurd.gate_modes_report.v1".to_string(),
        status: "completed".to_string(),
        spec_ref: "docs/101-focusa-bloatgaurd-spec.md#58-adaptive-bloatgaurd-routerclassifier"
            .to_string(),
        summary: format!(
            "Spec101 Bloatgaurd gate modes: {} deterministic modes (A advisory, B warning, C fail-candidate) with thresholds, allowlist, and report schema fields.",
            modes.len()
        ),
        modes,
    }
}

pub fn bloatgaurd_gate_mode(name: &str) -> Option<BloatgaurdGateMode> {
    let key = normalize_domain_name(name);
    bloatgaurd_gate_modes().into_iter().find(|mode| {
        normalize_domain_name(&mode.code) == key
            || normalize_domain_name(&mode.name) == key
            || normalize_domain_name(&mode.title) == key
    })
}

pub fn bloatgaurd_gate_modes() -> Vec<BloatgaurdGateMode> {
    vec![
        gate_mode(
            "A",
            "advisory",
            "Mode A — advisory report",
            "advisory_only",
            BloatgaurdGateThresholds {
                max_hot_response_bytes: 8_000,
                max_hot_response_items: 50,
                max_findings_before_warning: 999,
                max_findings_before_fail_candidate: 999,
            },
            vec![
                "all_existing_surfaces",
                "baseline_collection",
                "manual_review",
            ],
            vec!["report", "document"],
        ),
        gate_mode(
            "B",
            "warning",
            "Mode B — warning gate",
            "warning_nonblocking",
            BloatgaurdGateThresholds {
                max_hot_response_bytes: 8_000,
                max_hot_response_items: 50,
                max_findings_before_warning: 1,
                max_findings_before_fail_candidate: 999,
            },
            vec![
                "documented_exception",
                "generated_surface",
                "compatibility_surface",
            ],
            vec!["report", "document", "isolate", "migrate"],
        ),
        gate_mode(
            "C",
            "fail-candidate",
            "Mode C — fail-candidate gate",
            "fail_candidate_with_allowlist",
            BloatgaurdGateThresholds {
                max_hot_response_bytes: 12_000,
                max_hot_response_items: 75,
                max_findings_before_warning: 1,
                max_findings_before_fail_candidate: 3,
            },
            vec![
                "explicit_ci_allowlist",
                "protected_public_surface",
                "operator_approved_exception",
            ],
            vec![
                "report",
                "document",
                "isolate",
                "deprecate",
                "split",
                "migrate",
                "delete_candidate",
            ],
        ),
    ]
}

fn gate_mode(
    code: &str,
    name: &str,
    title: &str,
    enforcement: &str,
    thresholds: BloatgaurdGateThresholds,
    allowlist: Vec<&str>,
    allowed_actions: Vec<&str>,
) -> BloatgaurdGateMode {
    BloatgaurdGateMode {
        code: code.to_string(),
        name: name.to_string(),
        title: title.to_string(),
        status: "ready".to_string(),
        enforcement: enforcement.to_string(),
        thresholds,
        allowlist: allowlist.into_iter().map(str::to_string).collect(),
        report_schema_fields: vec![
            "schema".to_string(),
            "status".to_string(),
            "mode".to_string(),
            "finding_count".to_string(),
            "thresholds".to_string(),
            "allowlist_matches".to_string(),
            "recommended_gate_mode".to_string(),
            "allowed_actions".to_string(),
            "evidence_refs".to_string(),
        ],
        allowed_actions: allowed_actions.into_iter().map(str::to_string).collect(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BloatgaurdProfile {
    pub name: String,
    pub title: String,
    pub mode: String,
    pub output_firewall: bool,
    pub tool_history_elision: String,
    pub context_compiler: String,
    pub semantic_cache: String,
    pub full_payload_policy: String,
    pub dynamic_slice_budget: String,
    pub rehydration_bias: String,
    pub toggles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BloatgaurdProfilesReport {
    pub schema: String,
    pub status: String,
    pub spec_ref: String,
    pub safety_rules: Vec<String>,
    pub operator_switches: Vec<String>,
    pub profiles: Vec<BloatgaurdProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BloatgaurdRoutine {
    pub name: String,
    pub title: String,
    pub command: String,
    pub manual: bool,
    pub automatic_triggers: Vec<String>,
    pub writes_code: bool,
    pub can_fail_ci: String,
    pub cold_payload_allowed: String,
    pub safe_default: String,
    pub recommended_profiles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BloatgaurdRoutinesReport {
    pub schema: String,
    pub status: String,
    pub spec_ref: String,
    pub automation_rules: Vec<String>,
    pub routines: Vec<BloatgaurdRoutine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BloatgaurdRolloutReport {
    pub schema: String,
    pub status: String,
    pub spec_ref: String,
    pub phases: Vec<String>,
    pub acceptance_checks: Vec<String>,
    pub resolved_open_questions: Vec<String>,
    pub proof_commands: Vec<String>,
}

pub fn bloatgaurd_profiles_report() -> BloatgaurdProfilesReport {
    BloatgaurdProfilesReport {
        schema: "focusa.bloatgaurd.profiles_report.v1".to_string(),
        status: "completed".to_string(),
        spec_ref: "docs/101-focusa-bloatgaurd-spec.md#16-preconfigured-profiles".to_string(),
        safety_rules: vec![
            "no automatic deletion".to_string(),
            "no classifier-only enforcement".to_string(),
            "no hidden full-payload default in hot paths".to_string(),
            "no cross-project semantic cache without project_root + continuity_id".to_string(),
            "no elision that drops active blocker evidence without a rehydrate ref".to_string(),
        ],
        operator_switches: vec![
            "temporary_disable".to_string(),
            "strict_mode".to_string(),
            "advisory_mode".to_string(),
            "cold_opt_in_exact_raw_payloads".to_string(),
            "protected_surface_allowlist".to_string(),
        ],
        profiles: bloatgaurd_profiles(),
    }
}

pub fn bloatgaurd_profile(name: &str) -> Option<BloatgaurdProfile> {
    let key = normalize_domain_name(name);
    bloatgaurd_profiles().into_iter().find(|profile| {
        normalize_domain_name(&profile.name) == key || normalize_domain_name(&profile.title) == key
    })
}

pub fn bloatgaurd_profiles() -> Vec<BloatgaurdProfile> {
    vec![
        profile(
            "daily_driver",
            "Daily Driver",
            "advisory",
            "after_checkpoint",
            "advisory",
            "scoped",
            "cold_opt_in",
            "medium",
            "medium",
            vec!["output_firewall", "semantic_cache", "context_compiler"],
        ),
        profile(
            "beast_mode",
            "Beast Mode",
            "advisory",
            "advisory",
            "advisory",
            "scoped",
            "cold_opt_in",
            "high",
            "high",
            vec!["output_firewall", "semantic_cache", "low_compression_bias"],
        ),
        profile(
            "speedy",
            "Speedy",
            "warning",
            "after_checkpoint",
            "required_for_broad_reads",
            "scoped",
            "cold_opt_in",
            "low",
            "low",
            vec!["output_firewall", "stable_prefix_churn_gate"],
        ),
        profile(
            "neat_freak",
            "Neat Freak",
            "advisory",
            "after_checkpoint",
            "required_for_broad_reads",
            "scoped",
            "cold_opt_in",
            "medium",
            "medium",
            vec![
                "dead_code_inventory",
                "duplicate_dedupe",
                "docs_budget",
                "shell_script_budget",
            ],
        ),
        profile(
            "tightwad",
            "Tightwad",
            "fail_closed",
            "after_checkpoint",
            "required_for_broad_reads",
            "scoped",
            "deny_hot_path",
            "low",
            "low",
            vec![
                "output_firewall",
                "duplicate_dedupe",
                "docs_budget",
                "shell_script_budget",
                "stable_prefix_churn_gate",
                "adaptive_router",
            ],
        ),
    ]
}

fn profile(
    name: &str,
    title: &str,
    mode: &str,
    tool_history_elision: &str,
    context_compiler: &str,
    semantic_cache: &str,
    full_payload_policy: &str,
    dynamic_slice_budget: &str,
    rehydration_bias: &str,
    toggles: Vec<&str>,
) -> BloatgaurdProfile {
    BloatgaurdProfile {
        name: name.to_string(),
        title: title.to_string(),
        mode: mode.to_string(),
        output_firewall: true,
        tool_history_elision: tool_history_elision.to_string(),
        context_compiler: context_compiler.to_string(),
        semantic_cache: semantic_cache.to_string(),
        full_payload_policy: full_payload_policy.to_string(),
        dynamic_slice_budget: dynamic_slice_budget.to_string(),
        rehydration_bias: rehydration_bias.to_string(),
        toggles: toggles.into_iter().map(str::to_string).collect(),
    }
}

pub fn bloatgaurd_routines_report() -> BloatgaurdRoutinesReport {
    BloatgaurdRoutinesReport {
        schema: "focusa.bloatgaurd.routines_report.v1".to_string(),
        status: "completed".to_string(),
        spec_ref: "docs/101-focusa-bloatgaurd-spec.md#17-named-routines-and-automation-policy".to_string(),
        automation_rules: vec![
            "automatic routines require verified project_root + continuity_id scope".to_string(),
            "context routines consume Spec100 Context Cognition before separate context selection".to_string(),
            "automatic routines emit report handles or evidence refs".to_string(),
            "automatic routines do not delete, archive, rewrite, or disable code".to_string(),
            "CI failure belongs to The Gatekeeper under Tightwad or explicitly selected strict modes".to_string(),
        ],
        routines: bloatgaurd_routines(),
    }
}

pub fn bloatgaurd_routine(name: &str) -> Option<BloatgaurdRoutine> {
    let key = normalize_domain_name(name);
    bloatgaurd_routines().into_iter().find(|routine| {
        normalize_domain_name(&routine.name) == key || normalize_domain_name(&routine.title) == key
    })
}

pub fn bloatgaurd_routines() -> Vec<BloatgaurdRoutine> {
    vec![
        routine(
            "patrol",
            "The Patrol",
            "bloatgaurd scan",
            vec!["ci_advisory", "nightly", "pre_pr"],
            "profile-dependent",
            "no",
            "report_only",
            vec!["daily_driver", "neat_freak", "tightwad"],
        ),
        routine(
            "brief",
            "The Brief",
            "bloatgaurd report",
            vec!["after_scan", "daily", "weekly"],
            "no",
            "no",
            "report_only",
            vec!["daily_driver", "neat_freak"],
        ),
        routine(
            "squeezer",
            "The Squeezer",
            "bloatgaurd squeeze",
            vec![
                "after_checkpoint",
                "before_compaction",
                "token_pressure_high",
            ],
            "no",
            "no",
            "no_raw_deletion",
            vec!["daily_driver", "speedy", "tightwad"],
        ),
        routine(
            "librarian",
            "The Librarian",
            "bloatgaurd context",
            vec!["broad_read_trigger", "audit_start"],
            "no",
            "bounded context only",
            "bounded_bundle_only",
            vec!["beast_mode", "neat_freak"],
        ),
        routine(
            "janitor",
            "The Janitor",
            "bloatgaurd dedupe",
            vec!["ci_advisory", "docs_changed"],
            "warning/fail candidate only",
            "no",
            "advisory_only",
            vec!["neat_freak", "tightwad"],
        ),
        routine(
            "pantry",
            "The Pantry",
            "bloatgaurd cache",
            vec!["project_open", "docs_changed", "checkpoint"],
            "no",
            "no",
            "scoped_cache_only",
            vec!["daily_driver", "speedy"],
        ),
        routine(
            "gatekeeper",
            "The Gatekeeper",
            "bloatgaurd check",
            vec!["ci", "pre_commit", "release_gate"],
            "yes",
            "no",
            "profile_controls_failure_mode",
            vec!["tightwad"],
        ),
        routine(
            "xray",
            "The X-Ray",
            "bloatgaurd explain",
            vec![],
            "no",
            "no",
            "read_only",
            vec!["daily_driver", "beast_mode", "neat_freak"],
        ),
        routine(
            "deep_dive",
            "The Deep Dive",
            "bloatgaurd rehydrate",
            vec!["active_blocker_only"],
            "no",
            "yes, explicit handle only",
            "explicit_handle_required",
            vec!["beast_mode"],
        ),
        routine(
            "scout",
            "The Scout",
            "bloatgaurd route",
            vec!["before_broad_read", "before_audit", "token_pressure_high"],
            "no",
            "no",
            "advisory_only",
            vec!["daily_driver", "speedy", "neat_freak"],
        ),
    ]
}

fn routine(
    name: &str,
    title: &str,
    command: &str,
    automatic_triggers: Vec<&str>,
    can_fail_ci: &str,
    cold_payload_allowed: &str,
    safe_default: &str,
    recommended_profiles: Vec<&str>,
) -> BloatgaurdRoutine {
    BloatgaurdRoutine {
        name: name.to_string(),
        title: title.to_string(),
        command: command.to_string(),
        manual: true,
        automatic_triggers: automatic_triggers.into_iter().map(str::to_string).collect(),
        writes_code: false,
        can_fail_ci: can_fail_ci.to_string(),
        cold_payload_allowed: cold_payload_allowed.to_string(),
        safe_default: safe_default.to_string(),
        recommended_profiles: recommended_profiles
            .into_iter()
            .map(str::to_string)
            .collect(),
    }
}

pub fn bloatgaurd_rollout_report() -> BloatgaurdRolloutReport {
    BloatgaurdRolloutReport {
        schema: "focusa.bloatgaurd.rollout_report.v1".to_string(),
        status: "completed".to_string(),
        spec_ref: "docs/101-focusa-bloatgaurd-spec.md#10-rollout-plan".to_string(),
        phases: vec![
            "measure_repo_baseline_in_advisory_mode".to_string(),
            "classify_false_positives_and_intentional_exceptions".to_string(),
            "add_allowlist_with_review_dates".to_string(),
            "turn_on_warning_gate_for_new_violations".to_string(),
            "promote_high_confidence_checks_to_fail_closed".to_string(),
            "periodically_reduce_allowlist_via_core_migrations".to_string(),
        ],
        acceptance_checks: vec![
            "profiles_report_exposes_daily_driver_to_tightwad".to_string(),
            "routines_report_exposes_patrol_to_scout".to_string(),
            "gate_modes_report_exposes_A_B_C_thresholds_allowlist_schema".to_string(),
            "tokenbloat_report_exposes_5_9_5_10_controls".to_string(),
            "all_surfaces_have_api_cli_pi_docs_and_static_audit_coverage".to_string(),
        ],
        resolved_open_questions: vec![
            "deletion remains out_of_scope_without_operator_review".to_string(),
            "classifier_output_is_advisory_only".to_string(),
            "strict CI failure is limited to Gatekeeper/Tightwad or selected strict mode"
                .to_string(),
            "full payload remains cold_opt_in with explicit handle".to_string(),
        ],
        proof_commands: vec![
            "python3 tests/spec101_bloatgaurd_budgets_static_test.py".to_string(),
            "node scripts/audit-focusa-bloatgaurd-budgets.mjs".to_string(),
            "node scripts/validate-focusa-tool-contracts.mjs".to_string(),
            "cargo test --workspace".to_string(),
            "cargo clippy --workspace -- -D warnings".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_has_all_spec101_budget_domains() {
        let report = bloatgaurd_report();
        assert_eq!(report.domains.len(), 8);
        assert!(report.domains.iter().any(|d| d.section == "5.1"));
        assert!(
            report
                .domains
                .iter()
                .all(|d| d.budget.full_payload_requires_opt_in)
        );
        assert!(
            report
                .domains
                .iter()
                .all(|d| d.budget.deletion_requires_human_review)
        );
    }

    #[test]
    fn domain_lookup_accepts_slug_or_title() {
        assert_eq!(bloatgaurd_domain("output_firewall").unwrap().section, "5.1");
        assert_eq!(
            bloatgaurd_domain("5.8 Adaptive Bloatgaurd router/classifier")
                .unwrap()
                .name,
            "adaptive-router"
        );
        assert!(bloatgaurd_domain("missing").is_none());
    }

    #[test]
    fn tokenbloat_report_has_control_and_elision_domains() {
        let report = tokenbloat_report();
        assert_eq!(report.controls.len(), 2);
        assert!(report.controls.iter().any(|d| d.section == "5.9"));
        assert!(report.controls.iter().any(|d| d.section == "5.10"));
        assert_eq!(
            tokenbloat_control("tool_call_history_elision")
                .unwrap()
                .section,
            "5.10"
        );
    }

    #[test]
    fn gate_modes_report_has_modes_thresholds_and_schema_fields() {
        let report = bloatgaurd_gate_modes_report();
        assert_eq!(report.modes.len(), 3);
        assert!(report.modes.iter().any(|mode| mode.code == "A"));
        assert!(report.modes.iter().any(|mode| mode.code == "B"));
        assert!(report.modes.iter().any(|mode| mode.code == "C"));
        let mode_c = bloatgaurd_gate_mode("fail_candidate").unwrap();
        assert_eq!(mode_c.code, "C");
        assert!(mode_c.thresholds.max_findings_before_fail_candidate > 0);
        assert!(
            mode_c
                .report_schema_fields
                .contains(&"recommended_gate_mode".to_string())
        );
        assert!(
            mode_c
                .allowlist
                .contains(&"explicit_ci_allowlist".to_string())
        );
    }

    #[test]
    fn profiles_routines_and_rollout_are_complete() {
        let profiles = bloatgaurd_profiles_report();
        assert_eq!(profiles.profiles.len(), 5);
        assert!(bloatgaurd_profile("daily-driver").is_some());
        assert_eq!(bloatgaurd_profile("tightwad").unwrap().mode, "fail_closed");
        assert!(
            profiles
                .operator_switches
                .contains(&"strict_mode".to_string())
        );

        let routines = bloatgaurd_routines_report();
        assert_eq!(routines.routines.len(), 10);
        assert!(bloatgaurd_routine("patrol").is_some());
        assert_eq!(
            bloatgaurd_routine("scout").unwrap().safe_default,
            "advisory_only"
        );
        assert!(
            routines
                .automation_rules
                .iter()
                .any(|rule| rule.contains("project_root"))
        );

        let rollout = bloatgaurd_rollout_report();
        assert_eq!(rollout.phases.len(), 6);
        assert!(
            rollout
                .acceptance_checks
                .iter()
                .any(|check| check.contains("profiles_report"))
        );
        assert!(
            rollout
                .resolved_open_questions
                .iter()
                .any(|q| q.contains("deletion"))
        );
    }
}
