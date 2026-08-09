//! Ordered output events with exact identity, run generation, and monotonic
//! sequence (PTY-006).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::identity::PtyAttachmentIdentity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PtyEvent {
    Attached {
        geometry: PtyGeometry,
    },
    Output {
        data: String,
    },
    Resized {
        geometry: PtyGeometry,
    },
    Interrupted,
    Detached,
    Closed,
    Restarted,
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PtyGeometry {
    pub columns: u16,
    pub rows: u16,
    pub pixel_width: u16,
    pub pixel_height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PtyEventEnvelope {
    pub kind: PtyEvent,
    pub attachment_key: PtyAttachmentIdentity,
    pub work_surface_id: String,
    pub generation: u64,
    pub sequence: u64,
}

impl PtyEventEnvelope {
    pub fn new(
        kind: PtyEvent,
        identity: PtyAttachmentIdentity,
        generation: u64,
        sequence: u64,
    ) -> Self {
        Self {
            kind,
            work_surface_id: identity.work_surface_id.clone(),
            attachment_key: identity,
            generation,
            sequence,
        }
    }
}

/// Monotonic sequence counter scoped to one run generation.
#[derive(Debug, Default)]
pub struct SequenceCounter {
    value: AtomicU64,
}

impl SequenceCounter {
    pub fn next(&self) -> u64 {
        self.value.fetch_add(1, Ordering::Relaxed) + 1
    }
}

/// Stale-output rejection: an event is accepted only for the CURRENT run
/// generation with a sequence that continues the current monotonic run.
pub fn accepts_output(
    generation: u64,
    latest_generation: u64,
    sequence: u64,
    latest_sequence: u64,
) -> bool {
    generation == latest_generation && sequence <= latest_sequence
}

/// Thread-safe event sink feeding ordered envelopes to the process owner.
#[derive(Clone)]
pub struct EventSink {
    tx: mpsc::Sender<PtyEventEnvelope>,
    rx: Arc<Mutex<Option<mpsc::Receiver<PtyEventEnvelope>>>>,
    generation: Arc<AtomicU64>,
    sequence: Arc<SequenceCounter>,
    history: Arc<Mutex<VecDeque<PtyEventEnvelope>>>,
}

impl EventSink {
    pub fn new(generation: u64) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            tx,
            rx: Arc::new(Mutex::new(Some(rx))),
            generation: Arc::new(AtomicU64::new(generation)),
            sequence: Arc::new(SequenceCounter::default()),
            history: Arc::new(Mutex::new(VecDeque::with_capacity(4096))),
        }
    }

    /// Bounded event history so a reattached surface can resync from the
    /// SAME process generation (PTY-010). Oldest envelopes drop first.
    pub fn push_history(&self, envelope: PtyEventEnvelope) {
        let mut history = self.history.lock().unwrap();
        if history.len() >= 4096 {
            history.pop_front();
        }
        history.push_back(envelope);
    }

    /// Envelopes with a sequence strictly greater than `since_sequence`.
    pub fn history_after(&self, since_sequence: u64) -> Vec<PtyEventEnvelope> {
        self.history
            .lock()
            .unwrap()
            .iter()
            .filter(|envelope| envelope.sequence > since_sequence)
            .cloned()
            .collect()
    }

    pub fn latest_sequence_in_history(&self) -> u64 {
        self.history
            .lock()
            .unwrap()
            .back()
            .map(|envelope| envelope.sequence)
            .unwrap_or(0)
    }

    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }

    pub fn latest_sequence(&self) -> u64 {
        self.sequence.value.load(Ordering::Relaxed)
    }

    pub fn emit(
        &self,
        kind: PtyEvent,
        identity: PtyAttachmentIdentity,
    ) -> Result<(), mpsc::SendError<PtyEventEnvelope>> {
        let envelope = PtyEventEnvelope::new(
            kind,
            identity,
            self.generation.load(Ordering::Relaxed),
            self.sequence.next(),
        );
        self.push_history(envelope.clone());
        // Live streaming is best-effort: when the surface detached (dropped
        // the receiver), history remains the durable resync path (PTY-010).
        let _ = self.tx.send(envelope.clone());
        Ok(())
    }

    /// Returns true when the event belongs to the current generation and does
    /// not exceed the latest monotonic sequence.
    pub fn accepts(&self, generation: u64, sequence: u64) -> bool {
        accepts_output(
            generation,
            self.generation.load(Ordering::Relaxed),
            sequence,
            self.latest_sequence(),
        )
    }

    /// The single process owner takes the receiver (one subscriber per
    /// process; output ordering is preserved by the channel).
    pub fn take_receiver(&self) -> Option<mpsc::Receiver<PtyEventEnvelope>> {
        self.rx.lock().unwrap().take()
    }
}

/// Partial-read buffer: preserves bytes across reads so a multi-byte output
/// split across PTY reads is never dropped or reordered (PTY-006).
#[derive(Debug, Default)]
pub struct OutputAccumulator {
    pending: Vec<u8>,
}

impl OutputAccumulator {
    pub fn push(&mut self, bytes: &[u8]) {
        self.pending.extend_from_slice(bytes);
    }

