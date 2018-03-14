//! Services is a core layer for the app business logic like
//! validation, authorization, etc.

pub mod stores;
pub mod user_roles;
pub mod attributes;
pub mod categories;
pub mod products;
pub mod system;
pub mod error;
pub mod types;

pub use self::products::*;
pub use self::attributes::*;
pub use self::categories::*;
pub use self::stores::*;
pub use self::system::*;
pub use self::user_roles::*;
