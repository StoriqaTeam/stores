ALTER TABLE products DROP COLUMN vendor_code;
ALTER TABLE products ADD COLUMN vendor_code VARCHAR NOT NULL DEFAULT uuid_generate_v1();
