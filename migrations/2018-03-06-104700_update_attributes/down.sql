-- This file should undo anything in `up.sql`
ALTER TABLE attributes DROP COLUMN meta_field;
ALTER TABLE attributes ADD COLUMN ui_type VARCHAR NOT NULL DEFAULT 'textbox';