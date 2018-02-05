//! Models contains all structures that are used in different
//! modules of the app

pub mod store;
pub mod product;
pub mod language;

pub use self::store::{NewStore, Store, UpdateStore};
pub use self::product::{NewProduct, Product, UpdateProduct};
pub use self::language::Language;