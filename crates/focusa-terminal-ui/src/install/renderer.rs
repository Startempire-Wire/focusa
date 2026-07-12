//! Ratatui renderer for the Hybrid AC install surface.

use super::{
    canvas::{BlockCanvas, Pixel},
    completion::InstallCompletionSummary,
    continuity_core::ContinuityCore,
    glow_base::GlowBase,
    layout::{Layout, LayoutKind},
    matrix_rain::MatrixRain,
    palette::{Ansi256Palette, MonochromePalette, TrueColorPalette},
    state::{InstallState, PhaseStatus},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout as RLayout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table},
    Frame,
};

pub struct HybridRenderer {
    pub seed: u64,
    pub tick: u64,
    pub core: ContinuityCore,
    pub rain: MatrixRain,
}
fn display_color(color: Color, mode: super::super::capabilities::InstallRendererMode) -> Color {
    match mode {
        super::super::capabilities::InstallRendererMode::TrueColorAnimated => color,
        super::super::capabilities::InstallRendererMode::Ansi256Animated => match color {
            Color::Rgb(r, g, b) => {
                let index =
                    16 + 36 * (r as u16 * 5 / 255) + 6 * (g as u16 * 5 / 255) + b as u16 * 5 / 255;
                Color::Indexed(index.min(231) as u8)
            }
            other => other,
        },
        _ => match color {
            Color::Rgb(r, g, b) => {
                let luma = (u16::from(r) * 30 + u16::from(g) * 59 + u16::from(b) * 11) / 100;
                if luma < 48 {
                    Color::Black
                } else if luma < 160 {
                    Color::DarkGray
                } else {
                    Color::White
                }
            }
            Color::Indexed(_) => Color::White,
            other => other,
        },
    }
}

