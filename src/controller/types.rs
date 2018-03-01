use futures::future::Future;
use super::error::ControllerError as Error;

pub type ControllerFuture = Box<Future<Item = String, Error = Error>>;
