INSERT INTO custom_attributes (base_product_id, attribute_id)
	SELECT DISTINCT base_prod_id, attr_id 
	FROM prod_attr_values 
	LEFT JOIN custom_attributes 
		ON prod_attr_values.base_prod_id = custom_attributes.base_product_id 
		AND prod_attr_values.attr_id = custom_attributes.attribute_id
	WHERE
		custom_attributes.base_product_id IS NULL