DELETE FROM prod_attr_values;
DELETE FROM products;
DELETE FROM base_products;
DELETE FROM attributes;

ALTER SEQUENCE attributes_id_seq RESTART WITH 1;

INSERT INTO attributes (uuid, name, value_type, meta_field) VALUES
(uuid_generate_v4(), '[{"lang": "en", "text": "Size"}, {"lang": "ru", "text": "Размер"}]' ,'str'	,'{"values": ["44", "46", "48", "50", "52"], "ui_element": null, "translated_values": null}'),
(uuid_generate_v4(), '[{"lang": "en", "text": "Material"}, {"lang": "ru", "text": "Материал"}]' ,'str'	,'{"values": null, "ui_element": null, "translated_values": [[{"lang": "en", "text": "Tree"}, {"lang": "ru", "text": "Дерево"}], [{"lang": "en", "text": "Glass"}, {"lang": "ru", "text": "Стекло"}], [{"lang": "en", "text": "Metal"}, {"lang": "ru", "text": "Металл"}]]}'),
(uuid_generate_v4(), '[{"lang": "en", "text": "Colour"}, {"lang": "ru", "text": "Цвет"}]' ,'str'	,'{"values": null, "ui_element": null, "translated_values": [[{"lang": "en", "text": "Black"},{"lang": "ru", "text": "Черный"}], [{"lang": "en", "text": "Red"}, {"lang": "ru", "text": "Красный"}], [{"lang": "en", "text": "Blue"}, {"lang": "ru", "text": "Синий"}], [{"lang": "en", "text": "Brown"}, {"lang": "ru", "text": "Коричневый"}]]}');
