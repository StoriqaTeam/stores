-- Your SQL goes here
CREATE TABLE moderator_product_comments (
    id SERIAL PRIMARY KEY,
    moderator_id INTEGER,
    base_product_id INTEGER NOT NULL REFERENCES base_products (id),
    comments VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX IF NOT EXISTS moderator_product_comments_id_idx ON moderator_product_comments (id);
