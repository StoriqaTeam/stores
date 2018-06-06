use hyper::StatusCode;
use failure::Error;
use validator::ValidationErrors;

use stq_http::errors::Codeable;

#[derive(Debug, Fail)]
pub enum ControllerError {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Parse error")]
    Parse,
    #[fail(display = "Validation error: {}", _0)]
    Validate(ValidationErrors),
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
            ControllerError::Validate(_)=> StatusCode::BadRequest,
            ControllerError::Parse => StatusCode::UnprocessableEntity,
            ControllerError::Connection(_) | ControllerError::ElasticSearch(_) => StatusCode::InternalServerError,
            ControllerError::Forbidden => StatusCode::Forbidden,
        }
    }
}