impl HybridRenderer {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            tick: 0,
            core: ContinuityCore::new(seed),
            rain: MatrixRain::new(seed, 120, 32),
        }
    }
    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.rain.update(1.0 / 30.0);
    }
    pub fn render(
        &mut self,
        frame: &mut Frame<'_>,
        state: &InstallState,
        mode: super::super::capabilities::InstallRendererMode,
    ) {
        let area = frame.area();
        let layout = Layout::for_area(area);
        if layout.kind == LayoutKind::Plain {
            return;
        }
        let (bg, text, muted, accent) = match mode {
            super::super::capabilities::InstallRendererMode::TrueColorAnimated => (
                TrueColorPalette::BACKGROUND,
                TrueColorPalette::TEXT,
                TrueColorPalette::MUTED,
                TrueColorPalette::CYAN,
            ),
            super::super::capabilities::InstallRendererMode::Ansi256Animated => (
                Ansi256Palette::BACKGROUND,
                Ansi256Palette::TEXT,
                Ansi256Palette::MUTED,
                Ansi256Palette::CYAN,
            ),
            _ => (
                MonochromePalette::BACKGROUND,
                MonochromePalette::TEXT,
                MonochromePalette::MUTED,
                MonochromePalette::CYAN,
            ),
        };
        frame.render_widget(Block::default().style(Style::default().bg(bg)), area);
        let chunks = RLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(4),
                Constraint::Length(6),
            ])
            .split(area);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled("FOCUSA INSTALL", Style::default().fg(accent))),
                Line::from(Span::styled(
                    "Local-first mission cohesion for AI coding agents.",
                    Style::default().fg(muted),
                )),
            ])
            .alignment(Alignment::Center),
            chunks[0],
        );
        self.render_art(frame, layout.art, state, mode, bg, accent, muted);
        let pct = (state.phase_completion * 100.0).clamp(0.0, 100.0);
        frame.render_widget(
            Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::TOP)
                        .title("phase completion"),
                )
                .gauge_style(Style::default().fg(accent))
                .percent(pct as u16),
            chunks[2],
        );
        if layout.rail.width > 0 {
            let rows = state.phases.iter().map(|(p, s)| {
                Row::new(vec![format!(
                    "{} {}",
                    InstallState::status_symbol(*s),
                    p.label()
                )])
            });
            frame.render_widget(
                Table::new(rows, [Constraint::Percentage(100)])
                    .block(Block::default().borders(Borders::LEFT).title("phases"))
                    .style(Style::default().fg(text)),
                layout.rail,
            );
        }
    }
    fn render_art(
        &mut self,
        frame: &mut Frame<'_>,
        area: Rect,
        state: &InstallState,
        mode: super::super::capabilities::InstallRendererMode,
        background: Color,
        accent: Color,
        muted: Color,
    ) {
        // The art is a real logical half-block canvas. The renderer owns no
        // installer decisions: assembly and failure colors are derived only
        // from the already-reduced presentation state.
        let logical_width = area.width;
        let logical_height = area.height.saturating_mul(2);
        let mut canvas = BlockCanvas::new(logical_width, logical_height);
        canvas.clear(background);

        let base_rows = (logical_height / 5).max(2);
        let base = GlowBase::new(logical_width, base_rows);
        let failed = state.failure.is_some() || state.rollback_active;
        let active = !failed && state.phase_completion < 1.0;
        for row in 0..base_rows {
            let y = logical_height.saturating_sub(base_rows) + row;
            for x in 0..logical_width {
                let color = base.color_at(x, row, active, failed);
                canvas.set(
                    x,
                    y,
                    Pixel {
                        top: color,
                        bottom: color,
                    },
                );
            }
        }

        let core_width = 32u16.min(logical_width.saturating_sub(2));
        let core_height = 32u16.min(logical_height.saturating_sub(base_rows + 1));
        if core_width >= 16 && core_height >= 16 {
            let origin_x = logical_width.saturating_sub(core_width) / 2;
            let origin_y = logical_height.saturating_sub(base_rows + core_height) / 2;
            let assembly = state.phase_completion.clamp(0.0, 1.0);
            self.core.render(&mut canvas, origin_x, origin_y, assembly);
            if state.phases.iter().any(|(phase, status)| {
                *phase == super::event::InstallPhase::VerifyIntegrity
                    && *status == PhaseStatus::Active
            }) {
                self.core.render_scan_line(
                    &mut canvas,
                    origin_x,
                    origin_y,
                    (self.tick % 90) as f32 / 89.0,
                    accent,
                );
            }
        }
        self.rain.render(&mut canvas, 0, 0);

        let lines = (0..area.height)
            .map(|row| {
                let spans = (0..area.width)
                    .map(|x| {
                        let top = canvas.get(x, row.saturating_mul(2)).unwrap_or_default();
                        let bottom = canvas
                            .get(x, row.saturating_mul(2).saturating_add(1))
                            .unwrap_or_default();
                        Span::styled(
                            "▄",
                            Style::default()
                                .fg(display_color(bottom.bottom, mode))
                                .bg(display_color(top.top, mode)),
                        )
                    })
                    .collect::<Vec<_>>();
                Line::from(spans)
            })
            .collect::<Vec<_>>();
        let title = if state.rollback_active {
            "↶ Rolling back safely"
        } else if state.failure.is_some() {
            "✗ Installation failed"
        } else if state.phase_completion >= 1.0 {
            "FOCUSA INSTALL COMPLETE"
        } else {
            "Continuity Core"
        };
        let title_style = if failed {
            Style::default().fg(display_color(Color::Red, mode))
        } else {
            Style::default().fg(display_color(accent, mode))
        };
        let mut content = vec![Line::from(Span::styled(title, title_style))];
        content.push(Line::from(Span::styled(
            "Matrix field · infrastructure platform",
            Style::default().fg(muted),
        )));
        content.extend(lines);
        frame.render_widget(Paragraph::new(content).alignment(Alignment::Center), area);
        let _ = mode; // mode is consumed by the palette selected by the caller.
    }
    pub fn completion_hold_ms() -> u16 {
        700
    }
    pub fn completion_summary(&self, _summary: &InstallCompletionSummary) {}
}
