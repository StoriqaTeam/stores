CREATE TABLE base_products (
    id SERIAL PRIMARY KEY,
    store_id INTEGER NOT NULL REFERENCES stores (id),
    is_active BOOLEAN NOT NULL DEFAULT 't',
    name JSONB NOT NULL,
    short_description JSONB NOT NULL,
    long_description JSONB,
    currency_id INTEGER NOT NULL REFERENCES currencies (id),
    category_id INTEGER NOT NULL REFERENCES categories (id),
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX stores_base_product_id_idx ON base_products (id);

SELECT diesel_manage_updated_at('base_products');