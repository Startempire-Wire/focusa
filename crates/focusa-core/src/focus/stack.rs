//! Focus Stack — Hierarchical Execution Context.
//!
//! Source: 03-focus-stack.md, G1-detail-05-focus-stack-hec.md
//!
//! Core Invariants:
//!   1. Exactly one active Focus Frame exists at any time
//!   2. Every Focus Frame has a concrete intent
//!   3. Every Focus Frame maps to a Beads issue
//!   4. Frames are entered and exited explicitly
//!   5. Completed frames are archived, not forgotten
//!
//! Invalid Operations (Forbidden):
//!   - Multiple active frames
//!   - Implicit frame switching
//!   - Editing archived frames
//!   - Frames without Beads linkage
//!   - Skipping completion reasons

use crate::types::*;
use chrono::Utc;
use uuid::Uuid;

/// Push a new child frame under the current active frame.
///
/// Rules:
///   - New frame becomes Active
///   - Previous active becomes Paused
///   - Emits: FocusFramePushed, focus.active_changed
pub fn push_frame(
    stack: &mut FocusStackState,
    title: String,
    goal: String,
    beads_issue_id: String,
    constraints: Vec<String>,
    tags: Vec<String>,
) -> Result<(FrameId, Vec<FocusaEvent>), String> {
    if beads_issue_id.is_empty() {
        return Err("beads_issue_id is required — frames without Beads linkage are forbidden".into());
    }

    let frame_id = Uuid::now_v7();
    let now = Utc::now();

    // Pause current active frame
    if let Some(active_id) = stack.active_id {
        if let Some(frame) = stack.frames.iter_mut().find(|f| f.id == active_id) {
            frame.status = FrameStatus::Paused;
            frame.updated_at = now;
        }
    }

    let parent_id = stack.active_id;

    let frame_constraints = constraints.clone();
    let frame_tags = tags.clone();

    let frame = FrameRecord {
        id: frame_id,
        parent_id,
        created_at: now,
        updated_at: now,
        status: FrameStatus::Active,
        title: title.clone(),
        goal: goal.clone(),
        beads_issue_id: beads_issue_id.clone(),
        tags,
        priority_hint: None,
        ascc_checkpoint_id: None,
        stats: FrameStats::default(),
        handles: vec![],
        constraints,
        focus_state: FocusState::default(),
    };

    stack.frames.push(frame);
    stack.active_id = Some(frame_id);
    if stack.root_id.is_none() {
        stack.root_id = Some(frame_id);
    }
    rebuild_stack_path(stack);
    stack.version += 1;

    let events = vec![FocusaEvent::FocusFramePushed {
        frame_id,
        beads_issue_id,
        title,
        goal,
        constraints: frame_constraints,
        tags: frame_tags,
    }];

    Ok((frame_id, events))
}

/// Pop (complete) the active frame, returning focus to parent.
///
/// Rules:
///   - Current active frame → Completed
///   - Requires completion_reason
///   - Parent frame restores to Active
pub fn pop_frame(
    stack: &mut FocusStackState,
    completion_reason: CompletionReason,
) -> Result<Vec<FocusaEvent>, String> {
    let active_id = stack
        .active_id
        .ok_or("No active frame to pop")?;

    // ── Phase 1: Validate everything BEFORE mutating ──
    let active_idx = stack
        .frames
        .iter()
        .position(|f| f.id == active_id)
        .ok_or("Active frame not found")?;

    let parent_id = stack.frames[active_idx].parent_id;

    // If parent exists, it must be Paused (push_frame pauses the parent).
    // Anything else means state corruption — refuse to proceed.
    if let Some(pid) = parent_id {
        let parent = stack
            .frames
            .iter()
            .find(|f| f.id == pid)
            .ok_or(format!("Parent frame {} not found", pid))?;
        if parent.status != FrameStatus::Paused {
            return Err(format!(
                "Parent frame {} has status {:?}, expected Paused — state is corrupt",
                pid, parent.status
            ));
        }
    }

    // ── Phase 2: All checks passed — now mutate ──
    let now = Utc::now();

    stack.frames[active_idx].status = FrameStatus::Completed;
    stack.frames[active_idx].updated_at = now;

    if let Some(pid) = parent_id {
        if let Some(parent) = stack.frames.iter_mut().find(|f| f.id == pid) {
            parent.status = FrameStatus::Active;
            parent.updated_at = now;
        }
        stack.active_id = Some(pid);
    } else {
        // Root frame was popped — clear both so next push starts fresh.
        stack.active_id = None;
        stack.root_id = None;
    }

    rebuild_stack_path(stack);
    stack.version += 1;

    Ok(vec![FocusaEvent::FocusFrameCompleted {
        frame_id: active_id,
        completion_reason,
    }])
}

/// Rebuild the stack path cache from root to active.
///
/// Includes cycle detection to prevent infinite loops on corrupt data.
fn rebuild_stack_path(stack: &mut FocusStackState) {
    stack.stack_path_cache.clear();
    if let Some(active_id) = stack.active_id {
        let mut current = Some(active_id);
        let mut path = Vec::new();
        let max_depth = stack.frames.len();
        while let Some(id) = current {
            path.push(id);
            if path.len() > max_depth {
                tracing::error!("Cycle detected in focus stack parent pointers — aborting path rebuild");
                break;
            }
            current = stack
                .frames
                .iter()
                .find(|f| f.id == id)
                .and_then(|f| f.parent_id);
        }
        path.reverse();
        stack.stack_path_cache = path;
    }
}
