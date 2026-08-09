//! Application state for the TUI.

use crate::activation_presenter::{
    TuiActivationView, TuiLicensePosture, project_activation_status, project_license_status,
};
use crate::api::ApiClient;
use chrono::{DateTime, Local};
use serde::Deserialize;
use std::collections::HashMap;

/// Modal layer opened on top of the Mission Control canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalKind {
    Recall,
    Learn,
    Help,
    About,
    CommandPalette,
}

impl ModalKind {
    pub fn title(self) -> &'static str {
        match self {
            ModalKind::Recall => " Recall (advisory) ",
            ModalKind::Learn => " Learn · walkthroughs ",
            ModalKind::Help => " Help · concepts overlay ",
            ModalKind::About => " About Focusa ",
            ModalKind::CommandPalette => " Command Palette ",
        }
    }
}

/// Active tab in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    DeckHome,
    FocusState,
    FocusStack,
    Gate,
    Events,
    Metrics,
    Lineage,
    WorkLoop,
    Recall,
    About,
    Autonomy,
    Constitution,
    Telemetry,
    Rfm,
    Proposals,
    Skills,
    Uxp,
    Training,
    References,
    Cache,
    Contribution,
    Intuition,
    Walkthroughs,
}

impl Tab {
    pub const ALL: &[Tab] = &[
        Tab::DeckHome,
        Tab::FocusState,
        Tab::FocusStack,
        Tab::Gate,
        Tab::Events,
        Tab::Metrics,
        Tab::Lineage,
        Tab::WorkLoop,
        Tab::Recall,
        Tab::About,
        Tab::Walkthroughs,
        Tab::Autonomy,
        Tab::Constitution,
        Tab::Telemetry,
        Tab::Rfm,
        Tab::Proposals,
        Tab::Skills,
        Tab::Uxp,
        Tab::Training,
        Tab::References,
        Tab::Cache,
        Tab::Contribution,
        Tab::Intuition,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Tab::DeckHome => "Deck Home",
            Tab::FocusState => "State",
            Tab::FocusStack => "Stack",
            Tab::Gate => "Gate",
            Tab::Events => "Events",
            Tab::Metrics => "Metrics",
            Tab::Lineage => "CLT",
            Tab::WorkLoop => "Loop",
            Tab::Recall => "Recall",
            Tab::About => "About",
            Tab::Walkthroughs => "Learn",
            Tab::Autonomy => "Autonomy",
            Tab::Constitution => "ACP",
            Tab::Telemetry => "Telemetry",
            Tab::Rfm => "RFM",
            Tab::Proposals => "PRE",
            Tab::Skills => "Skills",
            Tab::Uxp => "UXP",
            Tab::Training => "Export",
            Tab::References => "Refs",
            Tab::Cache => "Cache",
            Tab::Contribution => "Contrib",
            Tab::Intuition => "Intuition",
        }
    }

    pub fn hotkey(&self) -> &'static str {
        match self {
            Tab::DeckHome => "d",
            Tab::FocusState => "1",
            Tab::FocusStack => "2",
            Tab::Gate => "3",
            Tab::Events => "4",
            Tab::Metrics => "5",
            Tab::Lineage => "6",
            Tab::WorkLoop => "w",
            Tab::Recall => "/",
            Tab::About => "A",
            Tab::Walkthroughs => "L",
            Tab::Autonomy => "7",
            Tab::Constitution => "8",
            Tab::Telemetry => "9",
            Tab::Rfm => "0",
            Tab::Proposals => "p",
            Tab::Skills => "s",
            Tab::Uxp => "u",
            Tab::Training => "x",
            Tab::References => "r",
            Tab::Cache => "c",
            Tab::Contribution => "o",
            Tab::Intuition => "i",
        }
    }
}

