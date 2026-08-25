//! Server-owned organization wall-view contracts.
//!
//! A wall view is a revocable, read-only projection scope. It is not a bearer
//! token and it never grants mutation authority. API persistence and token
//! issuance belong to the daemon layer.

use serde::{Deserialize, Serialize};

pub const WALL_VIEW_SCHEMA: &str = "focusa.wall_view.v1";
pub const WALL_LAYOUT_SCHEMA: &str = "focusa.wall_layout.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WallScope {
    pub organization_id: String,
    pub project_refs: Vec<String>,
    pub workstream_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WallGrid {
    pub columns: u16,
    pub row_height: u16,
    pub gap: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WallWidgetPlacement {
    pub widget_id: String,
    pub widget_revision: u32,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WallLayout {
    pub schema: String,
    pub layout_id: String,
    pub revision: u32,
    pub name: String,
    pub grid: WallGrid,
    pub widgets: Vec<WallWidgetPlacement>,
    pub approved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WallViewStatus {
    Active,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WallView {
    pub schema: String,
    pub wall_view_id: String,
    pub scope: WallScope,
    pub layout: WallLayout,
    pub status: WallViewStatus,
    pub created_at: String,
    pub expires_at: String,
    pub revoked_at: Option<String>,
    pub source_revision: String,
}

impl WallLayout {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != WALL_LAYOUT_SCHEMA
            || self.layout_id.trim().is_empty()
            || self.revision == 0
        {
            return Err("wall layout identity is invalid".into());
        }
        if self.name.trim().is_empty() || self.name.len() > 160 {
            return Err("wall layout name is invalid".into());
        }
        if self.grid.columns == 0
            || self.grid.columns > 48
            || self.grid.row_height == 0
            || self.grid.gap > 128
        {
            return Err("wall layout grid is invalid".into());
        }
        if self.widgets.is_empty() || self.widgets.len() > 64 {
            return Err("wall layout must contain 1..64 widgets".into());
        }
        for widget in &self.widgets {
            if widget.widget_id.trim().is_empty()
                || widget.widget_revision == 0
                || widget.width == 0
                || widget.height == 0
            {
                return Err("wall widget placement is invalid".into());
            }
            if u32::from(widget.x) + u32::from(widget.width) > u32::from(self.grid.columns) {
                return Err(format!("widget {} exceeds wall grid", widget.widget_id));
            }
        }
        if !self.approved {
            return Err("wall layout must be approved before publication".into());
        }
        Ok(())
    }
}

impl WallView {
    pub fn validate(&self, now: &str) -> Result<(), String> {
        if self.schema != WALL_VIEW_SCHEMA || self.wall_view_id.trim().is_empty() {
            return Err("wall view identity is invalid".into());
        }
        if self.scope.organization_id.trim().is_empty() {
            return Err("wall view organization scope is required".into());
        }
        if self.created_at.trim().is_empty()
            || self.expires_at.trim().is_empty()
            || self.source_revision.trim().is_empty()
        {
            return Err("wall view timestamps and source revision are required".into());
        }
        self.layout.validate()?;
        if matches!(self.status, WallViewStatus::Active) && self.expires_at.as_str() <= now {
            return Err("active wall view is expired".into());
        }
        if matches!(self.status, WallViewStatus::Revoked) && self.revoked_at.is_none() {
            return Err("revoked wall view requires revoked_at".into());
        }
        Ok(())
    }

    pub fn is_read_only(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(approved: bool) -> WallLayout {
        WallLayout {
            schema: WALL_LAYOUT_SCHEMA.into(),
            layout_id: "layout:ops".into(),
            revision: 1,
            name: "Operations wall".into(),
            grid: WallGrid {
                columns: 12,
                row_height: 72,
                gap: 16,
            },
            widgets: vec![WallWidgetPlacement {
                widget_id: "focusa.workforce.status".into(),
                widget_revision: 1,
                x: 0,
                y: 0,
                width: 8,
                height: 3,
            }],
            approved,
        }
    }

    fn view(status: WallViewStatus, approved: bool) -> WallView {
        WallView {
            schema: WALL_VIEW_SCHEMA.into(),
            wall_view_id: "wall:ops".into(),
            scope: WallScope {
                organization_id: "org:demo".into(),
                project_refs: vec![],
                workstream_refs: vec![],
            },
            layout: layout(approved),
            status,
            created_at: "2026-01-01T00:00:00Z".into(),
            expires_at: "2099-01-01T00:00:00Z".into(),
            revoked_at: None,
            source_revision: "layout:1".into(),
        }
    }

    #[test]
    fn approved_wall_view_is_valid_and_read_only() {
        let wall = view(WallViewStatus::Active, true);
        wall.validate("2025-01-01T00:00:00Z").unwrap();
        assert!(wall.is_read_only());
    }

    #[test]
    fn unapproved_layout_and_expired_active_view_fail_closed() {
        assert!(
            view(WallViewStatus::Active, false)
                .validate("2025-01-01T00:00:00Z")
                .is_err()
        );
        let mut expired = view(WallViewStatus::Active, true);
        expired.expires_at = "2020-01-01T00:00:00Z".into();
        assert!(expired.validate("2025-01-01T00:00:00Z").is_err());
    }

    #[test]
    fn revoked_view_requires_revocation_timestamp() {
        assert!(
            view(WallViewStatus::Revoked, true)
                .validate("2025-01-01T00:00:00Z")
                .is_err()
        );
        let mut revoked = view(WallViewStatus::Revoked, true);
        revoked.revoked_at = Some("2026-01-02T00:00:00Z".into());
        assert!(revoked.validate("2025-01-01T00:00:00Z").is_ok());
    }
}
