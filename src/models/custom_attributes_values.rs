/// diesel table for custom attributes values
use stq_types::ProductId;

use models::{CustomAttribute, Product};
use schema::custom_attributes_values;

/// Payload for querying custom attributes values
#[derive(Debug, Deserialize, Serialize, Associations, Queryable, Clone, Identifiable)]
#[belongs_to(Product, foreign_key = "product_id")]
#[belongs_to(CustomAttribute, foreign_key = "custom_attribute_id")]
#[table_name = "custom_attributes_values"]
pub struct CustomAttributeValue {
    pub id: i32,
    pub product_id: ProductId,
    pub custom_attribute_id: i32,
    pub value: String,
}

/// Payload for creating custom attributes values
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "custom_attributes_values"]
pub struct NewCustomAttributeValue {
    pub product_id: ProductId,
    pub custom_attribute_id: i32,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewCustomAttributeValuePayload {
    pub custom_attribute_id: i32,
    pub value: String,
}

impl NewCustomAttributeValue {
    pub fn into_vec(product_id: ProductId, new_attrs: Vec<NewCustomAttributeValuePayload>) -> Vec<NewCustomAttributeValue> {
        let mut res = vec![];
        for value in new_attrs {
            res.push(NewCustomAttributeValue {
                product_id,
                custom_attribute_id: value.custom_attribute_id,
                value: value.value,
            })
        }
        res
    }
}
