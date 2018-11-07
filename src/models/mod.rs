//! Models contains all structures that are used in different
//! modules of the app

pub mod attributes;
pub mod authorization;
pub mod base_product;
pub mod category;
pub mod coupons;
pub mod currency_exchange;
pub mod custom_attributes;
pub mod elastic;
pub mod moderator_product_comment;
pub mod moderator_store_comment;
pub mod pagination;
pub mod product;
pub mod store;
pub mod user_role;
pub mod validation_rules;
pub mod visibility;
pub mod wizard_store;

pub use self::attributes::*;
pub use self::authorization::*;
pub use self::base_product::*;
pub use self::category::*;
pub use self::coupons::*;
pub use self::currency_exchange::*;
pub use self::custom_attributes::*;
pub use self::elastic::*;
pub use self::moderator_product_comment::*;
pub use self::moderator_store_comment::*;
pub use self::pagination::*;
pub use self::product::*;
pub use self::store::*;
pub use self::user_role::*;
pub use self::validation_rules::*;
pub use self::visibility::*;
pub use self::wizard_store::*;
