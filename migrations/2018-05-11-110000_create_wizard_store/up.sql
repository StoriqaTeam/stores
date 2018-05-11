CREATE TABLE wizard_stores (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    store_id INTEGER,
    name VARCHAR,
    short_description VARCHAR,
    default_language VARCHAR,
    slug VARCHAR,
    country VARCHAR,
    address VARCHAR
);

CREATE UNIQUE INDEX wizard_stores_user_id_idx ON wizard_stores (user_id);
