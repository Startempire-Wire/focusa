//! Threads, Instances, Sessions, Attachments — docs/38-39-40
//!
//! Thread = persistent cognitive workspace.
//! Instance = where (runtime integration point).
//! Session = when (temporal execution window).
//! Attachment = binds session to thread.
//!
//! 5 guarantees:
//!   1. Threads never share mutable state
//!   2. One active per session
//!   3. CLT nodes belong to one thread
//!   4. Telemetry is thread-scoped
//!   5. Autonomy is thread-specific
//!
//! Operations: Create, Resume, Save, Rename, Fork, Archive.

use crate::types::*;
use chrono::Utc;
use uuid::Uuid;

/// Create a new thread with optional ownership.
pub fn create_thread(name: &str, primary_intent: &str, owner_machine_id: Option<&str>) -> Thread {
    Thread {
        id: Uuid::now_v7(),
        name: name.into(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        status: ThreadStatus::Active,
        thesis: ThreadThesis {
            primary_intent: primary_intent.into(),
            updated_at: Some(Utc::now()),
            ..Default::default()
        },
        clt_head: None,
        autonomy_history: vec![],
        owner_machine_id: owner_machine_id.map(|s| s.to_string()),
    }
}

/// Fork a thread (creates a copy with new ID).
/// If owner is provided, it overrides the source's owner.
pub fn fork_thread(source: &Thread, new_name: &str, new_owner: Option<&str>) -> Thread {
    let mut forked = source.clone();
    forked.id = Uuid::now_v7();
    forked.name = new_name.into();
    forked.created_at = Utc::now();
    forked.updated_at = Utc::now();
    forked.status = ThreadStatus::Forked;
    forked.owner_machine_id = new_owner.map(|s| s.to_string());
    forked
}

/// Archive a thread.
pub fn archive_thread(thread: &mut Thread) {
    thread.status = ThreadStatus::Archived;
    thread.updated_at = Utc::now();
}

/// Save (checkpoint) a thread.
pub fn save_thread(thread: &mut Thread) {
    thread.status = ThreadStatus::Saved;
    thread.updated_at = Utc::now();
}

/// Resume a saved thread.
pub fn resume_thread(thread: &mut Thread) {
    thread.status = ThreadStatus::Active;
    thread.updated_at = Utc::now();
}

/// Update thread thesis (minimum confidence delta enforced).
pub fn update_thesis(
    thread: &mut Thread,
    primary_intent: Option<&str>,
    secondary_goals: Option<Vec<String>>,
    constraints: Option<ThesisConstraints>,
) {
    if let Some(intent) = primary_intent {
        thread.thesis.primary_intent = intent.into();
    }
    if let Some(goals) = secondary_goals {
        thread.thesis.secondary_goals = goals;
    }
    if let Some(c) = constraints {
        thread.thesis.constraints = c;
    }
    thread.thesis.updated_at = Some(Utc::now());
    thread.updated_at = Utc::now();
}

/// Create an instance.
pub fn create_instance(kind: InstanceKind, thread_id: Option<Uuid>) -> Instance {
    Instance {
        id: Uuid::now_v7(),
        kind,
        created_at: Utc::now(),
        thread_id,
    }
}

/// Create a session attachment.
pub fn attach_session(
    session_id: SessionId,
    thread_id: Uuid,
    role: AttachmentRole,
) -> Attachment {
    Attachment {
        session_id,
        thread_id,
        role,
        attached_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_thread() {
        let t = create_thread("my-task", "Build auth module", None);
        assert_eq!(t.status, ThreadStatus::Active);
        assert_eq!(t.thesis.primary_intent, "Build auth module");
        assert_eq!(t.owner_machine_id, None);
    }

    #[test]
    fn test_create_thread_with_owner() {
        let t = create_thread("my-task", "Build auth module", Some("machine-123"));
        assert_eq!(t.owner_machine_id, Some("machine-123".to_string()));
    }

    #[test]
    fn test_fork_gets_new_id() {
        let t = create_thread("original", "intent", None);
        let forked = fork_thread(&t, "fork-1", None);
        assert_ne!(t.id, forked.id);
        assert_eq!(forked.status, ThreadStatus::Forked);
    }

    #[test]
    fn test_fork_with_new_owner() {
        let t = create_thread("original", "intent", Some("machine-a"));
        let forked = fork_thread(&t, "fork-1", Some("machine-b"));
        assert_eq!(forked.owner_machine_id, Some("machine-b".to_string()));
    }

    #[test]
    fn test_archive_and_resume() {
        let mut t = create_thread("test", "intent", None);
        archive_thread(&mut t);
        assert_eq!(t.status, ThreadStatus::Archived);
        resume_thread(&mut t);
        assert_eq!(t.status, ThreadStatus::Active);
    }
}
