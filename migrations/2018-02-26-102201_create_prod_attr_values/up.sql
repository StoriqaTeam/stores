CREATE TABLE prod_attr_values (
    id SERIAL PRIMARY KEY,
    prod_id INTEGER NOT NULL REFERENCES products (id),
    attr_id INTEGER NOT NULL REFERENCES attributes (id)
    value VARCHAR NOT NULL
);