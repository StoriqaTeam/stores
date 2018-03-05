//! Enum for resources available in ACLs
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Resource {
    Products,
    ProductAttrs,
    Attributes,
    Stores,
    UserRoles,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Resource::Products => write!(f, "products"),
            Resource::Stores => write!(f, "stores"),
            Resource::UserRoles => write!(f, "user roles"),
            Resource::ProductAttrs => write!(f, "prod attrs"),
            Resource::Attributes => write!(f, "attributes"),
        }
    }
}
