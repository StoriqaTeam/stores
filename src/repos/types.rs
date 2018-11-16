use failure::Error as FailureError;
use futures::future::Future;
use models::authorization::*;
use repos::legacy_acl::Acl;

/// Repos layer Future
pub type RepoFuture<T> = Box<Future<Item = T, Error = FailureError> + Send>;
pub type RepoResult<T> = Result<T, FailureError>;
pub type RepoAcl<T> = Acl<Resource, Action, Scope, Rule, FailureError, T>;
