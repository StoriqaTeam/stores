//! EAV model attributes
use serde_json;
use stq_static_resources::Translation;
use uuid::Uuid;
use validator::Validate;

use stq_static_resources::AttributeType;
use stq_types::{AttributeId, AttributeValueCode, AttributeValueId};

use models::validation_rules::*;
use schema::attributes;

#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "attributes"]
pub struct Attribute {
    pub id: AttributeId,
    pub name: serde_json::Value,
    pub value_type: AttributeType,
    pub meta_field: Option<serde_json::Value>,
    pub uuid: Uuid,
}

/// Payload for creating attributes
#[derive(Serialize, Deserialize, Insertable, Clone, Validate, Debug)]
#[table_name = "attributes"]
pub struct NewAttribute {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub value_type: AttributeType,
    pub meta_field: Option<serde_json::Value>,
    pub uuid: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AttributeMetaField {
    pub values: Option<Vec<String>>,
    pub translated_values: Option<Vec<Vec<Translation>>>,
    pub ui_element: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate, PartialEq)]
pub struct CreateAttributePayload {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub value_type: AttributeType,
    pub meta_field: Option<AttributeMetaField>,
    pub values: Option<Vec<CreateAttributeWithAttribute>>,
    pub uuid: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate, PartialEq)]
pub struct CreateAttributeWithAttribute {
    #[validate(custom = "validate_translation")]
    pub translations: Option<serde_json::Value>,
    pub code: AttributeValueCode,
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
    pub attr_value_id: Option<AttributeValueId>,
    pub value: AttributeValueCode,
    pub meta_field: Option<String>,
}
