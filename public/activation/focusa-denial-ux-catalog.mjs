// Spec 152F.05.06 cross-presenter denial, purchase, and recovery UX fixture
// (schema focusa.spec152f.denial_ux_catalog.v1).
//
// One message catalog bound by CLI, desktop/menubar, TUI, Pi/agent, and
// branded-facade surfaces. Every message carries a plain-language blocked
// action, a reason, the retained always-reachable access, and ONE safe next
// action with a stable account/evaluation/checkout/recovery link. No internal
// route/lease details, no false urgency, no account enumeration, no raw
// email/key/token material, and no dead-end paywalls.
//
// Deterministic and dependency-free so presenters and tests can import it
// directly. Unknown states, families, and codes fail closed (never rendered).

// Generated from docs/contracts/spec152f-denial-ux-catalog.v1.json by
// the spec152f_denial_ux_parity_test generator; the embedded catalog is
// byte-identical to the JSON contract.
export const DENIAL_UX_CATALOG = {
  "schema": "focusa.spec152f.denial_ux_catalog.v1",
  "contract_version": 1,
  "authority": "docs/152f-simple-entitlement-gating-and-future-granularity-addendum.md",
  "objective": "Cross-presenter denial, purchase, and recovery UX: for each state/family, plain-language blocked action, reason, retained access, and one safe next action with a stable account/evaluation/checkout/recovery link.",
  "rules": [
    "plain language blocked action and reason",
    "retained access is always listed",
    "exactly one safe next action per message",
    "no internal route or lease details",
    "no false urgency",
    "no account enumeration or raw email/key/token",
    "no dead-end paywalls; every denial preserves a route to purchase or recovery"
  ],
  "always_reachable": [
    "navigation",
    "status",
    "account",
    "read",
    "export",
    "recovery",
    "repair",
    "update",
    "uninstall"
  ],
  "links": {
    "account": "/account",
    "evaluation": "/activate/evaluate",
    "checkout": "/activate/checkout",
    "recovery": "/activate/recovery"
  },
  "link_ids": [
    "account",
    "evaluation",
    "checkout",
    "recovery"
  ],
  "actions": [
    {
      "id": "continue",
      "label": "Continue"
    },
    {
      "id": "evaluate",
      "label": "Start a free Evaluation or purchase Focusa"
    },
    {
      "id": "purchase",
      "label": "Purchase or renew this optional family"
    },
    {
      "id": "recovery",
      "label": "Continue recovery"
    },
    {
      "id": "manage",
      "label": "Manage entitlement"
    },
    {
      "id": "verify_identity",
      "label": "Verify your account"
    },
    {
      "id": "diagnostics",
      "label": "Run diagnostics or update policy"
    }
  ],
  "error_registry": [
    {
      "code": "ENTITLEMENT_BASE_REQUIRED",
      "category": "authority",
      "http_status": 403,
      "retryable": false,
      "public_message": "A verified Evaluation or paid Focusa entitlement is required for value-producing Focusa work. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "safe_next_action": "evaluate",
      "action_label": "Start a free Evaluation or purchase Focusa",
      "link": "evaluation"
    },
    {
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "category": "feature",
      "http_status": 403,
      "retryable": false,
      "public_message": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "code": "ENTITLEMENT_REQUIRED",
      "category": "authority",
      "http_status": 403,
      "retryable": false,
      "public_message": "A usable authority-issued Focusa entitlement is required for this operation. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "safe_next_action": "evaluate",
      "action_label": "Start a free Evaluation or purchase Focusa",
      "link": "evaluation"
    },
    {
      "code": "ENTITLEMENT_LIMIT_EXHAUSTED",
      "category": "limit",
      "http_status": 429,
      "retryable": false,
      "public_message": "The authority-granted capacity for this operation is unavailable or exhausted. Manage capacity or retry after settlement.",
      "safe_next_action": "manage",
      "action_label": "Manage capacity or retry after settlement",
      "link": "account"
    },
    {
      "code": "ENTITLEMENT_RECOVERY_ONLY",
      "category": "recovery",
      "http_status": 403,
      "retryable": false,
      "public_message": "Your account is in recovery mode. Account, reading, export, recovery, repair, updates, and uninstall remain available.",
      "safe_next_action": "recovery",
      "action_label": "Continue recovery",
      "link": "recovery"
    },
    {
      "code": "ENTITLEMENT_SNAPSHOT_MISSING",
      "category": "authority",
      "http_status": 503,
      "retryable": true,
      "public_message": "Entitlement status is unavailable right now. Refresh status or run diagnostics; recovery remains available.",
      "safe_next_action": "diagnostics",
      "action_label": "Refresh status or run diagnostics",
      "link": "recovery"
    },
    {
      "code": "ENTITLEMENT_ROUTE_UNCLASSIFIED",
      "category": "classification",
      "http_status": 403,
      "retryable": false,
      "public_message": "This operation has no registered entitlement classification and is blocked before execution. Run diagnostics or update policy.",
      "safe_next_action": "diagnostics",
      "action_label": "Run diagnostics or update policy",
      "link": "recovery"
    },
    {
      "code": "ENTITLEMENT_POLICY_UNKNOWN",
      "category": "policy",
      "http_status": 403,
      "retryable": false,
      "public_message": "The entitlement policy is unavailable or unrecognized and this operation is blocked. Run diagnostics or update policy.",
      "safe_next_action": "diagnostics",
      "action_label": "Run diagnostics or update policy",
      "link": "recovery"
    },
    {
      "code": "ENTITLEMENT_RESERVATION_FAILED",
      "category": "reservation",
      "http_status": 503,
      "retryable": true,
      "public_message": "Licensed capacity could not be reserved safely before execution. Retry with the same request or run diagnostics.",
      "safe_next_action": "diagnostics",
      "action_label": "Retry with the same request or run diagnostics",
      "link": "recovery"
    },
    {
      "code": "ENTITLEMENT_IDEMPOTENCY_REQUIRED",
      "category": "idempotency",
      "http_status": 428,
      "retryable": true,
      "public_message": "A stable request identifier is required before reserving licensed capacity. Retry with the same request identifier.",
      "safe_next_action": "manage",
      "action_label": "Retry with the same request identifier",
      "link": "account"
    }
  ],
  "message_grid": [
    {
      "state": "pending_unverified",
      "family": "account_recovery",
      "kind": "available",
      "code": null,
      "blocked_action": "Account, recovery, repair, and uninstall actions",
      "reason": "Account, recovery, repair, update, and uninstall actions remain available in every entitlement state.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "pending_unverified",
      "family": "read_projection",
      "kind": "denied_read",
      "code": "ENTITLEMENT_REQUIRED",
      "blocked_action": "Reading your existing local data",
      "reason": "Verifying your account unlocks read access to your existing local data. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "verify_identity",
      "action_label": "Verify your account",
      "link": "account"
    },
    {
      "state": "pending_unverified",
      "family": "base_focusa",
      "kind": "denied_base",
      "code": "ENTITLEMENT_BASE_REQUIRED",
      "blocked_action": "Creating or changing projects, missions, Focus State, Workpoints, Trajectories, and evidence",
      "reason": "A verified Evaluation or paid Focusa entitlement is required for value-producing Focusa work. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "evaluate",
      "action_label": "Start a free Evaluation or purchase Focusa",
      "link": "evaluation"
    },
    {
      "state": "pending_unverified",
      "family": "automation",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Silent sessions, scheduled, parallel, and unattended work",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "pending_unverified",
      "family": "team_remote",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Adding devices, peers, and remote collaboration",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "pending_unverified",
      "family": "release_proof",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Release orchestration and governed proof bundles",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "pending_unverified",
      "family": "premium_updates",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Unattended and preview or nightly updates",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "pending_unverified",
      "family": "customer_data_export",
      "kind": "available",
      "code": null,
      "blocked_action": "Exporting your own customer data",
      "reason": "You always retain access to your own data, including export, even when execution is locked.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "pending_unverified",
      "family": "internal_maintenance",
      "kind": "denied_maintenance",
      "code": "ENTITLEMENT_ROUTE_UNCLASSIFIED",
      "blocked_action": "Background maintenance work",
      "reason": "This operation has no registered entitlement classification and is blocked before execution.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "diagnostics",
      "action_label": "Run diagnostics or update policy",
      "link": "recovery"
    },
    {
      "state": "verified_no_license",
      "family": "account_recovery",
      "kind": "available",
      "code": null,
      "blocked_action": "Account, recovery, repair, and uninstall actions",
      "reason": "Account, recovery, repair, update, and uninstall actions remain available in every entitlement state.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "verified_no_license",
      "family": "read_projection",
      "kind": "available",
      "code": null,
      "blocked_action": "Reading your existing local data",
      "reason": "Read-only projection of your existing local data remains available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "verified_no_license",
      "family": "base_focusa",
      "kind": "limited",
      "code": "ENTITLEMENT_BASE_REQUIRED",
      "blocked_action": "Creating or changing projects, missions, Focus State, Workpoints, Trajectories, and evidence",
      "reason": "Your identity is verified, but there is no active entitlement yet; the one-project manual Focusa subset remains available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "evaluate",
      "action_label": "Start a free Evaluation or purchase Focusa",
      "link": "evaluation"
    },
    {
      "state": "verified_no_license",
      "family": "automation",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Silent sessions, scheduled, parallel, and unattended work",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "verified_no_license",
      "family": "team_remote",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Adding devices, peers, and remote collaboration",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "verified_no_license",
      "family": "release_proof",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Release orchestration and governed proof bundles",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "verified_no_license",
      "family": "premium_updates",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Unattended and preview or nightly updates",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "verified_no_license",
      "family": "customer_data_export",
      "kind": "available",
      "code": null,
      "blocked_action": "Exporting your own customer data",
      "reason": "You always retain access to your own data, including export, even when execution is locked.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "verified_no_license",
      "family": "internal_maintenance",
      "kind": "denied_maintenance",
      "code": "ENTITLEMENT_ROUTE_UNCLASSIFIED",
      "blocked_action": "Background maintenance work",
      "reason": "This operation has no registered entitlement classification and is blocked before execution.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "diagnostics",
      "action_label": "Run diagnostics or update policy",
      "link": "recovery"
    },
    {
      "state": "active_paid",
      "family": "account_recovery",
      "kind": "available",
      "code": null,
      "blocked_action": "Account, recovery, repair, and uninstall actions",
      "reason": "Account, recovery, repair, update, and uninstall actions remain available in every entitlement state.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "active_paid",
      "family": "read_projection",
      "kind": "available",
      "code": null,
      "blocked_action": "Reading your existing local data",
      "reason": "Read-only projection of your existing local data remains available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "active_paid",
      "family": "base_focusa",
      "kind": "available",
      "code": null,
      "blocked_action": "Creating or changing projects, missions, Focus State, Workpoints, Trajectories, and evidence",
      "reason": "A verified Evaluation or paid Focusa entitlement enables the complete base Focusa value loop.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "active_paid",
      "family": "automation",
      "kind": "feature",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Silent sessions, scheduled, parallel, and unattended work",
      "reason": "This optional family requires an additional authority-issued grant. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "manage",
      "action_label": "Manage entitlement",
      "link": "account"
    },
    {
      "state": "active_paid",
      "family": "team_remote",
      "kind": "feature",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Adding devices, peers, and remote collaboration",
      "reason": "This optional family requires an additional authority-issued grant. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "manage",
      "action_label": "Manage entitlement",
      "link": "account"
    },
    {
      "state": "active_paid",
      "family": "release_proof",
      "kind": "feature",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Release orchestration and governed proof bundles",
      "reason": "This optional family requires an additional authority-issued grant. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "manage",
      "action_label": "Manage entitlement",
      "link": "account"
    },
    {
      "state": "active_paid",
      "family": "premium_updates",
      "kind": "feature",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Unattended and preview or nightly updates",
      "reason": "This optional family requires an additional authority-issued grant. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "manage",
      "action_label": "Manage entitlement",
      "link": "account"
    },
    {
      "state": "active_paid",
      "family": "customer_data_export",
      "kind": "available",
      "code": null,
      "blocked_action": "Exporting your own customer data",
      "reason": "You always retain access to your own data, including export, even when execution is locked.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "active_paid",
      "family": "internal_maintenance",
      "kind": "denied_maintenance",
      "code": "ENTITLEMENT_ROUTE_UNCLASSIFIED",
      "blocked_action": "Background maintenance work",
      "reason": "This operation has no registered entitlement classification and is blocked before execution.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "diagnostics",
      "action_label": "Run diagnostics or update policy",
      "link": "recovery"
    },
    {
      "state": "offline_grace",
      "family": "account_recovery",
      "kind": "available",
      "code": null,
      "blocked_action": "Account, recovery, repair, and uninstall actions",
      "reason": "Account, recovery, repair, update, and uninstall actions remain available in every entitlement state.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "offline_grace",
      "family": "read_projection",
      "kind": "available",
      "code": null,
      "blocked_action": "Reading your existing local data",
      "reason": "Read-only projection of your existing local data remains available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "offline_grace",
      "family": "base_focusa",
      "kind": "available",
      "code": null,
      "blocked_action": "Creating or changing projects, missions, Focus State, Workpoints, Trajectories, and evidence",
      "reason": "A verified Evaluation or paid Focusa entitlement enables the complete base Focusa value loop.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "offline_grace",
      "family": "automation",
      "kind": "feature",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Silent sessions, scheduled, parallel, and unattended work",
      "reason": "This optional family requires an additional authority-issued grant. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "manage",
      "action_label": "Manage entitlement",
      "link": "account"
    },
    {
      "state": "offline_grace",
      "family": "team_remote",
      "kind": "feature",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Adding devices, peers, and remote collaboration",
      "reason": "This optional family requires an additional authority-issued grant. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "manage",
      "action_label": "Manage entitlement",
      "link": "account"
    },
    {
      "state": "offline_grace",
      "family": "release_proof",
      "kind": "feature",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Release orchestration and governed proof bundles",
      "reason": "This optional family requires an additional authority-issued grant. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "manage",
      "action_label": "Manage entitlement",
      "link": "account"
    },
    {
      "state": "offline_grace",
      "family": "premium_updates",
      "kind": "feature",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Unattended and preview or nightly updates",
      "reason": "This optional family requires an additional authority-issued grant. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "manage",
      "action_label": "Manage entitlement",
      "link": "account"
    },
    {
      "state": "offline_grace",
      "family": "customer_data_export",
      "kind": "available",
      "code": null,
      "blocked_action": "Exporting your own customer data",
      "reason": "You always retain access to your own data, including export, even when execution is locked.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "offline_grace",
      "family": "internal_maintenance",
      "kind": "denied_maintenance",
      "code": "ENTITLEMENT_ROUTE_UNCLASSIFIED",
      "blocked_action": "Background maintenance work",
      "reason": "This operation has no registered entitlement classification and is blocked before execution.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "diagnostics",
      "action_label": "Run diagnostics or update policy",
      "link": "recovery"
    },
    {
      "state": "expired",
      "family": "account_recovery",
      "kind": "available",
      "code": null,
      "blocked_action": "Account, recovery, repair, and uninstall actions",
      "reason": "Account, recovery, repair, update, and uninstall actions remain available in every entitlement state.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "expired",
      "family": "read_projection",
      "kind": "available",
      "code": null,
      "blocked_action": "Reading your existing local data",
      "reason": "Read-only projection of your existing local data remains available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "expired",
      "family": "base_focusa",
      "kind": "denied_base",
      "code": "ENTITLEMENT_BASE_REQUIRED",
      "blocked_action": "Creating or changing projects, missions, Focus State, Workpoints, Trajectories, and evidence",
      "reason": "A verified Evaluation or paid Focusa entitlement is required for value-producing Focusa work. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "evaluate",
      "action_label": "Start a free Evaluation or purchase Focusa",
      "link": "evaluation"
    },
    {
      "state": "expired",
      "family": "automation",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Silent sessions, scheduled, parallel, and unattended work",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "expired",
      "family": "team_remote",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Adding devices, peers, and remote collaboration",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "expired",
      "family": "release_proof",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Release orchestration and governed proof bundles",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "expired",
      "family": "premium_updates",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Unattended and preview or nightly updates",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "expired",
      "family": "customer_data_export",
      "kind": "available",
      "code": null,
      "blocked_action": "Exporting your own customer data",
      "reason": "You always retain access to your own data, including export, even when execution is locked.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "expired",
      "family": "internal_maintenance",
      "kind": "denied_maintenance",
      "code": "ENTITLEMENT_ROUTE_UNCLASSIFIED",
      "blocked_action": "Background maintenance work",
      "reason": "This operation has no registered entitlement classification and is blocked before execution.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "diagnostics",
      "action_label": "Run diagnostics or update policy",
      "link": "recovery"
    },
    {
      "state": "refunded_or_revoked",
      "family": "account_recovery",
      "kind": "available",
      "code": null,
      "blocked_action": "Account, recovery, repair, and uninstall actions",
      "reason": "Account, recovery, repair, update, and uninstall actions remain available in every entitlement state.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "refunded_or_revoked",
      "family": "read_projection",
      "kind": "available",
      "code": null,
      "blocked_action": "Reading your existing local data",
      "reason": "Read-only projection of your existing local data remains available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "refunded_or_revoked",
      "family": "base_focusa",
      "kind": "denied_base",
      "code": "ENTITLEMENT_BASE_REQUIRED",
      "blocked_action": "Creating or changing projects, missions, Focus State, Workpoints, Trajectories, and evidence",
      "reason": "A verified Evaluation or paid Focusa entitlement is required for value-producing Focusa work. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "evaluate",
      "action_label": "Start a free Evaluation or purchase Focusa",
      "link": "evaluation"
    },
    {
      "state": "refunded_or_revoked",
      "family": "automation",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Silent sessions, scheduled, parallel, and unattended work",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "refunded_or_revoked",
      "family": "team_remote",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Adding devices, peers, and remote collaboration",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "refunded_or_revoked",
      "family": "release_proof",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Release orchestration and governed proof bundles",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "refunded_or_revoked",
      "family": "premium_updates",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Unattended and preview or nightly updates",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "refunded_or_revoked",
      "family": "customer_data_export",
      "kind": "available",
      "code": null,
      "blocked_action": "Exporting your own customer data",
      "reason": "You always retain access to your own data, including export, even when execution is locked.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "refunded_or_revoked",
      "family": "internal_maintenance",
      "kind": "denied_maintenance",
      "code": "ENTITLEMENT_ROUTE_UNCLASSIFIED",
      "blocked_action": "Background maintenance work",
      "reason": "This operation has no registered entitlement classification and is blocked before execution.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "diagnostics",
      "action_label": "Run diagnostics or update policy",
      "link": "recovery"
    },
    {
      "state": "missing_or_corrupt",
      "family": "account_recovery",
      "kind": "available",
      "code": null,
      "blocked_action": "Account, recovery, repair, and uninstall actions",
      "reason": "Account, recovery, repair, update, and uninstall actions remain available in every entitlement state.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "missing_or_corrupt",
      "family": "read_projection",
      "kind": "available",
      "code": null,
      "blocked_action": "Reading your existing local data",
      "reason": "Read-only projection of your existing local data remains available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "missing_or_corrupt",
      "family": "base_focusa",
      "kind": "denied_base",
      "code": "ENTITLEMENT_BASE_REQUIRED",
      "blocked_action": "Creating or changing projects, missions, Focus State, Workpoints, Trajectories, and evidence",
      "reason": "A verified Evaluation or paid Focusa entitlement is required for value-producing Focusa work. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "evaluate",
      "action_label": "Start a free Evaluation or purchase Focusa",
      "link": "evaluation"
    },
    {
      "state": "missing_or_corrupt",
      "family": "automation",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Silent sessions, scheduled, parallel, and unattended work",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "missing_or_corrupt",
      "family": "team_remote",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Adding devices, peers, and remote collaboration",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "missing_or_corrupt",
      "family": "release_proof",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Release orchestration and governed proof bundles",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "missing_or_corrupt",
      "family": "premium_updates",
      "kind": "denied_premium",
      "code": "ENTITLEMENT_FEATURE_REQUIRED",
      "blocked_action": "Unattended and preview or nightly updates",
      "reason": "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "purchase",
      "action_label": "Purchase or renew this optional family",
      "link": "checkout"
    },
    {
      "state": "missing_or_corrupt",
      "family": "customer_data_export",
      "kind": "available",
      "code": null,
      "blocked_action": "Exporting your own customer data",
      "reason": "You always retain access to your own data, including export, even when execution is locked.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "continue",
      "action_label": "Continue",
      "link": "account"
    },
    {
      "state": "missing_or_corrupt",
      "family": "internal_maintenance",
      "kind": "denied_maintenance",
      "code": "ENTITLEMENT_ROUTE_UNCLASSIFIED",
      "blocked_action": "Background maintenance work",
      "reason": "This operation has no registered entitlement classification and is blocked before execution.",
      "retained_access": [
        "navigation",
        "status",
        "account",
        "read",
        "export",
        "recovery",
        "repair",
        "update",
        "uninstall"
      ],
      "safe_next_action": "diagnostics",
      "action_label": "Run diagnostics or update policy",
      "link": "recovery"
    }
  ],
  "presenter_bindings": {
    "cli": {
      "fixture": "crates/focusa-cli/tests/fixtures/spec152f-cli-presenter-fixtures.v1.json",
      "binds": [
        "recovery_allowance",
        "base_product",
        "premium",
        "preflight"
      ],
      "action_vocabulary": [
        "evaluate",
        "purchase",
        "recovery",
        "manage"
      ],
      "notes": "CLI fixtures project the canonical base/premium/recovery decisions; denial UX renders the catalog message with one safe next action."
    },
    "desktop": {
      "fixture": "docs/contracts/spec152f-menubar-action-map.v1.json",
      "binds": [
        "always_reachable",
        "recovery_account",
        "action_classes",
        "navigation_display"
      ],
      "action_vocabulary": [
        "evaluate",
        "purchase",
        "recovery",
        "manage"
      ],
      "notes": "Menubar accessibility fixtures share the always-reachable set and the Evaluation/purchase/recovery action guide."
    },
    "tui": {
      "fixture": "crates/focusa-tui/src/activation_presenter.rs",
      "binds": [
        "ALWAYS_REACHABLE",
        "presenter_state",
        "next_action",
        "action_guide"
      ],
      "action_vocabulary": [
        "evaluate",
        "purchase",
        "recovery",
        "manage"
      ],
      "notes": "TUI presenter renders the same always-reachable set and the shared action guide on Deck Home."
    },
    "pi": {
      "fixture": "apps/pi-extension/src/entitlement-policy-adapter.ts",
      "binds": [
        "failure_class",
        "entitlement_blocked",
        "recovery",
        "allowed",
        "status_path"
      ],
      "action_vocabulary": [
        "recovery",
        "evaluate",
        "purchase",
        "manage"
      ],
      "notes": "Pi/agent adapter returns stable machine JSON with recovery actions; catalog messages explain the denial."
    },
    "facade": {
      "fixture": "public/activation/focusa-facade-policy-presenter.mjs",
      "binds": [
        "family",
        "posture",
        "action",
        "action_label",
        "recovery_action",
        "always_reachable"
      ],
      "action_vocabulary": [
        "evaluate",
        "purchase",
        "recovery",
        "manage"
      ],
      "notes": "Branded facades present the same canonical decision with the catalog's action and recovery vocabulary."
    }
  },
  "accessibility": {
    "retained_access_always_present": true,
    "exactly_one_safe_next_action": true,
    "no_disabled_traps": true,
    "links_are_relative": true,
    "denied_messages_never_empty": true
  },
  "privacy": {
    "no_raw_email": true,
    "no_raw_key_or_token": true,
    "no_account_enumeration": true,
    "no_internal_route_or_lease_details": true,
    "no_false_urgency": true,
    "no_dead_end_paywalls": true
  }
};

