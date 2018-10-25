
CREATE TABLE used_coupons (
    coupon_id INTEGER NOT NULL REFERENCES coupons (id),
    user_id INTEGER NOT NULL,
    primary key(coupon_id, user_id)
);
