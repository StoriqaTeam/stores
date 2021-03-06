//! Repos is a module responsible for interacting with postgres db
#[macro_use]
pub mod acl;
pub mod attribute_values;
pub mod attributes;
pub mod base_products;
pub mod categories;
pub mod coupons;
pub mod currency_exchange;
pub mod custom_attributes;
pub mod moderator_product;
pub mod moderator_store;
pub mod product_attrs;
pub mod products;
pub mod repo_factory;
pub mod stores;
pub mod types;
pub mod user_roles;
pub mod wizard_stores;

pub use self::acl::*;
pub use self::attribute_values::*;
pub use self::attributes::*;
pub use self::base_products::*;
pub use self::categories::*;
pub use self::coupons::*;
pub use self::currency_exchange::*;
pub use self::custom_attributes::*;
pub use self::moderator_product::*;
pub use self::moderator_store::*;
pub use self::product_attrs::*;
pub use self::products::*;
pub use self::repo_factory::*;
pub use self::stores::*;
pub use self::types::*;
pub use self::user_roles::*;
pub use self::wizard_stores::*;
