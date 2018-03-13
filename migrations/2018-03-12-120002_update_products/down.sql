-- This file should undo anything in `up.sql`
ALTER TABLE products ADD COLUMN store_id INTEGER NOT NULL REFERENCES stores (id);
ALTER TABLE products ADD COLUMN name JSONB NOT NULL;
ALTER TABLE products ADD COLUMN short_description JSONB NOT NULL;
ALTER TABLE products ADD COLUMN long_description JSONB;
ALTER TABLE products ADD COLUMN DOUBLE PRECISION NOT NULL;
ALTER TABLE products ADD COLUMN currency_id INTEGER NOT NULL DEFAULT '1';
ALTER TABLE products ADD COLUMN category_id INTEGER NOT NULL REFERENCES categories (id);
