//! Enum for resources available in ACLs
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Resource {
    Products,
    BaseProducts,
    ProductAttrs,
    Attributes,
    Stores,
    UserRoles,
    Categories,
    CategoryAttrs,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Resource::Products => write!(f, "products"),
            Resource::BaseProducts => write!(f, "base_products"),
            Resource::ProductAttrs => write!(f, "prod attrs"),
            Resource::Attributes => write!(f, "attributes"),
            Resource::Stores => write!(f, "stores"),
            Resource::UserRoles => write!(f, "user roles"),
            Resource::CategoryAttrs => write!(f, "cat attrs"),
            Resource::Categories => write!(f, "categories"),
        }
    }
}
