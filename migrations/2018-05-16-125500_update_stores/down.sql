-- This file should undo anything in `up.sql`
ALTER TABLE stores DROP COLUMN IF EXISTS administrative_area_level_1;
ALTER TABLE stores DROP COLUMN IF EXISTS administrative_area_level_2;
ALTER TABLE stores DROP COLUMN IF EXISTS locality;
ALTER TABLE stores DROP COLUMN IF EXISTS political;
ALTER TABLE stores DROP COLUMN IF EXISTS postal_code;
ALTER TABLE stores DROP COLUMN IF EXISTS route;
ALTER TABLE stores DROP COLUMN IF EXISTS street_number;
