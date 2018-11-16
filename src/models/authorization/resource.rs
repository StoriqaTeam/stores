//! Enum for resources available in ACLs
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Resource {
    Products,
    BaseProducts,
    ProductAttrs,
    Attributes,
    AttributeValues,
    Stores,
    UserRoles,
    Categories,
    CategoryAttrs,
    CustomAttributes,
    CurrencyExchange,
    WizardStores,
    ModeratorProductComments,
    ModeratorStoreComments,
    Coupons,
    CouponScopeBaseProducts,
    CouponScopeCategories,
    UsedCoupons,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Resource::Products => write!(f, "products"),
            Resource::BaseProducts => write!(f, "base_products"),
            Resource::ProductAttrs => write!(f, "prod attrs"),
            Resource::Attributes => write!(f, "attributes"),
            Resource::AttributeValues => write!(f, "attribute_values"),
            Resource::Stores => write!(f, "stores"),
            Resource::UserRoles => write!(f, "user roles"),
            Resource::CategoryAttrs => write!(f, "cat attrs"),
            Resource::Categories => write!(f, "categories"),
            Resource::CustomAttributes => write!(f, "custom_attributes"),
            Resource::CurrencyExchange => write!(f, "currency_exchange"),
            Resource::WizardStores => write!(f, "wizard_stores"),
            Resource::ModeratorProductComments => write!(f, "moderator_product_comments"),
            Resource::ModeratorStoreComments => write!(f, "moderator_store_comments"),
            Resource::Coupons => write!(f, "coupons"),
            Resource::CouponScopeBaseProducts => write!(f, "coupon_scope_base_products"),
            Resource::CouponScopeCategories => write!(f, "coupon_scope_categories"),
            Resource::UsedCoupons => write!(f, "used_coupons"),
        }
    }
}
