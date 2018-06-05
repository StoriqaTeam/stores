use hyper::StatusCode;
use failure::Error;
use validator::ValidationErrors;

use stq_http::errors::Codeable;

#[derive(Debug, Fail)]
pub enum ControllerError {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Parse error: {}", _0)]
    Parse(String),
    #[fail(display = "Bad request: {}", _0)]
    BadRequest(Error),
    #[fail(display = "Validation error: {}", _0)]
    Validate(ValidationErrors),
    #[fail(display = "Unprocessable entity: {}", _0)]
    UnprocessableEntity(Error),
    #[fail(display = "Internal server error: {}", _0)]
    InternalServerError(Error),
    #[fail(display = "Server is refusing to fullfil the reqeust")]
    Forbidden,
    #[fail(display = "Server is refusing to fullfil the reqeust: {}", _0)]
    Connection(Error),
    #[fail(display = "Server is refusing to fullfil the reqeust: {}", _0)]
    ElasticSearch(Error),
}

impl Codeable for ControllerError {
    fn code(&self) -> StatusCode {
       match *self {
            ControllerError::NotFound => StatusCode::NotFound,
            ControllerError::BadRequest(_) | ControllerError::Validate(_)=> StatusCode::BadRequest,
            ControllerError::Parse(_) | ControllerError::UnprocessableEntity(_) => StatusCode::UnprocessableEntity,
            ControllerError::InternalServerError(_) | ControllerError::Connection(_) | ControllerError::ElasticSearch(_) => StatusCode::InternalServerError,
            ControllerError::Forbidden => StatusCode::Forbidden,
        }
    }
}

