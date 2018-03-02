ALTER TABLE products DROP COLUMN language_id;
ALTER TABLE products DROP COLUMN name;
ALTER TABLE products ADD COLUMN name JSONB NOT NULL;
ALTER TABLE products DROP COLUMN short_description;
ALTER TABLE products ADD COLUMN short_description JSONB NOT NULL;
ALTER TABLE products DROP COLUMN long_description;
ALTER TABLE products ADD COLUMN long_description JSONB;