//! Recovery controls share the writer owner without acquiring execution authority.
use super::*;
use std::collections::HashMap;

fn recovery_writer(
    claims: &HashMap<String, WriterLease>,
    key: &str,
    headers: &HeaderMap,
    now: DateTime<Utc>,
) -> Result<Option<WriterLease>, (StatusCode, Json<Value>)> {
    let writer_id = writer_id_from_headers(headers)?;
    let token = if headers.contains_key(FENCING_HEADER) {
        Some(fencing_token_from_headers(headers)?)
    } else {
        None
    };
    match claims.get(key) {
        Some(lease) if lease.expires_at > now => validate_writer_for_key(
            claims,
            key,
            &writer_id,
            token
                .ok_or_else(|| bad_request(format!("missing required header: {FENCING_HEADER}")))?,
            now,
        )
        .map(Some),
        // No live owner can be displaced. This grants no lease, token or progress.
        _ => Ok(None),
    }
}

fn validate_recovery_partition(
    claims: &HashMap<String, WriterLease>,
    key: &str,
    scope: &WorkstreamKey,
    now: DateTime<Utc>,
) -> Result<(), (StatusCode, Json<Value>)> {
    scope
        .validate()
        .map_err(|error| bad_request(error.to_string()))?;
    let prefix = writer_claim_scope_prefix(
        scope.root_scope.root_path.to_string_lossy().as_ref(),
        &scope.continuity_id,
    );
    if let Some((_, lease)) = claims.iter().find(|(candidate, lease)| {
        candidate.as_str() != key && candidate.starts_with(&prefix) && lease.expires_at > now
    }) {
        return Err(conflict(
            "another work item in this workstream has a live writer; recovery cannot displace it",
            Some(lease.writer_id.clone()),
        ));
    }
    Ok(())
}

async fn dispatch_recovery<F>(
    claims: &mut HashMap<String, WriterLease>,
    key: &str,
    headers: &HeaderMap,
    stop: bool,
    dispatch: F,
) -> Result<Option<WriterLease>, (StatusCode, Json<Value>)>
where
    F: std::future::Future<Output = Result<(), (StatusCode, Json<Value>)>>,
{
    let writer = recovery_writer(claims, key, headers, Utc::now())?;
    dispatch.await?;
    // Failed dispatch preserves the claim; pause neither removes nor renews it.
    if stop {
        claims.remove(key);
    }
    Ok(writer)
}

