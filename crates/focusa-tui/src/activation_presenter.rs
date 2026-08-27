//! TUI activation/entitlement presenter (Spec 152E §21 surface consolidation).
//!
//! The TUI renders the same shared activation states, next actions, allowed
//! actions, masked identity, checkout/verify links, terminal delivery, node
//! management, denial/recovery, and resume handles as the menubar, the daemon
//! REST license routes, and lifecycle receipts. It never reimplements
//! identity, product, payment, Evaluation, license, node, or lease decisions:
//! this module only projects daemon payloads onto the frozen Spec 152E
//! presenter-state vocabulary (docs/contracts/spec152e-activation-internal.v1.json
//! `presenter_states` and the frozen next-action table). Unknown states,
//! unmasked emails, and raw credentials fail closed.
//!
//! The TUI is read-only: it renders posture and actions; every action is
//! executed through the daemon REST surface, never locally.

use serde_json::Value;

/// Frozen Spec 152E presenter states. Rendering only — the shared reducer in
/// focusa-license remains the only decision authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiPresenterState {
    EmailRequired,
    EmailVerificationPending,
    EmailVerified,
    SelectionRequired,
    CheckoutRequired,
    PaymentPending,
    LicenseDeliveryReady,
    Activated,
    Denied,
    RecoveryOnly,
}

impl TuiPresenterState {
    pub const ALL: [TuiPresenterState; 10] = [
        Self::EmailRequired,
        Self::EmailVerificationPending,
        Self::EmailVerified,
        Self::SelectionRequired,
        Self::CheckoutRequired,
        Self::PaymentPending,
        Self::LicenseDeliveryReady,
        Self::Activated,
        Self::Denied,
        Self::RecoveryOnly,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::EmailRequired => "email_required",
            Self::EmailVerificationPending => "email_verification_pending",
            Self::EmailVerified => "email_verified",
            Self::SelectionRequired => "selection_required",
            Self::CheckoutRequired => "checkout_required",
            Self::PaymentPending => "payment_pending",
            Self::LicenseDeliveryReady => "license_delivery_ready",
            Self::Activated => "activated",
            Self::Denied => "denied",
            Self::RecoveryOnly => "recovery_only",
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Activated | Self::Denied | Self::RecoveryOnly)
    }

    /// Frozen next-action table (mirror of the shared presenter projection).
    pub const fn next_action(self) -> &'static str {
        match self {
            Self::EmailRequired => "provide_email",
            Self::EmailVerificationPending => "verify_email",
            Self::EmailVerified => "select_offer",
            Self::SelectionRequired => "select_offer",
            Self::CheckoutRequired => "open_checkout",
            Self::PaymentPending => "poll_after_retry_after",
            Self::LicenseDeliveryReady => "deliver_license",
            Self::Activated => "activated",
            Self::Denied => "activate_or_manage_entitlement",
            Self::RecoveryOnly => "recovery",
        }
    }

    /// Equivalent allowed actions exposed by every presenter for the same
    /// canonical registration. Rendering guidance only; the reducer decides.
    pub const fn allowed_actions(self) -> &'static [&'static str] {
        match self {
            Self::EmailRequired => &["provide_email"],
            Self::EmailVerificationPending => &["verify_email", "resend_code"],
            Self::EmailVerified => &["select_offer"],
            Self::SelectionRequired => &[
                "select_purchase",
                "select_limited_access",
                "select_existing_key",
            ],
            Self::CheckoutRequired => &["open_checkout"],
            Self::PaymentPending => &["poll", "open_checkout"],
            Self::LicenseDeliveryReady => &["deliver_license", "activate"],
            Self::Activated => &["resume"],
            Self::Denied => &["activate_or_manage_entitlement", "recovery"],
            Self::RecoveryOnly => &["recovery", "repair", "export", "uninstall"],
        }
    }

    /// Fail-closed parse of a frozen presenter-state label.
    pub fn from_label(label: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|state| state.label() == label)
    }
}

