//! Repos is a module responsible for interacting with postgres db
pub mod stores;
pub mod products;
pub mod attributes;

pub use self::products::*;
pub use self::stores::*;
pub use self::attributes::*;
