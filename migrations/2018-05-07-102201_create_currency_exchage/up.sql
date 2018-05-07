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