/// Presenter-safe TUI activation view projected from the daemon
/// `GET /v1/activation/status` payload. No raw email, license key, poll
/// credential, or card data field exists by construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiActivationView {
    pub registration_id: String,
    pub state: String,
    pub terminal: bool,
    pub next_action: String,
    pub actions: Vec<String>,
    pub masked_email: Option<String>,
    pub safe_url: Option<String>,
    pub retry_posture: String,
    pub resume_handle: Option<String>,
}

impl TuiActivationView {
    /// Render one compact status line for the Deck Home surface.
    pub fn status_line(&self) -> String {
        if self.terminal {
            format!(
                "activation={} next={} handle={}",
                self.state,
                self.next_action,
                self.resume_handle.as_deref().unwrap_or("none")
            )
        } else {
            format!(
                "activation={} next={} actions=[{}] handle={}",
                self.state,
                self.next_action,
                self.actions.join(","),
                self.resume_handle.as_deref().unwrap_or("none")
            )
        }
    }
}

/// Presenter-safe TUI license posture projected from the daemon
/// `GET /v1/license/status` payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiLicensePosture {
    pub presenter_state: String,
    pub next_action: String,
    pub actions: Vec<String>,
    pub masked_identity: Option<String>,
}

/// Always-reachable surface families (Spec 152F P6 / §11.5, §13):
/// navigation, status, account, read, export, recovery, repair, update, and
/// uninstall are never disabled by a denied entitlement decision. Frozen
/// fixture shared with the menubar presenter
/// (apps/menubar/src/lib/entitlementPosture.ts
/// `ALWAYS_REACHABLE_ACTIONS`) and the menubar action map
/// (docs/contracts/spec152f-menubar-action-map.v1.json
/// `accessibility_fixtures.always_reachable`).
pub const ALWAYS_REACHABLE: [&str; 9] = [
    "navigation",
    "status",
    "account",
    "read",
    "export",
    "recovery",
    "repair",
    "update",
    "uninstall",
];

impl TuiLicensePosture {
    pub fn status_line(&self) -> String {
        format!(
            "entitlement={} next={} actions=[{}] identity={} | Recovery, export, repair, and uninstall remain available when execution is locked",
            self.presenter_state,
            self.next_action,
            self.actions.join(","),
            self.masked_identity.as_deref().unwrap_or("masked")
        )
    }

    /// Accessibility fixture shared with the menubar presenter: the next
    /// action label and the always-reachable set. Never disabled and never
    /// empty; parity-bound by
    /// tests/spec152f_presenter_accessibility_test.mjs.
    pub fn action_guide(&self) -> String {
        format!(
            "next={} label={} always_reachable=[{}]",
            self.next_action,
            self.next_action_label(),
            ALWAYS_REACHABLE.join(",")
        )
    }

    /// Human-readable next-action label (mirror of the menubar action guide;
    /// rendering only, never a commercial decision).
    fn next_action_label(&self) -> &'static str {
        match self.next_action.as_str() {
            "provide_email" => "Provide email",
            "verify_email" => "Verify email",
            "select_offer" => "Select offer",
            "open_checkout" => "Open checkout",
            "poll_after_retry_after" => "Poll after retry-after",
            "deliver_license" => "Deliver license",
            "activated" => "Entitlement active",
            "activate_or_manage_entitlement" => "Evaluate or manage entitlement",
            "recovery" => "Recovery",
            _ => "Manage entitlement",
        }
    }
}

/// Map the daemon license-status `status` label onto the frozen presenter
/// vocabulary. Identical mapping is enforced in the daemon REST surface and
/// bound by tests/spec152e_tui_rest_activation_test.py so every presenter
/// renders the same posture for the same entitlement.
pub fn presenter_state_for_entitlement_status(status: &str) -> TuiPresenterState {
    match status {
        "active" | "offline_grace" => TuiPresenterState::Activated,
        "recovery_only" => TuiPresenterState::RecoveryOnly,
        "expired" | "revoked" => TuiPresenterState::Denied,
        // Unactivated and legacy-migration-only postures re-enter the shared
        // activation flow; they never grant anything locally.
        _ => TuiPresenterState::EmailRequired,
    }
}

