CREATE TABLE attribute_values (
    id SERIAL PRIMARY KEY,
    attr_id INTEGER NOT NULL REFERENCES attributes (id),
    code VARCHAR NOT NULL,
    translations jsonb
);

CREATE UNIQUE INDEX stores_attribute_values_id_idx ON attribute_values (id);

ALTER TABLE prod_attr_values ADD COLUMN attr_value_id INTEGER REFERENCES attribute_values (id);
