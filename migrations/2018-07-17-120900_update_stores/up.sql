ALTER TABLE stores DROP CONSTRAINT stores_slug_key;
ALTER TABLE stores ADD CONSTRAINT unique_slug UNIQUE (slug, is_active);
