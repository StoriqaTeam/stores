-- Your SQL goes here
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name JSONB NOT NULL,
    meta_field VARCHAR,
    parent_id INTEGER
);

CREATE UNIQUE INDEX stores_categories_id_idx ON categories (id);
