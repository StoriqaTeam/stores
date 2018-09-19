ALTER TABLE categories DROP COLUMN IF EXISTS meta_field;
ALTER TABLE categories ADD COLUMN meta_field JSONB;