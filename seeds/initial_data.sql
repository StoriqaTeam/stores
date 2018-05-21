
DELETE FROM prod_attr_values;
DELETE FROM products;
DELETE FROM base_products;
DELETE FROM stores;
DELETE FROM cat_attr_values;
DELETE FROM attributes;
DELETE FROM categories;

ALTER SEQUENCE categories_id_seq RESTART WITH 1;
ALTER SEQUENCE attributes_id_seq RESTART WITH 1;
ALTER SEQUENCE prod_attr_values_id_seq RESTART WITH 1;
ALTER SEQUENCE products_id_seq RESTART WITH 1;
ALTER SEQUENCE base_products_id_seq RESTART WITH 1;
ALTER SEQUENCE stores_id_seq RESTART WITH 1;

INSERT INTO categories (name, meta_field, parent_id, level) VALUES
('[{"lang": "en", "text": "Jewelry & Accessories"}, {"lang": "ru", "text": "Ювелирные изделия и аксессуары"}]' ,NULL	,NULL, 1),
('[{"lang": "en", "text": "Clothing & Shoes"}, {"lang": "ru", "text": "Одежда и обувь"}]' ,NULL	,NULL, 1),
('[{"lang": "en", "text": "Accessories"}, {"lang": "ru", "text": "Аксессуары"}]' ,NULL,	1, 2),
('[{"lang": "en", "text": "Bags & purses"}, {"lang": "ru", "text": "Сумки и чехлы"}]' ,NULL,	1, 2),
('[{"lang": "en", "text": "Necklaces"}, {"lang": "ru", "text": "Ожерелья"}]' ,NULL,	1, 2),
('[{"lang": "en", "text": "Bracelets"}, {"lang": "ru", "text": "Браслеты"}]' ,NULL,	1, 2),
('[{"lang": "en", "text": "Womens"}, {"lang": "ru", "text": "Женский"}]' ,NULL,	2, 2),
('[{"lang": "en", "text": "Mens"}, {"lang": "ru", "text": "Мужской"}]' ,NULL,	2, 2),
('[{"lang": "en", "text": "Hair Accessories"}, {"lang": "ru", "text": "Аксессуары для волос"}]' ,NULL,	3, 3),
('[{"lang": "en", "text": "Sunglasses & Eyewear"}, {"lang": "ru", "text": "Солнцезащитные очки и очки"}]' ,NULL,	3, 3),
('[{"lang": "en", "text": "Belts & Suspenders"}, {"lang": "ru", "text": "Ремни и подтяжки"}]' ,NULL,	3, 3),
('[{"lang": "en", "text": "Keychains & Lanyards"}, {"lang": "ru", "text": "Брелки и ремешки"}]' ,NULL,	3, 3),
('[{"lang": "en", "text": "Umbrellas & Rain Accessories"}, {"lang": "ru", "text": "Зонтики и аксессуары"}]' ,NULL,	3, 3),
('[{"lang": "en", "text": "Backpacks"}, {"lang": "ru", "text": "Рюкзаки"}]' ,NULL,	4, 3),
('[{"lang": "en", "text": "Phone Case"}, {"lang": "ru", "text": "Чехол для телефона"}]' ,NULL,	4, 3),
('[{"lang": "en", "text": "Beaded Bracelets"}, {"lang": "ru", "text": "Браслеты из бисера"}]' ,NULL,	6, 3),
('[{"lang": "en", "text": "Cuff Bracelets"}, {"lang": "ru", "text": "Браслеты с манжетами"}]' ,NULL,	6, 3),
('[{"lang": "en", "text": "Dresses"}, {"lang": "ru", "text": "Платья"}]' ,NULL,	7, 3),
('[{"lang": "en", "text": "Jackets & Coats"}, {"lang": "ru", "text": "Куртки и пальто"}]' ,NULL,	7, 3),
('[{"lang": "en", "text": "Shirts"}, {"lang": "ru", "text": "Рубашки"}]' ,NULL,	8, 3),
('[{"lang": "en", "text": "Sweaters"}, {"lang": "ru", "text": "Свитера"}]' ,NULL,	8, 3),
('[{"lang": "en", "text": "Chains"}, {"lang": "ru", "text": "Цепочки"}]' ,NULL,	5, 3),
('[{"lang": "en", "text": "Pendants"}, {"lang": "ru", "text": "Подвески"}]' ,NULL,	5, 3);

