-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS stores_slug_is_active_idx;
ALTER TABLE stores ADD CONSTRAINT unique_slug UNIQUE (slug, is_active);
