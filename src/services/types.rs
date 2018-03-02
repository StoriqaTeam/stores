use futures::future::Future;

use super::error::ServiceError as Error;

/// Service layer Future
pub type ServiceFuture<T> = Box<Future<Item = T, Error = Error>>;
