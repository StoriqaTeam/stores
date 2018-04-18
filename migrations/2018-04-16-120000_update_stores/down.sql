-- This file should undo anything in `up.sql`
ALTER TABLE stores DROP COLUMN IF EXISTS rating;
ALTER TABLE stores DROP COLUMN IF EXISTS country;
ALTER TABLE stores DROP COLUMN IF EXISTS product_categories;
