/// diesel table for custom attributes
use stq_types::{BaseProductId, CustomAttributeId};

use models::{Attribute, BaseProduct};
use schema::custom_attributes;

/// Payload for querying custom attributes
#[derive(Debug, Deserialize, Serialize, Associations, Queryable, Clone, Identifiable)]
#[belongs_to(BaseProduct, foreign_key = "base_product_id")]
#[belongs_to(Attribute, foreign_key = "attribute_id")]
#[table_name = "custom_attributes"]
pub struct CustomAttribute {
    pub id: CustomAttributeId,
    pub base_product_id: BaseProductId,
    pub attribute_id: i32,
}

/// Payload for creating custom attributes
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "custom_attributes"]
pub struct NewCustomAttribute {
    pub base_product_id: BaseProductId,
    pub attribute_id: i32,
}
