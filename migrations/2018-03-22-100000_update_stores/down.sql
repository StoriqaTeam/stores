-- This file should undo anything in `up.sql`
ALTER TABLE stores ADD COLUMN IF NOT EXISTS currency_id INTEGER NOT NULL DEFAULT '1';