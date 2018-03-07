
ALTER TABLE attributes DROP COLUMN name;
ALTER TABLE attributes ADD COLUMN name JSONB NOT NULL DEFAULT '[{"lang" : "en", "text" : "unknown"}]';
