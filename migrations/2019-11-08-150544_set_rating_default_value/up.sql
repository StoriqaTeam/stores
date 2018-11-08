ALTER TABLE base_products
ALTER COLUMN rating SET DEFAULT '4.9'::double precision;

ALTER TABLE stores
ALTER COLUMN rating SET DEFAULT '4.9'::double precision;
