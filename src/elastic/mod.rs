//! Elastic search modules
pub mod products;
pub mod stores;

pub use self::products::*;
pub use self::stores::*;

use std::fmt::Debug;

pub fn log_elastic_req<T: Debug>(item: &T) {
    debug!("Searching in elastic {:?}.", item);
}

pub fn log_elastic_resp<T: Debug>(item: &T) {
    trace!("Result of searching in elastic {:?}.", item)
}
