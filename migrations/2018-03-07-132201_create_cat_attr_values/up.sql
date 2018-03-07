CREATE TABLE cat_attr_values (
    id SERIAL PRIMARY KEY,
    cat_id INTEGER NOT NULL REFERENCES categories (id),
    attr_id INTEGER NOT NULL REFERENCES attributes (id),
);