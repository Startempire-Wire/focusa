//! Application state for the TUI.

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
    client: ApiClient,
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
