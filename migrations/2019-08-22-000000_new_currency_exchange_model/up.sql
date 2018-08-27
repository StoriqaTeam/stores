DROP TABLE IF EXISTS currency_exchange;

CREATE TABLE currency_exchange (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    data JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('currency_exchange');

CREATE UNIQUE INDEX IF NOT EXISTS currency_exchange_id_idx ON currency_exchange (id);

INSERT INTO currency_exchange (data) VALUES ('{
    "RUB" : { "RUB" : 1.0,      "EUR" : 75.0,    "USD" : 63.0000, "BTC" : 500000.0, "ETH" : 66666.0, "STQ" : 5.5      },
    "EUR" : { "RUB" : 0.013,    "EUR" : 1.0,     "USD" : 1.10000, "BTC" : 7000.0,   "ETH" : 1000.0,  "STQ" : 0.01     },
    "USD" : { "RUB" : 0.016,    "EUR" : 0.9,     "USD" : 1.00000, "BTC" : 8000.0,   "ETH" : 1200.0,  "STQ" : 0.013    },
    "BTC" : { "RUB" : 0.000002, "EUR" : 0.00014, "USD" : 0.00012, "BTC" : 1.0,      "ETH" : 0.01,    "STQ" : 0.000001 },
    "ETH" : { "RUB" : 0.00015,  "EUR" : 0.001,   "USD" : 0.00080, "BTC" : 100.0,    "ETH" : 1.0,     "STQ" : 0.0001   },
    "STQ" : { "RUB" : 0.18,     "EUR" : 100.0,   "USD" : 80.0000, "BTC" : 100000.0, "ETH" : 10000.0, "STQ" : 1.0      }
  }');
