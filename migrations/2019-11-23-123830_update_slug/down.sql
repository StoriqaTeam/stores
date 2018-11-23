ALTER TABLE base_products ALTER COLUMN slug SET DEFAULT uuid_generate_v1();

alter table base_products ADD CONSTRAINT base_products_slug_key UNIQUE (slug);

DROP SEQUENCE base_products_slug_seq;
