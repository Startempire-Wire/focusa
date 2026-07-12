//! Ratatui renderer for the Hybrid AC install surface.

use ratatui::{Frame, layout::{Alignment, Constraint, Direction, Layout as RLayout, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Gauge, Paragraph, Row, Table}};
use super::{completion::InstallCompletionSummary, continuity_core::ContinuityCore, glow_base::GlowBase, layout::{Layout, LayoutKind}, matrix_rain::MatrixRain, palette::{Ansi256Palette, MonochromePalette, TrueColorPalette}, state::{InstallState, PhaseStatus}};

pub struct HybridRenderer { pub seed: u64, pub tick: u64, pub core: ContinuityCore, pub rain: MatrixRain }
impl HybridRenderer {
    pub fn new(seed: u64) -> Self { Self { seed, tick: 0, core: ContinuityCore::new(seed), rain: MatrixRain::new(seed, 120, 32) } }
    pub fn tick(&mut self) { self.tick = self.tick.wrapping_add(1); self.rain.update(1.0 / 30.0); }
    pub fn render(&mut self, frame: &mut Frame<'_>, state: &InstallState, mode: super::super::capabilities::InstallRendererMode) {
        let area=frame.area(); let layout=Layout::for_area(area); if layout.kind==LayoutKind::Plain { return; }
        let (bg, text, muted, accent) = match mode { super::super::capabilities::InstallRendererMode::TrueColorAnimated => (TrueColorPalette::BACKGROUND,TrueColorPalette::TEXT,TrueColorPalette::MUTED,TrueColorPalette::CYAN), super::super::capabilities::InstallRendererMode::Ansi256Animated => (Ansi256Palette::BACKGROUND,Ansi256Palette::TEXT,Ansi256Palette::MUTED,Ansi256Palette::CYAN), _ => (MonochromePalette::BACKGROUND,MonochromePalette::TEXT,MonochromePalette::MUTED,MonochromePalette::CYAN) };
        frame.render_widget(Block::default().style(Style::default().bg(bg)), area);
        let chunks=RLayout::default().direction(Direction::Vertical).constraints([Constraint::Length(3),Constraint::Min(4),Constraint::Length(6)]).split(area);
        frame.render_widget(Paragraph::new(vec![Line::from(Span::styled("FOCUSA INSTALL",Style::default().fg(accent))),Line::from(Span::styled("Local-first mission cohesion for AI coding agents.",Style::default().fg(muted)))]).alignment(Alignment::Center),chunks[0]);
        self.render_art(frame, layout.art, state, accent, muted);
        let pct=(state.phase_completion*100.0).clamp(0.0,100.0); frame.render_widget(Gauge::default().block(Block::default().borders(Borders::TOP).title("phase completion")).gauge_style(Style::default().fg(accent)).percent(pct as u16),chunks[2]);
        if layout.rail.width>0 { let rows=state.phases.iter().map(|(p,s)| Row::new(vec![format!("{} {}",InstallState::status_symbol(*s),p.label())])); frame.render_widget(Table::new(rows,[Constraint::Percentage(100)]).block(Block::default().borders(Borders::LEFT).title("phases")).style(Style::default().fg(text)),layout.rail); }
    }
    fn render_art(&self, frame: &mut Frame<'_>, area: Rect, state: &InstallState, accent: Color, muted: Color) { let title=if state.rollback_active{"↶ Rolling back safely"} else if state.failure.is_some(){"✗ Installation failed"} else {"Continuity Core"}; let lines=vec![Line::from(Span::styled(title,Style::default().fg(accent))),Line::from(Span::styled("Matrix field · infrastructure platform",Style::default().fg(muted))),Line::from(""),Line::from(Span::styled("        ◇  ◇  ◇",Style::default().fg(accent))),Line::from(Span::styled("      ╱   ◉   ╲",Style::default().fg(accent))),Line::from(Span::styled("        ▔▔▔",Style::default().fg(accent)))]; frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center),area); }
    pub fn completion_hold_ms() -> u16 { 700 }
    pub fn completion_summary(&self, _summary: &InstallCompletionSummary) {}
}
