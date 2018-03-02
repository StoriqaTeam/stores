use hyper;
use serde_json;

use failure::Error;

use validator::ValidationErrors;

#[derive(Debug, Fail)]
pub enum ControllerError {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Parse error")]
    Parse(String),
    #[fail(display = "Bad request")]
    BadRequest(Error),
    #[fail(display = "Validation error: {}", _0)]
    Validate(ValidationErrors),
    #[fail(display = "Unprocessable entity")]
    UnprocessableEntity(Error),
    #[fail(display = "Internal server error")]
    InternalServerError(Error),
}

impl From<serde_json::error::Error> for ControllerError {
    fn from(e: serde_json::error::Error) -> Self {
        ControllerError::UnprocessableEntity(e.into())
    }
}

impl ControllerError {
    /// Converts `Error` to HTTP Status Code
    pub fn code(&self) -> hyper::StatusCode {
        use hyper::StatusCode;

        match *self {
            ControllerError::NotFound => StatusCode::NotFound,
            ControllerError::Parse(_) | ControllerError::BadRequest(_) | ControllerError::Validate(_) => StatusCode::BadRequest,
            ControllerError::UnprocessableEntity(_) => StatusCode::UnprocessableEntity,
            ControllerError::InternalServerError(_) => StatusCode::InternalServerError,
        }
    }

    /// Converts `Error` to string
    pub fn message(&self) -> String {
        match *self {
            ControllerError::NotFound => "Not found".to_string(),
            ControllerError::Parse(_) | ControllerError::BadRequest(_) => "Bad request".to_string(),
            ControllerError::Validate(ref valid_err) => match serde_json::to_string(valid_err) {
                Ok(res) => res,
                Err(_) => "Bad request".to_string(),
            },
            ControllerError::UnprocessableEntity(_) => "Unprocessable entity".to_string(),
            ControllerError::InternalServerError(_) => "Internal server error".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorMessage {
    pub code: u16,
    pub message: String,
}
