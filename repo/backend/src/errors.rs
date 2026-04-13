use actix_web::{HttpResponse, http::StatusCode};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<FieldError>,
}

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub body: ApiErrorBody,
}

impl ApiError {
    /// Creates a 400 Bad Request error with the given code and message.
    pub fn bad_request(code: &str, message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ApiErrorBody {
                code: code.to_string(),
                message: message.to_string(),
                details: vec![],
            },
        }
    }

    /// Creates a 401 Unauthorized error.
    pub fn unauthorized(message: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            body: ApiErrorBody {
                code: "UNAUTHORIZED".to_string(),
                message: message.to_string(),
                details: vec![],
            },
        }
    }

    /// Creates a 403 Forbidden error.
    pub fn forbidden(message: &str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            body: ApiErrorBody {
                code: "FORBIDDEN".to_string(),
                message: message.to_string(),
                details: vec![],
            },
        }
    }

    /// Creates a 404 Not Found error for the specified entity type.
    pub fn not_found(entity: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: ApiErrorBody {
                code: "NOT_FOUND".to_string(),
                message: format!("{} not found", entity),
                details: vec![],
            },
        }
    }

    /// Creates a 409 Conflict error.
    pub fn conflict(message: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: ApiErrorBody {
                code: "CONFLICT".to_string(),
                message: message.to_string(),
                details: vec![],
            },
        }
    }

    /// Creates a 422 Unprocessable Entity error with the given code and message.
    pub fn unprocessable(code: &str, message: &str) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            body: ApiErrorBody {
                code: code.to_string(),
                message: message.to_string(),
                details: vec![],
            },
        }
    }

    /// Creates a 422 Unprocessable Entity error with per-field validation details.
    pub fn unprocessable_fields(code: &str, message: &str, details: Vec<FieldError>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            body: ApiErrorBody {
                code: code.to_string(),
                message: message.to_string(),
                details,
            },
        }
    }

    /// Creates a 500 Internal Server Error.
    pub fn internal(message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ApiErrorBody {
                code: "INTERNAL_ERROR".to_string(),
                message: message.to_string(),
                details: vec![],
            },
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.status, self.body.code, self.body.message)
    }
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.status
    }

    fn error_response(&self) -> HttpResponse {
        tracing::error!(
            code = %self.body.code,
            message = %self.body.message,
            status = %self.status.as_u16(),
            "API error"
        );
        HttpResponse::build(self.status).json(&self.body)
    }
}

impl From<diesel::result::Error> for ApiError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => Self::not_found("Resource"),
            _ => {
                tracing::error!(error = %e, "Database error");
                Self::internal("Database error")
            }
        }
    }
}

impl From<r2d2::Error> for ApiError {
    fn from(e: r2d2::Error) -> Self {
        tracing::error!(error = %e, "Connection pool error");
        Self::internal("Service temporarily unavailable")
    }
}
