//! Model coupon_scope_base_products table

use stq_types::{BaseProductId, CouponId};

use schema::coupon_scope_base_products;

#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "coupon_scope_base_products"]
pub struct CouponScopeBaseProducts {
    pub id: i32,
    pub coupon_id: CouponId,
    pub base_product_id: BaseProductId,
}

/// Payload for creating coupon_scope_base_products
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "coupon_scope_base_products"]
pub struct NewCouponScopeBaseProducts {
    pub coupon_id: CouponId,
    pub base_product_id: BaseProductId,
}
