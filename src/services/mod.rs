//! Services is a core layer for the app business logic like
//! validation, authorization, etc.

pub mod attribute_values;
pub mod attributes;
pub mod base_products;
pub mod categories;
pub mod coupons;
pub mod currency_exchange;
pub mod custom_attributes;
pub mod moderator_comments;
pub mod products;
pub mod stores;
pub mod types;
pub mod user_roles;
pub mod wizard_stores;

pub use self::attribute_values::*;
pub use self::attributes::*;
pub use self::base_products::*;
pub use self::categories::*;
pub use self::coupons::*;
pub use self::currency_exchange::*;
pub use self::custom_attributes::*;
pub use self::moderator_comments::*;
pub use self::products::*;
pub use self::stores::*;
pub use self::types::*;
pub use self::user_roles::*;
pub use self::wizard_stores::*;
