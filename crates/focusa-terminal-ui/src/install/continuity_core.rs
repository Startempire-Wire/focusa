//! Deterministic assembling Continuity Core.
//!
//! §8.4: fixed mask, 32×32 logical, fragments assemble progressively,
//! stabilize into luminous ring/core, open dark center.

use super::canvas::{BlockCanvas, Pixel};
use super::palette::TrueColorPalette;
use ratatui::style::Color;

/// Radius of the core ring in logical pixels.
const CORE_RADIUS: f32 = 10.0;
const CORE_CENTER: (f32, f32) = (16.0, 14.0);

/// The continuity core mask. Each cell has a target color and a lock threshold.
#[derive(Clone, Copy)]
struct CoreCell {
    x: u16,
    y: u16,
    target: Color,
    /// 0.0 = immediately visible, 1.0 = last to lock
    lock_threshold: f32,
    is_spark: bool,
}

/// Continuity Core with deterministic assembly.
pub struct ContinuityCore {
    cells: Vec<CoreCell>,
    seed: u64,
}

impl ContinuityCore {
    pub fn new(seed: u64) -> Self {
        let mut cells = Vec::new();
        for y in 0..32u16 {
            for x in 0..32u16 {
                let fx = x as f32 + 0.5;
                let fy = y as f32 + 0.5;
                let dx = fx - CORE_CENTER.0;
                let dy = fy - CORE_CENTER.1;
                let dist = (dx * dx + dy * dy).sqrt();

                // Ring shape: between radius 7 and 12
                let in_ring = (7.0..=12.0).contains(&dist);
                let in_core = dist < 7.0;

                if in_ring {
                    let angle = dy.atan2(dx);
                    let (target, is_spark) = core_color_for_angle(angle, dist, seed);
                    let lock = deterministic_float(seed, x, y);
                    cells.push(CoreCell {
                        x,
                        y,
                        target,
                        lock_threshold: lock,
                        is_spark,
                    });
                } else if in_core {
                    // Inner core energy (dim cyan/blue glow)
                    let lock = deterministic_float(seed.wrapping_add(1), x, y);
                    cells.push(CoreCell {
                        x,
                        y,
                        target: TrueColorPalette::ELECTRIC_BLUE,
                        lock_threshold: lock,
                        is_spark: false,
                    });
                }
            }
        }
        ContinuityCore { cells, seed }
    }

    /// Render the core into the canvas at the given origin with current phase assembly.
    /// `assembly` is 0.0 (dispersed) to 1.0 (fully assembled).
    pub fn render(&self, canvas: &mut BlockCanvas, origin_x: u16, origin_y: u16, assembly: f32) {
        for cell in &self.cells {
            let pos = if assembly >= cell.lock_threshold {
                // Cell is locked in place
                (cell.x, cell.y)
            } else {
                // Cell is dispersed based on seed and time
                let dispersion = 1.0 - (assembly / cell.lock_threshold.max(0.01));
                let dx = (deterministic_float(self.seed, cell.x + 100, cell.y) - 0.5)
                    * 24.0
                    * dispersion;
                let dy = (deterministic_float(self.seed, cell.x, cell.y + 200) - 0.5)
                    * 24.0
                    * dispersion;
                let px = (cell.x as f32 + dx).clamp(0.0, 31.0) as u16;
                let py = (cell.y as f32 + dy).clamp(0.0, 31.0) as u16;
                (px, py)
            };

            let color = if assembly >= cell.lock_threshold {
                cell.target
            } else {
                // Dimmer while dispersing
                dim_color(cell.target, 0.4)
            };

            let canvas_x = origin_x + pos.0;
            let canvas_y = origin_y + pos.1;
            if let Some(p) = canvas.get(canvas_x, canvas_y) {
                let merged = if cell.is_spark && assembly >= cell.lock_threshold {
                    blend_over(
                        p,
                        Pixel {
                            top: color,
                            bottom: color,
                        },
                    )
                } else {
                    Pixel {
                        top: color,
                        bottom: p.bottom,
                    }
                };
                canvas.set(canvas_x, canvas_y, merged);
            }
        }
    }

