//! CRDT-based multi-device sync — docs/43-multi-device-sync.md
//!
//! Conflict-free Replicated Data Types for event log synchronization.

use crate::types::EventLogEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Vector clock for causal ordering.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VectorClock {
    /// Map from machine_id to event counter.
    pub clocks: HashMap<String, u64>,
}

impl VectorClock {
    /// Create a new empty vector clock.
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    /// Increment the counter for a machine.
    pub fn increment(&mut self, machine_id: &str) {
        let counter = self.clocks.entry(machine_id.to_string()).or_insert(0);
        *counter += 1;
    }

    /// Get the counter for a machine.
    pub fn get(&self, machine_id: &str) -> u64 {
        self.clocks.get(machine_id).copied().unwrap_or(0)
    }

    /// Merge two vector clocks (taking the maximum of each counter).
    pub fn merge(&mut self, other: &VectorClock) {
        for (machine, counter) in &other.clocks {
            let entry = self.clocks.entry(machine.clone()).or_insert(0);
            *entry = (*entry).max(*counter);
        }
    }

    /// Compare two vector clocks for causality.
    ///
    /// Returns:
    /// - Some(Ordering::Less) if self happens before other
    /// - Some(Ordering::Greater) if self happens after other  
    /// - Some(Ordering::Equal) if they are the same
    /// - None if concurrent (incomparable)
    pub fn compare(&self, other: &VectorClock) -> Option<std::cmp::Ordering> {
        let mut has_less = false;
        let mut has_greater = false;

        // Check all keys from both clocks.
        let all_keys: std::collections::HashSet<_> = self
            .clocks
            .keys()
            .chain(other.clocks.keys())
            .collect();

        for key in all_keys {
            let self_val = self.get(key);
            let other_val = other.get(key);

            if self_val < other_val {
                has_less = true;
            } else if self_val > other_val {
                has_greater = true;
            }
        }

        match (has_less, has_greater) {
            (true, true) => None,                    // Concurrent.
            (true, false) => Some(std::cmp::Ordering::Less),   // self < other.
            (false, true) => Some(std::cmp::Ordering::Greater), // self > other.
            (false, false) => Some(std::cmp::Ordering::Equal), // Equal.
        }
    }

    /// Check if this clock descends from (has seen) another clock.
    pub fn descends_from(&self, other: &VectorClock) -> bool {
        matches!(
            self.compare(other),
            Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal)
        )
    }
}

/// CRDT operation for event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtEvent {
    /// Event payload.
    pub entry: EventLogEntry,
    /// Vector clock at time of creation.
    pub vector_clock: VectorClock,
    /// Lamport timestamp for total ordering fallback.
    pub lamport_ts: u64,
}

/// CRDT-based event log.
#[derive(Debug, Clone, Default)]
pub struct CrdtLog {
    pub events: Vec<CrdtEvent>,
    pub local_clock: VectorClock,
    pub lamport_counter: u64,
}

impl CrdtLog {
    /// Create a new empty CRDT log.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            local_clock: VectorClock::new(),
            lamport_counter: 0,
        }
    }

    /// Add a local event.
    pub fn add_local_event(
        &mut self,
        entry: EventLogEntry,
        machine_id: &str,
    ) -> CrdtEvent {
        self.local_clock.increment(machine_id);
        self.lamport_counter += 1;

        let event = CrdtEvent {
            entry,
            vector_clock: self.local_clock.clone(),
            lamport_ts: self.lamport_counter,
        };

        self.events.push(event.clone());
        event
    }

    /// Merge events from a remote peer.
    ///
    /// Returns the number of new events added.
    pub fn merge_remote(&mut self, remote_events: &[CrdtEvent]) -> usize {
        let mut added = 0;

        for remote in remote_events {
            // Check if we already have this event.
            if self.has_event(&remote.entry.id) {
                continue;
            }

            // Add the event.
            self.events.push(remote.clone());
            added += 1;

            // Update our vector clock to include remote's causality.
            self.local_clock.merge(&remote.vector_clock);

            // Update lamport counter.
            self.lamport_counter = self.lamport_counter.max(remote.lamport_ts) + 1;
        }

        // Re-sort events by causal order (vector clock), then lamport timestamp.
        self.sort_events();

        added
    }

    /// Check if we already have an event.
    fn has_event(&self, event_id: &Uuid) -> bool {
        self.events.iter().any(|e| e.entry.id == *event_id)
    }

    /// Sort events by causal order.
    fn sort_events(&mut self) {
        self.events.sort_by(|a, b| {
            // First try vector clock comparison.
            match a.vector_clock.compare(&b.vector_clock) {
                Some(ordering) => ordering,
                None => {
                    // Concurrent - use lamport timestamp as tiebreaker.
                    a.lamport_ts.cmp(&b.lamport_ts)
                }
            }
        });
    }

    /// Get all events that are causally after a given vector clock.
    pub fn events_after(&self, clock: &VectorClock) -> Vec<&CrdtEvent> {
        self.events
            .iter()
            .filter(|e| !clock.descends_from(&e.vector_clock))
            .collect()
    }

    /// Get current vector clock.
    pub fn clock(&self) -> &VectorClock {
        &self.local_clock
    }

    /// Get all events.
    pub fn all_events(&self) -> &[CrdtEvent] {
        &self.events
    }

    /// Count of events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Conflict resolver for concurrent updates.
