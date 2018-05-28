-- This file should undo anything in `up.sql`
ALTER TABLE wizard_stores DROP COLUMN IF EXISTS place_id;
