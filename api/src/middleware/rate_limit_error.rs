use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

pub fn rete_limit_error_response(err: tower_governor::GovernorError) -> Response {
    match err {
        tower_governor::GovernorError::TooManyRequests { headers, .. } => {
            let retry_after = headers
                .as_ref()
                .and_then(|h| h.get("retry-after"))
                .and_then(|v| v.to_str().ok())
                .unwrap_or("1");
            let body = json!({
              "error": "too_many_requests",
              "message": "요청이 너무 많습니다. 잠시 후 다시 시도해주세요.",
              "retry_after_seconds": retry_after,
            });

            let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response();
            if let Ok(val) = retry_after.parse::<u64>() {
                response.headers_mut().insert(
                    "retry-after",
                    axum::http::HeaderValue::from_str(val.to_string().as_str()).unwrap(),
                );
            }

            response
        }

        tower_governor::GovernorError::UnableToExtractKey => {
            let body = json!({
              "error": "unable_to_extract_key",
              "message": "요청자를 식별할 수 없습니다."
            });
            (StatusCode::FORBIDDEN, Json(body)).into_response()
        }
        _ => {
            let body = json!({
              "error": "rate_limit_error",
              "message": "Rate Limit 처리 중 오류가 발생했습니다.",
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
        }
    }
}
