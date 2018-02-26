-- This file should undo anything in `up.sql`
ALTER TABLE products DROP COLUMN IF EXISTS default_language;
ALTER TABLE products ADD COLUMN IF NOT EXISTS default_language VARCHAR NOT NULL;