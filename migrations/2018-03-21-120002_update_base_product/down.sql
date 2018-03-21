-- This file should undo anything in `up.sql`
ALTER TABLE base_products DROP COLUMN IF EXISTS seo_title;
ALTER TABLE base_products DROP COLUMN IF EXISTS seo_description;
