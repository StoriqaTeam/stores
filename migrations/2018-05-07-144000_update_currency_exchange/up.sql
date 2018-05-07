CREATE UNIQUE INDEX currency_exchange_id_idx ON currency_exchange (id);

INSERT INTO currency_exchange (rouble, euro, dollar, bitcoin, etherium, stq) VALUES (
    '{ "rouble" : 1.0, "euro" : 2.0, "dollar" : 3.0, "bitcoin" : 4.0, "etherium" : 5.0, "stq" : 6.0 }',
    '{ "rouble" : 0.5, "euro" : 1.0, "dollar" : 3.0, "bitcoin" : 4.0, "etherium" : 5.0, "stq" : 6.0 }',
    '{ "rouble" : 0.5, "euro" : 0.5, "dollar" : 1.0, "bitcoin" : 4.0, "etherium" : 5.0, "stq" : 6.0 }',
    '{ "rouble" : 0.5, "euro" : 7.0, "dollar" : 3.0, "bitcoin" : 1.0, "etherium" : 5.0, "stq" : 6.0 }',
    '{ "rouble" : 0.5, "euro" : 8.0, "dollar" : 3.0, "bitcoin" : 4.0, "etherium" : 1.0, "stq" : 6.0 }',
    '{ "rouble" : 0.5, "euro" : 9.0, "dollar" : 3.0, "bitcoin" : 4.0, "etherium" : 5.0, "stq" : 1.0 }'
);
