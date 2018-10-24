//! Model coupons
use std::time::SystemTime;

use validator::Validate;

use stq_types::{CouponCode, CouponId, StoreId};

use models::validation_rules::*;

use schema::coupons;

/// DB presenting by coupon
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "coupons"]
pub struct Coupon {
    pub id: CouponId,
    pub code: CouponCode,
    pub title: String,
    pub store_id: StoreId,
    pub scope: CouponScope,
    pub percent: i32,
    pub quantity: i32,
    pub expired_at: Option<SystemTime>,
    pub is_active: bool,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Payload for creating coupon
#[derive(Serialize, Deserialize, Insertable, Clone, Validate, Debug)]
#[table_name = "coupons"]
pub struct NewCoupon {
    #[validate(custom = "validate_coupon_code")]
    pub code: CouponCode,
    pub title: String,
    pub store_id: StoreId,
    pub scope: CouponScope,
    #[validate(range(min = "0", max = "100"))]
    pub percent: i32,
    #[validate(custom = "validate_non_negative_coupon_quantity")]
    pub quantity: i32,
    pub expired_at: Option<SystemTime>,
}

impl Coupon {
    pub const MIN_LENGTH_CODE: u64 = 4;
    pub const MAX_LENGTH_CODE: u64 = 12;
    pub const MIN_GENERATE_LENGTH_CODE: usize = 6;
}

/// Payload for updating coupon
#[derive(Serialize, Deserialize, Insertable, AsChangeset, Validate, Debug)]
#[table_name = "coupons"]
pub struct UpdateCoupon {
    #[validate(range(min = "0", max = "100"))]
    pub percent: Option<i32>,
    #[validate(custom = "validate_non_negative_coupon_quantity")]
    pub quantity: Option<i32>,
    pub expired_at: Option<SystemTime>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, DieselTypes)]
pub enum CouponScope {
    Store,
    Categories,
    BaseProducts,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CouponsSearchCodePayload {
    pub code: CouponCode,
    pub store_id: StoreId,
}
