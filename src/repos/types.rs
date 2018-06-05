use futures::future::Future;
use failure::Error as FailureError;


/// Repos layer Future
pub type RepoFuture<T> = Box<Future<Item = T, Error = FailureError> + Send>;
pub type RepoResult<T> = Result<T, FailureError>;
