-- Your SQL goes here
CREATE TABLE moderator_store_comments (
    id SERIAL PRIMARY KEY,
    moderator_id INTEGER,
    store_id INTEGER NOT NULL REFERENCES store (id),
    comments VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX moderator_product_comments_store_id_idx ON moderator_store_comments (store_id);
