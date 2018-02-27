-- This file should undo anything in `up.sql`
ALTER TABLE products DROP COLUMN IF EXISTS language_id;
ALTER TABLE products ADD COLUMN IF NOT EXISTS default_language VARCHAR NOT NULL;
ALTER TABLE products ADD COLUMN IF NOT EXISTS category INTEGER;
ALTER TABLE stores DROP COLUMN IF EXISTS currency_id;
ALTER TABLE stores ADD COLUMN IF NOT EXISTS currency_id INTEGER NOT NULL REFERENCES currencies (id);