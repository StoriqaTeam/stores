CREATE TABLE coupon_scope_categories (
    id SERIAL PRIMARY KEY,
    coupon_id INTEGER NOT NULL REFERENCES coupons (id),
    category_id INTEGER NOT NULL REFERENCES categories (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS coupon_scope_categories_unique_idx ON coupon_scope_categories (coupon_id, category_id);
