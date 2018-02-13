use hyper;
use serde_json;

use services::error::Error as ServiceError;

#[derive(Debug)]
pub enum Error {
    NotFound,
    BadRequest(String),
    UnprocessableEntity(String),
    InternalServerError(String),
    UnAuthorized(String),
}

impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Self {
        Error::UnprocessableEntity(format!("{}", e).to_string())
    }
}

impl From<ServiceError> for Error {
    fn from(e: ServiceError) -> Self {
        match e {
            ServiceError::NotFound => Error::NotFound,
            ServiceError::Rollback => Error::BadRequest("Transaction rollback".to_string()),
            ServiceError::Validate(msg) => {
                Error::BadRequest(serde_json::to_string(&msg).unwrap_or("Unable to serialize validation errors".to_string()))
            }
            ServiceError::Parse(msg) => Error::UnprocessableEntity(format!("Parse error: {}", msg)),
            ServiceError::Database(msg) => Error::InternalServerError(format!("Database error: {}", msg)),
            ServiceError::HttpClient(msg) => Error::InternalServerError(format!("Http Client error: {}", msg)),
            ServiceError::UnAuthorized(msg) => Error::UnAuthorized(msg),
            ServiceError::Unknown(msg) => Error::InternalServerError(format!("Unknown: {}", msg)),
        }
    }
}

impl Error {
    /// Converts `Error` to HTTP Status Code
    pub fn code(&self) -> hyper::StatusCode {
        use super::error::Error::*;
        use hyper::StatusCode;

        match self {
            &NotFound => StatusCode::NotFound,
            &BadRequest(_) => StatusCode::BadRequest,
            &UnprocessableEntity(_) => StatusCode::UnprocessableEntity,
            &InternalServerError(_) => StatusCode::InternalServerError,
            &UnAuthorized(_) => StatusCode::Unauthorized,
        }
    }

    /// Converts `Error` to string
    pub fn message(&self) -> String {
        use super::error::Error::*;

        match self {
            &NotFound => "Not found".to_string(),
            &BadRequest(ref msg) => msg.to_string(),
            &UnprocessableEntity(ref msg) => msg.to_string(),
            &InternalServerError(ref msg) => msg.to_string(),
            &UnAuthorized(ref msg) => msg.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::Error;
    use hyper::StatusCode;

    #[test]
    fn error_to_code_test() {
        let mut error = Error::NotFound.code();
        assert_eq!(error, StatusCode::NotFound);
        error = Error::BadRequest("bad".to_string()).code();
        assert_eq!(error, StatusCode::BadRequest);
        error = Error::UnprocessableEntity("bad".to_string()).code();
        assert_eq!(error, StatusCode::UnprocessableEntity);
        error = Error::InternalServerError("bad".to_string()).code();
        assert_eq!(error, StatusCode::InternalServerError);
    }

    #[test]
    fn error_to_message_test() {
        let mut error = Error::NotFound.message();
        assert_eq!(error, "Not found".to_string());
        error = Error::BadRequest("bad".to_string()).message();
        assert_eq!(error, "bad".to_string());
        error = Error::UnprocessableEntity("bad".to_string()).message();
        assert_eq!(error, "bad".to_string());
        error = Error::InternalServerError("bad".to_string()).message();
        assert_eq!(error, "bad".to_string());
    }
}
