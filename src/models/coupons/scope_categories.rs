//! Model coupon_scope_categories table

use stq_types::{CategoryId, CouponId};

use schema::coupon_scope_categories;

#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "coupon_scope_categories"]
pub struct CouponScopeCategories {
    pub id: i32,
    pub coupon_id: CouponId,
    pub category_id: CategoryId,
}

/// Payload for creating coupon_scope_categories
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "coupon_scope_categories"]
pub struct NewCouponScopeCategories {
    pub coupon_id: CouponId,
    pub category_id: CategoryId,
}
