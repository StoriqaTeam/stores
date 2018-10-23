CREATE TABLE coupon_scope_base_products (
    id SERIAL PRIMARY KEY,
    coupon_id INTEGER NOT NULL REFERENCES coupons (id),
    base_product_id INTEGER NOT NULL REFERENCES base_products (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS coupon_scope_base_products_unique_idx ON coupon_scope_base_products (coupon_id, base_product_id);
