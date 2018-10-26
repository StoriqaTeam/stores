pub mod coupons;
pub mod scope_base_products;
pub mod scope_categories;
pub mod used_coupons;

pub use self::coupons::*;
pub use self::scope_base_products::*;
pub use self::scope_categories::*;
pub use self::used_coupons::*;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum CouponValidate {
    NotActive,
    HasExpired,
    NoActivationsAvailable,
    AlreadyActivated,
    Valid,
}