/// Fail-closed masked-email check: `^[^@]*\*[^@]*@[^@]+$` (frozen contract).
fn looks_masked(value: &str) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    if local.is_empty() || domain.is_empty() || domain.contains('@') || domain.contains('*') {
        return false;
    }
    let Some((head, tail)) = local.split_once('*') else {
        return false;
    };
    !head.is_empty() && (!tail.is_empty() || local.ends_with('*'))
}

/// Authority-owned safe link: https only and no userinfo credentials.
fn safe_authority_link(value: &str) -> Option<String> {
    let parsed = url_parse(value)?;
    if parsed.scheme != "https" || parsed.userinfo.is_some() {
        return None;
    }
    Some(value.to_string())
}

/// Minimal, dependency-free URL split (the TUI crate has no URL parser).
fn url_parse(value: &str) -> Option<UrlParts<'_>> {
    let rest = value.strip_prefix("https://")?;
    let (authority, _path) = rest.split_once('/').unwrap_or((rest, ""));
    let (userinfo, _host) = authority
        .split_once('@')
        .map_or((None, authority), |(user, host)| (Some(user), host));
    Some(UrlParts {
        scheme: "https",
        userinfo,
        _host,
    })
}

struct UrlParts<'a> {
    scheme: &'a str,
    userinfo: Option<&'a str>,
    _host: &'a str,
}

/// Project one daemon `GET /v1/activation/status` payload into a typed TUI
/// view. Deterministic: the first valid registration snapshot wins (the
/// daemon persists registrations in sorted order). Unknown states, unmasked
/// emails, and non-authority links fail closed.
pub fn project_activation_status(payload: &Value) -> Option<TuiActivationView> {
    let registrations = payload.get("registrations")?.as_array()?;
    for registration in registrations {
        let state_label = registration.get("state")?.as_str()?;
        let state = TuiPresenterState::from_label(state_label)?;
        let registration_id = registration.get("registration_id")?.as_str()?;
        if registration_id.trim().is_empty() {
            continue;
        }
        let masked_email = registration
            .get("masked_email")
            .and_then(Value::as_str)
            .filter(|value| looks_masked(value))
            .map(str::to_string);
        let safe_url = registration
            .get("safe_url")
            .and_then(Value::as_str)
            .and_then(safe_authority_link);
        let retry_posture = registration
            .get("retry_posture")
            .and_then(Value::as_str)
            .unwrap_or("none")
            .to_string();
        return Some(TuiActivationView {
            registration_id: registration_id.to_string(),
            state: state.label().to_string(),
            terminal: state.is_terminal(),
            next_action: state.next_action().to_string(),
            actions: state
                .allowed_actions()
                .iter()
                .map(|a| (*a).to_string())
                .collect(),
            masked_email,
            safe_url,
            retry_posture,
            resume_handle: (!state.is_terminal()).then(|| registration_id.to_string()),
        });
    }
    None
}

