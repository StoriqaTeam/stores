use diesel::result::Error as DieselError;
use models::authorization::*;
use stq_http::client::Error as HttpError;

use failure::Error;

#[derive(Debug, Fail)]
pub enum RepoError {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Rollback")]
    Rollback,
    #[fail(display = "Unauthorized: {} {}", _0, _1)]
    Unauthorized(Resource, Action),
    #[fail(display = "Constraint violation: {}", _0)]
    ContstaintViolation(Error),
    #[fail(display = "Mismatched type: {}", _0)]
    MismatchedType(Error),
    #[fail(display = "Connection: {}", _0)]
    Connection(Error),
    #[fail(display = "Unknown: {}", _0)]
    Unknown(Error),
}

impl From<DieselError> for RepoError {
    fn from(err: DieselError) -> Self {
        error!("Diesel error occured: '{}'.", err);
        match err {
            DieselError::InvalidCString(e) => RepoError::Unknown(DieselError::InvalidCString(e).into()),
            DieselError::DatabaseError(kind, info) => RepoError::ContstaintViolation(DieselError::DatabaseError(kind, info).into()),
            DieselError::NotFound => RepoError::NotFound,
            DieselError::QueryBuilderError(e) => RepoError::Unknown(DieselError::QueryBuilderError(e).into()),
            DieselError::SerializationError(e) => RepoError::MismatchedType(DieselError::SerializationError(e).into()),
            DieselError::DeserializationError(e) => RepoError::MismatchedType(DieselError::DeserializationError(e).into()),
            DieselError::RollbackTransaction => RepoError::Rollback,
            _ => RepoError::Unknown(DieselError::__Nonexhaustive.into()),
        }
    }
}

impl From<HttpError> for RepoError {
    fn from(err: HttpError) -> Self {
        error!("Http error occured: '{}'.", err);
        RepoError::Connection(format_err!("Http error. {}", err))
    }
}
