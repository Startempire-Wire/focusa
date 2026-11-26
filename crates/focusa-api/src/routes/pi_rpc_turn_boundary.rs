//! Tracks one outer Pi RPC turn. A transport acknowledgement or low-level
//! agent run is not evidence that Pi has settled its retries and follow-ups.

#[derive(Default, Debug)]
pub(crate) struct PiRpcTurnBoundary {
    prompt_id: Option<String>,
    running: bool,
}

impl PiRpcTurnBoundary {
    pub(crate) fn is_busy(&self) -> bool {
        self.running || self.prompt_id.is_some()
    }

    pub(crate) fn reserve(&mut self, id: String) -> bool {
        if self.is_busy() {
            return false;
        }
        self.prompt_id = Some(id);
        true
    }

    pub(crate) fn started(&mut self) {
        self.running = true;
    }

    pub(crate) fn prompt_did_not_start(&mut self, id: &str) {
        if !self.running && self.prompt_id.as_deref() == Some(id) {
            self.prompt_id = None;
        }
    }

    pub(crate) fn settle(&mut self) -> bool {
        let was_busy = self.is_busy();
        *self = Self::default();
        was_busy
    }
}

/// Pi can finish its outer run without an abort even when the final model call failed.
/// This disposition is independent of the process exit code and the settlement event.
pub(crate) fn assistant_terminal_failure(stop_reason: Option<&str>) -> Option<&'static str> {
    match stop_reason {
        Some("error") => Some("Pi model turn failed; reconcile the saved frontier before retrying"),
        Some("aborted") => {
            Some("Pi assistant turn aborted; verify the saved frontier before resuming")
        }
        _ => None,
    }
}

/// `agent_settled` with an explicit abort disposition was added in Pi 1.1.0.
pub(crate) fn supports_settled_abort_disposition(version: &str) -> bool {
    let Some((major, minor, _patch)) = parse_stable_version(version.trim()) else {
        return false;
    };
    (major, minor) >= (1, 1)
}

fn parse_stable_version(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.split('.');
    let numbers = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    parts.next().is_none().then_some(numbers)
}

#[cfg(test)]
mod tests {
    use super::{
        PiRpcTurnBoundary, assistant_terminal_failure, supports_settled_abort_disposition,
    };

    #[test]
    fn normal_rpc_settlement_can_follow_a_model_error() {
        assert!(assistant_terminal_failure(Some("error")).is_some());
        assert!(assistant_terminal_failure(Some("aborted")).is_some());
        assert!(assistant_terminal_failure(Some("stop")).is_none());
        assert!(assistant_terminal_failure(Some("toolUse")).is_none());
    }

    #[test]
    fn only_compatible_pi_protocol_versions_are_admitted() {
        assert!(!supports_settled_abort_disposition("0.81.1"));
        assert!(!supports_settled_abort_disposition("1.0.4"));
        assert!(!supports_settled_abort_disposition("1.6.1-beta"));
        assert!(supports_settled_abort_disposition("1.1.0\n"));
        assert!(supports_settled_abort_disposition("1.6.1"));
    }

    #[test]
    fn one_prompt_through_retry_compaction_and_follow_up() {
        let mut turn = PiRpcTurnBoundary::default();
        assert!(!turn.settle());
        assert!(turn.reserve("first".into()));
        assert!(!turn.reserve("second".into()));
        turn.started();
        // Low-level agent_end/compaction and another agent_start retain custody.
        turn.started();
        turn.prompt_did_not_start("first");
        assert!(!turn.reserve("second".into()));
        assert!(turn.settle());
        assert!(!turn.settle());
        assert!(turn.reserve("second".into()));
    }

    #[test]
    fn only_matching_rejection_can_clear_pending_prompt() {
        let mut turn = PiRpcTurnBoundary::default();
        assert!(turn.reserve("current".into()));
        turn.prompt_did_not_start("older");
        assert!(turn.is_busy());
        turn.prompt_did_not_start("current");
        assert!(!turn.is_busy());
    }
}
