//! Application state for the TUI.

use crate::api::ApiClient;
use serde::Deserialize;

/// Active tab in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    FocusState,
    FocusStack,
    Gate,
    Events,
    Metrics,
}

impl Tab {
    pub const ALL: &[Tab] = &[
        Tab::FocusState,
        Tab::FocusStack,
        Tab::Gate,
        Tab::Events,
        Tab::Metrics,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Tab::FocusState => "Focus State",
            Tab::FocusStack => "Focus Stack",
            Tab::Gate => "Gate",
            Tab::Events => "Events",
            Tab::Metrics => "Metrics",
        }
    }

    pub fn hotkey(&self) -> &'static str {
        match self {
            Tab::FocusState => "1",
            Tab::FocusStack => "2",
            Tab::Gate => "3",
            Tab::Events => "4",
            Tab::Metrics => "5",
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
    #[serde(default)]
    #[allow(dead_code)]
    pub started_at: Option<String>,
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
    #[serde(default)]
    #[allow(dead_code)]
    pub parent_id: Option<String>,
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
            }
            Err(e) => {
                self.connected = false;
                self.last_error = Some(format!("{}", e));
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
