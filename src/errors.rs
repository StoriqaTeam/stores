use diesel::result::{DatabaseErrorKind, Error as DieselError};
use failure::Fail;
use hyper::StatusCode;
use serde_json;
use validator::{ValidationError, ValidationErrors};

use stq_http::errors::{Codeable, PayloadCarrier};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Parse error")]
    Parse,
    #[fail(display = "Validation error")]
    Validate(ValidationErrors),
    #[fail(display = "Server is refusing to fullfil the request")]
    Forbidden,
    #[fail(display = "R2D2 connection error")]
    Connection,
    #[fail(display = "Elastic search error")]
    ElasticSearch,
    #[fail(display = "service error - internal")]
    Internal,
}

impl Codeable for Error {
    fn code(&self) -> StatusCode {
        match *self {
            Error::NotFound => StatusCode::NotFound,
            Error::Validate(_) => StatusCode::BadRequest,
            Error::Parse => StatusCode::UnprocessableEntity,
            Error::Connection | Error::ElasticSearch | Error::Internal => StatusCode::InternalServerError,
            Error::Forbidden => StatusCode::Forbidden,
        }
    }
}

impl PayloadCarrier for Error {
    fn payload(&self) -> Option<serde_json::Value> {
        match *self {
            Error::Validate(ref e) => serde_json::to_value(e.clone()).ok(),
            _ => None,
        }
    }
}

impl<'a> From<DieselError> for Error {
    fn from(e: DieselError) -> Self {
        match e {
            DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, ref info) => {
                let mut errors = ValidationErrors::new();
                let mut error = ValidationError::new("not_unique");
                let message: &str = info.message();
                error.add_param("message".into(), &message);
                errors.add("repo", error);
                Error::Validate(errors)
            }
            DieselError::NotFound => Error::NotFound,
            _ => Error::Internal,
        }
    }
}
