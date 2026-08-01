//! Canonical Mission Canvas composition authority.
//!
//! This module owns portable composition types, append-only events, deterministic
//! projection reducers, and SQLite persistence. UI and HTTP layers remain adapters.

pub mod layout;
pub mod memory;
pub mod model;
pub mod persistence;
pub mod profiles;
pub mod reducer;
pub mod resolver;

pub use layout::{
    resolve_layout, validate_no_dead_chrome, LayoutConstraints, LayoutError, LayoutNode,
};
pub use memory::{reduce_layout_memory, ProfileLayoutMemory};
pub use model::*;
pub use persistence::{MissionCanvasStore, MissionCanvasStoreError};
pub use profiles::{
    ActivityModeDefinition, CompositionRegistry, DomainPack, RegistryDefinition,
    WorkspaceProfileDefinition,
};
pub use reducer::{resolve_projection, RecompositionResult, ResolveProjectionInput};
pub use resolver::{
    collect_candidates, resolve_eligibility, EligibilityContext, EligibilityResolution,
};
