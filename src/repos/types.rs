use futures::future::Future;
use super::error::RepoError;

/// Repos layer Future
pub type RepoFuture<T> = Box<Future<Item = T, Error = RepoError> + Send>;
pub type RepoResult<T> = Result<T, RepoError>;
