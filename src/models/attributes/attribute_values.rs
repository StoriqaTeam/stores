use validator::Validate;

use stq_types::{AttributeId, AttributeValueCode, AttributeValueId};

use models::validation_rules::*;
use schema::attribute_values;

#[derive(Debug, Clone, Serialize, Deserialize, Associations, Queryable, Identifiable)]
#[table_name = "attribute_values"]
pub struct AttributeValue {
    pub id: AttributeValueId,
    pub attr_id: AttributeId,
    pub code: AttributeValueCode,
    pub translations: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Insertable, Validate, Clone, Debug)]
#[table_name = "attribute_values"]
pub struct NewAttributeValue {
    pub attr_id: AttributeId,
    pub code: AttributeValueCode,
    #[validate(custom = "validate_translation")]
    pub translations: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Validate, Debug)]
#[table_name = "attribute_values"]
pub struct UpdateAttributeValue {
    #[validate(custom = "validate_translation")]
    pub translations: Option<serde_json::Value>,
    pub code: Option<AttributeValueCode>,
}
