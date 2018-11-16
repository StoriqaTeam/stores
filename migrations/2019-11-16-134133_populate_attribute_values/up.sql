insert into attribute_values (attr_id, code, translations)

select
trans.id as attr_id,
'' as code,
jsonb_array_elements(trans.meta_field->'translated_values') as translations
from attributes trans where trans.meta_field->'translated_values' != 'null'::jsonb
union
select
codes.id as attr_id,
jsonb_array_elements_text(codes.meta_field->'values') as code,
null as translations
from attributes codes where codes.meta_field->'values' != 'null'::jsonb;

update attribute_values set code=(
  select av.translations->0->'text' from attribute_values as av where av.id=attribute_values.id
) where code='';

update prod_attr_values set attr_value_id=(select av.id from attribute_values av where av.attr_id=attr_id and av.code=value);