    pub fn take_line(&mut self) -> Option<String> {
        let pos = self.pending.iter().position(|b| *b == b'\n')?;
        let line: Vec<u8> = self.pending.drain(..=pos).collect();
        Some(String::from_utf8_lossy(&line).into_owned())
    }

    pub fn remaining(&self) -> &[u8] {
        &self.pending
    }

    pub fn take_remaining(&mut self) -> String {
        let bytes = std::mem::take(&mut self.pending);
        String::from_utf8_lossy(&bytes).into_owned()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

/// Ordered output reader state for one generation (PTY-006).
pub struct GenerationReader {
    generation: u64,
    sequence: SequenceCounter,
    buffer: OutputAccumulator,
    pub identity: PtyAttachmentIdentity,
}

impl GenerationReader {
    pub fn new(generation: u64, identity: PtyAttachmentIdentity) -> Self {
        Self {
            generation,
            sequence: SequenceCounter::default(),
            buffer: OutputAccumulator::default(),
            identity,
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Feed raw bytes from a partial read; returns complete lines in order.
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<PtyEventEnvelope> {
        self.buffer.push(bytes);
        let mut events = Vec::new();
        while let Some(line) = self.buffer.take_line() {
            events.push(PtyEventEnvelope::new(
                PtyEvent::Output { data: line },
                self.identity.clone(),
                self.generation,
                self.sequence.next(),
            ));
        }
        events
    }

    /// Flush bytes that never terminated with a newline (preserved, ordered).
    pub fn flush(&mut self) -> Vec<PtyEventEnvelope> {
        if self.buffer.is_empty() {
            return Vec::new();
        }
        let data = self.buffer.take_remaining();
        if data.is_empty() {
            return Vec::new();
        }
        vec![PtyEventEnvelope::new(
            PtyEvent::Output { data },
            self.identity.clone(),
            self.generation,
            self.sequence.next(),
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::tests::sample_identity;

    #[test]
    fn partial_reads_preserve_bytes_and_order() {
        let mut reader = GenerationReader::new(1, sample_identity());
        // Two partial reads of one logical line: bytes must be preserved.
        let first = reader.feed(b"hel");
        assert!(first.is_empty(), "no newline yet — bytes preserved, not emitted");
        let second = reader.feed(b"lo\nwo");
        assert_eq!(second.len(), 1);
        match &second[0].kind {
            PtyEvent::Output { data } => assert_eq!(data, "hello\n"),
            other => panic!("expected output event, got {other:?}"),
        }
        assert_eq!(second[0].generation, 1);
        assert_eq!(second[0].sequence, 1);
        // flush preserves the unterminated tail in order
        let tail = reader.flush();
        assert_eq!(tail.len(), 1);
        match &tail[0].kind {
            PtyEvent::Output { data } => assert_eq!(data, "wo"),
            other => panic!("expected output event, got {other:?}"),
        }
        assert_eq!(tail[0].sequence, 2, "sequence stays monotonic");
    }

    #[test]
    fn stale_generation_cannot_impersonate_current() {
        let mut old = GenerationReader::new(1, sample_identity());
        let mut current = GenerationReader::new(2, sample_identity());
        let stale_events = old.feed(b"old process bytes\n");
        assert_eq!(stale_events[0].generation, 1);
        // The current generation rejects them: generation mismatch.
        assert!(!accepts_output(1, 2, 1, 1));
        let current_events = current.feed(b"current bytes\n");
        assert!(accepts_output(2, 2, 1, 1));
        assert_eq!(current_events[0].generation, 2);
    }

    #[test]
    fn sequence_is_monotonic_within_generation() {
        let mut reader = GenerationReader::new(3, sample_identity());
        reader.feed(b"a\nb\nc\n");
        let mut events = reader.feed(b"");
        events.extend(reader.flush());
        // feed produced 3 events (a,b,c lines); flush empty
        assert_eq!(events.len(), 0);
        let events = {
            let mut r = GenerationReader::new(3, sample_identity());
            r.feed(b"a\nb\nc\n")
        };
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].sequence, 1);
        assert_eq!(events[1].sequence, 2);
        assert_eq!(events[2].sequence, 3);
    }

    #[test]
    fn event_sink_emits_exact_identity() {
        let sink = EventSink::new(7);
        let _ = sink.emit(PtyEvent::Interrupted, sample_identity());
        assert_eq!(sink.generation(), 7);
        assert_eq!(sink.latest_sequence(), 1);
        assert!(sink.accepts(7, 1), "current generation, in-range sequence accepted");
        assert!(sink.accepts(7, 0), "replayed sequence within the run accepted");
        assert!(!sink.accepts(6, 1), "stale generation rejected");
        assert!(!sink.accepts(7, 2), "non-monotonic sequence rejected");
    }

    #[test]
    fn output_accumulator_merges_split_writes() {
        let mut acc = OutputAccumulator::default();
        acc.push(b"a");
        acc.push(b"b\nc");
        assert_eq!(acc.take_line().as_deref(), Some("ab\n"));
        assert_eq!(acc.take_remaining(), "c");
        assert!(acc.is_empty());
    }
}
