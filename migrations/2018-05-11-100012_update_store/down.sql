-- This file should undo anything in `up.sql`
ALTER TABLE stores DROP COLUMN IF EXISTS status;
