use serde::Serialize;
use std::collections::VecDeque;
use std::fmt;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use url::Url;

pub const DEEP_LINK_EVENT: &str = "focusa://deep-link-intent";
pub const DEEP_LINK_REJECTED_EVENT: &str = "focusa://deep-link-rejected";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FocusaDeepLinkIntent {
    pub schema: &'static str,
    pub route: FocusaDeepLinkRoute,
    pub target_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub governed_connect_payload: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FocusaDeepLinkRoute {
    Connect,
    Mission,
    Card,
    Workpoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FocusaDeepLinkRejection {
    pub schema: &'static str,
    pub reason_code: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    WrongScheme,
    AuthorityNotAllowed,
    UnsupportedRoute,
    InvalidTarget,
    UnexpectedParameters,
    MissingGovernedPayload,
    InvalidGovernedPayload,
}

impl ParseError {
    pub fn reason_code(self) -> &'static str {
        match self {
            Self::WrongScheme => "wrong_scheme",
            Self::AuthorityNotAllowed => "authority_not_allowed",
            Self::UnsupportedRoute => "unsupported_route",
            Self::InvalidTarget => "invalid_target",
            Self::UnexpectedParameters => "unexpected_parameters",
            Self::MissingGovernedPayload => "missing_governed_payload",
            Self::InvalidGovernedPayload => "invalid_governed_payload",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delivery {
    Queued,
    Warm(FocusaDeepLinkIntent),
}

#[derive(Default)]
pub struct DeepLinkRuntimeState {
    frontend_ready: AtomicBool,
    pending: Mutex<VecDeque<FocusaDeepLinkIntent>>,
}

impl DeepLinkRuntimeState {
    pub fn accept(&self, intent: FocusaDeepLinkIntent) -> Delivery {
        if self.frontend_ready.load(Ordering::Acquire) {
            return Delivery::Warm(intent);
        }
        self.pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push_back(intent);
        Delivery::Queued
    }

    pub fn take_pending_and_mark_ready(&self) -> Vec<FocusaDeepLinkIntent> {
        let pending = self
            .pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .drain(..)
            .collect();
        self.frontend_ready.store(true, Ordering::Release);
        pending
    }
}

pub fn parse_focusa_deep_link(url: &Url) -> Result<FocusaDeepLinkIntent, ParseError> {
    if url.scheme() != "focusa" {
        return Err(ParseError::WrongScheme);
    }
    if !url.username().is_empty() || url.password().is_some() || url.port().is_some() {
        return Err(ParseError::AuthorityNotAllowed);
    }
    if url.fragment().is_some() {
        return Err(ParseError::UnexpectedParameters);
    }

    let route = match url.host_str() {
        Some("connect") => FocusaDeepLinkRoute::Connect,
        Some("mission") => FocusaDeepLinkRoute::Mission,
        Some("card") => FocusaDeepLinkRoute::Card,
        Some("workpoint") => FocusaDeepLinkRoute::Workpoint,
        _ => return Err(ParseError::UnsupportedRoute),
    };

    if route == FocusaDeepLinkRoute::Connect {
        return parse_connect(url);
    }
    parse_target_route(url, route)
}

fn parse_connect(url: &Url) -> Result<FocusaDeepLinkIntent, ParseError> {
    if !matches!(url.path(), "" | "/") {
        return Err(ParseError::InvalidTarget);
    }
    let pairs: Vec<_> = url.query_pairs().collect();
    if pairs.len() != 1 || pairs[0].0 != "payload" {
        return Err(if pairs.is_empty() {
            ParseError::MissingGovernedPayload
        } else {
            ParseError::UnexpectedParameters
        });
    }
    let payload = pairs[0].1.as_ref();
    if payload.is_empty()
        || payload.len() > 4096
        || payload.chars().any(|character| character.is_control())
    {
        return Err(ParseError::InvalidGovernedPayload);
    }
    Ok(FocusaDeepLinkIntent {
        schema: "focusa.deep_link_intent.v1",
        route: FocusaDeepLinkRoute::Connect,
        target_ref: None,
        governed_connect_payload: Some(payload.to_owned()),
    })
}

fn parse_target_route(
    url: &Url,
    route: FocusaDeepLinkRoute,
) -> Result<FocusaDeepLinkIntent, ParseError> {
    if url.query().is_some() {
        return Err(ParseError::UnexpectedParameters);
    }
    let mut segments = url.path().trim_start_matches('/').split('/');
    let target = segments.next().unwrap_or_default();
    if segments.next().is_some() || !is_opaque_ref(target) {
        return Err(ParseError::InvalidTarget);
    }
    Ok(FocusaDeepLinkIntent {
        schema: "focusa.deep_link_intent.v1",
        route,
        target_ref: Some(target.to_owned()),
        governed_connect_payload: None,
    })
}

fn is_opaque_ref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(value: &str) -> Result<FocusaDeepLinkIntent, ParseError> {
        parse_focusa_deep_link(&Url::parse(value).expect("test URL parses"))
    }

    #[test]
    fn parses_allowlisted_target_routes() {
        for (url, route, target) in [
            (
                "focusa://mission/project:alpha",
                FocusaDeepLinkRoute::Mission,
                "project:alpha",
            ),
            (
                "focusa://card/card_001",
                FocusaDeepLinkRoute::Card,
                "card_001",
            ),
            (
                "focusa://workpoint/workpoint-001",
                FocusaDeepLinkRoute::Workpoint,
                "workpoint-001",
            ),
        ] {
            let intent = parse(url).expect("allowlisted route");
            assert_eq!(intent.route, route);
            assert_eq!(intent.target_ref.as_deref(), Some(target));
            assert!(intent.governed_connect_payload.is_none());
        }
    }

    #[test]
    fn parses_connect_without_interpreting_governed_payload() {
        let intent = parse(
            "focusa://connect?payload=https%3A%2F%2Fpair.example%2Fpair%2Fdevice%23secret%3Dopaque",
        )
        .expect("governed connect route");
        assert_eq!(intent.route, FocusaDeepLinkRoute::Connect);
        assert_eq!(
            intent.governed_connect_payload.as_deref(),
            Some("https://pair.example/pair/device#secret=opaque")
        );
        assert!(intent.target_ref.is_none());
    }

    #[test]
    fn rejects_non_allowlisted_or_ambiguous_links() {
        for url in [
            "https://mission/project:alpha",
            "focusa://unknown/project:alpha",
            "focusa://user@mission/project:alpha",
            "focusa://mission/project:alpha/extra",
            "focusa://mission/%2Ftmp%2Fsecret",
            "focusa://mission/project:alpha?token=secret",
            "focusa://mission/project:alpha#secret",
            "focusa://connect",
            "focusa://connect?payload=",
            "focusa://connect?payload=one&payload=two",
            "focusa://connect?payload=opaque&token=secret",
        ] {
            assert!(parse(url).is_err(), "{url} must fail closed");
        }
    }

    #[test]
    fn queues_cold_activation_then_delivers_warm_activation() {
        let state = DeepLinkRuntimeState::default();
        let cold = parse("focusa://mission/project:alpha").expect("cold intent");
        assert_eq!(state.accept(cold.clone()), Delivery::Queued);
        assert_eq!(state.take_pending_and_mark_ready(), vec![cold]);

        let warm = parse("focusa://workpoint/workpoint-001").expect("warm intent");
        assert_eq!(state.accept(warm.clone()), Delivery::Warm(warm));
        assert!(state.take_pending_and_mark_ready().is_empty());
    }
}