INSERT INTO attributes (name, value_type, meta_field) VALUES
('[{"lang": "en", "text": "Size"}, {"lang": "ru", "text": "Размер"}]' ,'str'	,'{"values": ["44", "46", "48", "50", "52"], "ui_element": "Checkbox", "translated_values": null}'),
('[{"lang": "en", "text": "Material"}, {"lang": "ru", "text": "Материал"}]' ,'str'	,'{"values": null, "ui_element": "Combobox", "translated_values": [[{"lang": "en", "text": "Tree"}, {"lang": "ru", "text": "Дерево"}], [{"lang": "en", "text": "Glass"}, {"lang": "ru", "text": "Стекло"}], [{"lang": "en", "text": "Metal"}, {"lang": "ru", "text": "Металл"}]]}'),
('[{"lang": "en", "text": "Colour"}, {"lang": "ru", "text": "Цвет"}]' ,'str'	,'{"values": null, "ui_element": "ColorPicker", "translated_values": [[{"lang": "en", "text": "Black"},{"lang": "ru", "text": "Черный"}], [{"lang": "en", "text": "Red"}, {"lang": "ru", "text": "Красный"}], [{"lang": "en", "text": "Blue"}, {"lang": "ru", "text": "Синий"}], [{"lang": "en", "text": "Brown"}, {"lang": "ru", "text": "Коричневый"}]]}');

INSERT INTO cat_attr_values (cat_id, attr_id) VALUES
(31,1),
(31,2),
(31,3),
(32,1),
(32,2),
(32,3),
(34,1),
(34,2),
(34,3),
(9 ,1),
(9 ,2),
(9 ,3),
(30,1),
(30,2),
(30,3),
(35,1),
(38,1),
(38,2),
(38,3),
(35,2),
(35,3),
(41,1),
(41,2),
(41,3),
(42,1),
(42,2),
(42,3),
(44,3),
(44,2),
(44,1),
(45,1),
(45,2),
(45,3),
(48,1),
(48,2),
(48,3),
(49,1),
(49,2),
(49,3);

