CREATE TABLE custom_attributes (
    id SERIAL PRIMARY KEY,
    base_product_id INTEGER NOT NULL REFERENCES base_products (id),
    attribute_id INTEGER NOT NULL REFERENCES attributes (id)
);
