//! Type aliases for service module

use failure;
use futures::future::Future;

/// Service layer Future
pub type ServiceFuture<T> = Box<Future<Item = T, Error = failure::Error>>;
