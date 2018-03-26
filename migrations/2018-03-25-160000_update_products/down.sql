-- This file should undo anything in `up.sql`
ALTER TABLE products DROP COLUMN IF EXISTS additional_photos;
