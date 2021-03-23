use std::error::Error;

use actix_web::dev::HttpResponseBuilder;
use actix_web::http::StatusCode;
use actix_web::{error::ResponseError, HttpResponse};
use serde::Serialize;
use validator::ValidationErrors;

#[derive(Debug)]
pub enum AppError {
    Initialization(String),
    Validation(ValidationErrors),
    NotFound,
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AppError::*;
        match self {
            Initialization(message) => write!(f, "Failed to initialize app: {}", message),
            Validation(message) => write!(f, "Invalid request: {}", message),
            NotFound => write!(f, "Resource not found"),
            Internal(message) => write!(f, "Internal server error: {}", message),
        }
    }
}

impl Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::Internal(error.to_string())
    }
}

impl From<java_properties::PropertiesError> for AppError {
    fn from(error: java_properties::PropertiesError) -> Self {
        AppError::Internal(error.to_string())
    }
}

impl From<mongodb::error::Error> for AppError {
    fn from(error: mongodb::error::Error) -> Self {
        AppError::Internal(error.to_string())
    }
}

impl From<mongodb::bson::de::Error> for AppError {
    fn from(error: mongodb::bson::de::Error) -> Self {
        AppError::Internal(error.to_string())
    }
}

impl From<mongodb::bson::ser::Error> for AppError {
    fn from(error: mongodb::bson::ser::Error) -> Self {
        AppError::Internal(error.to_string())
    }
}

impl From<elasticsearch::Error> for AppError {
    fn from(error: elasticsearch::Error) -> Self {
        AppError::Internal(error.to_string())
    }
}

impl From<ValidationErrors> for AppError {
    fn from(errors: ValidationErrors) -> Self {
        AppError::Validation(errors)
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        use AppError::*;
        match self {
            NotFound => StatusCode::NOT_FOUND,
            Validation(_) => StatusCode::BAD_REQUEST,
            Initialization(_) | Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let mut errors = vec![];

        match self {
            AppError::Validation(v_errors) => {
                v_errors
                    .field_errors()
                    .iter()
                    .for_each(|(field, v_errors)| {
                        v_errors.iter().for_each(|err| {
                            let message = match err.message.as_ref() {
                                Some(m) => {
                                    format!("Field '{}' is invalid: {}", field, m.as_ref())
                                }
                                None => format!("Field '{}' is invalid", field),
                            };

                            errors.push(ErrorDetail::new(message));
                        })
                    });
            }
            _ => errors.push(ErrorDetail::new(self.to_string())),
        };

        HttpResponseBuilder::new(self.status_code()).json(errors)
    }
}

#[derive(Serialize, Debug)]
struct ErrorDetail {
    message: String,
}

impl ErrorDetail {
    fn new(message: String) -> Self {
        Self { message }
    }
}
