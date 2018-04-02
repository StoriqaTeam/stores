
ALTER TABLE attributes ADD COLUMN value_type VARCHAR NOT NULL DEFAULT 'str';
ALTER TABLE attributes DROP COLUMN meta_field;
ALTER TABLE attributes ADD COLUMN meta_field JSONB;
