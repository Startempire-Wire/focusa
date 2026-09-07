//! Compatibility export for the shared licensing-owned development-origin resolver.
//! Keep one implementation so core, CLI and daemon policy use the same evidence.

pub use focusa_license::developer_origin::{
    DeveloperOriginReport, developer_origin_active, developer_origin_report,
    invalidate_developer_origin_cache,
};
