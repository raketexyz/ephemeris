use std::collections::HashMap;

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use diesel::result::Error as DieselError;
use validator::ValidationErrors;
use serde_json::to_string;

#[derive(Debug)]
pub struct ApiError {
    pub status_code: u16,
    pub message: String,
}

impl ApiError {
    pub fn new(status_code: u16, message: String) -> Self {
        Self {
            status_code,
            message,
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.status_code, self.message)
    }
}

impl From<DieselError> for ApiError {
    fn from(e: DieselError) -> Self {
        match e {
            DieselError::NotFound => ApiError::new(404, "Record not found".to_string()),
            e => ApiError::new(500, format!("Diesel error: {}", e)),
        }
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(e: ValidationErrors) -> Self {
        let mut errors = HashMap::new();
        for (a, b) in e.into_errors() {
            errors.insert(a, b);
        }
        match to_string(&errors) {
            Ok(s) => Self::new(400, s),
            Err(s) => Self::new(500, s.to_string()),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let status_code = match StatusCode::from_u16(self.status_code) {
            Ok(status_code) => status_code,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let message = if status_code.as_u16() < 500 {
            self.message.to_owned()
        } else {
            error!("{}", self);
            "Internal Server Error".to_string()
        };

        HttpResponse::build(status_code).body(message)
    }
}
