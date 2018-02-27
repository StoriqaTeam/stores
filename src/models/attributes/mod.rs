//! Models contains all structures that are used in different
//! modules of the app

pub mod attribute;
pub mod attribute_product;
pub mod attribute_filter;

pub use self::attribute_product::*;
pub use self::attribute_filter::*;
pub use self::attribute::*;
