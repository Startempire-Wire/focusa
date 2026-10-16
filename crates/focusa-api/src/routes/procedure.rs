//! Stateless compiler transport; no ledger, dispatch, grant or runtime activation.
use axum::{
    Json, Router,
    extract::rejection::JsonRejection,
    http::{HeaderValue, StatusCode, header::CACHE_CONTROL},
    middleware,
    response::{IntoResponse, Response},
    routing::post,
};
use focusa_core::procedure_compiler::{
    Procedure, ProcedurePreview, compile_preview, verify_preview,
};
use focusa_core::tool_result::{FailureClass, ToolResultV1, ToolStatus};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifyPreviewRequest {
    pub definition: Procedure,
    pub preview: ProcedurePreview,
}

pub fn router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/v1/procedures/preview", post(preview))
        .route("/v1/procedures/verify-preview", post(verify))
        .layer(middleware::map_response(private_response))
}

async fn private_response(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

fn compiler_response<T: Serialize>(result: Result<T, String>) -> Response {
    match result {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(_) => validation_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            FailureClass::ValidationRejected,
        ),
    }
}

fn validation_response(status: StatusCode, class: FailureClass) -> Response {
    (status, Json(ToolResultV1::failure(ToolStatus::ValidationRejected, class,
        "Supply supported exact procedure/preview JSON; integrity never grants execution authority")))
        .into_response()
}

fn rejected_json(rejection: JsonRejection) -> Response {
    let status = rejection.status();
    let class = if status == StatusCode::PAYLOAD_TOO_LARGE {
        FailureClass::ResourceExhausted
    } else {
        FailureClass::SchemaInvalid
    };
    validation_response(status, class)
}

async fn preview(input: Result<Json<Procedure>, JsonRejection>) -> Response {
    match input {
        Ok(Json(definition)) => compiler_response(compile_preview(&definition)),
        Err(error) => rejected_json(error),
    }
}

async fn verify(input: Result<Json<VerifyPreviewRequest>, JsonRejection>) -> Response {
    match input {
        Ok(Json(request)) => {
            compiler_response(verify_preview(&request.definition, &request.preview))
        }
        Err(error) => rejected_json(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    fn source() -> Value {
        json!({"schema":"focusa.procedure.v1","procedure_id":"http-preview","version":1,
            "steps":[{"step_id":"transfer","kind":"api","effect_class":"financial",
            "require_confirmation":false,"rollback_step":"reverse","command":"provider.transfer"}]})
    }

    async fn request(path: &str, body: Value) -> (StatusCode, Value) {
        let response = router::<()>()
            .oneshot(
                Request::post(path)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.headers()[CACHE_CONTROL], "no-store");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1 << 20).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn preview_and_verification_share_core_contract_without_authority() {
        let definition = source();
        let (status, preview) = request("/v1/procedures/preview", definition.clone()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(preview["schema"], "focusa.procedure_preview.v1");
        assert_eq!(preview["canonical"], false);
        assert_eq!(preview["execution_authorized"], false);
        assert_eq!(
            preview["plan"]["plan_steps"][0]["confirmation_required"],
            true
        );
        let (status, result) = request(
            "/v1/procedures/verify-preview",
            json!({"definition":definition,"preview":preview}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(result["verified"], true);
        assert_eq!(result["canonical"], false);
        assert_eq!(result["execution_authorized"], false);
    }

    #[tokio::test]
    async fn changed_authority_and_safety_projections_are_rejected() {
        let definition = source();
        let (_, baseline) = request("/v1/procedures/preview", definition.clone()).await;
        for authority in [true, false] {
            let mut preview = baseline.clone();
            if authority {
                preview["execution_authorized"] = json!(true);
            } else {
                preview["plan"]["plan_steps"][0]["confirmation_required"] = json!(false);
            }
            let (status, error) = request(
                "/v1/procedures/verify-preview",
                json!({"definition":definition,"preview":preview}),
            )
            .await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(error["status"], "validation_rejected");
            assert_eq!(error["schema"], "focusa.tool_result.v1");
        }
    }

    #[tokio::test]
    async fn unsupported_body_meaning_fails_closed_with_private_caching() {
        let mut definition = source();
        definition["unregistered_policy_override"] = json!(true);
        let response = router::<()>()
            .oneshot(
                Request::post("/v1/procedures/preview")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&definition).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response.headers()[CACHE_CONTROL], "no-store");
    }

    #[tokio::test]
    async fn legacy_preview_cannot_qualify_through_http() {
        let definition = source();
        let (_, mut preview) = request("/v1/procedures/preview", definition.clone()).await;
        preview["digest_schema"] = json!("focusa.procedure-plan.digest.v1");
        let (status, _) = request(
            "/v1/procedures/verify-preview",
            json!({"definition":definition,"preview":preview}),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn unsupported_source_returns_bounded_validation_error() {
        let mut definition = source();
        definition["schema"] = json!("unsupported-private-source-value");
        let (status, error) = request("/v1/procedures/preview", definition).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(
            !error
                .to_string()
                .contains("unsupported-private-source-value")
        );
    }
}
