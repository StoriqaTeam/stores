-- Your SQL goes here
CREATE TABLE moderator_store_comments (
    id SERIAL PRIMARY KEY,
    moderator_id INTEGER,
    store_id INTEGER NOT NULL REFERENCES stores (id),
    comments VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX IF NOT EXISTS moderator_product_comments_id_idx ON moderator_store_comments (id);
