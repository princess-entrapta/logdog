use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug)]
pub enum AppError {
    DBError(sqlx::error::Error),
}

impl From<sqlx::error::Error> for AppError {
    fn from(error: sqlx::error::Error) -> Self {
        Self::DBError(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::DBError(err) => tracing::error!("{}", err.to_string()),
        }
        (StatusCode::INTERNAL_SERVER_ERROR, "Error".to_owned()).into_response()
    }
}
