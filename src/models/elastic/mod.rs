//! Elastic search models
use std::fmt;

pub mod shards;
pub mod index_response;
pub mod search_response;

pub use self::shards::*;
pub use self::index_response::*;
pub use self::search_response::*;

pub enum ElasticIndex {
    Store,
    Product,
}

impl fmt::Display for ElasticIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ElasticIndex::Store => write!(f, "stores"),
            ElasticIndex::Product => write!(f, "products"),
        }
    }
}
