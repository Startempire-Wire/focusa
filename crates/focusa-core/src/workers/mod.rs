//! Background Workers — async cognition pipeline.
//!
//! Source: G1-10-workers.md
//!
//! Design: async task queue, limited concurrency (1-2 workers),
//! strict time budget per job, never blocks hot path.
//!
//! Workers return results, not state changes.
//! Reducer decides whether to accept results.

pub mod executor;
pub mod priority_queue;
pub mod queue;
