DELETE FROM custom_attributes 
USING prod_attr_values
WHERE custom_attributes.base_product_id = prod_attr_values.base_prod_id AND custom_attributes.attribute_id = prod_attr_values.attr_id;