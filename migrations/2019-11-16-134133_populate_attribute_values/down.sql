DROP INDEX IF EXISTS stores_attribute_values_id_idx;

update prod_attr_values set attr_value_id=null;

delete from attribute_values;
