CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    store_id INTEGER NOT NULL REFERENCES stores (id),
    is_active BOOLEAN NOT NULL DEFAULT 't',
    name VARCHAR NOT NULL,
    short_description VARCHAR NOT NULL,
    long_description VARCHAR,
    price DOUBLE PRECISION NOT NULL,
    currency_id INTEGER NOT NULL REFERENCES currencies (id),
    discount FLOAT,
    category INTEGER,
    photo_main VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX stores_product_id_idx ON products (id);

SELECT diesel_manage_updated_at('products');