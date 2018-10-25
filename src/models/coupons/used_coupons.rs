//! Model used_coupons

use stq_types::{CouponId, UserId};

use schema::used_coupons;

/// DB presenting by coupon
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "used_coupons"]
#[primary_key(coupon_id, user_id)]
pub struct UsedCoupon {
    pub coupon_id: CouponId,
    pub user_id: UserId,
}

/// Payload for creating coupon
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "used_coupons"]
pub struct NewUsedCoupon {
    pub coupon_id: CouponId,
    pub user_id: UserId,
}
