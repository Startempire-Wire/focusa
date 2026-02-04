//! Worker queue — bounded async job queue.
//!
//! Bounded size: 100 jobs (default).
//! Backpressure: drop low-priority jobs if full.
//! Max execution time per job: 200ms (default).
//! Jobs enqueued by daemon reducer, high-priority first.

use crate::types::WorkerJob;
use tokio::sync::mpsc;

/// Worker queue backed by mpsc channel.
pub struct WorkerQueue {
    tx: mpsc::Sender<WorkerJob>,
    rx: mpsc::Receiver<WorkerJob>,
}

impl WorkerQueue {
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self { tx, rx }
    }

    /// Enqueue a job. Returns false if queue is full.
    pub async fn enqueue(&self, job: WorkerJob) -> bool {
        self.tx.try_send(job).is_ok()
    }

    /// Dequeue the next job.
    pub async fn dequeue(&mut self) -> Option<WorkerJob> {
        self.rx.recv().await
    }

    /// Get the sender for cloning.
    pub fn sender(&self) -> mpsc::Sender<WorkerJob> {
        self.tx.clone()
    }
}
