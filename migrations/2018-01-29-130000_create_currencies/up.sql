CREATE TABLE currencies (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL
);

CREATE UNIQUE INDEX stores_currency_id_idx ON currencies (id);

INSERT INTO currencies (name) VALUES ('roubles'); 