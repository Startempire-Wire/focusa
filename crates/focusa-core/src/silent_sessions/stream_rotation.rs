use std::collections::{HashMap, VecDeque};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{
    CanonicalStreamEvent, OutputChannel, PublishedChunk, SecureStreamStore, SilentSessionId,
    SilentSessionRunId, StreamStorageError, compress_chunk,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RotationPolicy {
    pub max_uncompressed_bytes: usize,
    pub max_compressed_bytes: usize,
    pub max_event_count: usize,
    pub max_chunk_age_seconds: i64,
}

impl Default for RotationPolicy {
    fn default() -> Self {
        Self {
            max_uncompressed_bytes: 1_048_576,
            max_compressed_bytes: 1_048_576,
            max_event_count: 250,
            max_chunk_age_seconds: 60,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationReason {
    ChannelChanged,
    UncompressedBytes,
    CompressedBytes,
    EventCount,
    Age,
    Checkpoint,
    Completion,
}

#[derive(Debug, Clone)]
pub struct SubscriberState {
    pub id: String,
    pub capacity: usize,
    pub queue: VecDeque<CanonicalStreamEvent>,
    pub last_acknowledged_cursor: Option<String>,
    pub disconnected: bool,
}

#[derive(Debug, Default)]
pub struct DurableFanout {
    subscribers: HashMap<String, SubscriberState>,
}

impl DurableFanout {
    pub fn subscribe(&mut self, id: impl Into<String>, capacity: usize) {
        let id = id.into();
        self.subscribers.insert(
            id.clone(),
            SubscriberState {
                id,
                capacity: capacity.max(1),
                queue: VecDeque::new(),
                last_acknowledged_cursor: None,
                disconnected: false,
            },
        );
    }

    pub fn acknowledge(&mut self, id: &str, cursor: String) {
        if let Some(subscriber) = self.subscribers.get_mut(id) {
            subscriber.last_acknowledged_cursor = Some(cursor);
        }
    }

    pub fn drain(&mut self, id: &str) -> Vec<CanonicalStreamEvent> {
        self.subscribers
            .get_mut(id)
            .map(|subscriber| subscriber.queue.drain(..).collect())
            .unwrap_or_default()
    }

    pub fn state(&self, id: &str) -> Option<&SubscriberState> {
        self.subscribers.get(id)
    }

    fn publish_durable(&mut self, events: &[CanonicalStreamEvent]) {
        for subscriber in self.subscribers.values_mut() {
            if subscriber.disconnected {
                continue;
            }
            if subscriber.queue.len() + events.len() > subscriber.capacity {
                subscriber.disconnected = true;
                subscriber.queue.clear();
                continue;
            }
            subscriber.queue.extend(events.iter().cloned());
        }
    }
}

pub struct StreamRotator {
    store: SecureStreamStore,
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    policy: RotationPolicy,
    active_channel: Option<OutputChannel>,
    active_since: Option<DateTime<Utc>>,
    active: Vec<CanonicalStreamEvent>,
    next_chunks: HashMap<OutputChannel, u64>,
    pub fanout: DurableFanout,
}

impl StreamRotator {
    pub fn new(
        store: SecureStreamStore,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
        policy: RotationPolicy,
    ) -> Self {
        Self {
            store,
            session_id,
            run_id,
            policy,
            active_channel: None,
            active_since: None,
            active: Vec::new(),
            next_chunks: HashMap::new(),
            fanout: DurableFanout::default(),
        }
    }

    pub fn push(
        &mut self,
        event: CanonicalStreamEvent,
        now: DateTime<Utc>,
    ) -> Result<Vec<(RotationReason, PublishedChunk)>, StreamStorageError> {
        let mut published = Vec::new();
        if self
            .active_channel
            .is_some_and(|channel| channel != event.channel)
        {
            if let Some(chunk) = self.flush()? {
                published.push((RotationReason::ChannelChanged, chunk));
            }
        }
        if !self.active.is_empty() {
            let prospective =
                encoded_size(&self.active) + encoded_size(std::slice::from_ref(&event));
            let reason = if self.active.len() >= self.policy.max_event_count {
                Some(RotationReason::EventCount)
            } else if prospective > self.policy.max_uncompressed_bytes {
                Some(RotationReason::UncompressedBytes)
            } else if self.active_since.is_some_and(|started| {
                (now - started).num_seconds() >= self.policy.max_chunk_age_seconds
            }) {
                Some(RotationReason::Age)
            } else {
                None
            };
            if let Some(reason) = reason
                && let Some(chunk) = self.flush()?
            {
                published.push((reason, chunk));
            }
        }
        self.active_channel = Some(event.channel);
        self.active_since.get_or_insert(now);
        self.active.push(event);
        if compress_chunk(&encode_events(&self.active)).len() > self.policy.max_compressed_bytes
            && self.active.len() > 1
        {
            let last = self.active.pop().expect("active event exists");
            if let Some(chunk) = self.flush()? {
                published.push((RotationReason::CompressedBytes, chunk));
            }
            self.active_channel = Some(last.channel);
            self.active_since = Some(now);
            self.active.push(last);
        }
        Ok(published)
    }

    pub fn checkpoint(&mut self) -> Result<Option<PublishedChunk>, StreamStorageError> {
        self.flush()
    }

    pub fn complete(&mut self) -> Result<Option<PublishedChunk>, StreamStorageError> {
        self.flush()
    }

    fn flush(&mut self) -> Result<Option<PublishedChunk>, StreamStorageError> {
        if self.active.is_empty() {
            return Ok(None);
        }
        let channel = self.active_channel.expect("active channel exists");
        let next_chunk = match self.next_chunks.get(&channel) {
            Some(value) => *value,
            None => {
                self.store
                    .resume_position(self.session_id, self.run_id, channel)?
                    .0
            }
        };
        let events = std::mem::take(&mut self.active);
        let published =
            self.store
                .publish_chunk(self.session_id, self.run_id, channel, next_chunk, &events)?;
        self.next_chunks.insert(channel, next_chunk + 1);
        self.active_channel = None;
        self.active_since = None;
        self.fanout.publish_durable(&events);
        Ok(Some(published))
    }
}

fn encoded_size(events: &[CanonicalStreamEvent]) -> usize {
    encode_events(events).len()
}

fn encode_events(events: &[CanonicalStreamEvent]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for event in events {
        if let Ok(encoded) = serde_json::to_vec(event) {
            bytes.extend_from_slice(&encoded);
            bytes.push(b'\n');
        }
    }
    bytes
}
