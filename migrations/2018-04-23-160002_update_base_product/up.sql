ALTER TABLE base_products ADD COLUMN slug VARCHAR UNIQUE NOT NULL DEFAULT uuid_generate_v1();
