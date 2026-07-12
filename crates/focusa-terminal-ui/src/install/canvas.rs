//! Half-block canvas for terminal pixel rendering.

use ratatui::style::Color;

/// A logical pixel on the block canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pixel {
    pub top: Color,
    pub bottom: Color,
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            top: Color::Black,
            bottom: Color::Black,
        }
    }
}

/// Block canvas using half-block technique.
/// Each terminal cell represents two vertical logical pixels.
pub struct BlockCanvas {
    pub width: u16,
    pub height: u16, // logical rows (each maps to half a terminal row)
    pub pixels: Vec<Pixel>,
}

impl BlockCanvas {
    pub fn new(width: u16, height: u16) -> Self {
        let len = (width as usize) * (height as usize);
        BlockCanvas {
            width,
            height,
            pixels: vec![Pixel::default(); len],
        }
    }

    pub fn set(&mut self, x: u16, y: u16, pixel: Pixel) {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.pixels[idx] = pixel;
        }
    }

    pub fn get(&self, x: u16, y: u16) -> Option<Pixel> {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            Some(self.pixels[idx])
        } else {
            None
        }
    }

    /// Fill entire canvas with a color.
    pub fn clear(&mut self, color: Color) {
        let p = Pixel {
            top: color,
            bottom: color,
        };
        for px in &mut self.pixels {
            *px = p;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_block_top_bottom_mapping() {
        let mut canvas = BlockCanvas::new(10, 10);
        canvas.set(
            5,
            5,
            Pixel {
                top: Color::Red,
                bottom: Color::Blue,
            },
        );
        let p = canvas.get(5, 5).unwrap();
        assert_eq!(p.top, Color::Red);
        assert_eq!(p.bottom, Color::Blue);
    }

    #[test]
    fn out_of_bounds_safe() {
        let canvas = BlockCanvas::new(10, 10);
        assert!(canvas.get(20, 20).is_none());
    }
}
