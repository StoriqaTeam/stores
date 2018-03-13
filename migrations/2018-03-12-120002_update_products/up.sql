ALTER TABLE products DROP COLUMN store_id;
ALTER TABLE products DROP COLUMN name;
ALTER TABLE products DROP COLUMN short_description;
ALTER TABLE products DROP COLUMN long_description;
ALTER TABLE products DROP COLUMN price;
ALTER TABLE products DROP COLUMN currency_id;
ALTER TABLE products DROP COLUMN category_id;
ALTER TABLE products ADD COLUMN base_product_id INTEGER NOT NULL REFERENCES base_products (id);
