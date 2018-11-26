CREATE SEQUENCE categories_slug_seq;

ALTER TABLE categories ADD COLUMN slug varchar NOT NULL UNIQUE DEFAULT nextval('categories_slug_seq');
