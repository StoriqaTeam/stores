-- Your SQL goes here
CREATE TABLE coupons (
    id SERIAL PRIMARY KEY,
    code VARCHAR NOT NULL,
    title VARCHAR NOT NULL,
    store_id INTEGER NOT NULL REFERENCES stores (id),
    scope VARCHAR NOT NULL,
    percent INTEGER NOT NULL,
    quantity INTEGER NOT NULL CHECK (quantity >= 0),
    expired_at TIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT 't',
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX IF NOT EXISTS coupons_code_store_idx ON coupons (code, store_id);
