-- This file should undo anything in `up.sql`
ALTER TABLE products ADD COLUMN IF NOT EXISTS category INTEGER;