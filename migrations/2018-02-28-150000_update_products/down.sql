-- This file should undo anything in `up.sql`
ALTER TABLE products ADD COLUMN IF NOT EXISTS language_id INTEGER NOT NULL;