-- This file should undo anything in `up.sql`
ALTER TABLE stores DROP COLUMN IF EXISTS default_language;
ALTER TABLE stores ADD COLUMN IF NOT EXISTS language_id INTEGER NOT NULL;
ALTER TABLE stores DROP COLUMN name;
ALTER TABLE stores ADD COLUMN name VARCHAR NOT NULL;
ALTER TABLE stores DROP COLUMN short_description;
ALTER TABLE stores ADD COLUMN short_description VARCHAR NOT NULL;
ALTER TABLE stores DROP COLUMN long_description;
ALTER TABLE stores ADD COLUMN long_description VARCHAR;