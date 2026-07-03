//! `GET /llms.txt` — agent-readable product context per Spec 109 AX-004.
//!
//! The transcript evaluator (2026-07-03) spent the entire session
//! reverse-engineering what focusa is FOR. This endpoint exists so an
//! AI agent can read a single, structured text document explaining the
//! product, the core concepts, the tool surface, and the next commands
//! to run — in one request.
//!
//! The document is plain text, not JSON, intentionally. LLM agents are
//! best primed with a single readable document they can quote back when
//! reasoning about focusa. JSON forms are available separately via
//! `/v1/agent/capabilities` (AX-001) and `/v1/agent/tools` (AX-002).
//!
//! Stable content rules: this endpoint returns the same body for a given
//! daemon version. Don't add volatile data (timestamps, run counts).

use axum::Router;
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;

pub fn router() -> Router {
    Router::new().route("/llms.txt", get(llms_txt))
}

async fn llms_txt() -> impl IntoResponse {
    let body = include_str!("../../../../docs/llms.txt");
    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (
                header::CACHE_CONTROL,
                "public, max-age=300, stale-while-revalidate=3600",
            ),
        ],
        body,
    )
}
