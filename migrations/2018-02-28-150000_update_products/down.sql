-- This file should undo anything in `up.sql`
ALTER TABLE products ADD COLUMN IF NOT EXISTS language_id INTEGER NOT NULL;
ALTER TABLE products DROP COLUMN name;
ALTER TABLE products ADD COLUMN name VARCHAR NOT NULL;
ALTER TABLE products DROP COLUMN short_description;
ALTER TABLE products ADD COLUMN short_description VARCHAR NOT NULL;
ALTER TABLE products DROP COLUMN long_description;
ALTER TABLE products ADD COLUMN long_description VARCHAR;