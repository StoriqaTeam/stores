//! Services is a core layer for the app business logic like
//! validation, authorization, etc.

pub mod attributes;
pub mod base_products;
pub mod categories;
pub mod error;
pub mod products;
pub mod stores;
pub mod system;
pub mod types;
pub mod user_roles;

pub use self::attributes::*;
pub use self::base_products::*;
pub use self::categories::*;
pub use self::products::*;
pub use self::stores::*;
pub use self::system::*;
pub use self::user_roles::*;
