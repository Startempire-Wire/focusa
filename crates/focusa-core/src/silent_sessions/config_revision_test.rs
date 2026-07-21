use crate::silent_sessions::*;

use super::config_resolution_test::requested;

#[derive(Default)]
struct Backend {
    calls: Vec<&'static str>,
    verify: bool,
    restored: Option<SilentSessionConfig>,
}

impl ConfigRevisionBackend for Backend {
    fn persist_pending(&mut self, _plan: &ConfigRevisionPlan) -> anyhow::Result<()> {
        self.calls.push("persist");
        Ok(())
    }

    fn apply_hot(
        &mut self,
        _config: &SilentSessionConfig,
        _fields: &[String],
    ) -> anyhow::Result<()> {
        self.calls.push("hot");
        Ok(())
    }

    fn create_restart_plan(
        &mut self,
        _config: &SilentSessionConfig,
        _fields: &[String],
    ) -> anyhow::Result<()> {
        self.calls.push("restart");
        Ok(())
    }

    fn verify(&mut self, _plan: &ConfigRevisionPlan) -> anyhow::Result<bool> {
        self.calls.push("verify");
        Ok(self.verify)
    }

    fn commit(&mut self, _plan: &ConfigRevisionPlan) -> anyhow::Result<()> {
        self.calls.push("commit");
        Ok(())
    }

    fn rollback(&mut self, prior: &SilentSessionConfig) -> anyhow::Result<()> {
        self.calls.push("rollback");
        self.restored = Some(prior.clone());
        Ok(())
    }
}

#[test]
fn hot_revision_runs_preview_persist_apply_verify_commit_in_order() {
    let current = requested();
    let mut changed = current.clone();
    changed.notifications.channels = vec!["operator".into()];
    let plan = preview_config_revision(current, changed, Vec::new()).unwrap();
    assert_eq!(plan.hot_fields, vec!["notifications.channels"]);
    assert!(plan.restart_required_fields.is_empty());

    let mut backend = Backend {
        verify: true,
        ..Backend::default()
    };
    let outcome = execute_config_revision(&plan, true, &mut backend).unwrap();
    assert!(outcome.committed);
    assert!(!outcome.restart_required);
    assert_eq!(backend.calls, ["persist", "hot", "verify", "commit"]);
}

#[test]
fn restart_plan_is_created_and_failed_verification_rolls_back_prior_revision() {
    let current = requested();
    let mut changed = current.clone();
    changed.model.model = "model-restart".into();
    let plan = preview_config_revision(current.clone(), changed, Vec::new()).unwrap();
    assert_eq!(plan.restart_required_fields, vec!["model.model"]);

    let mut backend = Backend::default();
    let outcome = execute_config_revision(&plan, true, &mut backend).unwrap();
    assert!(!outcome.committed);
    assert!(outcome.restart_required);
    assert_eq!(outcome.stage, ConfigRevisionStage::RolledBack);
    assert_eq!(backend.calls, ["persist", "restart", "verify", "rollback"]);
    assert_eq!(backend.restored, Some(current));
}

#[test]
fn immutable_changes_and_ungated_revisions_never_persist() {
    let current = requested();
    let mut immutable = current.clone();
    immutable.identity.continuity_id = "other-boundary".into();
    assert!(matches!(
        preview_config_revision(current.clone(), immutable, Vec::new()),
        Err(ConfigRevisionError::ImmutableMutation(_))
    ));

    let mut hot = current.clone();
    hot.supervision.checkpoint_interval_seconds += 1;
    let plan = preview_config_revision(current, hot, Vec::new()).unwrap();
    let mut backend = Backend {
        verify: true,
        ..Backend::default()
    };
    assert!(matches!(
        execute_config_revision(&plan, false, &mut backend),
        Err(ConfigRevisionError::GateRequired)
    ));
    assert!(backend.calls.is_empty());
}
