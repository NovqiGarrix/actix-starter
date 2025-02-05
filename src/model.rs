use super::config::AppConfig;
use crate::libs::redis_client::RedisClient;
use actix_web::{
    http::{header, StatusCode},
    HttpResponse,
};
use base64::DecodeError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct Error {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl Error {
    pub fn for_validation(field: String, message: String) -> Self {
        Error {
            error: None,
            field: Some(field),
            message: Some(message),
        }
    }

    pub fn common_error(error: &str) -> Self {
        Error {
            error: Some(error.to_owned()),
            field: None,
            message: None,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub redis_client: RedisClient,
    pub config: AppConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorResponseType {
    BadRequest,
    InternalServerError,
    Unauthorized,
    Forbidden,
    NotFound,
}

impl From<ErrorResponseType> for StatusCode {
    fn from(value: ErrorResponseType) -> Self {
        match value {
            ErrorResponseType::BadRequest => StatusCode::BAD_REQUEST,
            ErrorResponseType::NotFound => StatusCode::NOT_FOUND,
            ErrorResponseType::Forbidden => StatusCode::FORBIDDEN,
            ErrorResponseType::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorResponseType::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ServiceException {
    pub code: u16,
    pub status: String,
    pub errors: Vec<Error>,
    #[serde(skip_serializing)]
    pub response_type: ErrorResponseType,
}

// Only use in Integration Test
#[derive(Serialize, Debug, Deserialize)]
pub struct TestServiceException {
    pub code: u16,
    pub status: String,
    pub errors: Vec<Error>,
}

impl ServiceException {
    pub fn internal_server_error() -> Self {
        ServiceException {
            code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            status: String::from("Internal Server Error"),
            errors: vec![Error::common_error("Internal Server Error")],
            response_type: ErrorResponseType::InternalServerError,
        }
    }

    pub fn for_validation(errors: Vec<Error>) -> Self {
        ServiceException {
            code: StatusCode::BAD_REQUEST.as_u16(),
            errors,
            response_type: ErrorResponseType::BadRequest,
            status: String::from("Bad Request"),
        }
    }

    pub fn common_error(error: &str, error_type: ErrorResponseType) -> Self {
        match error_type {
            ErrorResponseType::Unauthorized => ServiceException {
                code: StatusCode::UNAUTHORIZED.as_u16(),
                status: "Unauthorized".to_owned(),
                errors: vec![Error::common_error(error)],
                response_type: error_type,
            },

            ErrorResponseType::BadRequest => ServiceException {
                code: StatusCode::BAD_REQUEST.as_u16(),
                status: "Bad Request".to_owned(),
                errors: vec![Error::common_error(error)],
                response_type: error_type,
            },

            ErrorResponseType::NotFound => ServiceException {
                code: StatusCode::NOT_FOUND.as_u16(),
                status: "Not Found".to_owned(),
                errors: vec![Error::common_error(error)],
                response_type: error_type,
            },

            ErrorResponseType::Forbidden => ServiceException {
                code: StatusCode::FORBIDDEN.as_u16(),
                status: "Forbidden".to_owned(),
                errors: vec![Error::common_error(error)],
                response_type: error_type,
            },

            _ => ServiceException {
                code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                status: String::from("Internal Server Error"),
                errors: vec![Error::common_error("Internal Server Error")],
                response_type: ErrorResponseType::InternalServerError,
            },
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl std::fmt::Display for ServiceException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.errors)
    }
}

impl actix_web::error::ResponseError for ServiceException {
    fn status_code(&self) -> StatusCode {
        self.response_type.clone().into()
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(header::ContentType::json())
            .json(self)
    }
}

impl From<ServiceException> for HttpResponse {
    fn from(service_exception: ServiceException) -> Self {
        HttpResponse::build(service_exception.response_type.clone().into())
            .insert_header(header::ContentType::json())
            .json(service_exception)
    }
}

impl From<DecodeError> for ServiceException {
    fn from(e: DecodeError) -> Self {
        tracing::error!("Failed to decode base64: {:?}", e);
        ServiceException::common_error(
            "You inputted an invalid Base64 encoding",
            ErrorResponseType::BadRequest,
        )
    }
}

// impl From<uuid::Error> for ServiceException {
//     fn from(value: uuid::Error) -> Self {
//         tracing::error!("UUID Error: {}", value);
//         ServiceException::common_error("Invalid UUID", ErrorResponseType::BadRequest)
//     }
// }
