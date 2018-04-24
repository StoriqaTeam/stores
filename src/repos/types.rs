use super::error::RepoError;
use futures::future::Future;

/// Repos layer Future
pub type RepoFuture<T> = Box<Future<Item = T, Error = RepoError> + Send>;
pub type RepoResult<T> = Result<T, RepoError>;
