-- Your SQL goes here
CREATE TABLE attributes (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    ui_type VARCHAR NOT NULL
);

CREATE UNIQUE INDEX stores_attribute_id_idx ON attributes (id);

INSERT INTO attributes (name, ui_type) VALUES ('price', 'textbox'); 
INSERT INTO attributes (name, ui_type) VALUES ('color', 'combobox'); 
INSERT INTO attributes (name, ui_type) VALUES ('size', 'combobox'); 