/// Project the daemon `GET /v1/license/status` payload into a typed TUI
/// posture. The masked identity is validated; unmasked values fail closed.
pub fn project_license_status(payload: &Value) -> Option<TuiLicensePosture> {
    let status = payload.get("status")?.as_str()?;
    let state = presenter_state_for_entitlement_status(status);
    let masked_identity = payload
        .get("masked_identity")
        .and_then(Value::as_str)
        .filter(|value| looks_masked(value))
        .map(str::to_string);
    Some(TuiLicensePosture {
        presenter_state: state.label().to_string(),
        next_action: state.next_action().to_string(),
        actions: state
            .allowed_actions()
            .iter()
            .map(|a| (*a).to_string())
            .collect(),
        masked_identity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn presenter_states_match_frozen_vocabulary() {
        let labels: Vec<&str> = TuiPresenterState::ALL.iter().map(|s| s.label()).collect();
        assert_eq!(
            labels,
            vec![
                "email_required",
                "email_verification_pending",
                "email_verified",
                "selection_required",
                "checkout_required",
                "payment_pending",
                "license_delivery_ready",
                "activated",
                "denied",
                "recovery_only",
            ]
        );
        assert!(TuiPresenterState::Activated.is_terminal());
        assert!(TuiPresenterState::Denied.is_terminal());
        assert!(TuiPresenterState::RecoveryOnly.is_terminal());
        assert!(!TuiPresenterState::PaymentPending.is_terminal());
    }

    #[test]
    fn projection_is_deterministic_and_fails_closed() {
        let payload = json!({
            "schema": "focusa.agent_activation_status.v1",
            "registrations": [
                {
                    "registration_id": "registration-0001",
                    "state": "checkout_required",
                    "masked_email": "c***@example.com",
                    "safe_url": "https://install.focusa.dev/pay/opaque-token",
                    "retry_posture": "none"
                },
                {
                    "registration_id": "registration-0002",
                    "state": "recovery_only",
                    "masked_email": "o***@example.com"
                }
            ]
        });
        let view = project_activation_status(&payload).expect("first registration wins");
        assert_eq!(view.registration_id, "registration-0001");
        assert_eq!(view.state, "checkout_required");
        assert_eq!(view.next_action, "open_checkout");
        assert_eq!(view.actions, vec!["open_checkout".to_string()]);
        assert_eq!(view.masked_email.as_deref(), Some("c***@example.com"));
        assert_eq!(
            view.safe_url.as_deref(),
            Some("https://install.focusa.dev/pay/opaque-token")
        );
        assert_eq!(view.resume_handle.as_deref(), Some("registration-0001"));
        // Deterministic: second run identical.
        assert_eq!(project_activation_status(&payload), Some(view));
    }

    #[test]
    fn unknown_states_unmasked_email_and_bad_links_fail_closed() {
        let unknown = json!({
            "registrations": [{"registration_id": "registration-0001", "state": "granted_now"}]
        });
        assert_eq!(project_activation_status(&unknown), None);
        let unmasked = json!({
            "registrations": [{
                "registration_id": "registration-0001",
                "state": "payment_pending",
                "masked_email": "raw@example.com"
            }]
        });
        assert_eq!(
            project_activation_status(&unmasked).unwrap().masked_email,
            None
        );
        let bad_link = json!({
            "registrations": [{
                "registration_id": "registration-0001",
                "state": "checkout_required",
                "safe_url": "http://evil.example.test/pay"
            }]
        });
        assert_eq!(project_activation_status(&bad_link).unwrap().safe_url, None);
        let credential_link = json!({
            "registrations": [{
                "registration_id": "registration-0001",
                "state": "checkout_required",
                "safe_url": "https://user:pass@evil.example.test/pay"
            }]
        });
        assert_eq!(
            project_activation_status(&credential_link)
                .unwrap()
                .safe_url,
            None
        );
    }

    #[test]
    fn license_status_projection_maps_shared_posture() {
        let active = json!({
            "status": "active",
            "masked_identity": "o***@example.com",
            "capabilities": []
        });
        let posture = project_license_status(&active).unwrap();
        assert_eq!(posture.presenter_state, "activated");
        assert_eq!(posture.next_action, "activated");
        assert!(posture.actions.contains(&"resume".to_string()));
        let recovery = json!({"status": "recovery_only"});
        assert_eq!(
            project_license_status(&recovery).unwrap().presenter_state,
            "recovery_only"
        );
        let unactivated = json!({"status": "unactivated"});
        assert_eq!(
            project_license_status(&unactivated)
                .unwrap()
                .presenter_state,
            "email_required"
        );
        let unmasked = json!({"status": "active", "masked_identity": "o@example.com"});
        assert_eq!(
            project_license_status(&unmasked).unwrap().masked_identity,
            None
        );
    }

    #[test]
    fn terminal_views_carry_no_resume_handle() {
        let payload = json!({
            "registrations": [{
                "registration_id": "registration-0009",
                "state": "recovery_only",
                "masked_email": "o***@example.com"
            }]
        });
        let view = project_activation_status(&payload).unwrap();
        assert!(view.terminal);
        assert_eq!(view.resume_handle, None);
        assert_eq!(view.next_action, "recovery");
    }
}