pub struct ConflictResolver;

impl ConflictResolver {
    /// Resolve a conflict between two concurrent events.
    ///
    /// Per docs/43 §11: Deterministic resolution prevents hidden drift.
    pub fn resolve(e1: &CrdtEvent, e2: &CrdtEvent) -> std::cmp::Ordering {
        // Strategy: Lamport timestamp wins, then event ID for determinism.
        match e1.lamport_ts.cmp(&e2.lamport_ts) {
            std::cmp::Ordering::Equal => e1.entry.id.cmp(&e2.entry.id),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // SMOKE TEST: Vector clock basics
    #[test]
    fn test_vector_clock_basics() {
        let mut vc = VectorClock::new();
        vc.increment("machine-a");
        vc.increment("machine-a");
        vc.increment("machine-b");

        assert_eq!(vc.get("machine-a"), 2);
        assert_eq!(vc.get("machine-b"), 1);
        assert_eq!(vc.get("machine-c"), 0);
    }

    // SMOKE TEST: Vector clock merge
    #[test]
    fn test_vector_clock_merge() {
        let mut vc1 = VectorClock::new();
        vc1.increment("a");
        vc1.increment("a");

        let mut vc2 = VectorClock::new();
        vc2.increment("b");
        vc2.increment("b");
        vc2.increment("b");

        vc1.merge(&vc2);

        assert_eq!(vc1.get("a"), 2);
        assert_eq!(vc1.get("b"), 3);
    }

    // SMOKE TEST: Causality - happens before
    #[test]
    fn test_causality_happens_before() {
        let mut vc1 = VectorClock::new();
        vc1.increment("a");

        let mut vc2 = vc1.clone();
        vc2.increment("a");

        // vc1 happens before vc2
        assert_eq!(vc1.compare(&vc2), Some(std::cmp::Ordering::Less));
        assert_eq!(vc2.compare(&vc1), Some(std::cmp::Ordering::Greater));
    }

    // SMOKE TEST: Concurrent events
    #[test]
    fn test_concurrent_events() {
        let mut vc1 = VectorClock::new();
        vc1.increment("a");

        let mut vc2 = VectorClock::new();
        vc2.increment("b");

        // Neither happens before the other - concurrent
        assert_eq!(vc1.compare(&vc2), None);
        assert_eq!(vc2.compare(&vc1), None);
    }

    // SMOKE TEST: CRDT log add and retrieve
    #[test]
    fn test_crdt_log_add() {
        let mut log = CrdtLog::new();

        // Need to create a minimal EventLogEntry for testing
        let entry = EventLogEntry {
            id: Uuid::now_v7(),
            timestamp: Utc::now(),
            event: crate::types::FocusaEvent::SessionStarted {
                session_id: Uuid::now_v7(),
                adapter_id: None,
                workspace_id: None,
            },
            correlation_id: None,
            origin: crate::types::SignalOrigin::Daemon,
            machine_id: Some("test".to_string()),
            instance_id: None,
            session_id: None,
            thread_id: None,
            is_observation: false,
        };

        let event = log.add_local_event(entry, "machine-a");
        assert_eq!(log.len(), 1);
        assert_eq!(event.vector_clock.get("machine-a"), 1);
    }

    // STRESS TEST: Merge multiple remote events
    #[test]
    fn test_crdt_merge_remote() {
        let mut log1 = CrdtLog::new();
        let mut log2 = CrdtLog::new();

        // Both logs add independent events
        for i in 0..10 {
            let entry = EventLogEntry {
                id: Uuid::now_v7(),
                timestamp: Utc::now(),
                event: crate::types::FocusaEvent::IntuitionSignalObserved {
                    signal_id: Uuid::now_v7(),
                    signal_type: crate::types::SignalKind::UserInput,
                    severity: "0.5".to_string(),
                    summary: format!("event {}", i),
                    related_frame_id: None,
                },
                correlation_id: None,
                origin: crate::types::SignalOrigin::Daemon,
                machine_id: Some("log1".to_string()),
                instance_id: None,
                session_id: None,
                thread_id: None,
                is_observation: false,
            };
            log1.add_local_event(entry, "machine-a");
        }

        for i in 0..10 {
            let entry = EventLogEntry {
                id: Uuid::now_v7(),
                timestamp: Utc::now(),
                event: crate::types::FocusaEvent::IntuitionSignalObserved {
                    signal_id: Uuid::now_v7(),
                    signal_type: crate::types::SignalKind::AssistantOutput,
                    severity: "0.5".to_string(),
                    summary: format!("event {}", i),
                    related_frame_id: None,
                },
                correlation_id: None,
                origin: crate::types::SignalOrigin::Daemon,
                machine_id: Some("log2".to_string()),
                instance_id: None,
                session_id: None,
                thread_id: None,
                is_observation: false,
            };
            log2.add_local_event(entry, "machine-b");
        }

        // Merge log2 into log1
        let added = log1.merge_remote(&log2.all_events().to_vec());
        assert_eq!(added, 10);
        assert_eq!(log1.len(), 20);
    }

    // STRESS TEST: Concurrent updates from multiple machines
    #[test]
    fn test_concurrent_updates() {
        let mut logs: Vec<CrdtLog> = (0..5).map(|_| CrdtLog::new()).collect();

        // Each machine adds events concurrently
        for (i, log) in logs.iter_mut().enumerate() {
            for j in 0..20 {
                let entry = EventLogEntry {
                    id: Uuid::now_v7(),
                    timestamp: Utc::now(),
                    event: crate::types::FocusaEvent::IntuitionSignalObserved {
                        signal_id: Uuid::now_v7(),
                        signal_type: crate::types::SignalKind::UserInput,
                        severity: "0.5".to_string(),
                        summary: format!("machine-{}-event-{}", i, j),
                        related_frame_id: None,
                    },
                    correlation_id: None,
                    origin: crate::types::SignalOrigin::Daemon,
                    machine_id: Some(format!("machine-{}", i)),
                    instance_id: None,
                    session_id: None,
                    thread_id: None,
                    is_observation: false,
                };
                log.add_local_event(entry, &format!("machine-{}", i));
            }
        }

        // Merge all logs into the first one
        for i in 1..5 {
            let remote_events: Vec<CrdtEvent> = logs[i].all_events().to_vec();
            logs[0].merge_remote(&remote_events);
        }

        // Should have all 100 events
        assert_eq!(logs[0].len(), 100);

        // Verify events are sorted by causality
        for window in logs[0].all_events().windows(2) {
            let e1 = &window[0];
            let e2 = &window[1];

            // If e1 causally precedes e2, that's correct
            // If they're concurrent, lamport ordering should be consistent
            if let Some(ordering) = e1.vector_clock.compare(&e2.vector_clock) {
                assert!(
                    ordering != std::cmp::Ordering::Greater,
                    "Events not in causal order"
                );
            }
        }
    }

    // STRESS TEST: Deterministic conflict resolution
    #[test]
    fn test_deterministic_resolution() {
        let mut e1 = CrdtEvent {
            entry: EventLogEntry {
                id: Uuid::now_v7(),
                timestamp: Utc::now(),
                event: crate::types::FocusaEvent::SessionStarted {
                    session_id: Uuid::now_v7(),
                    adapter_id: None,
                    workspace_id: None,
                },
                correlation_id: None,
                origin: crate::types::SignalOrigin::Daemon,
                machine_id: Some("a".to_string()),
                instance_id: None,
                session_id: None,
                thread_id: None,
                is_observation: false,
            },
            vector_clock: VectorClock::new(),
            lamport_ts: 1,
        };

        let mut e2 = e1.clone();
        e2.lamport_ts = 2;
        e2.entry.id = Uuid::now_v7();

        // Concurrent events - resolution should be deterministic
        let ordering = ConflictResolver::resolve(&e1, &e2);
        assert_eq!(ordering, std::cmp::Ordering::Less); // e1 has lower lamport

        // Same lamport - event ID decides
        e2.lamport_ts = 1;
        let ordering2 = ConflictResolver::resolve(&e1, &e2);
        // Deterministic based on UUID ordering
        assert!(ordering2 == std::cmp::Ordering::Less || ordering2 == std::cmp::Ordering::Greater);
    }
}
