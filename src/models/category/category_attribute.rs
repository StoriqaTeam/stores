use models::{Attribute, RawCategory};
use schema::cat_attr_values;
/// diesel table for category attributes
use stq_types::{AttributeId, CategoryId};

/// Payload for querying category attributes
#[derive(Debug, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[belongs_to(RawCategory, foreign_key = "cat_id")]
#[belongs_to(Attribute, foreign_key = "attr_id")]
#[table_name = "cat_attr_values"]
pub struct CatAttr {
    pub id: i32,
    pub cat_id: CategoryId,
    pub attr_id: AttributeId,
}

/// Payload for creating category attributes
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "cat_attr_values"]
pub struct NewCatAttr {
    pub cat_id: CategoryId,
    pub attr_id: AttributeId,
}

/// Payload for updating category attributes
#[derive(Serialize, Deserialize, Insertable, AsChangeset, Debug)]
#[table_name = "cat_attr_values"]
pub struct OldCatAttr {
    pub cat_id: CategoryId,
    pub attr_id: AttributeId,
}
