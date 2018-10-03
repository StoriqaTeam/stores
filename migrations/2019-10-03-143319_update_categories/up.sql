ALTER TABLE categories 
    ALTER COLUMN parent_id SET DEFAULT 0;

UPDATE categories SET parent_id = DEFAULT where parent_id IS NULL;