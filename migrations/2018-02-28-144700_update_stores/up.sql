
ALTER TABLE stores DROP COLUMN language_id;
ALTER TABLE stores ADD COLUMN default_language VARCHAR NOT NULL;
ALTER TABLE stores DROP COLUMN name;
ALTER TABLE stores ADD COLUMN name JSONB NOT NULL;
ALTER TABLE stores DROP COLUMN short_description;
ALTER TABLE stores ADD COLUMN short_description JSONB NOT NULL;
ALTER TABLE stores DROP COLUMN long_description;
ALTER TABLE stores ADD COLUMN long_description JSONB;