-- This file should undo anything in `up.sql`
ALTER TABLE attributes DROP COLUMN value_type;
ALTER TABLE attributes DROP COLUMN meta_field;
ALTER TABLE attributes ADD COLUMN meta_field VARCHAR;