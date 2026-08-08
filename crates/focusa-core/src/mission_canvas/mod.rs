//! Canonical Mission Canvas composition authority.
//!
//! This module owns portable composition types, append-only events, deterministic
//! projection reducers, and SQLite persistence. UI and HTTP layers remain adapters.

pub mod domain_pack;
pub mod host;
pub mod layout;
pub mod layout_mutation;
pub mod memory;
pub mod model;
pub mod persistence;
pub mod profiles;
pub mod reducer;
pub mod resolver;

pub use domain_pack::{
    DomainPackInstallCommand, DomainPackInstallError, DomainPackInstallService,
    DOMAIN_PACK_INSTALL_CAPABILITY, DOMAIN_PACK_INSTALL_OPERATION, DOMAIN_PACK_INSTALL_PERMISSION,
};
pub use host::{
    HostLifecycleError, HostLifecycleFocusCommand, HostLifecycleHideCommand,
    HostLifecycleLaunchCommand, HostLifecycleService, HostLifecycleState, HostPlatform,
    HostRendererResolution, HostRendererResolutionError, HostRendererResolutionService,
    DESKTOP_TAURI_CAPABILITY, DESKTOP_TAURI_RENDERER, HOST_RESOLVER_REVISION,
    PI_OVERLAY_COMPATIBILITY_CAPABILITY, PI_OVERLAY_RENDERER, RICH_HOST_FOCUS_OPERATION,
    RICH_HOST_HIDE_OPERATION, RICH_HOST_LAUNCH_OPERATION, RICH_HOST_PERMISSION,
    RICH_HOST_RESOLVE_CAPABILITY, RICH_HOST_RESOLVE_OPERATION,
};
pub use layout::{
    resolve_layout, validate_no_dead_chrome, LayoutConstraints, LayoutError, LayoutNode,
};
pub use layout_mutation::{
    LayoutMutationCommand, LayoutMutationError, LayoutMutationExecution, LayoutMutationResult,
    LayoutMutationService, LAYOUT_MUTATE_OPERATION, LAYOUT_MUTATE_PERMISSION,
};
pub use memory::{
    layout_memory_digest, reduce_layout_memory, validate_profile_layout_memory,
    LayoutMemoryUpdateCommand, LayoutMemoryUpdateError, LayoutMemoryUpdateService,
    ProfileLayoutMemory, LAYOUT_MEMORY_UPDATE_OPERATION, LAYOUT_MEMORY_UPDATE_PERMISSION,
};
pub use model::*;
pub use persistence::{MissionCanvasStore, MissionCanvasStoreError};
pub use profiles::{
    meaningful_activities_for_projection, meaningful_profiles_for_projection,
    ActivityModeDefinition, CompositionRegistry, DomainPack, RegistryDefinition,
    WorkspaceProfileDefinition,
};
pub use reducer::{
    resolve_projection, ActivitySelectionCommand, ActivitySelectionError, ActivitySelectionService,
    ProfileSelectionCommand, ProfileSelectionError, ProfileSelectionService, RecompositionResult,
    ResolveProjectionInput, ACTIVITY_SELECT_OPERATION, ACTIVITY_SELECT_PERMISSION,
    PROFILE_SELECT_OPERATION, PROFILE_SELECT_PERMISSION,
};
pub use resolver::{
    collect_candidates, resolve_eligibility, EligibilityContext, EligibilityResolution,
};
