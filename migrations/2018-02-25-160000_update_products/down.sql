-- This file should undo anything in `up.sql`
ALTER TABLE products DROP COLUMN IF EXISTS language_id;
ALTER TABLE products ADD COLUMN IF NOT EXISTS default_language VARCHAR NOT NULL;
ALTER TABLE products ADD COLUMN IF NOT EXISTS category INTEGER;