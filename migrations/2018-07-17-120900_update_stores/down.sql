-- This file should undo anything in `up.sql`
ALTER TABLE stores DROP CONSTRAINT unique_slug;
ALTER TABLE stores ADD CONSTRAINT stores_slug_key UNIQUE (slug);
