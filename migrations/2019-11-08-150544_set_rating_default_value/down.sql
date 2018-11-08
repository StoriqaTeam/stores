ALTER TABLE base_products
ALTER COLUMN rating SET DEFAULT '1.0'::double precision;

ALTER TABLE stores
ALTER COLUMN rating SET DEFAULT '1.0'::double precision;
