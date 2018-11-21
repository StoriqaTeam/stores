DROP INDEX IF EXISTS stores_stores_uuid_idx;
DROP INDEX IF EXISTS stores_products_uuid_idx;
DROP INDEX IF EXISTS stores_base_products_uuid_idx;
DROP INDEX IF EXISTS stores_categories_uuid_idx;
DROP INDEX IF EXISTS stores_attributes_uuid_idx;

ALTER TABLE stores DROP COLUMN uuid;
ALTER TABLE products DROP COLUMN uuid;
ALTER TABLE base_products DROP COLUMN uuid;
ALTER TABLE categories DROP COLUMN uuid;
ALTER TABLE attributes DROP COLUMN uuid;
