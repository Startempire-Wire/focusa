//! Canonical high-frequency palette.
//!
//! §8.2: exact RGB values and semantic purposes.

use ratatui::style::Color;

/// TrueColor palette tokens.
pub struct TrueColorPalette;

impl TrueColorPalette {
    pub const BACKGROUND: Color = Color::Rgb(2, 3, 10);
    pub const TEXT: Color = Color::Rgb(234, 247, 255);
    pub const MUTED: Color = Color::Rgb(120, 144, 168);
    pub const CYAN: Color = Color::Rgb(0, 229, 255);
    pub const ELECTRIC_BLUE: Color = Color::Rgb(0, 136, 255);
    pub const VIOLET: Color = Color::Rgb(138, 92, 255);
    pub const MAGENTA: Color = Color::Rgb(255, 43, 214);
    pub const LIME: Color = Color::Rgb(57, 255, 20);
    pub const YELLOW: Color = Color::Rgb(255, 230, 0);
    pub const ORANGE: Color = Color::Rgb(255, 138, 0);
    pub const SUCCESS: Color = Color::Rgb(53, 255, 120);
    pub const WARNING: Color = Color::Rgb(255, 216, 74);
    pub const ERROR: Color = Color::Rgb(255, 51, 79);
    pub const BORDER: Color = Color::Rgb(39, 58, 78);
}

/// ANSI-256 palette approximation.
pub struct Ansi256Palette;

impl Ansi256Palette {
    pub const BACKGROUND: Color = Color::Indexed(16);  // dark black
    pub const TEXT: Color = Color::Indexed(231);       // white
    pub const MUTED: Color = Color::Indexed(67);       // gray-blue
    pub const CYAN: Color = Color::Indexed(51);        // bright cyan
    pub const ELECTRIC_BLUE: Color = Color::Indexed(33); // blue
    pub const VIOLET: Color = Color::Indexed(93);      // purple
    pub const MAGENTA: Color = Color::Indexed(201);    // magenta
    pub const LIME: Color = Color::Indexed(82);        // green
    pub const YELLOW: Color = Color::Indexed(220);     // yellow
    pub const ORANGE: Color = Color::Indexed(208);     // orange
    pub const SUCCESS: Color = Color::Indexed(84);     // green
    pub const WARNING: Color = Color::Indexed(220);    // yellow
    pub const ERROR: Color = Color::Indexed(196);      // red
    pub const BORDER: Color = Color::Indexed(59);      // dark gray
}

/// Monochrome palette (grayscale only).
pub struct MonochromePalette;

impl MonochromePalette {
    pub const BACKGROUND: Color = Color::Black;
    pub const TEXT: Color = Color::White;
    pub const MUTED: Color = Color::DarkGray;
    pub const CYAN: Color = Color::White;
    pub const ELECTRIC_BLUE: Color = Color::Gray;
    pub const VIOLET: Color = Color::White;
    pub const MAGENTA: Color = Color::White;
    pub const LIME: Color = Color::White;
    pub const YELLOW: Color = Color::White;
    pub const ORANGE: Color = Color::Gray;
    pub const SUCCESS: Color = Color::White;
    pub const WARNING: Color = Color::Gray;
    pub const ERROR: Color = Color::White;
    pub const BORDER: Color = Color::DarkGray;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truecolor_cyan_exact() {
        assert_eq!(TrueColorPalette::CYAN, Color::Rgb(0, 229, 255));
    }

    #[test]
    fn ansi256_mapping_nonempty() {
        assert_ne!(Ansi256Palette::CYAN, Color::Reset);
    }

    #[test]
    fn monochrome_no_color_variety() {
        // Monochrome intentionally uses only black/white/gray
        let colors = [
            MonochromePalette::BACKGROUND,
            MonochromePalette::TEXT,
            MonochromePalette::MUTED,
        ];
        for c in &colors {
            assert!(!matches!(c, Color::Rgb(_, _, _)));
        }
    }
}
