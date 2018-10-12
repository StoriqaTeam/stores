//! Error for S3 service

use futures::future::err;
use futures::Future;
use rusoto_s3::PutObjectError;

/// Error for S3 service
#[derive(Debug, Fail)]
pub enum S3Error {
    #[fail(display = "Access Error: {}", _0)]
    Access(String),
    #[fail(display = "Network Error: {}", _0)]
    Network(String),
    #[fail(display = "Unknown error: {}", _0)]
    Unknown(String),
}

impl<T: 'static> Into<Box<Future<Item = T, Error = S3Error>>> for S3Error {
    fn into(self) -> Box<Future<Item = T, Error = S3Error>> {
        Box::new(err::<T, _>(self))
    }
}

impl From<PutObjectError> for S3Error {
    fn from(e: PutObjectError) -> Self {
        match e {
            PutObjectError::HttpDispatch(err) => S3Error::Network(format!("{}", err)),
            PutObjectError::Credentials(err) => S3Error::Access(format!("{}", err)),
            PutObjectError::Validation(err) => S3Error::Access(format!("{}", err)),
            PutObjectError::Unknown(err) => S3Error::Unknown(format!("{}", err)),
        }
    }
}
