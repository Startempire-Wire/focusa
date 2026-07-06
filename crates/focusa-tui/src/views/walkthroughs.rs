//! Mission Deck Walkthroughs + Education tab (Spec 117 §13).
//!
//! Renders the three core walkthroughs in-TUI: First Mission, Agent Handoff,
//! No Proof, No Done. Mobile-friendly. Hotkey L or l.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub const WALKTHROUGH_IDS: &[&str] = &[
    "first-mission",
    "agent-handoff",
    "no-proof-no-done",
    "clean-install",
    "clean-uninstall",
];
pub const WALKTHROUGH_TITLES: &[&str] = &[
    "First Mission",
    "Agent Handoff",
    "No Proof, No Done",
    "Clean Install",
    "Clean Uninstall",
];

pub const WALKTHROUGH_AUDIENCES: &[&str] =
    &["beginner", "agent", "beginner", "beginner", "beginner"];

pub const WALKTHROUGH_STEPS: &[&[&str]] = &[
    &[
        "1. Start daemon: focusa start",
        "2. Bind project: focusa init --quickstart",
        "3. Create Workpoint: focusa workpoint checkpoint",
        "4. Attach evidence or mark proof gap explicitly",
        "5. Resume: focusa workpoint resume",
        "6. Show mission is resumable across handoff",
    ],
    &[
        "1. Show current mission: focusa trajectory view",
        "2. Show current Workpoint: focusa workpoint resume",
        "3. Render handoff packet: focusa workpoint resume --mode compact_prompt",
        "4. Show what the new agent receives",
        "5. Show drift boundaries (do_not_drift)",
        "6. Show evidence and proof expectations",
    ],
    &[
        "1. Display the agent completion claim",
        "2. Check evidence refs",
        "3. Show proof gap if missing",
        "4. Attach proof or mark proof intentionally missing",
        "5. Re-render proof meter (none | linked | verified)",
    ],
    &[
        "1. Preview plan: focusa install --dry-run --json",
        "2. Run installer: focusa install (or shell bootstrapper)",
        "3. Verify service: focusa start; focusa doctor",
        "4. Open TUI: focusa-tui (press L for Learn, post-install)",
        "5. Prove uninstall: focusa uninstall --dry-run --json",
        "6. Pair with menubar: focusa pair start",
    ],
    &[
        "1. Preview: focusa uninstall --dry-run --json",
        "2. Run: focusa uninstall --yes",
        "3. Verify: focusa --version not found",
        "4. Check: daemon stopped (systemctl is-active inactive)",
        "5. Check: ~/.focusa absent or empty",
        "6. Trust signal: clean removal proven.",
    ],
];

pub const WALKTHROUGH_WHY_IT_MATTERS: &[&str] = &[
    "Teaches the core Focusa loop: daemon → project → Workpoint → evidence → resume.",
    "Shows why Focusa exists: a new agent can recover mission, Workpoint, boundaries, and proof expectations without transcript memory.",
    "Teaches evidence discipline: an agent completion claim is not done until proof is visible or the gap is explicit.",
    "Every evaluator starts with install. A frictionless CLI+daemon install makes the first impression safe, predictable, and uninstallable.",
    "Easy uninstall is a trust signal. Evaluators keep tools they can remove cleanly; they abandon tools that leave cruft.",
];

pub const WALKTHROUGH_SUCCESS_SIGNALS: &[&str] = &[
    "A second agent can run resume and continue the same mission safely.",
    "The handoff packet states mission, next action, evidence, and do-not-drift boundaries.",
    "The proof meter shows linked/verified evidence or an explicit proof gap.",
    "focusa install --dry-run shows a clean plan; focusa install runs to completion with symlinks, service unit, PATH, and walkthrough card.",
    "focusa uninstall --dry-run shows every file/step; focusa uninstall removes daemon, service, symlinks, install root, license, and PATH edits cleanly.",
];

pub const WALKTHROUGH_ANTI_DRIFT_RULES: &[&str] = &[
    "Do not rely on transcript tail when Workpoint resume is available.",
    "Do not start unrelated work until authority and scope are visible.",
    "Do not close a task without evidence citations or an explicit blocker.",
    "Do not ship install scripts that skip service wiring with a comment saying 'Phase 2'.",
    "Do not leave evaluators wondering how to uninstall. Include it in the help, CLI, TUI, and first-run card.",
];

pub fn selected_index(app: &App) -> usize {
    app.modal_selection
        .min(WALKTHROUGH_TITLES.len().saturating_sub(1))
}

