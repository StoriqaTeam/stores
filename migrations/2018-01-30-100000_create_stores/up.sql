CREATE TABLE stores (
    id SERIAL PRIMARY KEY,
    is_active BOOLEAN NOT NULL DEFAULT 't',
    name VARCHAR NOT NULL,
    currency_id INTEGER NOT NULL REFERENCES currencies (id),
    short_description VARCHAR NOT NULL,
    long_description VARCHAR,
    slug VARCHAR UNIQUE NOT NULL,
    cover VARCHAR,
    logo VARCHAR,
    phone VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    address VARCHAR NOT NULL,
    facebook_url VARCHAR,
    twitter_url VARCHAR,
    instagram_url VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX stores_store_id_idx ON stores (id);

SELECT diesel_manage_updated_at('stores');