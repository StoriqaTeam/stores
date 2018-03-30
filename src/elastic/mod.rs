//! Elastic search modules
pub mod stores;
pub mod products;

pub use self::products::*;
pub use self::stores::*;

use std::fmt::Debug;

pub fn log_elastic_req<T: Debug>(item: &T) {
    debug!("Searching in elastic {:?}.", item);
}

pub fn log_elastic_resp<T: Debug>(item: &T) {
    debug!("Result of searching in elastic {:?}.", item)
}
