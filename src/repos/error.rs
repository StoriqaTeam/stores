use diesel::result::Error as DieselError;
use models::authorization::*;
use http::client::Error as HttpError;

/// Repos layer Error
#[derive(Debug)]
pub enum Error {
    NotFound,
    Rollback,
    ContstaintViolation(String),
    Unauthorized(Resource, Action),
    MismatchedType(String),
    Connection(String),
    Unknown(String),
}

impl From<DieselError> for Error {
    fn from(err: DieselError) -> Self {
        match err {
            DieselError::InvalidCString(e) => Error::Unknown(format!("{}", e)),
            DieselError::DatabaseError(kind, info) => Error::ContstaintViolation(format!("{:?}: {:?}", kind, info)),
            DieselError::NotFound => Error::NotFound,
            DieselError::QueryBuilderError(e) => Error::Unknown(format!("{}", e)),
            DieselError::SerializationError(e) => Error::MismatchedType(format!("{}", e)),
            DieselError::DeserializationError(e) => Error::MismatchedType(format!("{}", e)),
            DieselError::RollbackTransaction => Error::Rollback,
            _ => Error::Unknown("Unknown diesel error".to_string()),
        }
    }
}


impl From<HttpError> for Error {
    fn from(err: HttpError) -> Self {
        match err {
            HttpError::Api(_, _) => Error::Connection(format!("Cant connect to elastic.")),
            HttpError::Network(_) => Error::Connection(format!("Cant connect to elastic.")),
            HttpError::Parse(_) => Error::Connection(format!("Cant connect to elastic.")),
            HttpError::Unknown(_) => Error::Connection(format!("Cant connect to elastic.")),
        }
    }
}