-- This file should undo anything in `up.sql`
ALTER TABLE stores DROP COLUMN IF EXISTS language;
ALTER TABLE stores ADD COLUMN IF NOT EXISTS language_id INTEGER NOT NULL;
