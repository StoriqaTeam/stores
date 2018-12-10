ALTER TABLE base_products
DROP CONSTRAINT base_products_store_id_fkey,
ADD CONSTRAINT base_products_store_id_fkey
   FOREIGN KEY (store_id)
   REFERENCES stores(id)
   ON DELETE RESTRICT;

ALTER TABLE coupons
DROP CONSTRAINT coupons_store_id_fkey,
ADD CONSTRAINT coupons_store_id_fkey
   FOREIGN KEY (store_id)
   REFERENCES stores(id)
   ON DELETE RESTRICT;

ALTER TABLE moderator_store_comments
DROP CONSTRAINT moderator_store_comments_store_id_fkey,
ADD CONSTRAINT moderator_store_comments_store_id_fkey
   FOREIGN KEY (store_id)
   REFERENCES stores(id)
   ON DELETE RESTRICT;

ALTER TABLE coupon_scope_base_products
DROP CONSTRAINT coupon_scope_base_products_base_product_id_fkey,
ADD CONSTRAINT coupon_scope_base_products_base_product_id_fkey
   FOREIGN KEY (base_product_id)
   REFERENCES base_products(id)
   ON DELETE RESTRICT;

ALTER TABLE custom_attributes
DROP CONSTRAINT custom_attributes_base_product_id_fkey,
ADD CONSTRAINT custom_attributes_base_product_id_fkey
   FOREIGN KEY (base_product_id)
   REFERENCES base_products(id)
   ON DELETE RESTRICT;

ALTER TABLE moderator_product_comments
DROP CONSTRAINT moderator_product_comments_base_product_id_fkey,
ADD CONSTRAINT moderator_product_comments_base_product_id_fkey
   FOREIGN KEY (base_product_id)
   REFERENCES base_products(id)
   ON DELETE RESTRICT;

ALTER TABLE products
DROP CONSTRAINT products_base_product_id_fkey,
ADD CONSTRAINT products_base_product_id_fkey
   FOREIGN KEY (base_product_id)
   REFERENCES base_products(id)
   ON DELETE RESTRICT;

ALTER TABLE prod_attr_values
DROP CONSTRAINT prod_attr_values_attr_value_id_fkey,
ADD CONSTRAINT prod_attr_values_attr_value_id_fkey
   FOREIGN KEY (attr_value_id)
   REFERENCES attribute_values(id)
   ON DELETE RESTRICT;

ALTER TABLE prod_attr_values
DROP CONSTRAINT prod_attr_values_base_prod_id_fkey,
ADD CONSTRAINT prod_attr_values_base_prod_id_fkey
   FOREIGN KEY (base_prod_id)
   REFERENCES base_products(id)
   ON DELETE RESTRICT;

ALTER TABLE prod_attr_values
DROP CONSTRAINT prod_attr_values_prod_id_fkey,
ADD CONSTRAINT prod_attr_values_prod_id_fkey
   FOREIGN KEY (prod_id)
   REFERENCES products(id)
   ON DELETE RESTRICT;
