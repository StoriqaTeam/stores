CREATE TABLE languages (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL
);

CREATE UNIQUE INDEX stores_languages_id_idx ON languages (id);

INSERT INTO languages (name) VALUES ('English'); 
INSERT INTO languages (name) VALUES ('Chinese'); 
INSERT INTO languages (name) VALUES ('German'); 
INSERT INTO languages (name) VALUES ('Russian'); 
INSERT INTO languages (name) VALUES ('Spanish'); 
INSERT INTO languages (name) VALUES ('French'); 
INSERT INTO languages (name) VALUES ('Korean'); 
INSERT INTO languages (name) VALUES ('Portuguese'); 
INSERT INTO languages (name) VALUES ('Japanese'); 