INSERT INTO stores (user_id, slug, name, logo, cover, short_description, default_language) VALUES
(2,'slug 1','[{"lang": "de", "text":  "ten store 1 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(2,'slug 2','[{"lang": "de", "text":  "ten store 2 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(2,'slug 3','[{"lang": "de", "text":  "ten store 3 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(3,'slug 4','[{"lang": "de", "text":  "ten store 4 bar"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(3,'slug 5','[{"lang": "de", "text":  "ten store 5 bar"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(3,'slug 6','[{"lang": "de", "text":  "ten store 6 bar"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(3,'slug 7','[{"lang": "de", "text":  "ten store 7 bar"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(4,'slug 8','[{"lang": "de", "text":  "ten store 8 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(4,'slug 9','[{"lang": "de", "text":  "ten store 9 baz"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(4,'slug 10','[{"lang": "de", "text": "ten store 10 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(4,'slug 11','[{"lang": "de", "text": "twelve store 11 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(4,'slug 12','[{"lang": "de", "text": "twelve store 12 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(4,'slug 13','[{"lang": "de", "text": "twelve store 13 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(4,'slug 14','[{"lang": "de", "text": "twelve store 14 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(4,'slug 15','[{"lang": "de", "text": "twelve store 15 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(5,'slug 16','[{"lang": "de", "text": "twelve store 16 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(5,'slug 17','[{"lang": "de", "text": "twelve store 17 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(5,'slug 18','[{"lang": "de", "text": "twelve store 18 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(5,'slug 19','[{"lang": "de", "text": "twelve store 19 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(5,'slug 20','[{"lang": "de", "text": "twelve store 20 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(5,'slug 21','[{"lang": "de", "text": "twelve store 21 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(5,'slug 22','[{"lang": "de", "text": "twelve store 22 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en'),
(5,'slug 23','[{"lang": "de", "text": "twelve store 23 foo"}, {"lang": "en", "text": "foobar"}]', 'https://image.ibb.co/isVHNH/35e3138162918f3eb2af0c5738423bf5.png', 'https://preview.ibb.co/bGDRGc/boutique_shop_free_cute_facebook_timeline_cover1.jpg', '[{"lang": "en", "text": "foobar"}]', 'en');

INSERT INTO base_products (store_id, name, short_description, currency_id, category_id, views) VALUES
(1, '[{"lang": "en", "text": "test product 16"}, {"lang": "ru", "text": "тест продукт16"}]','[{"lang": "en", "text": "test"}]',2,10,792),
(2, '[{"lang": "en", "text": "test product 15"}, {"lang": "ru", "text": "тест продукт15"}]','[{"lang": "en", "text": "test"}]',2,11,791),
(3, '[{"lang": "en", "text": "test product 17"}, {"lang": "ru", "text": "тест продукт17"}]','[{"lang": "en", "text": "test"}]',2,12,791),
(4, '[{"lang": "en", "text": "test product 18"}, {"lang": "ru", "text": "тест продукт18"}]','[{"lang": "en", "text": "test"}]',2,13,790),
(5, '[{"lang": "en", "text": "test product 04.09_113700"}, {"lang": "ru", "text": "тест продукт 04.09_113700"}]','[{"lang": "en", "text": "test"}]',2,14,80),
(6, '[{"lang": "en", "text": "test product 04.09_113710"}, {"lang": "ru", "text": "тест продукт 04.09_113710"}]','[{"lang": "en", "text": "test"}]',2,15,79),
(7, '[{"lang": "en", "text": "test product 04.09_120135"}, {"lang": "ru", "text": "тест продукт 04.09_120135"}]','[{"lang": "en", "text": "test"}]',2,16,61),
(8, '[{"lang": "en", "text": "test product 04.09_120604"}, {"lang": "ru", "text": "тест продукт 04.09_120604"}]','[{"lang": "en", "text": "test"}]',2,17,60),
(9, '[{"lang": "en", "text": "test product 04.09_120727"}, {"lang": "ru", "text": "тест продукт 04.09_120727"}]','[{"lang": "en", "text": "test"}]',2,18,59),
(10, '[{"lang": "en", "text": "test product 14"}, {"lang": "ru", "text": "тест продукт 14"}]','[{"lang": "en", "text": "test"}]',2,19, 806);

INSERT INTO products (discount, base_product_id, price, photo_main, currency_id) VALUES
(0.2, 1,100, 'https://image.ibb.co/k66QZn/1.jpg', 1),
(0.25, 2,200, 'https://image.ibb.co/k66QZn/1.jpg', 1),
(0.25, 3,200, 'https://image.ibb.co/k66QZn/1.jpg', 1),
(0.25, 4,200, 'https://image.ibb.co/k66QZn/1.jpg', 1),
(0.25, 5,200, 'https://image.ibb.co/bWz177/3.jpg', 1),
(0.35, 6,200, 'https://image.ibb.co/bWz177/3.jpg', 1),
(0.25, 7,200, 'https://image.ibb.co/bWz177/3.jpg', 1),
(0.25, 8,200, 'https://image.ibb.co/eRz177/2.jpg', 1),
(0.15, 9,200, 'https://image.ibb.co/eRz177/2.jpg', 1),
(0.25, 10,200, 'https://image.ibb.co/eRz177/2.jpg', 1);

INSERT INTO prod_attr_values (prod_id, attr_id, value, value_type, meta_field, base_prod_id) VALUES
(1,1,'44','str','dfasfas',1),
(1,2,'Tree','str','dfasfas',1),
(1,3,'Black','str','dfasfas',1),
(2,1,'44','str','dfasfas',2),
(2,2,'Tree','str','dfasfas',2),
(2,3,'Black','str','dfasfas',2),
(3,1,'44','str','dfasfas',3),
(3,2,'Tree','str','dfasfas',3),
(3,3,'Black','str','dfasfas',3),
(4,1,'44','str','dfasfas',4),
(4,2,'Tree','str','dfasfas',4),
(4,3,'Black','str','dfasfas',4),
(5,1,'44','str','dfasfas',5),
(5,2,'Tree','str','dfasfas',5),
(5,3,'Black','str','dfasfas',5),
(6,1,'44','str','dfasfas',6),
(6,2,'Tree','str','dfasfas',6),
(6,3,'Black','str','dfasfas',6),
(7,1,'44','str','dfasfas',7),
(7,2,'Tree','str','dfasfas',7),
(7,3,'Black','str','dfasfas',7),
(8,1,'44','str','dfasfas',8),
(8,2,'Tree','str','dfasfas',8),
(8,3,'Black','str','dfasfas',8),
(9,1,'44','str','dfasfas',9),
(9,2,'Tree','str','dfasfas',9),
(9,3,'Black','str','dfasfas',9),
(10,1,'44','str','dfasfas',10),
(10,2,'Tree','str','dfasfas',10),
(10,3,'Black','str','dfasfas',10);

