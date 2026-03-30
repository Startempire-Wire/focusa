//! Application state for the TUI.

use crate::api::ApiClient;
use serde::Deserialize;
use std::collections::HashMap;

/// Active tab in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    FocusState,
    FocusStack,
    Gate,
    Events,
    Metrics,
    Lineage,
    Autonomy,
    Constitution,
    Telemetry,
    Rfm,
    Proposals,
    Skills,
    Uxp,
    Training,
}

impl Tab {
    pub const ALL: &[Tab] = &[
        Tab::FocusState,
        Tab::FocusStack,
        Tab::Gate,
        Tab::Events,
        Tab::Metrics,
        Tab::Lineage,
        Tab::Autonomy,
        Tab::Constitution,
        Tab::Telemetry,
        Tab::Rfm,
        Tab::Proposals,
        Tab::Skills,
        Tab::Uxp,
        Tab::Training,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Tab::FocusState => "State",
            Tab::FocusStack => "Stack",
            Tab::Gate => "Gate",
            Tab::Events => "Events",
            Tab::Metrics => "Metrics",
            Tab::Lineage => "CLT",
            Tab::Autonomy => "Autonomy",
            Tab::Constitution => "ACP",
            Tab::Telemetry => "Telemetry",
            Tab::Rfm => "RFM",
            Tab::Proposals => "PRE",
            Tab::Skills => "Skills",
            Tab::Uxp => "UXP",
            Tab::Training => "Export",
        }
    }

    pub fn hotkey(&self) -> &'static str {
        match self {
            Tab::FocusState => "1",
            Tab::FocusStack => "2",
            Tab::Gate => "3",
            Tab::Events => "4",
            Tab::Metrics => "5",
            Tab::Lineage => "6",
            Tab::Autonomy => "7",
            Tab::Constitution => "8",
            Tab::Telemetry => "9",
            Tab::Rfm => "0",
            Tab::Proposals => "p",
            Tab::Skills => "s",
            Tab::Uxp => "u",
            Tab::Training => "x",
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
    pub version: u64,
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
    pub connected: bool,
    pub last_error: Option<String>,
    client: ApiClient,
}

impl App {
    pub fn new(api_url: String) -> Self {
        Self {
            tab: Tab::FocusState,
            state: StateSnapshot::default(),
            extra_data: HashMap::new(),
            scroll_offset: 0,
            connected: false,
            last_error: None,
            client: ApiClient::new(api_url),
        }
    }

    pub async fn refresh(&mut self) {
        match self.client.fetch_state().await {
            Ok(snapshot) => {
                self.state = snapshot;
                self.connected = true;
                self.last_error = None;

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
            ("autonomy", "/v1/autonomy"),
            ("constitution", "/v1/constitution/active"),
            ("telemetry", "/v1/telemetry/tokens"),
            ("rfm", "/v1/rfm"),
            ("proposals", "/v1/proposals"),
            ("skills", "/v1/skills"),
            ("uxp", "/v1/uxp"),
            ("ufi", "/v1/ufi"),
            ("training", "/v1/training/status"),
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
