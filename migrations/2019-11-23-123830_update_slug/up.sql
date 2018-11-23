ALTER TABLE base_products DROP CONSTRAINT base_products_slug_key;

CREATE SEQUENCE base_products_slug_seq;

ALTER TABLE base_products ALTER COLUMN slug SET DEFAULT nextval('base_products_slug_seq');