pub(super) async fn control(
    scope: &WorkLoopScope,
    state: &Arc<AppState>,
    headers: &HeaderMap,
    action: Action,
) -> Result<Option<WriterLease>, (StatusCode, Json<Value>)> {
    let (stop, action_name) = match &action {
        Action::StopContinuousWork { .. } => (true, "work_loop_stop"),
        Action::PauseContinuousWork { .. } => (false, "work_loop_pause"),
        _ => return Err(bad_request("only pause and stop are recovery controls")),
    };
    let key = require_authoritative_claim_key(writer_claim_key(scope, state).await)?;
    let mut claims = state.writer_claims.write().await;
    validate_recovery_partition(&claims, &key, &scope.0, Utc::now())?;
    // Keep the existing claim lock through bounded queue acceptance. A replacement
    // writer cannot acquire and enqueue enable ahead of this pause/stop.
    dispatch_recovery(
        &mut claims,
        &key,
        headers,
        stop,
        send_work_loop_action(state, action_name, action),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(writer: &str, token: Option<u64>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(WRITER_HEADER, writer.parse().unwrap());
        if let Some(token) = token {
            headers.insert(FENCING_HEADER, token.to_string().parse().unwrap());
        }
        headers
    }

    #[tokio::test]
    async fn unclaimed_pause_and_stop_do_not_acquire_authority() {
        for stop in [false, true] {
            let mut claims = HashMap::new();
            let owner = dispatch_recovery(
                &mut claims,
                "exact-key",
                &headers("operator", None),
                stop,
                async { Ok(()) },
            )
            .await
            .unwrap();
            assert!(owner.is_none());
            assert!(claims.is_empty());
            assert!(
                validate_and_renew_writer_for_key(
                    &mut claims,
                    "exact-key",
                    "operator",
                    1,
                    Utc::now()
                )
                .is_err()
            );
        }
    }

    #[tokio::test]
    async fn expired_owner_can_be_stopped_without_renewal_or_foreign_changes() {
        let mut claims = HashMap::new();
        let now = Utc::now();
        acquire_writer_for_key(
            &mut claims,
            "expired".into(),
            "old".into(),
            41,
            now - chrono::Duration::milliseconds(WRITER_LEASE_TTL_MS + 1),
        )
        .unwrap();
        let other = acquire_writer_for_key(
            &mut claims,
            "other-partition".into(),
            "foreign".into(),
            42,
            now,
        )
        .unwrap();
        assert!(
            dispatch_recovery(
                &mut claims,
                "expired",
                &headers("operator", None),
                true,
                async { Ok(()) }
            )
            .await
            .unwrap()
            .is_none()
        );
        assert!(!claims.contains_key("expired"));
        assert_eq!(claims["other-partition"].writer_id, other.writer_id);
        assert_eq!(claims["other-partition"].fencing_token, other.fencing_token);
        assert_eq!(claims["other-partition"].expires_at, other.expires_at);
    }

    #[tokio::test]
    async fn live_owner_requires_the_exact_writer_and_fence_before_dispatch() {
        let now = Utc::now();
        let mut claims = HashMap::new();
        let owner =
            acquire_writer_for_key(&mut claims, "key".into(), "owner".into(), 9, now).unwrap();
        for request in [
            headers("foreign", None),
            headers("foreign", Some(9)),
            headers("owner", Some(8)),
        ] {
            let dispatched = std::sync::atomic::AtomicBool::new(false);
            let result = dispatch_recovery(&mut claims, "key", &request, true, async {
                dispatched.store(true, Ordering::SeqCst);
                Ok(())
            })
            .await;
            assert!(result.is_err());
            assert!(!dispatched.load(Ordering::SeqCst));
            assert_eq!(claims["key"].fencing_token, owner.fencing_token);
            assert_eq!(claims["key"].expires_at, owner.expires_at);
        }
    }

    #[tokio::test]
    async fn owned_pause_does_not_renew_and_failed_stop_does_not_release() {
        let mut claims = HashMap::new();
        let owner =
            acquire_writer_for_key(&mut claims, "key".into(), "owner".into(), 4, Utc::now())
                .unwrap();
        let request = headers("owner", Some(4));
        dispatch_recovery(&mut claims, "key", &request, false, async { Ok(()) })
            .await
            .unwrap();
        assert_eq!(claims["key"].renewed_at, owner.renewed_at);
        assert_eq!(claims["key"].expires_at, owner.expires_at);
        let result = dispatch_recovery(&mut claims, "key", &request, true, async {
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"status":"failed"})),
            ))
        })
        .await;
        assert!(result.is_err());
        assert_eq!(claims["key"].fencing_token, owner.fencing_token);
        dispatch_recovery(&mut claims, "key", &request, true, async { Ok(()) })
            .await
            .unwrap();
        assert!(!claims.contains_key("key"));
    }

    #[test]
    fn malformed_fencing_is_rejected_even_without_a_live_claim() {
        let mut request = headers("operator", None);
        request.insert(FENCING_HEADER, "invalid".parse().unwrap());
        assert!(recovery_writer(&HashMap::new(), "key", &request, Utc::now()).is_err());
        assert!(recovery_writer(&HashMap::new(), "key", &HeaderMap::new(), Utc::now()).is_err());
    }

    #[test]
    fn missing_exact_claim_does_not_hide_a_live_writer_on_another_work_item() {
        use focusa_core::scoped_state::ScopeRef;
        let root = tempfile::tempdir().unwrap();
        let scope = WorkstreamKey::new(
            ScopeRef::project("project:test", root.path(), "test", "sha256:test").unwrap(),
            "continuity:test",
        )
        .unwrap();
        let path = root.path().to_string_lossy();
        let key = writer_claim_key_for_partition(&path, &scope.continuity_id, "new-task");
        let previous = writer_claim_key_for_partition(&path, &scope.continuity_id, "old-task");
        let foreign = writer_claim_key_for_partition(&path, "other-continuity", "other-task");
        let now = Utc::now();
        let mut claims = HashMap::new();
        acquire_writer_for_key(&mut claims, previous.clone(), "other-writer".into(), 7, now)
            .unwrap();
        assert!(validate_recovery_partition(&claims, &key, &scope, now).is_err());
        claims.get_mut(&previous).unwrap().expires_at = now;
        acquire_writer_for_key(&mut claims, foreign.clone(), "foreign".into(), 8, now).unwrap();
        assert!(validate_recovery_partition(&claims, &key, &scope, now).is_ok());
        assert_eq!(claims[&foreign].writer_id, "foreign");
    }

    #[tokio::test]
    async fn replacement_waits_until_recovery_dispatch_is_accepted() {
        let claims = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let locked = claims.clone();
        let (entered, entered_rx) = tokio::sync::oneshot::channel();
        let (release, release_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let mut guard = locked.write().await;
            dispatch_recovery(&mut guard, "key", &headers("operator", None), true, async {
                entered.send(()).unwrap();
                release_rx.await.unwrap();
                Ok(())
            })
            .await
            .unwrap();
        });
        entered_rx.await.unwrap();
        assert!(claims.try_write().is_err());
        release.send(()).unwrap();
        task.await.unwrap();
        let mut guard = claims.write().await;
        let new_owner =
            acquire_writer_for_key(&mut guard, "key".into(), "new-owner".into(), 43, Utc::now())
                .unwrap();
        assert_eq!(new_owner.fencing_token, 43);
    }
}
