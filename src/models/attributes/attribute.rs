//! EAV model attributes
use serde_json;
use validator::Validate;

use stq_static_resources::AttributeType;
use stq_types::{AttributeId, AttributeValue};

use models::validation_rules::*;
use models::*;
use schema::attributes;

#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "attributes"]
pub struct Attribute {
    pub id: AttributeId,
    pub name: serde_json::Value,
    pub value_type: AttributeType,
    pub meta_field: Option<serde_json::Value>,
}

/// Payload for creating attributes
#[derive(Serialize, Deserialize, Insertable, Clone, Validate, Debug)]
#[table_name = "attributes"]
pub struct NewAttribute {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub value_type: AttributeType,
    pub meta_field: Option<serde_json::Value>,
}

/// Payload for updating attributes
#[derive(Serialize, Deserialize, Insertable, AsChangeset, Validate, Debug)]
#[table_name = "attributes"]
pub struct UpdateAttribute {
    #[validate(custom = "validate_translation")]
    pub name: Option<serde_json::Value>,
    pub meta_field: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttrValue {
    pub attr_id: AttributeId,
    pub value: AttributeValue,
    pub meta_field: Option<String>,
}

impl From<ProdAttr> for AttrValue {
    fn from(pr: ProdAttr) -> Self {
        Self {
            attr_id: pr.attr_id,
            value: pr.value,
            meta_field: pr.meta_field,
        }
    }
}
