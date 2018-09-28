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
    CustomAttributes,
    CustomAttributesValues,
    CurrencyExchange,
    WizardStores,
    ModeratorProductComments,
    ModeratorStoreComments,
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
            Resource::CustomAttributes => write!(f, "custom_attributes"),
            Resource::CustomAttributesValues => write!(f, "custom_attributes_values"),
            Resource::CurrencyExchange => write!(f, "currency_exchange"),
            Resource::WizardStores => write!(f, "wizard_stores"),
            Resource::ModeratorProductComments => write!(f, "moderator_product_comments"),
            Resource::ModeratorStoreComments => write!(f, "moderator_store_comments"),
        }
    }
}
