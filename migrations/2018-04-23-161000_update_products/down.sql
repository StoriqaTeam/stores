-- This file should undo anything in `up.sql`
ALTER TABLE products DROP COLUMN vendor_code;
ALTER TABLE products ADD COLUMN vendor_code VARCHAR;
