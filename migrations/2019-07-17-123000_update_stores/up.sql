ALTER TABLE stores DROP CONSTRAINT unique_slug;
CREATE UNIQUE INDEX stores_slug_is_active_idx ON stores (slug) WHERE is_active = 'true';
