CREATE TABLE currencies (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL
);

CREATE UNIQUE INDEX stores_currency_id_idx ON currencies (id);

INSERT INTO currencies (name) VALUES ('rouble'); 
INSERT INTO currencies (name) VALUES ('euro'); 
INSERT INTO currencies (name) VALUES ('dollar'); 
INSERT INTO currencies (name) VALUES ('bitcoin'); 
INSERT INTO currencies (name) VALUES ('etherium'); 
INSERT INTO currencies (name) VALUES ('stq'); 