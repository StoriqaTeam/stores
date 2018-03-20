-- This file should undo anything in `up.sql`
DELETE FROM prod_attr_values;
DELETE FROM products;
DELETE FROM base_products;
DELETE FROM categories CASCADE;
