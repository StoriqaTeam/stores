ALTER TABLE products DROP COLUMN default_language;
ALTER TABLE products ADD COLUMN language_id INTEGER NOT NULL DEFAULT 1;
ALTER TABLE products DROP COLUMN category;