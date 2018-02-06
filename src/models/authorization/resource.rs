//! Enum for resources available in ACLs
use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Resource {
    Products,
    Stores,
    UserRoles,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Resource::Products => write!(f, "products"),
            Resource::Stores => write!(f, "stores"),
            Resource::UserRoles => write!(f, "user roles"),
        }
    }
}