    pub fn render_scan_line(
        &self,
        canvas: &mut BlockCanvas,
        origin_x: u16,
        origin_y: u16,
        scan_t: f32,
        scan_color: Color,
    ) {
        // Horizontal scan line across the core center
        let y = (CORE_CENTER.1 + 0.5) as u16;
        for x in 0..32u16 {
            let canvas_x = origin_x + x;
            let canvas_y = origin_y + y;
            if (0.0..=1.0).contains(&scan_t) {
                let scan_x = (scan_t * 32.0) as u16;
                if x == scan_x || x.saturating_add(1) == scan_x {
                    if let Some(p) = canvas.get(canvas_x, canvas_y) {
                        canvas.set(
                            canvas_x,
                            canvas_y,
                            Pixel {
                                top: scan_color,
                                bottom: p.bottom,
                            },
                        );
                    }
                }
            }
        }
    }
}

fn core_color_for_angle(angle: f32, dist: f32, _seed: u64) -> (Color, bool) {
    use std::f32::consts::PI;
    let normalized = (angle + PI) / (2.0 * PI); // 0..1
    let is_spark = deterministic_float(_seed, (normalized * 1000.0) as u16, 999) > 0.92;

    // Lower half: cyan/blue energy
    // Upper half: violet/magenta structure
    let color = if normalized > 0.5 {
        if normalized > 0.75 {
            TrueColorPalette::MAGENTA
        } else {
            TrueColorPalette::VIOLET
        }
    } else {
        if normalized > 0.25 {
            TrueColorPalette::ELECTRIC_BLUE
        } else {
            TrueColorPalette::CYAN
        }
    };

    if is_spark {
        (TrueColorPalette::ORANGE, true)
    } else if dist > 10.5 {
        (TrueColorPalette::VIOLET, false)
    } else {
        (color, false)
    }
}

fn deterministic_float(seed: u64, x: u16, y: u16) -> f32 {
    // Simple hash-based deterministic float in [0, 1)
    let mut h = seed.wrapping_add((x as u64) << 16 | (y as u64));
    h = h.wrapping_mul(0x9e3779b97f4a7c15);
    h = h ^ (h >> 30);
    h = h.wrapping_mul(0xbf58476d1ce4e5b9);
    h = h ^ (h >> 27);
    h = h.wrapping_mul(0x94d049bb133111eb);
    h = h ^ (h >> 31);
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

fn blend_over(_base: Pixel, _top: Pixel) -> Pixel {
    // Simple replacement for sparks
    _top
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_mask_same_seed() {
        let c1 = ContinuityCore::new(42);
        let c2 = ContinuityCore::new(42);
        assert_eq!(c1.cells.len(), c2.cells.len());
        for (a, b) in c1.cells.iter().zip(&c2.cells) {
            assert_eq!(a.x, b.x);
            assert_eq!(a.y, b.y);
            assert_eq!(a.lock_threshold, b.lock_threshold);
        }
    }

    #[test]
    fn different_seed_different_thresholds() {
        let c1 = ContinuityCore::new(42);
        let c2 = ContinuityCore::new(43);
        let differs = c1
            .cells
            .iter()
            .zip(&c2.cells)
            .any(|(a, b)| a.lock_threshold != b.lock_threshold);
        assert!(differs);
    }

    #[test]
    fn assembly_zero_disperses() {
        let core = ContinuityCore::new(1);
        let mut canvas = BlockCanvas::new(40, 40);
        core.render(&mut canvas, 4, 4, 0.0);
        // Most cells should be at displaced positions, not forming ring
        let ring_pixels: Vec<_> = (0..32u16)
            .flat_map(|y| (0..32u16).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let dx = x as f32 - CORE_CENTER.0;
                let dy = y as f32 - CORE_CENTER.1;
                let d = (dx * dx + dy * dy).sqrt();
                (7.0..=12.0).contains(&d)
            })
            .collect();
        // With assembly=0, ring cells are dispersed; fewer ring pixels should be present
        let set_in_ring = ring_pixels
            .iter()
            .filter(|&&(x, y)| {
                canvas
                    .get(4 + x, 4 + y)
                    .map(|p| p.top != Color::Black)
                    .unwrap_or(false)
            })
            .count();
        // Should be very few because cells are scattered
        assert!(set_in_ring < ring_pixels.len() / 2);
    }

    #[test]
    fn assembly_full_locks() {
        let core = ContinuityCore::new(1);
        let mut canvas = BlockCanvas::new(40, 40);
        core.render(&mut canvas, 4, 4, 1.0);
        let mut locked = 0;
        for cell in &core.cells {
            if let Some(p) = canvas.get(4 + cell.x, 4 + cell.y) {
                if p.top != Color::Black {
                    locked += 1;
                }
            }
        }
        assert_eq!(locked, core.cells.len());
    }
}
