//! Priority queue for worker jobs.
//!
//! Per G1-10 §Scheduling: "high-priority jobs first"
//!
//! Replaces FIFO mpsc channel with a binary heap ordered by priority.
//! Uses tokio::sync::Notify for efficient async wakeup.

use crate::types::{JobPriority, WorkerJob};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

/// Wrapper for WorkerJob that implements Ord for priority ordering.
#[derive(Debug, Clone)]
struct PrioritizedJob {
    priority: u8, // 0=High, 1=Normal, 2=Low for Reverse ordering
    seq: u64,     // Sequence number for FIFO within same priority
    job: WorkerJob,
}

impl PartialEq for PrioritizedJob {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.seq == other.seq
    }
}

impl Eq for PrioritizedJob {}

impl PartialOrd for PrioritizedJob {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedJob {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse ordering: lower priority value = higher actual priority
        self.priority
            .cmp(&other.priority)
            .then_with(|| self.seq.cmp(&other.seq))
    }
}

/// Async priority queue for worker jobs.
#[derive(Debug)]
pub struct PriorityQueue {
    heap: Mutex<BinaryHeap<Reverse<PrioritizedJob>>>,
    notify: Notify,
    seq: Mutex<u64>,
    capacity: usize,
}

impl PriorityQueue {
    /// Create a new priority queue with given capacity.
    pub fn new(capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            heap: Mutex::new(BinaryHeap::with_capacity(capacity)),
            notify: Notify::new(),
            seq: Mutex::new(0),
            capacity,
        })
    }

    /// Try to send a job without blocking.
    ///
    /// Returns true if job was queued, false if queue is full.
    pub async fn try_send(&self, job: WorkerJob) -> bool {
        let mut heap = self.heap.lock().await;
        
        if heap.len() >= self.capacity {
            return false;
        }

        let priority = match job.priority {
            JobPriority::High => 0,
            JobPriority::Normal => 1,
            JobPriority::Low => 2,
        };

        let mut seq = self.seq.lock().await;
        let current_seq = *seq;
        *seq += 1;
        drop(seq);

        let pjob = PrioritizedJob {
            priority,
            seq: current_seq,
            job,
        };

        heap.push(Reverse(pjob));
        drop(heap);

        self.notify.notify_one();
        true
    }

    /// Receive a job, waiting if queue is empty.
    ///
    /// Jobs are returned in priority order (High > Normal > Low),
    /// with FIFO ordering within the same priority.
    pub async fn recv(&self) -> Option<WorkerJob> {
        loop {
            let mut heap = self.heap.lock().await;
            
            if let Some(Reverse(pjob)) = heap.pop() {
                return Some(pjob.job);
            }

            // Queue is empty - wait for notification.
            drop(heap);
            self.notify.notified().await;
        }
    }

    /// Get current queue length.
    pub async fn len(&self) -> usize {
        self.heap.lock().await.len()
    }

    /// Check if queue is empty.
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

/// Sender handle for priority queue.
#[derive(Debug, Clone)]
pub struct PrioritySender {
    queue: Arc<PriorityQueue>,
}

impl PrioritySender {
    /// Try to send a job without blocking.
    pub async fn try_send(&self, job: WorkerJob) -> bool {
        self.queue.try_send(job).await
    }

    /// Get queue length.
    pub async fn len(&self) -> usize {
        self.queue.len().await
    }
}

/// Receiver handle for priority queue.
#[derive(Debug)]
pub struct PriorityReceiver {
    queue: Arc<PriorityQueue>,
}

impl PriorityReceiver {
    /// Receive a job, waiting if queue is empty.
    pub async fn recv(&self) -> Option<WorkerJob> {
        self.queue.recv().await
    }

    /// Check if queue is empty.
    pub async fn is_empty(&self) -> bool {
        self.queue.is_empty().await
    }
}

/// Create a new priority queue with given capacity.
pub fn priority_channel(capacity: usize) -> (PrioritySender, PriorityReceiver) {
    let queue = PriorityQueue::new(capacity);
    let sender = PrioritySender {
        queue: queue.clone(),
    };
    let receiver = PriorityReceiver { queue };
    (sender, receiver)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{JobPriority, WorkerJobKind};
    use chrono::Utc;
    use uuid::Uuid;

    fn make_job(priority: JobPriority, kind: WorkerJobKind) -> WorkerJob {
        WorkerJob {
            id: Uuid::now_v7(),
            kind,
            created_at: Utc::now(),
            priority,
            payload_ref: None,
            frame_context: None,
            correlation_id: None,
            timeout_ms: 200,
        }
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let (tx, rx) = priority_channel(10);

        // Send jobs in reverse priority order.
        let low = make_job(JobPriority::Low, WorkerJobKind::ClassifyTurn);
        let normal = make_job(JobPriority::Normal, WorkerJobKind::ScanForErrors);
        let high = make_job(JobPriority::High, WorkerJobKind::ExtractAsccDelta);

        tx.try_send(low).await;
        tx.try_send(normal).await;
        tx.try_send(high).await;

        // Should receive in priority order: High, Normal, Low.
        let job1 = rx.recv().await.unwrap();
        let job2 = rx.recv().await.unwrap();
        let job3 = rx.recv().await.unwrap();

        assert!(matches!(job1.kind, WorkerJobKind::ExtractAsccDelta));
        assert!(matches!(job2.kind, WorkerJobKind::ScanForErrors));
        assert!(matches!(job3.kind, WorkerJobKind::ClassifyTurn));
    }

    #[tokio::test]
    async fn test_fifo_within_priority() {
        let (tx, rx) = priority_channel(10);

        // Send multiple high priority jobs.
        let high1 = make_job(JobPriority::High, WorkerJobKind::ClassifyTurn);
        let high2 = make_job(JobPriority::High, WorkerJobKind::ScanForErrors);
        let high3 = make_job(JobPriority::High, WorkerJobKind::ExtractAsccDelta);

        tx.try_send(high1.clone()).await;
        tx.try_send(high2.clone()).await;
        tx.try_send(high3.clone()).await;

        // Should receive in FIFO order within same priority.
        let job1 = rx.recv().await.unwrap();
        let job2 = rx.recv().await.unwrap();
        let job3 = rx.recv().await.unwrap();

        assert!(matches!(job1.kind, WorkerJobKind::ClassifyTurn));
        assert!(matches!(job2.kind, WorkerJobKind::ScanForErrors));
        assert!(matches!(job3.kind, WorkerJobKind::ExtractAsccDelta));
    }

    #[tokio::test]
    async fn test_capacity_limit() {
        let (tx, _rx) = priority_channel(2);

        let job1 = make_job(JobPriority::Low, WorkerJobKind::ClassifyTurn);
        let job2 = make_job(JobPriority::Low, WorkerJobKind::ScanForErrors);
        let job3 = make_job(JobPriority::Low, WorkerJobKind::ExtractAsccDelta);

        assert!(tx.try_send(job1).await);
        assert!(tx.try_send(job2).await);
        assert!(!tx.try_send(job3).await); // Queue full.
    }

    #[tokio::test]
    async fn test_recv_waits_for_send() {
        let (tx, rx) = priority_channel(10);

        // Start recv in background.
        let recv_handle = tokio::spawn(async move {
            rx.recv().await
        });

        // Small delay to ensure recv is waiting.
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Now send.
        let job = make_job(JobPriority::Normal, WorkerJobKind::ClassifyTurn);
        tx.try_send(job).await;

        let result = recv_handle.await.unwrap();
        assert!(result.is_some());
    }
}
