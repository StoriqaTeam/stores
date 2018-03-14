-- This file should undo anything in `up.sql`
ALTER TABLE attributes DROP COLUMN name;
ALTER TABLE attributes ADD COLUMN name VARCHAR NOT NULL DEFAULT 'unknown';