/// Snapshot of Focusa state for rendering.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct StateSnapshot {
    #[serde(default)]
    pub session: Option<SessionInfo>,
    #[serde(default)]
    pub focus_stack: StackInfo,
    #[serde(default)]
    pub focus_state: Option<FocusStateInfo>,
    #[serde(default)]
    pub candidates: Vec<CandidateInfo>,
    #[serde(default)]
    pub events: Vec<EventInfo>,
    #[serde(default)]
    pub update_notification: Option<UpdateNotificationInfo>,
    #[serde(default)]
    pub version: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateNotificationInfo {
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub stale_parts: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct StackInfo {
    #[serde(default)]
    pub active_id: Option<String>,
    #[serde(default)]
    pub frames: Vec<FrameInfo>,
    #[serde(default)]
    pub stack_path: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FrameInfo {
    pub frame_id: String,
    #[serde(default)]
    pub beads_id: String,
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub depth: u32,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct FocusStateInfo {
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub decisions: Vec<String>,
    #[serde(default)]
    pub next_steps: Vec<String>,
    #[serde(default)]
    pub current_state: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CandidateInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub pressure: f64,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventInfo {
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub event_type: String,
    #[serde(default)]
    pub event_id: String,
}

/// Main application state.
pub struct App {
    pub tab: Tab,
    pub state: StateSnapshot,
    pub extra_data: HashMap<String, Option<serde_json::Value>>,
    pub scroll_offset: u16,
    pub show_help: bool,
    pub show_intro: bool,
    pub throbber_state: throbber_widgets_tui::ThrobberState,
    pub modal: Option<ModalKind>,
    pub modal_selection: usize,
    pub palette_open: bool,
    pub palette_buffer: String,
    pub connected: bool,
    pub last_error: Option<String>,
    pub last_refresh_at: Option<DateTime<Local>>,
    /// Presenter-safe activation view from `GET /v1/activation/status`
    /// (Spec 152E §21: the TUI renders the shared activation states/actions,
    /// masked identity, checkout/verify links, denial/recovery, and resume
    /// handles; it never re-decides a transition).
    pub activation: Option<TuiActivationView>,
    /// Presenter-safe entitlement posture from `GET /v1/license/status`.
    pub license: Option<TuiLicensePosture>,
    client: ApiClient,
}

fn find_json_string<'a>(value: &'a serde_json::Value, key: &str, depth: usize) -> Option<&'a str> {
    if depth == 0 {
        return None;
    }
    if let Some(found) = value.get(key).and_then(serde_json::Value::as_str) {
        return Some(found);
    }
    match value {
        serde_json::Value::Object(map) => map
            .values()
            .find_map(|child| find_json_string(child, key, depth - 1)),
        serde_json::Value::Array(values) => values
            .iter()
            .find_map(|child| find_json_string(child, key, depth - 1)),
        _ => None,
    }
}

fn encode_query_component(value: &str) -> String {
    value
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

impl App {
    pub fn new(api_url: String) -> Self {
        Self::new_with_intro(api_url, true)
    }

    pub fn new_with_intro(api_url: String, show_intro: bool) -> Self {
        Self {
            tab: Tab::DeckHome,
            state: StateSnapshot::default(),
            extra_data: HashMap::new(),
            scroll_offset: 0,
            show_help: false,
            show_intro,
            throbber_state: throbber_widgets_tui::ThrobberState::default(),
            modal: None,
            modal_selection: 0,
            palette_open: false,
            palette_buffer: String::new(),
            connected: false,
            last_error: None,
            last_refresh_at: None,
            activation: None,
            license: None,
            client: ApiClient::new(api_url),
        }
    }

    pub async fn refresh(&mut self) {
        match self.client.fetch_state().await {
            Ok(snapshot) => {
                self.state = snapshot;
                self.connected = true;
                self.last_error = None;
                self.last_refresh_at = Some(Local::now());

                // Fetch extra data for the active tab.
                self.refresh_tab_data().await;
            }
            Err(e) => {
                self.connected = false;
                self.last_error = Some(format!("{}", e));
            }
        }
    }

    async fn refresh_tab_data(&mut self) {
        let endpoints: &[(&str, &str)] = &[
            ("clt", "/v1/clt/nodes"),
            ("work_loop_status", "/v1/work-loop/status"),
            ("work_loop_replay", "/v1/work-loop/replay/closure-evidence"),
            (
                "work_loop_closure_bundle",
                "/v1/work-loop/replay/closure-bundle",
            ),
            ("autonomy", "/v1/autonomy"),
            ("constitution", "/v1/constitution/active"),
            ("telemetry", "/v1/telemetry/tokens"),
            ("rfm", "/v1/rfm"),
            ("proposals", "/v1/proposals"),
            ("skills", "/v1/skills"),
            ("uxp", "/v1/uxp"),
            ("ufi", "/v1/ufi"),
            ("training", "/v1/training/status"),
            ("project_identity", "/v1/project/identity"),
            ("workpoint_resume", "/v1/workpoint/resume"),
            ("trajectory_view", "/v1/trajectory/view"),
            (
                "instruction_integrity",
                "/v1/agent-runtime/instruction-integrity/status",
            ),
        ];

        for (key, endpoint) in endpoints {
            match self.client.fetch_json(endpoint).await {
                Ok(data) => {
                    self.extra_data.insert(key.to_string(), Some(data));
                }
                Err(_) => {
                    self.extra_data.insert(key.to_string(), None);
                }
            }
        }
        // Shared activation/entitlement presenter projections (Spec 152E §21).
        // Fail closed: an unreachable daemon or unknown posture renders as
        // `None` and the TUI shows the posture as unavailable rather than
        // inventing an activation state.
        let activation_status = self.client.fetch_json("/v1/activation/status").await.ok();
        self.activation = activation_status
            .as_ref()
            .and_then(project_activation_status);
        let license_status = self.client.fetch_json("/v1/license/status").await.ok();
        self.license = license_status.as_ref().and_then(project_license_status);
        let authority_scope = self
            .extra_data
            .get("workpoint_resume")
            .and_then(Option::as_ref)
            .and_then(|value| find_json_string(value, "continuity_id", 8))
            .and_then(|continuity_id| {
                let scope = self
                    .extra_data
                    .get("project_identity")
                    .and_then(Option::as_ref)?
                    .pointer("/project_identity/scope_ref")?;
                Some((
                    scope.get("scope_kind")?.as_str()?.to_string(),
                    scope.get("scope_id")?.as_str()?.to_string(),
                    scope.get("root_path")?.as_str()?.to_string(),
                    scope.get("canonical_name")?.as_str()?.to_string(),
                    scope.get("fingerprint")?.as_str()?.to_string(),
                    continuity_id.to_string(),
                ))
            });
        if let Some((scope_kind, scope_id, root_path, canonical_name, fingerprint, continuity_id)) =
            authority_scope
        {
            let endpoint = format!(
                "/v1/prediction-authority/projection?scope_kind={}&scope_id={}&root_path={}&canonical_name={}&fingerprint={}&continuity_id={}",
                encode_query_component(&scope_kind),
                encode_query_component(&scope_id),
                encode_query_component(&root_path),
                encode_query_component(&canonical_name),
                encode_query_component(&fingerprint),
                encode_query_component(&continuity_id),
            );
            let value = self.client.fetch_json(&endpoint).await.ok();
            self.extra_data.insert("prediction_authority".into(), value);
            let compaction_endpoint = format!(
                "/v1/compaction/policy?project_root={}&continuity_id={}",
                encode_query_component(&root_path),
                encode_query_component(&continuity_id),
            );
            let compaction_policy = self.client.fetch_json(&compaction_endpoint).await.ok();
            self.extra_data
                .insert("compaction_policy".into(), compaction_policy);
        } else {
            self.extra_data.insert("prediction_authority".into(), None);
            self.extra_data.insert("compaction_policy".into(), None);
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn dismiss_intro(&mut self) {
        self.show_intro = false;
    }

    pub fn api_url(&self) -> &str {
        self.client.base_url()
    }

    pub fn tick_throbber(&mut self) {
        self.throbber_state.calc_next();
    }

    pub fn tick_intro_dismiss(&mut self, elapsed_ms: u128) {
        if self.show_intro && elapsed_ms >= 2500 {
            self.show_intro = false;
        }
    }

    pub fn open_modal(&mut self, modal: ModalKind) {
        self.modal = Some(modal);
        self.modal_selection = 0;
    }

    pub fn close_modal(&mut self) {
        self.modal = None;
        self.modal_selection = 0;
    }

    pub fn toggle_palette(&mut self) {
        self.palette_open = !self.palette_open;
        if !self.palette_open {
            self.palette_buffer.clear();
        }
    }

    pub fn next_tab(&mut self) {
        let idx = Tab::ALL.iter().position(|t| *t == self.tab).unwrap_or(0);
        self.tab = Tab::ALL[(idx + 1) % Tab::ALL.len()];
        self.scroll_offset = 0;
    }

    pub fn prev_tab(&mut self) {
        let idx = Tab::ALL.iter().position(|t| *t == self.tab).unwrap_or(0);
        self.tab = Tab::ALL[(idx + Tab::ALL.len() - 1) % Tab::ALL.len()];
        self.scroll_offset = 0;
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }
}
