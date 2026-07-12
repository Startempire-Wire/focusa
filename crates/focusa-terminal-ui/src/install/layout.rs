//! Responsive layout selection for the installer presentation.

use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutKind {
    Full,
    Standard,
    Compact,
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    pub kind: LayoutKind,
    pub art: Rect,
    pub rail: Rect,
    pub status: Rect,
    pub warnings: Rect,
}

impl Layout {
    pub fn select(width: u16, height: u16) -> LayoutKind {
        if width < 70 || height < 22 {
            LayoutKind::Plain
        } else if width >= 120 && height >= 36 {
            LayoutKind::Full
        } else if width >= 90 && height >= 28 {
            LayoutKind::Standard
        } else {
            LayoutKind::Compact
        }
    }

    pub fn for_area(area: Rect) -> Self {
        let kind = Self::select(area.width, area.height);
        let rail_width = match kind {
            LayoutKind::Full => 30,
            LayoutKind::Standard => 24,
            _ => 0,
        };
        let bottom = area.height.min(8);
        let status = Rect::new(
            area.x,
            area.y + area.height.saturating_sub(bottom),
            area.width,
            bottom,
        );
        let art_height = area.height.saturating_sub(bottom);
        let rail = if rail_width > 0 {
            Rect::new(
                area.x + area.width.saturating_sub(rail_width),
                area.y,
                rail_width,
                art_height,
            )
        } else {
            Rect::default()
        };
        let art_width = area.width.saturating_sub(rail_width);
        Layout {
            kind,
            art: Rect::new(area.x, area.y, art_width, art_height),
            rail,
            status,
            warnings: Rect::new(
                area.x,
                status.y + status.height.saturating_sub(2),
                area.width,
                2,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn breakpoints_are_safe() {
        assert_eq!(Layout::select(69, 30), LayoutKind::Plain);
        assert_eq!(Layout::select(70, 22), LayoutKind::Compact);
        assert_eq!(Layout::select(90, 28), LayoutKind::Standard);
        assert_eq!(Layout::select(120, 36), LayoutKind::Full);
    }
}