pub fn render(frame: &mut ratatui::Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(WALKTHROUGH_TITLES.len() as u16 + 2),
            Constraint::Min(0),
        ])
        .split(area);

    // Header
    let head = Paragraph::new(Line::from(vec![
        Span::styled("Learn", theme::title()),
        Span::raw(" — walkthroughs to teach Focusa end-to-end."),
    ]))
    .block(
        Block::default()
            .title(" Mission Deck · Learn ")
            .borders(Borders::ALL),
    );
    frame.render_widget(head.wrap(Wrap { trim: true }), chunks[0]);

    // Catalog
    let catalog: Vec<Line> = WALKTHROUGH_TITLES
        .iter()
        .enumerate()
        .map(|(idx, t)| {
            Line::from(format!(
                "  [{}] {} · {} · {}",
                idx + 1,
                t,
                WALKTHROUGH_AUDIENCES.get(idx).copied().unwrap_or("?"),
                WALKTHROUGH_IDS.get(idx).copied().unwrap_or("?")
            ))
        })
        .collect();
    let cat_block = Block::default().title(" Catalog ").borders(Borders::ALL);
    frame.render_widget(Paragraph::new(catalog).block(cat_block), chunks[1]);

    // First walkthrough steps (mobile-friendly detail card)
    let selected = 0usize;
    let mut detail: Vec<Line> = Vec::new();
    detail.push(Line::from(format!(
        "Currently showing: {}",
        WALKTHROUGH_TITLES[selected]
    )));
    detail.push(Line::from(""));
    detail.push(Line::from(Span::styled("Why", theme::title())));
    detail.push(Line::from(WALKTHROUGH_WHY_IT_MATTERS[selected]));
    detail.push(Line::from(""));
    detail.push(Line::from(Span::styled("Success signal", theme::title())));
    detail.push(Line::from(WALKTHROUGH_SUCCESS_SIGNALS[selected]));
    detail.push(Line::from(""));
    detail.push(Line::from(Span::styled("Anti-drift rule", theme::title())));
    detail.push(Line::from(WALKTHROUGH_ANTI_DRIFT_RULES[selected]));
    detail.push(Line::from(""));
    detail.push(Line::from(Span::styled("Steps", theme::title())));
    for step in WALKTHROUGH_STEPS[selected] {
        detail.push(Line::from(format!("  {step}")));
    }
    detail.push(Line::from(""));
    detail.push(Line::from(
        "Tip: open the CLI for full control — focusa walkthrough show --walkthrough <id>.",
    ));
    let detail_block = Block::default()
        .title(" First Mission detail ")
        .borders(Borders::ALL);
    frame.render_widget(
        Paragraph::new(detail)
            .block(detail_block)
            .wrap(Wrap { trim: true }),
        chunks[2],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walkthrough_titles_match_spec_117() {
        assert_eq!(WALKTHROUGH_TITLES.len(), 5);
        assert!(WALKTHROUGH_TITLES.contains(&"First Mission"));
        assert!(WALKTHROUGH_TITLES.contains(&"Agent Handoff"));
        assert!(WALKTHROUGH_TITLES.contains(&"No Proof, No Done"));
    }

    #[test]
    fn walkthrough_steps_match_titles_count() {
        assert_eq!(WALKTHROUGH_TITLES.len(), WALKTHROUGH_IDS.len());
        assert_eq!(WALKTHROUGH_TITLES.len(), WALKTHROUGH_STEPS.len());
        assert_eq!(WALKTHROUGH_TITLES.len(), WALKTHROUGH_AUDIENCES.len());
        assert_eq!(WALKTHROUGH_TITLES.len(), WALKTHROUGH_WHY_IT_MATTERS.len());
        assert_eq!(WALKTHROUGH_TITLES.len(), WALKTHROUGH_SUCCESS_SIGNALS.len());
        assert_eq!(WALKTHROUGH_TITLES.len(), WALKTHROUGH_ANTI_DRIFT_RULES.len());
    }

    #[test]
    fn first_mission_has_five_or_six_steps() {
        assert!(WALKTHROUGH_STEPS[0].len() >= 5);
    }

    #[test]
    fn walkthrough_lines_are_mobile_friendly() {
        for line in WALKTHROUGH_TITLES
            .iter()
            .chain(WALKTHROUGH_IDS)
            .chain(WALKTHROUGH_AUDIENCES)
        {
            assert!(line.chars().count() <= 40);
        }
    }

    #[test]
    fn education_copy_has_success_and_anti_drift_signals() {
        assert!(WALKTHROUGH_SUCCESS_SIGNALS.iter().all(|s| !s.is_empty()));
        assert!(
            WALKTHROUGH_ANTI_DRIFT_RULES
                .iter()
                .all(|s| s.contains("Do not"))
        );
    }
}
