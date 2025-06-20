use crate::response::AppResponse;
use salvo::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct AppError {
    #[serde(skip)]
    status_code: StatusCode,
    message: String,
}

impl AppError {
    pub fn new(code: StatusCode, message: String) -> Self {
        Self {
            status_code: code,
            message: message.to_string(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        let message = message.into();
        Self::new(StatusCode::FORBIDDEN, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        let message = message.into();
        Self::new(StatusCode::CONFLICT, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        let message = message.into();
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    pub fn unauth(message: impl Into<String>) -> Self {
        let message = message.into();
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        let message = message.into();
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn code(&self) -> StatusCode {
        self.status_code
    }
}

impl From<AppError> for AppResponse {
    fn from(value: AppError) -> Self {
        AppResponse::error(value.status_code, &value.message)
    }
}

#[async_trait]
impl Writer for AppError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.render(AppResponse::from(self))
    }
}

pub type AppResult<T = ()> = Result<T, AppError>;

pub type AppResponseResult<T = (), M = ()> = AppResult<AppResponse<T, M>>;
