ALTER TABLE stores ADD COLUMN uuid uuid;
ALTER TABLE products ADD COLUMN uuid uuid;
ALTER TABLE base_products ADD COLUMN uuid uuid;
ALTER TABLE categories ADD COLUMN uuid uuid;
ALTER TABLE attributes ADD COLUMN uuid uuid;

UPDATE stores SET uuid = uuid_generate_v4();
UPDATE products SET uuid = uuid_generate_v4();
UPDATE base_products SET uuid = uuid_generate_v4();
UPDATE categories SET uuid = uuid_generate_v4();
UPDATE attributes SET uuid = uuid_generate_v4();

CREATE UNIQUE INDEX IF NOT EXISTS stores_stores_uuid_idx ON stores (uuid);
CREATE UNIQUE INDEX IF NOT EXISTS stores_products_uuid_idx ON products (uuid);
CREATE UNIQUE INDEX IF NOT EXISTS stores_base_products_uuid_idx ON base_products (uuid);
CREATE UNIQUE INDEX IF NOT EXISTS stores_categories_uuid_idx ON categories (uuid);
CREATE UNIQUE INDEX IF NOT EXISTS stores_attributes_uuid_idx ON attributes (uuid);

ALTER TABLE stores ALTER COLUMN uuid SET NOT NULL;
ALTER TABLE products ALTER COLUMN uuid SET NOT NULL;
ALTER TABLE base_products ALTER COLUMN uuid SET NOT NULL;
ALTER TABLE categories ALTER COLUMN uuid SET NOT NULL;
ALTER TABLE attributes ALTER COLUMN uuid SET NOT NULL;
