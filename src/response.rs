use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
pub struct AppResponse<T = (), M = ()> {
    // status code (enum)
    #[serde(skip)]
    status_code: StatusCode,
    // status code (u16)
    code: u16,
    // status message
    status: String,
    // error message
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    // data
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    // metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<M>,
}

impl<T, M> AppResponse<T, M> {
    pub fn new(data: T, metadata: M) -> Self {
        Self {
            status_code: StatusCode::OK,
            code: StatusCode::OK.as_u16(),
            status: StatusCode::OK
                .canonical_reason()
                .unwrap_or("Unknown")
                .to_string(),
            message: None,
            data: Some(data),
            metadata: Some(metadata),
        }
    }

    pub fn created() -> Self {
        Self {
            status_code: StatusCode::CREATED,
            code: StatusCode::CREATED.as_u16(),
            status: StatusCode::CREATED
                .canonical_reason()
                .unwrap_or("Unknown")
                .to_string(),
            message: None,
            data: None,
            metadata: None,
        }
    }

    pub fn with_data(data: T) -> Self {
        Self {
            status_code: StatusCode::OK,
            code: StatusCode::OK.as_u16(),
            status: StatusCode::OK
                .canonical_reason()
                .unwrap_or("Unknown")
                .to_string(),
            message: None,
            data: Some(data),
            metadata: None,
        }
    }
}

impl AppResponse {
    pub fn ok() -> Self {
        Self {
            status_code: StatusCode::OK,
            code: StatusCode::OK.as_u16(),
            status: StatusCode::OK
                .canonical_reason()
                .unwrap_or("Unknown")
                .to_string(),
            message: None,
            data: None,
            metadata: None,
        }
    }

    pub fn error(status_code: StatusCode, message: &str) -> Self {
        Self {
            status_code,
            code: status_code.as_u16(),
            status: status_code
                .canonical_reason()
                .unwrap_or("Unknown")
                .to_string(),
            message: Some(message.to_string()),
            data: None,
            metadata: None,
        }
    }
}

impl<T: Serialize, M: Serialize> IntoResponse for AppResponse<T, M> {
    fn into_response(self) -> Response {
        (self.status_code, Json(json!(self))).into_response()
    }
}
