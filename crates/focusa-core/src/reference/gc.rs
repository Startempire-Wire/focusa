//! Garbage collection for ECS (MVP minimal).
//!
//! MVP: keep everything by default.
//! Optional: delete blobs older than N days.
//! Ensure index consistency on startup (repair pass).

// TODO: Implement GC per G1-detail-08-ecs.md
