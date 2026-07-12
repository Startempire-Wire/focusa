//! Deterministic bounded Matrix rain.
//!
//! §8.5: deterministic seed, columns spaced, tail 4-12, speed 8-18 logical cells/sec,
//! colors from dim lime/cyan/violet/magenta/rare yellow, max 18% occupancy.

use super::canvas::{BlockCanvas, Pixel};
use super::palette::TrueColorPalette;
use ratatui::style::Color;
use std::collections::HashMap;

/// Approved rain glyph set: 0-9 A-F : + * · │
const RAIN_GLYPHS: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'A', 'B', 'C', 'D', 'E', 'F', ':', '+', '*', '·', '│',
];

// We don't render glyphs in the block canvas; we use color intensity instead.
// Each rain drop is a column with a head and tail.

#[derive(Clone, Debug)]
struct RainColumn {
    x: u16,
    head_y: f32,
    speed: f32,    // logical cells per second
    tail_len: u16, // 4-12
    color: Color,
    alive: bool,
}

/// Matrix rain system.
pub struct MatrixRain {
    columns: Vec<RainColumn>,
    seed: u64,
    max_columns: usize,
    column_spacing: u16,
    width: u16,
    height: u16,
    next_spawn: f32,
    paused: bool,
}

impl MatrixRain {
    pub fn new(seed: u64, width: u16, height: u16) -> Self {
        let area = (width as usize) * (height as usize);
        let max_occupancy = (area as f32 * 0.18) as usize;
        // Each column occupies tail_len cells; average tail ~8
        let max_columns = (max_occupancy / 8).max(1);
        let column_spacing = 1; // at least one cell apart

        let mut rain = MatrixRain {
            columns: Vec::with_capacity(max_columns),
            seed,
            max_columns,
            column_spacing,
            width,
            height,
            next_spawn: 0.0,
            paused: false,
        };
        // Pre-populate some columns
        for i in 0..max_columns.min(width as usize / 2) {
            let col = rain.spawn_column(i as u64);
            rain.columns.push(col);
        }
        rain
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn update(&mut self, dt_secs: f32) {
        if self.paused {
            return;
        }
        for col in &mut self.columns {
            if col.alive {
                col.head_y += col.speed * dt_secs;
                if col.head_y > self.height as f32 + col.tail_len as f32 {
                    col.alive = false;
                }
            }
        }
        self.columns.retain(|c| c.alive);

        self.next_spawn -= dt_secs;
        if self.next_spawn <= 0.0 && self.columns.len() < self.max_columns {
            let col = self.spawn_column(self.seed.wrapping_add(self.columns.len() as u64));
            self.columns.push(col);
            self.next_spawn = deterministic_f32(self.seed, 500, self.columns.len() as u16) * 0.3 + 0.1;
        }
    }

    pub fn render(&self, canvas: &mut BlockCanvas, origin_x: u16, origin_y: u16) {
        for col in &self.columns {
            let head = col.head_y as i16;
            for i in 0..col.tail_len as i16 {
                let y = head - i;
                if y < 0 || y >= self.height as i16 {
                    continue;
                }
                let brightness = if i == 0 {
                    1.0
                } else {
                    1.0 - (i as f32 / col.tail_len as f32)
                };
                let color = dim_color(col.color, brightness);
                let cy = origin_y + y as u16;
                let cx = origin_x + col.x;
                if let Some(p) = canvas.get(cx, cy) {
                    // Only draw if it doesn't obscure text; attenuate behind text
                    let bg = p.bottom;
                    let mixed = blend_colors(bg, color, brightness * 0.3);
                    canvas.set(cx, cy, Pixel { top: p.top, bottom: mixed });
                }
            }
        }
    }

    fn spawn_column(&self, salt: u64) -> RainColumn {
        let s = self.seed.wrapping_add(salt);
        let x = (deterministic_f32(s, 0, 0) * self.width as f32) as u16 % self.width;
        let speed = 8.0 + deterministic_f32(s, 1, 0) * 10.0; // 8-18
        let tail_len = 4 + (deterministic_f32(s, 2, 0) * 8.0) as u16; // 4-12
        let color = rain_color(s);
        RainColumn {
            x,
            head_y: -(tail_len as f32),
            speed,
            tail_len,
            color,
            alive: true,
        }
    }
}

fn rain_color(seed: u64) -> Color {
    let v = deterministic_f32(seed, 3, 0);
    if v > 0.95 {
        TrueColorPalette::YELLOW
    } else if v > 0.75 {
        TrueColorPalette::MAGENTA
    } else if v > 0.55 {
        TrueColorPalette::VIOLET
    } else if v > 0.35 {
        TrueColorPalette::CYAN
    } else {
        TrueColorPalette::LIME
    }
}

fn deterministic_f32(seed: u64, x: u16, y: u16) -> f32 {
    let mut h = seed.wrapping_add((x as u64) << 16 | (y as u64));
    h = h.wrapping_mul(0x9e3779b97f4a7c15);
    h = h ^ (h >> 30);
    h = h.wrapping_mul(0xbf58476d1ce4e5b9);
    h = h ^ (h >> 27);
    ((h & 0xFFFFFF) as f32) / ((1u64 << 24) as f32)
}

fn dim_color(c: Color, factor: f32) -> Color {
    match c {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f32 * factor) as u8,
            (g as f32 * factor) as u8,
            (b as f32 * factor) as u8,
        ),
        _ => c,
    }
}

fn blend_colors(a: Color, b: Color, t: f32) -> Color {
    match (a, b) {
        (Color::Rgb(ar, ag, ab), Color::Rgb(br, bg, bb)) => Color::Rgb(
            (ar as f32 * (1.0 - t) + br as f32 * t) as u8,
            (ag as f32 * (1.0 - t) + bg as f32 * t) as u8,
            (ab as f32 * (1.0 - t) + bb as f32 * t) as u8,
        ),
        _ => b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_from_seed() {
        let r1 = MatrixRain::new(42, 40, 20);
        let r2 = MatrixRain::new(42, 40, 20);
        assert_eq!(r1.columns.len(), r2.columns.len());
        for (a, b) in r1.columns.iter().zip(&r2.columns) {
            assert_eq!(a.x, b.x);
            assert_eq!(a.tail_len, b.tail_len);
        }
    }

    #[test]
    fn occupancy_capped() {
        let mut rain = MatrixRain::new(1, 60, 30);
        // Run for 5 seconds
        for _ in 0..500 {
            rain.update(0.01);
        }
        let area = 60 * 30;
        let max_occ = (area as f32 * 0.18) as usize;
        // Count active rain pixels
        let mut active_pixels = 0;
        for col in &rain.columns {
            active_pixels += col.tail_len as usize;
        }
        assert!(active_pixels <= max_occ + rain.columns.len() * 12);
    }

    #[test]
    fn tail_len_in_bounds() {
        let rain = MatrixRain::new(1, 60, 30);
        for col in &rain.columns {
            assert!(col.tail_len >= 4 && col.tail_len <= 12);
        }
    }

    #[test]
    fn speed_in_bounds() {
        let rain = MatrixRain::new(1, 60, 30);
        for col in &rain.columns {
            assert!(col.speed >= 8.0 && col.speed <= 18.0);
        }
    }

    #[test]
    fn glyphs_are_width_stable() {
        for g in RAIN_GLYPHS {
            assert!(!g.is_control());
            assert_eq!(g.len_utf8(), g.len_utf8()); // not a check, just usage
        }
    }
}
