//! Models for working with authorization (acl - access control list)

pub mod action;
pub mod permission;
pub mod resource;
pub mod rule;
pub mod scope;

pub use self::action::Action;
pub use self::permission::Permission;
pub use self::resource::Resource;
pub use self::rule::Rule;
pub use self::scope::Scope;
