-- Your SQL goes here
CREATE TABLE attributes (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    ty VARCHAR NOT NULL
);

CREATE UNIQUE INDEX stores_attribute_id_idx ON attributes (id);

INSERT INTO attributes (name, ty) VALUES ('price', 'float'); 
INSERT INTO attributes (name, ty) VALUES ('color', 'str'); 
INSERT INTO attributes (name, ty) VALUES ('size', 'str'); 
