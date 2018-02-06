use validator::ValidationErrors;
use diesel::result::Error as DieselError;

use ::repos::error::Error as RepoError;

/// Service layer Error
#[derive(Debug)]
pub enum Error {
    NotFound,
    Rollback,
    Validate(ValidationErrors),
    Parse(String),
    Database(String),
    HttpClient(String),
    UnAuthorized(String),
    Unknown(String)
}

impl From<RepoError> for Error {
    fn from(err: RepoError) -> Self {
        match err {
            RepoError::NotFound => Error::NotFound,
            RepoError::Rollback => Error::Rollback,
            RepoError::ContstaintViolation(msg) => Error::Database(format!("Constraint violation: {}", msg)),
            RepoError::MismatchedType(msg) => Error::Database(format!("Mismatched type: {}", msg)),
            RepoError::Connection(msg) => Error::Database(format!("Connection error: {}", msg)),
            RepoError::UnAuthorized(res, act) => Error::UnAuthorized(format!("Unauthorized access: Resource: {}, Action: {}", res, act)),
            RepoError::Unknown(msg) => Error::Database(format!("Unknown: {}", msg)),
        }
    }
}

impl From<DieselError> for Error {
    fn from(err: DieselError) -> Self {
        match err {
            DieselError::NotFound => Error::NotFound,
            _ => Error::Database("Database error".into()),
        }
    }
}