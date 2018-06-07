//! Services is a core layer for the app business logic like
//! validation, authorization, etc.

pub mod attributes;
pub mod base_products;
pub mod categories;
pub mod currency_exchange;
pub mod moderator_comments;
pub mod products;
pub mod stores;
pub mod system;
pub mod types;
pub mod user_roles;
pub mod wizard_stores;

pub use self::attributes::*;
pub use self::base_products::*;
pub use self::categories::*;
pub use self::currency_exchange::*;
pub use self::moderator_comments::*;
pub use self::products::*;
pub use self::stores::*;
pub use self::system::*;
pub use self::user_roles::*;
pub use self::wizard_stores::*;
