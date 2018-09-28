CREATE TABLE custom_attributes_values (
    id SERIAL PRIMARY KEY,
    product_id INTEGER NOT NULL REFERENCES products (id),
    custom_attribute_id INTEGER NOT NULL REFERENCES custom_attributes (id),
    value VARCHAR NOT NULL
);
