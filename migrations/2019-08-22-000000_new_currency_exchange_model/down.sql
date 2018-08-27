DROP TABLE IF EXISTS currency_exchange;

CREATE TABLE currency_exchange (
    id SERIAL PRIMARY KEY,
    rouble JSONB NOT NULL,
    euro JSONB NOT NULL,
    dollar JSONB NOT NULL,
    bitcoin JSONB NOT NULL,
    etherium JSONB NOT NULL,
    stq JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('currency_exchange');

CREATE UNIQUE INDEX IF NOT EXISTS currency_exchange_id_idx ON currency_exchange (id);

INSERT INTO currency_exchange (rouble, euro, dollar, bitcoin, etherium, stq) VALUES (
    '{ "rouble" : 1.0, "euro" : 75.0, "dollar" : 63.0, "bitcoin" : 500000.0, "etherium" : 66666.0, "stq" : 5.5 }',
    '{ "rouble" : 0.013, "euro" : 1.0, "dollar" : 1.1, "bitcoin" : 7000.0, "etherium" : 1000.0, "stq" : 0.01 }',
    '{ "rouble" : 0.016, "euro" : 0.9, "dollar" : 1.0, "bitcoin" : 8000.0, "etherium" : 1200.0, "stq" : 0.013 }',
    '{ "rouble" : 0.000002, "euro" : 0.00014, "dollar" : 0.00012, "bitcoin" : 1.0, "etherium" : 0.01, "stq" : 0.000001 }',
    '{ "rouble" : 0.00015, "euro" : 0.001, "dollar" : 0.0008, "bitcoin" : 100.0, "etherium" : 1.0, "stq" : 0.0001 }',
    '{ "rouble" : 0.18, "euro" : 100.0, "dollar" : 80.0, "bitcoin" : 100000.0, "etherium" : 10000.0, "stq" : 1.0 }'
);