const DENIAL_UX_SCHEMA = DENIAL_UX_CATALOG.schema;
const RETAINED_ACCESS = Object.freeze([...DENIAL_UX_CATALOG.always_reachable]);
const LINKS = Object.freeze({ ...DENIAL_UX_CATALOG.links });
const REGISTRY = Object.freeze(DENIAL_UX_CATALOG.error_registry.map((entry) => Object.freeze({ ...entry })));
const GRID = Object.freeze(DENIAL_UX_CATALOG.message_grid.map((cell) => Object.freeze({ ...cell })));

/** Stable relative link for an account/evaluation/checkout/recovery route. */
export function denialUxLink(linkId) {
  if (typeof linkId !== "string") return null;
  const path = LINKS[linkId];
  if (typeof path !== "string") return null;
  if (!path.startsWith("/")) return null;
  if (/[?&#]/.test(path)) return null;
  return path;
}

/** Canonical message for a stable error code; unknown codes fail closed. */
export function messageForErrorCode(code) {
  if (typeof code !== "string") return null;
  const entry = REGISTRY.find((row) => row.code === code);
  if (!entry) return null;
  const link = denialUxLink(entry.link);
  if (!link) return null;
  return Object.freeze({
    code: entry.code,
    blocked_action: entry.public_message,
    reason: entry.public_message,
    retained_access: RETAINED_ACCESS,
    safe_next_action: entry.safe_next_action,
    action_label: entry.action_label,
    link: entry.link,
    link_path: link,
  });
}

/**
 * Project the canonical catalog message for one (state, family) pair or a
 * stable error code. Input must be a plain object carrying exactly one of
 * {state, family} or {code}; anything else (raw emails, keys, tokens, links,
 * grants, prices, extra fields) fails closed with null.
 */
export function projectDenialUxMessage(input) {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const keys = Object.keys(input);
  const hasGrid = keys.includes("state") && keys.includes("family");
  const hasCode = keys.includes("code");
  if (hasGrid === hasCode) return null;
  if (keys.some((key) => !["state", "family", "code"].includes(key))) return null;
  if (hasCode) return messageForErrorCode(input.code);
  const cell = GRID.find((row) => row.state === input.state && row.family === input.family);
  if (!cell) return null;
  const link = denialUxLink(cell.link);
  if (!link) return null;
  return Object.freeze({
    kind: cell.kind,
    code: cell.code,
    blocked_action: cell.blocked_action,
    reason: cell.reason,
    retained_access: RETAINED_ACCESS,
    safe_next_action: cell.safe_next_action,
    action_label: cell.action_label,
    link: cell.link,
    link_path: link,
  });
}

/** Frozen contract surface for presenters and parity tests. */
export const denialUxCatalog = Object.freeze({
  schema: DENIAL_UX_SCHEMA,
  role: "presenter_message_fixture",
  links: LINKS,
  link_ids: Object.freeze([...DENIAL_UX_CATALOG.link_ids]),
  always_reachable: RETAINED_ACCESS,
  actions: Object.freeze([...DENIAL_UX_CATALOG.actions]),
  error_registry: REGISTRY,
  grid_cells: GRID.length,
  rules: Object.freeze([...DENIAL_UX_CATALOG.rules]),
});
